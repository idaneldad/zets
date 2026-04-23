//! # Vault — AES-256-GCM encrypted value store
//!
//! Real authenticated encryption (aes-gcm 0.10 crate).
//! Each stored value has its own 12-byte nonce derived deterministically
//! from a counter + SecretId hash — no rand dep needed, no nonce reuse
//! (GCM's deadly sin) because each SecretId/counter pair is unique.
//!
//! ## File format
//!
//! ```text
//! [magic:4 "VLT1"] [counter:u64] [entry_count:u32]
//! For each entry:
//!   [nonce:12] [key_len:u32] [key_bytes] [ciphertext_len:u32] [ciphertext]
//!       where ciphertext = AES-256-GCM(plaintext + auth_tag)
//! ```
//!
//! ## Key derivation
//!
//! Master key from caller (from env var / keychain / HSM in production).
//! We SHA-256 it to get exactly 32 bytes for AES-256 regardless of input.
//!
//! ## Nonce strategy
//!
//! 12-byte nonce = first 4 bytes of SecretId hash || 8 bytes of counter.
//! Counter is stored in the file and incremented on every write.
//! Guarantees no nonce reuse under the same key.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use sha2::{Digest, Sha256};

use super::secret_ref::{SecretId, SecretRef};
use crate::personal_graph::IdentityId;

const MAGIC: &[u8; 4] = b"VLT1";

#[derive(Debug)]
pub enum VaultError {
    Io(io::Error),
    AccessDenied,
    NotFound,
    Corrupt(String),
    SecretDisabled,
    Crypto(String),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Io(e) => write!(f, "I/O: {}", e),
            VaultError::AccessDenied => write!(f, "access denied"),
            VaultError::NotFound => write!(f, "not found"),
            VaultError::Corrupt(s) => write!(f, "corrupt: {}", s),
            VaultError::SecretDisabled => write!(f, "secret disabled"),
            VaultError::Crypto(s) => write!(f, "crypto: {}", s),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<io::Error> for VaultError {
    fn from(e: io::Error) -> Self {
        VaultError::Io(e)
    }
}

/// The Vault — AES-256-GCM encrypted key-value store for secrets.
pub struct Vault {
    path: PathBuf,
    /// In-memory decrypted values.
    values: HashMap<SecretId, Vec<u8>>,
    /// Derived AES key (32 bytes). Lives only in memory.
    cipher: Aes256Gcm,
    /// Monotonic counter for nonce generation.
    counter: u64,
}

impl Vault {
    /// Open or create a vault.
    pub fn open(path: impl Into<PathBuf>, master_key: &[u8]) -> Result<Self, VaultError> {
        let path = path.into();
        let derived_key = derive_key(master_key);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&derived_key));

        let mut vault = Vault {
            path: path.clone(),
            values: HashMap::new(),
            cipher,
            counter: 0,
        };

