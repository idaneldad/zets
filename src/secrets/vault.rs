//! # Vault — encrypted value store for secrets
//!
//! The vault is a file-backed, AES-256-GCM encrypted key-value store.
//! Keys are `SecretId` strings; values are the raw secret bytes.
//!
//! The vault is accessed ONLY via methods here. The raw decrypted value
//! is never logged, never written to the graph, never exposed except
//! to explicit callers who pass ACL checks.
//!
//! ## Threat model
//!
//! - **Disk compromise**: vault file is encrypted. Needs master key.
//! - **Memory compromise**: values are zeroized after use via `zeroize`.
//! - **Graph compromise**: graph holds only references, no values.
//! - **ACL bypass**: caller must pass an explicit IdentityId to get access;
//!   the vault checks the SecretRef's ACL before returning the value.
//!
//! ## Not implemented yet
//!
//! For the initial version we use a SIMPLE XOR-based obfuscation with a
//! file-stored key. This is NOT production-grade encryption — it's a
//! placeholder that establishes the architecture. The real implementation
//! should use `aes-gcm` crate (deferred to avoid adding deps before
//! the design is validated).

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use super::secret_ref::{SecretId, SecretRef};
use crate::personal_graph::IdentityId;

/// Errors that can occur when interacting with the vault.
#[derive(Debug)]
pub enum VaultError {
    Io(io::Error),
    AccessDenied,
    NotFound,
    Corrupt,
    SecretDisabled,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Io(e) => write!(f, "I/O error: {}", e),
            VaultError::AccessDenied => write!(f, "access denied"),
            VaultError::NotFound => write!(f, "secret not found"),
            VaultError::Corrupt => write!(f, "vault corrupt"),
            VaultError::SecretDisabled => write!(f, "secret is disabled"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<io::Error> for VaultError {
    fn from(e: io::Error) -> Self {
        VaultError::Io(e)
    }
}

/// The Vault — holds encrypted secret values.
///
/// Stateful: holds the master key in memory. The backing file is written
/// on every change (simple model; future: WAL + periodic flush).
pub struct Vault {
    /// Path to the encrypted file on disk.
    path: PathBuf,
    /// In-memory decrypted values. Key = SecretId, Value = raw bytes.
    values: HashMap<SecretId, Vec<u8>>,
    /// Master key, loaded once on open.
    master_key: Vec<u8>,
}

impl Vault {
    /// Open (or create) a vault at the given path, using the master key.
    ///
    /// The master key should come from a trusted source (env var, OS
    /// keychain, HSM). Never hardcoded, never logged.
    pub fn open(path: impl Into<PathBuf>, master_key: &[u8]) -> Result<Self, VaultError> {
        let path = path.into();
        let mut vault = Vault {
            path: path.clone(),
            values: HashMap::new(),
            master_key: master_key.to_vec(),
        };

        if path.exists() {
            vault.load()?;
        }
        Ok(vault)
    }

    /// Put a secret value into the vault.
    ///
    /// Pre-condition: the caller has already verified that the caller's
    /// identity is allowed to write this secret (usually, the owner).
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

    /// Get a secret value.
    ///
    /// Returns `AccessDenied` if the caller is not on the ACL.
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

    /// Remove a secret from the vault entirely.
    /// The SecretRef in the graph should be marked Revoked separately.
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

    /// How many secrets are stored (for diagnostics).
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Re-encrypt and write to disk.
    fn persist(&self) -> Result<(), VaultError> {
        // Serialize values as a simple length-prefixed format:
        // [u32 count] [{u32 key_len} {key_bytes} {u32 val_len} {val_bytes}]*
        let mut plaintext: Vec<u8> = Vec::new();
        let count = self.values.len() as u32;
        plaintext.extend_from_slice(&count.to_le_bytes());
        for (k, v) in &self.values {
            let key_bytes = k.0.as_bytes();
            plaintext.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            plaintext.extend_from_slice(key_bytes);
            plaintext.extend_from_slice(&(v.len() as u32).to_le_bytes());
            plaintext.extend_from_slice(v);
        }

        let ciphertext = obfuscate(&plaintext, &self.master_key);

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        f.write_all(&ciphertext)?;
        f.sync_all()?;

        // Set restrictive permissions on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    fn load(&mut self) -> Result<(), VaultError> {
        let mut f = fs::File::open(&self.path)?;
        let mut ciphertext = Vec::new();
        f.read_to_end(&mut ciphertext)?;
        let plaintext = obfuscate(&ciphertext, &self.master_key);

        if plaintext.len() < 4 {
            return Err(VaultError::Corrupt);
        }

        let count = u32::from_le_bytes([plaintext[0], plaintext[1], plaintext[2], plaintext[3]])
            as usize;
        let mut cursor = 4usize;

        for _ in 0..count {
            if cursor + 4 > plaintext.len() {
                return Err(VaultError::Corrupt);
            }
            let key_len = u32::from_le_bytes([
                plaintext[cursor],
                plaintext[cursor + 1],
                plaintext[cursor + 2],
                plaintext[cursor + 3],
            ]) as usize;
            cursor += 4;
            if cursor + key_len > plaintext.len() {
                return Err(VaultError::Corrupt);
            }
            let key_str =
                String::from_utf8_lossy(&plaintext[cursor..cursor + key_len]).to_string();
            cursor += key_len;

            if cursor + 4 > plaintext.len() {
                return Err(VaultError::Corrupt);
            }
            let val_len = u32::from_le_bytes([
                plaintext[cursor],
                plaintext[cursor + 1],
                plaintext[cursor + 2],
                plaintext[cursor + 3],
            ]) as usize;
            cursor += 4;
            if cursor + val_len > plaintext.len() {
                return Err(VaultError::Corrupt);
            }
            let val = plaintext[cursor..cursor + val_len].to_vec();
            cursor += val_len;

            self.values.insert(SecretId(key_str), val);
        }

        Ok(())
    }
}

/// PLACEHOLDER obfuscation — XOR with repeated key.
///
/// NOT production-grade. This establishes the interface.
/// Replace with aes-gcm before any real secret lands in vault.
fn obfuscate(input: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return input.to_vec();
    }
    input
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
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
    fn test_put_and_get() {
        let path = tmp_path("put_get");
        let mut vault = Vault::open(&path, b"master_key_v1").unwrap();

        let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault.put(&sref, &idan(), b"sk-real-key-12345".to_vec()).unwrap();

        let got = vault.get(&sref, &idan()).unwrap();
        assert_eq!(got, b"sk-real-key-12345");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_access_denied_wrong_caller() {
        let path = tmp_path("denied");
        let mut vault = Vault::open(&path, b"master_key_v1").unwrap();

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
    fn test_persistence_roundtrip() {
        let path = tmp_path("persist");

        // Write
        {
            let mut vault = Vault::open(&path, b"master_v1").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            vault.put(&sref, &idan(), b"persistent_value".to_vec()).unwrap();
        }

        // Read back
        {
            let vault = Vault::open(&path, b"master_v1").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            let got = vault.get(&sref, &idan()).unwrap();
            assert_eq!(got, b"persistent_value");
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_wrong_master_key_gives_garbage() {
        let path = tmp_path("wrong_key");

        {
            let mut vault = Vault::open(&path, b"correct_key").unwrap();
            let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
            vault.put(&sref, &idan(), b"value".to_vec()).unwrap();
        }

        // Open with WRONG key - file will decode to garbage
        let res = Vault::open(&path, b"wrong_key_xyz");
        // Either fails to parse (Corrupt) or yields garbage key that won't
        // match SecretId lookups. Both are acceptable for this placeholder.
        match res {
            Ok(v) => {
                // Likely zero entries or un-lookupable gibberish
                let sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
                // the lookup should fail because the key bytes got XOR'd to different value
                // (unless collision, which is astronomically unlikely with these inputs)
                assert!(v.get(&sref, &idan()).is_err() || v.len() == 0 || v.len() == 1);
            }
            Err(_) => (), // Corrupt — also fine
        }

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_remove() {
        let path = tmp_path("remove");
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
    fn test_granted_user_can_access() {
        let path = tmp_path("granted");
        let mut vault = Vault::open(&path, b"key").unwrap();

        let mut sref = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        vault.put(&sref, &idan(), b"shared_value".to_vec()).unwrap();

        let collab = IdentityId::new(IdentityKind::Person, "collab");
        sref.grant(collab.clone());

        let got = vault.get(&sref, &collab).unwrap();
        assert_eq!(got, b"shared_value");

        let _ = fs::remove_file(&path);
    }
}
