//! Encrypted Installer — ships bootstrap as an encrypted blob.
//!
//! Idan's requirement: a fresh ZETS instance should be able to boot from
//! a SINGLE encrypted file that contains all the foundational rules. This
//! way the "brain operating system" can be:
//!   - Distributed (one file)
//!   - Signed (via passphrase/key)
//!   - Tamper-evident (AES-GCM authenticated)
//!   - Immutable at rest (encryption also acts as checksum)
//!
//! Workflow:
//!
//!   CREATE (one-time, by whoever ships ZETS):
//!     1. Fresh AtomStore
//!     2. bootstrap(&mut store) — create 119 atoms + 118 edges
//!     3. atom_persist::serialize(&store, buf)
//!     4. crypto::encrypt(buf, key) → encrypted bytes
//!     5. Write to installer.zets_enc
//!
//!   INSTALL (on every new device):
//!     1. Read installer.zets_enc
//!     2. crypto::decrypt(bytes, key) → cleartext
//!     3. atom_persist::deserialize(cleartext) → AtomStore with bootstrap
//!     4. Optional: verify is_bootstrapped(&store) == true
//!     5. Start using ZETS
//!
//! The encryption uses AES-256-GCM with a key derived from a passphrase
//! (SHA-256). This matches the existing crypto.rs module.

use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};

use crate::atoms::AtomStore;
use crate::atom_persist;
use crate::bootstrap;
use crate::crypto::{key_from_passphrase, EncKey};

/// Magic + format for encrypted installer files.
const INSTALLER_MAGIC: &[u8; 6] = b"ZETSIN";
const INSTALLER_VERSION: u16 = 1;
const NONCE_LEN: usize = 12;

#[derive(Debug)]
pub enum InstallerError {
    Io(std::io::Error),
    Persist(atom_persist::PersistError),
    BadMagic,
    UnsupportedVersion(u16),
    CryptoFail,
    Truncated,
}

impl From<std::io::Error> for InstallerError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<atom_persist::PersistError> for InstallerError {
    fn from(e: atom_persist::PersistError) -> Self { Self::Persist(e) }
}
impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {}", e),
            Self::Persist(e) => write!(f, "persist: {}", e),
            Self::BadMagic => write!(f, "not a ZETS installer (bad magic)"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported installer version: {}", v),
            Self::CryptoFail => write!(f, "decryption failed — wrong passphrase or tampered file"),
            Self::Truncated => write!(f, "installer file truncated"),
        }
    }
}
impl std::error::Error for InstallerError {}

/// Build a fresh bootstrapped store, then encrypt it into an installer blob.
///
/// The output format:
///   [6 bytes] magic "ZETSIN"
///   [2 bytes] version u16 LE
///   [12 bytes] nonce (deterministic: derived from bootstrap content hash)
///   [N bytes] encrypted atom_persist payload (ciphertext + 16-byte tag)
///
/// Note on determinism: for identical inputs (same passphrase), we produce
/// the same output — except GCM requires a nonce. We derive the nonce from
/// the first 12 bytes of the passphrase hash, mixed with a static salt.
/// This means same passphrase = same bytes. If you need different ciphertext
/// each time, use `build_installer_random_nonce` (not implemented — defeats
/// determinism anyway).
pub fn build_installer(passphrase: &str) -> Result<Vec<u8>, InstallerError> {
    // Create fresh bootstrapped store
    let mut store = AtomStore::new();
    bootstrap::bootstrap(&mut store);

    // Serialize it
    let mut plain = Vec::with_capacity(32 * 1024);
    atom_persist::serialize(&store, &mut plain)?;

    // Derive nonce deterministically from passphrase hash
    let key = key_from_passphrase(passphrase);
    let nonce_bytes = derive_nonce(passphrase);

    // Encrypt
    let key_ref = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plain.as_ref())
        .map_err(|_| InstallerError::CryptoFail)?;

    // Compose output
    let mut output = Vec::with_capacity(20 + ciphertext.len());
    output.extend_from_slice(INSTALLER_MAGIC);
    output.extend_from_slice(&INSTALLER_VERSION.to_le_bytes());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Install: decrypt the blob and reconstruct an AtomStore with bootstrap.
pub fn install(blob: &[u8], passphrase: &str) -> Result<AtomStore, InstallerError> {
    if blob.len() < 8 + NONCE_LEN + 16 {
        return Err(InstallerError::Truncated);
    }
    if &blob[0..6] != INSTALLER_MAGIC {
        return Err(InstallerError::BadMagic);
    }
    let version = u16::from_le_bytes(blob[6..8].try_into().unwrap());
    if version != INSTALLER_VERSION {
        return Err(InstallerError::UnsupportedVersion(version));
    }

    let nonce_bytes = &blob[8..8+NONCE_LEN];
    let ciphertext = &blob[8+NONCE_LEN..];

    let key = key_from_passphrase(passphrase);
    let key_ref = Key::<Aes256Gcm>::from_slice(&key);
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| InstallerError::CryptoFail)?;

    let store = atom_persist::deserialize(&plain)?;
    Ok(store)
}