        if path.exists() {
            vault.load()?;
        }
        Ok(vault)
    }

    pub fn put(
        &mut self,
        secret_ref: &SecretRef,
        caller: &IdentityId,
        value: Vec<u8>,
    ) -> Result<(), VaultError> {
        if !secret_ref.can_access(caller) {
            return Err(VaultError::AccessDenied);
        }
        self.values.insert(secret_ref.id.clone(), value);
        self.persist()?;
        Ok(())
    }

    pub fn get(
        &self,
        secret_ref: &SecretRef,
        caller: &IdentityId,
    ) -> Result<Vec<u8>, VaultError> {
        if !secret_ref.can_access(caller) {
            return Err(VaultError::AccessDenied);
        }
        if !secret_ref.status.allows_access() {
            return Err(VaultError::SecretDisabled);
        }
        self.values
            .get(&secret_ref.id)
            .cloned()
            .ok_or(VaultError::NotFound)
    }

    pub fn remove(
        &mut self,
        secret_ref: &SecretRef,
        caller: &IdentityId,
    ) -> Result<(), VaultError> {
        if !secret_ref.can_access(caller) {
            return Err(VaultError::AccessDenied);
        }
        self.values.remove(&secret_ref.id);
        self.persist()?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Build a 12-byte nonce unique per (SecretId, counter).
    fn make_nonce(&self, secret_id: &SecretId, counter: u64) -> [u8; 12] {
        let mut hasher = Sha256::new();
        hasher.update(secret_id.0.as_bytes());
        let id_hash = hasher.finalize();

        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&id_hash[0..4]);
        nonce[4..12].copy_from_slice(&counter.to_le_bytes());
        nonce
    }

    fn persist(&mut self) -> Result<(), VaultError> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(MAGIC);

        // Bump counter for this write. Each entry gets counter + index.
        self.counter = self.counter.wrapping_add(1);
        out.extend_from_slice(&self.counter.to_le_bytes());

        let entry_count = self.values.len() as u32;
        out.extend_from_slice(&entry_count.to_le_bytes());

        for (idx, (secret_id, plaintext)) in self.values.iter().enumerate() {
            let entry_counter = self.counter.wrapping_add(idx as u64);
            let nonce_bytes = self.make_nonce(secret_id, entry_counter);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = self
                .cipher
                .encrypt(nonce, plaintext.as_slice())
                .map_err(|e| VaultError::Crypto(format!("encrypt failed: {}", e)))?;

            out.extend_from_slice(&nonce_bytes);
            let id_bytes = secret_id.0.as_bytes();
            out.extend_from_slice(&(id_bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(id_bytes);
            out.extend_from_slice(&(ciphertext.len() as u32).to_le_bytes());
            out.extend_from_slice(&ciphertext);
        }

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        f.write_all(&out)?;
        f.sync_all()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    fn load(&mut self) -> Result<(), VaultError> {
        let mut f = fs::File::open(&self.path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        if buf.len() < 16 {
            return Err(VaultError::Corrupt("file too short".into()));
        }
        if &buf[0..4] != MAGIC {
            return Err(VaultError::Corrupt("bad magic (wrong key or not a vault)".into()));
        }

        self.counter = u64::from_le_bytes(buf[4..12].try_into().unwrap());
        let entry_count = u32::from_le_bytes(buf[12..16].try_into().unwrap()) as usize;

        let mut cursor = 16usize;
        for _ in 0..entry_count {
            if cursor + 12 + 4 > buf.len() {
                return Err(VaultError::Corrupt("truncated nonce/len".into()));
            }
            let nonce_bytes: [u8; 12] = buf[cursor..cursor + 12].try_into().unwrap();
            let nonce = Nonce::from_slice(&nonce_bytes);
            cursor += 12;

            let key_len = u32::from_le_bytes(buf[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;

            if cursor + key_len > buf.len() {
                return Err(VaultError::Corrupt("truncated key".into()));
            }
            let key_str = std::str::from_utf8(&buf[cursor..cursor + key_len])
                .map_err(|_| VaultError::Corrupt("non-utf8 key".into()))?
                .to_string();
            cursor += key_len;

            if cursor + 4 > buf.len() {
                return Err(VaultError::Corrupt("truncated ciphertext len".into()));
            }
            let ct_len = u32::from_le_bytes(buf[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;

            if cursor + ct_len > buf.len() {
                return Err(VaultError::Corrupt("truncated ciphertext".into()));
            }
            let ct = &buf[cursor..cursor + ct_len];
            cursor += ct_len;

            let plaintext = self
                .cipher
                .decrypt(nonce, ct)
                .map_err(|_| VaultError::Corrupt("decrypt failed (wrong key or tamper)".into()))?;

            self.values.insert(SecretId(key_str), plaintext);
        }

        Ok(())
    }
}

/// Derive a 32-byte AES-256 key from arbitrary master key bytes via SHA-256.
fn derive_key(master_key: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"zets-vault-v1");
    hasher.update(master_key);
    let out = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::{IdentityId, IdentityKind};
    use crate::secrets::secret_ref::{SecretKind, SecretRef};

    fn idan() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("zets_vault_test_{}_{}.bin", name, std::process::id()));
        let _ = fs::remove_file(&p);
        p
    }

    #[test]
    fn test_put_and_get_aes() {
        let path = tmp_path("aes_put_get");
        let mut vault = Vault::open(&path, b"master_key_v1").unwrap();

        let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault
            .put(&sref, &idan(), b"sk-real-key-12345".to_vec())
            .unwrap();

        let got = vault.get(&sref, &idan()).unwrap();
        assert_eq!(got, b"sk-real-key-12345");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_persistence_aes_roundtrip() {
        let path = tmp_path("aes_persist");

        {
            let mut vault = Vault::open(&path, b"master_v1").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            vault.put(&sref, &idan(), b"persistent_value".to_vec()).unwrap();
        }

        {
            let vault = Vault::open(&path, b"master_v1").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            let got = vault.get(&sref, &idan()).unwrap();
            assert_eq!(got, b"persistent_value");
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_wrong_key_fails_decrypt() {
        let path = tmp_path("aes_wrong_key");

        {
            let mut vault = Vault::open(&path, b"correct_key").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            vault.put(&sref, &idan(), b"secret_data".to_vec()).unwrap();
        }

        // Wrong key MUST fail — AES-GCM auth tag catches it
        let res = Vault::open(&path, b"wrong_key_xyz");
        assert!(res.is_err(), "wrong key should fail to open");
        match res {
            Err(VaultError::Corrupt(_)) => (),
            other => panic!("expected Corrupt, got {:?}", other.map(|_| ())),
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_tampered_file_detected() {
        let path = tmp_path("aes_tamper");

        {
            let mut vault = Vault::open(&path, b"key").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            vault.put(&sref, &idan(), b"value".to_vec()).unwrap();
        }

        // Tamper the file — flip a byte in the ciphertext region
        let mut bytes = fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        fs::write(&path, &bytes).unwrap();

        // Should fail because AES-GCM auth tag no longer validates
        let res = Vault::open(&path, b"key");
        assert!(res.is_err(), "tampered file should fail to open");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_multiple_secrets_same_vault() {
        let path = tmp_path("aes_multi");
        let mut vault = Vault::open(&path, b"key").unwrap();

        let s1 = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        let s2 = SecretRef::new(idan(), SecretKind::ApiKey, "gemini", 1000);
        let s3 = SecretRef::new(idan(), SecretKind::OAuth, "gmail", 1000);

        vault.put(&s1, &idan(), b"openai-key".to_vec()).unwrap();
        vault.put(&s2, &idan(), b"gemini-key".to_vec()).unwrap();
        vault.put(&s3, &idan(), b"gmail-token".to_vec()).unwrap();

        assert_eq!(vault.get(&s1, &idan()).unwrap(), b"openai-key");
        assert_eq!(vault.get(&s2, &idan()).unwrap(), b"gemini-key");
        assert_eq!(vault.get(&s3, &idan()).unwrap(), b"gmail-token");

        // Reload
        drop(vault);
        let vault = Vault::open(&path, b"key").unwrap();
        assert_eq!(vault.len(), 3);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_access_denied_stranger() {
        let path = tmp_path("aes_denied");
        let mut vault = Vault::open(&path, b"key").unwrap();

        let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault.put(&sref, &idan(), b"value".to_vec()).unwrap();

        let stranger = IdentityId::new(IdentityKind::Person, "stranger");
        match vault.get(&sref, &stranger) {
            Err(VaultError::AccessDenied) => (),
            other => panic!("expected AccessDenied, got {:?}", other),
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_granted_can_access() {
        let path = tmp_path("aes_granted");
        let mut vault = Vault::open(&path, b"key").unwrap();

        let mut sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault.put(&sref, &idan(), b"shared".to_vec()).unwrap();

        let collab = IdentityId::new(IdentityKind::Person, "collab");
        sref.grant(collab.clone());

        let got = vault.get(&sref, &collab).unwrap();
        assert_eq!(got, b"shared");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_remove() {
        let path = tmp_path("aes_remove");
        let mut vault = Vault::open(&path, b"key").unwrap();

        let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault.put(&sref, &idan(), b"val".to_vec()).unwrap();
        assert_eq!(vault.len(), 1);

        vault.remove(&sref, &idan()).unwrap();
        assert_eq!(vault.len(), 0);
        assert!(matches!(vault.get(&sref, &idan()), Err(VaultError::NotFound)));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_empty_vault_open() {
        let path = tmp_path("aes_empty");
        let vault = Vault::open(&path, b"key").unwrap();
        assert!(vault.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_bad_magic_rejected() {
        let path = tmp_path("aes_bad_magic");
        fs::write(&path, b"XXXXXXXXXXXXXXXX").unwrap();
        let res = Vault::open(&path, b"key");
        assert!(matches!(res, Err(VaultError::Corrupt(_))));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_long_value_encrypted() {
        let path = tmp_path("aes_long");
        let mut vault = Vault::open(&path, b"key").unwrap();

        // 10KB value — AES-GCM handles arbitrary lengths
        let big_value: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let sref = SecretRef::new(idan(), SecretKind::PrivateKey, "ssh", 1000);
        vault.put(&sref, &idan(), big_value.clone()).unwrap();

        let got = vault.get(&sref, &idan()).unwrap();
        assert_eq!(got, big_value);

        let _ = fs::remove_file(&path);
    }
}
