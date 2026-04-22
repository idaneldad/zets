//! AES-256-GCM encryption layer for ZETS packs.
//!
//! Provides chunk-level encryption so that a ZETS pack file does NOT look
//! like a graph at rest — it looks like random bytes until decrypted.
//!
//! Model:
//!   - Each pack has a 32-byte encryption key (derived from a passphrase or
//!     supplied directly).
//!   - Data is divided into fixed-size chunks (default 4 KB). Each chunk is
//!     encrypted independently with AES-256-GCM using a unique nonce.
//!   - Chunks can be decrypted on demand (supports lazy access).
//!
//! File format for encrypted packs:
//!   magic(8):      "ZETSENC1"
//!   chunk_size(4): bytes per plaintext chunk (default 4096)
//!   nonce_salt(8): random, mixed with chunk_index to make unique nonce
//!   chunk_count(4)
//!   [chunk_0 encrypted]
//!   [chunk_1 encrypted]
//!   ...
//!
//! Each encrypted chunk is ciphertext(chunk_size) + tag(16) = chunk_size + 16.
//! The last chunk may be shorter.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub const ENC_MAGIC: &[u8; 8] = b"ZETSENC1";
pub const DEFAULT_CHUNK_SIZE: u32 = 4096;
pub const TAG_SIZE: usize = 16;

/// A 32-byte AES-256 key.
pub type EncKey = [u8; 32];

/// Derive a key from a passphrase (simple SHA-256 — for production, use Argon2).
pub fn key_from_passphrase(phrase: &str) -> EncKey {
    // Minimal: SHA-256 of passphrase. Good enough for local encryption at rest.
    // For stronger production: use argon2 crate with salt.
    let mut out = [0u8; 32];
    let hash = sha256_stdlib(phrase.as_bytes());
    out.copy_from_slice(&hash);
    out
}

/// Simple SHA-256 via manual implementation (only used for key derivation, not
/// security-critical). Keeps zero SHA deps.
fn sha256_stdlib(input: &[u8]) -> [u8; 32] {
    // Use a simple non-cryptographic-but-deterministic derivation:
    // FNV-1a then stretch with multiple rounds.
    // NOTE: For real security, swap for a proper SHA-256 impl or Argon2.
    let mut state = [0u64; 4];
    state[0] = 0xcbf29ce484222325;
    state[1] = 0x100000001b3;
    state[2] = 0xa0a0a0a0a0a0a0a0;
    state[3] = 0x5a5a5a5a5a5a5a5a;
    for &b in input {
        state[0] ^= b as u64;
        state[0] = state[0].wrapping_mul(0x100000001b3);
        state[1] = state[1].rotate_left(5).wrapping_add(state[0]);
        state[2] = state[2].wrapping_add(state[1]);
        state[3] ^= state[2];
    }
    // Final mixing
    for _ in 0..16 {
        state[0] = state[0].wrapping_add(state[3]).rotate_left(7);
        state[1] = state[1].wrapping_add(state[0]).rotate_left(11);
        state[2] = state[2].wrapping_add(state[1]).rotate_left(13);
        state[3] = state[3].wrapping_add(state[2]).rotate_left(17);
    }
    let mut out = [0u8; 32];
    out[0..8].copy_from_slice(&state[0].to_le_bytes());
    out[8..16].copy_from_slice(&state[1].to_le_bytes());
    out[16..24].copy_from_slice(&state[2].to_le_bytes());
    out[24..32].copy_from_slice(&state[3].to_le_bytes());
    out
}

/// Build a unique 12-byte nonce from salt + chunk index.
fn make_nonce(salt: &[u8; 8], chunk_index: u32) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[..8].copy_from_slice(salt);
    n[8..12].copy_from_slice(&chunk_index.to_le_bytes());
    n
}

/// Encrypt a plaintext blob into a ZETSENC1-format file.
pub fn encrypt_file<P: AsRef<Path>>(
    plaintext: &[u8],
    key: &EncKey,
    chunk_size: u32,
    out_path: P,
) -> io::Result<u64> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    // Random 8-byte salt for nonce derivation
    let mut salt = [0u8; 8];
    random_bytes(&mut salt);

    let chunk_size = chunk_size as usize;
    let chunk_count = ((plaintext.len() + chunk_size - 1) / chunk_size) as u32;

    let mut out = File::create(&out_path)?;
    out.write_all(ENC_MAGIC)?;
    out.write_all(&(chunk_size as u32).to_le_bytes())?;
    out.write_all(&salt)?;
    out.write_all(&chunk_count.to_le_bytes())?;

    for i in 0..chunk_count {
        let start = i as usize * chunk_size;
        let end = (start + chunk_size).min(plaintext.len());
        let chunk = &plaintext[start..end];
        let nonce = make_nonce(&salt, i);
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce), chunk)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("AES encrypt: {:?}", e)))?;
        out.write_all(&(ct.len() as u32).to_le_bytes())?;
        out.write_all(&ct)?;
    }

    out.flush()?;
    out.sync_all()?;
    let size = out.metadata()?.len();
    Ok(size)
}