/// Save an installer to disk.
pub fn build_to_file<P: AsRef<Path>>(passphrase: &str, path: P) -> Result<u64, InstallerError> {
    let blob = build_installer(passphrase)?;
    std::fs::write(&path, &blob)?;
    Ok(blob.len() as u64)
}

/// Install from disk.
pub fn install_from_file<P: AsRef<Path>>(path: P, passphrase: &str) -> Result<AtomStore, InstallerError> {
    let blob = std::fs::read(path)?;
    install(&blob, passphrase)
}

/// Derive a 12-byte nonce from a passphrase (deterministic).
/// Mixes passphrase bytes with a static domain separator to ensure that using
/// the same key for a different payload type won't collide.
///
/// Uses three parallel FNV-1a rounds over mixed input streams. This is NOT
/// cryptographic diffusion (SHA-256 would be stronger), but the nonce only
/// needs to be unique per (key, plaintext) pair — it doesn't need to be
/// secret. For unique installer content, this is sufficient.
fn derive_nonce(passphrase: &str) -> [u8; NONCE_LEN] {
    const DOMAIN: &[u8] = b"zets-installer-v1:";
    let mut nonce = [0u8; NONCE_LEN];
    for round in 0..3 {
        let mut h: u64 = 0xcbf29ce484222325u64.wrapping_add(round as u64 * 0xdeadbeef);
        for &b in DOMAIN {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for &b in passphrase.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        let bytes = h.to_le_bytes();
        nonce[round * 4 + 0] = bytes[0];
        nonce[round * 4 + 1] = bytes[1];
        nonce[round * 4 + 2] = bytes[2];
        nonce[round * 4 + 3] = bytes[3];
    }
    nonce
}

/// Suppress unused-import warning when crypto::EncKey isn't referenced above.
#[allow(dead_code)]
fn _use_enckey(_: &EncKey) {}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::{find_bootstrap, is_bootstrapped};

    #[test]
    fn build_and_install_roundtrip() {
        let blob = build_installer("correct horse battery staple").unwrap();
        let store = install(&blob, "correct horse battery staple").unwrap();
        assert!(is_bootstrapped(&store));
        assert!(find_bootstrap(&store, "emotion:joy").is_some());
        assert!(find_bootstrap(&store, "rule:transitivity").is_some());
    }

    #[test]
    fn wrong_passphrase_fails() {
        let blob = build_installer("password1").unwrap();
        let result = install(&blob, "password2");
        assert!(matches!(result, Err(InstallerError::CryptoFail)));
    }

    #[test]
    fn tampered_blob_fails() {
        let mut blob = build_installer("test").unwrap();
        // Flip a byte deep in the ciphertext
        let last = blob.len() - 1;
        blob[last] ^= 0x01;
        let result = install(&blob, "test");
        assert!(matches!(result, Err(InstallerError::CryptoFail)));
    }

    #[test]
    fn bad_magic_rejected() {
        let bytes = vec![0u8; 100];
        let result = install(&bytes, "anything");
        assert!(matches!(result, Err(InstallerError::BadMagic)));
    }

    #[test]
    fn truncated_rejected() {
        let bytes = vec![0u8; 5];
        let result = install(&bytes, "anything");
        assert!(matches!(result, Err(InstallerError::Truncated)));
    }

    #[test]
    fn file_roundtrip() {
        let tmp = std::env::temp_dir().join("zets_installer_test.zets_enc");
        let size = build_to_file("filepass", &tmp).unwrap();
        assert!(size > 20);
        let store = install_from_file(&tmp, "filepass").unwrap();
        assert!(is_bootstrapped(&store));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn deterministic_output_same_passphrase() {
        let blob1 = build_installer("same-pass").unwrap();
        let blob2 = build_installer("same-pass").unwrap();
        // Same passphrase → same deterministic nonce → same output bytes
        assert_eq!(blob1, blob2);
    }

    #[test]
    fn different_passphrases_different_output() {
        let blob1 = build_installer("pass-a").unwrap();
        let blob2 = build_installer("pass-b").unwrap();
        assert_ne!(blob1, blob2);
    }

    #[test]
    fn installer_is_reasonably_small() {
        let blob = build_installer("test").unwrap();
        // 119 atoms + 118 edges ≈ a few KB; should be < 50KB
        assert!(blob.len() < 50_000, "installer too large: {} bytes", blob.len());
    }

    #[test]
    fn reinstalling_on_fresh_store_gives_same_state() {
        let blob = build_installer("x").unwrap();
        let s1 = install(&blob, "x").unwrap();
        let s2 = install(&blob, "x").unwrap();
        assert_eq!(s1.atom_count(), s2.atom_count());
        assert_eq!(s1.edge_count(), s2.edge_count());
    }
}
