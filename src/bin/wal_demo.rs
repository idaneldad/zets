//! `zets-wal-demo` — end-to-end WAL demonstration.
//!
//! 1. Append several learned updates to a WAL
//! 2. Reopen, replay, verify checksum
//! 3. Print the effective state
//!
//! Proves WAL works independently of the pack format.

use std::path::PathBuf;
use std::time::Instant;

use zets::wal::{wal_path_for_core, Record, RecordKind, WalReader, WalWriter};

fn main() -> std::io::Result<()> {
    let packs_dir = PathBuf::from("data/packs");
    std::fs::create_dir_all(&packs_dir)?;
    let path = wal_path_for_core(&packs_dir);
    println!("═══ ZETS WAL Demo ═══");
    println!("Path: {:?}", path);
    println!();

    // Fresh start
    let _ = std::fs::remove_file(&path);

    // Write 10,000 records
    let t = Instant::now();
    {
        let mut w = WalWriter::open(&path)?;
        for i in 0..10_000u32 {
            w.append(&Record::learn_pos(
                (i % 16) as u8,
                i,
                ((i % 4) + 1) as u8,
            ))?;
        }
        w.append(&Record::learn_order(0, 1, 98))?; // en=adj_first
        w.append(&Record::learn_order(4, 2, 95))?; // he=noun_first
        w.append(&Record::delete_pos(0, 42))?; // retract
        w.sync()?;
    }
    let write_ms = t.elapsed();

    let size = std::fs::metadata(&path)?.len();
    println!("Wrote 10,003 records in {:?}", write_ms);
    println!("WAL file size: {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
    println!("Avg bytes/record: {:.1}", size as f64 / 10_003.0);
    println!();

    // Replay
    let t = Instant::now();
    let mut r = WalReader::open(&path)?;
    let records = r.read_all()?;
    let read_ms = t.elapsed();
    println!("Replayed {} records in {:?}", records.len(), read_ms);

    // Summarize
    let mut by_kind = std::collections::HashMap::new();
    for rec in &records {
        *by_kind.entry(rec.kind).or_insert(0usize) += 1;
    }
    for (kind, count) in &by_kind {
        let name = match kind {
            RecordKind::LearnPos => "LearnPos",
            RecordKind::LearnOrder => "LearnOrder",
            RecordKind::DeletePos => "DeletePos",
            RecordKind::UserNote => "UserNote",
            RecordKind::Unknown => "Unknown",
        };
        println!("  {}: {}", name, count);
    }
    println!();

    // Show the two LearnOrder entries
    for rec in &records {
        if rec.kind == RecordKind::LearnOrder {
            if let Some((lang, rule, conf)) = rec.as_learn_order() {
                let rule_name = match rule {
                    1 => "adj_first",
                    2 => "noun_first",
                    _ => "undetermined",
                };
                println!(
                    "  word-order rule: lang_id={} rule={} confidence={}",
                    lang, rule_name, conf
                );
            }
        }
    }

    println!();
    println!("DONE. The WAL approach allows learned updates to persist");
    println!("without rewriting the main pack file.");
    println!();
    println!("To clean: rm {:?}", path);

    Ok(())
}