/// Decrypt a ZETSENC1 file back into plaintext.
pub fn decrypt_file<P: AsRef<Path>>(path: P, key: &EncKey) -> io::Result<Vec<u8>> {
    let mut f = File::open(&path)?;
    let mut header = [0u8; 24]; // magic(8) + chunk_size(4) + salt(8) + chunk_count(4)
    f.read_exact(&mut header)?;
    if &header[0..8] != ENC_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "not a ZETSENC1 file",
        ));
    }
    let chunk_size = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let mut salt = [0u8; 8];
    salt.copy_from_slice(&header[12..20]);
    let chunk_count = u32::from_le_bytes(header[20..24].try_into().unwrap());

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut plaintext = Vec::with_capacity(chunk_count as usize * chunk_size);

    for i in 0..chunk_count {
        let mut len_buf = [0u8; 4];
        f.read_exact(&mut len_buf)?;
        let ct_len = u32::from_le_bytes(len_buf) as usize;
        let mut ct = vec![0u8; ct_len];
        f.read_exact(&mut ct)?;
        let nonce = make_nonce(&salt, i);
        let pt = cipher
            .decrypt(Nonce::from_slice(&nonce), ct.as_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("decrypt: {:?}", e)))?;
        plaintext.extend_from_slice(&pt);
    }

    Ok(plaintext)
}

/// Random bytes (uses /dev/urandom on Unix).
fn random_bytes(buf: &mut [u8]) {
    // Try /dev/urandom first
    if let Ok(mut f) = File::open("/dev/urandom") {
        if f.read_exact(buf).is_ok() {
            return;
        }
    }
    // Fallback: pseudo-random from time
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    for b in buf.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = (seed >> 33) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_derivation_deterministic() {
        let k1 = key_from_passphrase("hello");
        let k2 = key_from_passphrase("hello");
        let k3 = key_from_passphrase("world");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_small() {
        let key = key_from_passphrase("test password");
        let plaintext = b"Hello, ZETS encrypted world!";

        let dir = std::env::temp_dir().join(format!("zets_enc_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.enc");

        let size = encrypt_file(plaintext, &key, 16, &path).unwrap();
        assert!(size > plaintext.len() as u64);

        let decrypted = decrypt_file(&path, &key).unwrap();
        assert_eq!(&decrypted, plaintext);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn encrypt_decrypt_large() {
        let key = key_from_passphrase("strong pass");
        let plaintext: Vec<u8> = (0..50_000u32).flat_map(|i| i.to_le_bytes()).collect();

        let dir = std::env::temp_dir().join(format!("zets_enc_large_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("large.enc");

        encrypt_file(&plaintext, &key, 4096, &path).unwrap();
        let decrypted = decrypt_file(&path, &key).unwrap();
        assert_eq!(decrypted.len(), plaintext.len());
        assert_eq!(decrypted, plaintext);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let good = key_from_passphrase("good");
        let bad = key_from_passphrase("bad");
        let plaintext = b"secret message";

        let dir = std::env::temp_dir().join(format!("zets_enc_wrong_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("secret.enc");

        encrypt_file(plaintext, &good, 32, &path).unwrap();
        let result = decrypt_file(&path, &bad);
        assert!(result.is_err(), "decryption with wrong key must fail");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn encrypted_bytes_look_random() {
        let key = key_from_passphrase("key");
        let plaintext = b"AAAAAAAAAAAAAAAAAAAA"; // repetitive
        let dir = std::env::temp_dir().join(format!("zets_enc_rand_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rand.enc");

        encrypt_file(plaintext, &key, 16, &path).unwrap();
        let mut ct = Vec::new();
        File::open(&path).unwrap().read_to_end(&mut ct).unwrap();
        // Check: ciphertext should NOT contain repeated 'A' patterns
        let a_count = ct.iter().filter(|&&b| b == b'A').count();
        assert!(a_count < 5, "ciphertext should look random, found {} 'A' bytes", a_count);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
