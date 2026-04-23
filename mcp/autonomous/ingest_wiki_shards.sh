#!/usr/bin/env bash
# ingest_wiki_shards.sh — the glue between multi_lang_wiki.py (download+parse)
#                         and stream_ingest (merge-to-graph), with delete-after-ingest.
#
# This is the SHORT-TERM P-R bridge (Python MVP path), living at mcp/autonomous/.
# The full Rust rewrite per MISSION_P_R_WIKIPEDIA_SCALE.md is the longer arc.
#
# What it does, per shard (<lang>_parsed.jsonl.gz):
#   1. Check manifest — skip if already ingested.
#   2. Pipe decompressed JSONL into stream_ingest with unique snapshot name.
#   3. On success (exit 0 + snapshot created):
#        a. Append to manifest with timestamp + stats.
#        b. DELETE the .jsonl.gz (source).
#        c. Optionally delete intermediate snapshots older than N days.
#   4. On failure:
#        - Leave .jsonl.gz in place.
#        - Append failure entry to manifest.
#        - Continue with next shard.
#
# Idempotent: safe to re-run.
# Budget: refuses to run if /home/dinio/zets disk usage > 90%.
#
# Usage:
#   ./ingest_wiki_shards.sh                    # process all pending shards
#   ./ingest_wiki_shards.sh --lang he,en,ar    # only specific languages
#   ./ingest_wiki_shards.sh --dry-run          # show what would happen, do nothing
#
# Runs forever if you want: loop it in systemd (see zets-ingest.service).

set -euo pipefail

REPO="/home/dinio/zets"
DUMPS="$REPO/data/wikipedia_dumps"
MANIFEST="$REPO/data/autonomous/ingest_manifest.jsonl"
LOG_DIR="$REPO/logs/ingest"
BIN="$REPO/target/release/stream_ingest"

mkdir -p "$LOG_DIR" "$(dirname "$MANIFEST")"
touch "$MANIFEST"

# -------- parse args --------
LANGS=""
DRY_RUN=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --lang) LANGS="$2"; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        *) echo "Unknown arg: $1" >&2; exit 2 ;;
    esac
done

# -------- sanity checks --------
if [[ ! -x "$BIN" ]]; then
    echo "ERROR: $BIN not found. Build it first:" >&2
    echo "  cd $REPO && cargo build --release --bin stream_ingest" >&2
    exit 3
fi

# Disk budget
USED_PCT=$(df "$REPO" | awk 'NR==2 {gsub(/%/,"",$5); print $5}')
if [[ "$USED_PCT" -gt 90 ]]; then
    echo "ERROR: disk usage ${USED_PCT}% > 90% threshold. Pausing." >&2
    exit 4
fi

# -------- helpers --------
manifest_has() {
    local shard_name="$1"
    grep -q "\"shard\":\"$shard_name\",\"status\":\"ingested\"" "$MANIFEST" 2>/dev/null
}

manifest_append() {
    local shard_name="$1"
    local status="$2"     # "ingested" | "failed"
    local lines_in="${3:-0}"
    local elapsed_s="${4:-0}"
    local err="${5:-}"
    local ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    printf '{"shard":"%s","status":"%s","lines_in":%d,"elapsed_s":%d,"ts":"%s","err":"%s"}\n' \
        "$shard_name" "$status" "$lines_in" "$elapsed_s" "$ts" "$err" >> "$MANIFEST"
}

is_lang_enabled() {
    local lang="$1"
    [[ -z "$LANGS" ]] && return 0
    [[ ",$LANGS," == *",$lang,"* ]]
}

# -------- main loop --------
SHARDS=$(ls -1 "$DUMPS"/*_parsed.jsonl.gz 2>/dev/null || true)
if [[ -z "$SHARDS" ]]; then
    echo "No *_parsed.jsonl.gz shards found in $DUMPS. Nothing to do."
    exit 0
fi

TOTAL=0
SUCCEEDED=0
FAILED=0
SKIPPED=0
DELETED_BYTES=0

for shard_path in $SHARDS; do
    TOTAL=$((TOTAL+1))
    shard_file=$(basename "$shard_path")
    lang=$(echo "$shard_file" | sed -n 's/^\([a-z][a-z]\)_parsed.jsonl.gz$/\1/p')

    if ! is_lang_enabled "$lang"; then
        SKIPPED=$((SKIPPED+1))
        continue
    fi

    if manifest_has "$shard_file"; then
        echo "[skip] $shard_file already ingested per manifest"
        SKIPPED=$((SKIPPED+1))
        if [[ "$DRY_RUN" -eq 0 && -f "$shard_path" ]]; then
            size=$(stat -c%s "$shard_path")
            rm -f "$shard_path"
            DELETED_BYTES=$((DELETED_BYTES+size))
            echo "[rm]   $shard_file (already-ingested leftover, freed $size bytes)"
        fi
        continue
    fi

    echo "[run ] $shard_file (lang=$lang)"
    if [[ "$DRY_RUN" -eq 1 ]]; then
        echo "       (dry-run: would ingest + delete)"
        continue
    fi

    t_start=$(date +%s)
    snapshot_name="wiki_${lang}_$(date +%Y%m%d_%H%M%S)"
    log_file="$LOG_DIR/${snapshot_name}.log"

    # Pipe gunzipped JSONL into stream_ingest.
    # --checkpoint-every 5000 → snapshot persists every 5k articles.
    # --source labels each atom with its origin for provenance.
    if gunzip -c "$shard_path" \
        | "$BIN" \
            --name "$snapshot_name" \
            --base v1_bootstrap \
            --source "wikipedia:$lang" \
            --checkpoint-every 5000 \
            >> "$log_file" 2>&1 ; then

        t_end=$(date +%s)
        elapsed=$((t_end - t_start))
        lines_in=$(grep -c '^Articles:' "$log_file" 2>/dev/null || echo 0)

        manifest_append "$shard_file" "ingested" "$lines_in" "$elapsed"

        # DELETE the source
        size=$(stat -c%s "$shard_path")
        rm -f "$shard_path"
        DELETED_BYTES=$((DELETED_BYTES+size))
        SUCCEEDED=$((SUCCEEDED+1))
        echo "[done] $shard_file → snapshot=$snapshot_name, elapsed=${elapsed}s, freed=${size}B"
    else
        err=$(tail -1 "$log_file" | tr -d '"' | head -c 200)
        manifest_append "$shard_file" "failed" 0 0 "$err"
        FAILED=$((FAILED+1))
        echo "[fail] $shard_file — see $log_file"
    fi

    # Re-check disk after each shard
    USED_PCT=$(df "$REPO" | awk 'NR==2 {gsub(/%/,"",$5); print $5}')
    if [[ "$USED_PCT" -gt 90 ]]; then
        echo "[pause] disk usage ${USED_PCT}% exceeded threshold mid-run. Stopping gracefully."
        break
    fi
done

echo ""
echo "=== SUMMARY ==="
echo "total shards seen:    $TOTAL"
echo "ingested + deleted:   $SUCCEEDED"
echo "failed:               $FAILED"
echo "skipped:              $SKIPPED"
echo "bytes freed:          $DELETED_BYTES"
echo "manifest:             $MANIFEST"
echo ""
echo "Disk after run:"
df -h "$REPO" | tail -1
