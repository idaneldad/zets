//! `zets-crypto-demo` — encrypt + decrypt full ZETS core pack.
//!
//! End-to-end test:
//!   1. Read data/packs/zets.core
//!   2. Encrypt it with AES-256-GCM → data/packs/zets.core.enc
//!   3. Verify encrypted bytes don't look like the original
//!   4. Decrypt and verify roundtrip

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use zets::crypto::{decrypt_file, encrypt_file, key_from_passphrase, DEFAULT_CHUNK_SIZE};

fn main() -> std::io::Result<()> {
    let packs_dir = PathBuf::from("data/packs");
    let core_path = packs_dir.join("zets.core");
    let enc_path = packs_dir.join("zets.core.enc");
    let dec_path = packs_dir.join("zets.core.dec");

    if !core_path.exists() {
        eprintln!("zets.core not found; run `pack-write` first.");
        return Ok(());
    }

    println!("═══ ZETS Crypto Demo ═══");
    println!();

    let key = key_from_passphrase("demo-passphrase-change-me-in-production");

    // Read original
    let original = fs::read(&core_path)?;
    let orig_size = original.len();
    println!(
        "Original zets.core: {} bytes ({:.1} MB)",
        orig_size,
        orig_size as f64 / 1_048_576.0
    );

    // First few bytes should be "ZETS" magic
    println!("  magic bytes     : {:?}", &original[0..4]);
    println!();

    // Encrypt
    let t = Instant::now();
    let enc_size = encrypt_file(&original, &key, DEFAULT_CHUNK_SIZE, &enc_path)?;
    println!(
        "Encrypt time     : {:?}  ({:.1} MB/s)",
        t.elapsed(),
        orig_size as f64 / 1_048_576.0 / t.elapsed().as_secs_f64()
    );
    println!(
        "Encrypted size   : {} bytes ({:.1} MB) — overhead {:.2}%",
        enc_size,
        enc_size as f64 / 1_048_576.0,
        (enc_size as f64 - orig_size as f64) / orig_size as f64 * 100.0
    );

    // Check encrypted bytes don't reveal the graph
    let enc = fs::read(&enc_path)?;
    println!("  enc magic bytes : {:?}", &enc[0..8]);
    // Look for any occurrence of "ZETS" in the encrypted body (after header)
    let zets_count = enc[24..]
        .windows(4)
        .filter(|w| *w == b"ZETS")
        .count();
    println!("  'ZETS' occurrences in encrypted body: {}", zets_count);
    println!("  (should be ~0 — cipher makes bytes random)");
    println!();

    // Decrypt
    let t = Instant::now();
    let decrypted = decrypt_file(&enc_path, &key)?;
    println!(
        "Decrypt time     : {:?}  ({:.1} MB/s)",
        t.elapsed(),
        orig_size as f64 / 1_048_576.0 / t.elapsed().as_secs_f64()
    );

    // Verify roundtrip
    assert_eq!(decrypted.len(), original.len());
    assert_eq!(decrypted, original);
    println!("Roundtrip verified: {} bytes match exactly", decrypted.len());

    fs::write(&dec_path, &decrypted)?;
    println!();
    println!("Files:");
    println!("  {:?}  ({} bytes)", core_path, orig_size);
    println!("  {:?}  ({} bytes)", enc_path, enc_size);
    println!("  {:?}  ({} bytes)", dec_path, decrypted.len());
    println!();
    println!("DONE.");
    println!();
    println!("Cleanup: rm data/packs/zets.core.enc data/packs/zets.core.dec");

    Ok(())
}
