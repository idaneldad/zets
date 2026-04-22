#!/bin/bash
# Fetch Wikipedia articles via API, emit JSONL to stdout.
#
# Usage:  scripts/fetch_wikipedia.sh topics.txt > articles.jsonl
#         (or via stdin)
#         cat topics.txt | scripts/fetch_wikipedia.sh /dev/stdin
#
# Rate-limited to ~8 req/sec to respect Wikipedia's polite-bot policy.
# Resumable: if interrupted, just re-run with a topics file that has
# the remaining topics.
#
# Output format (JSONL, one article per line):
#   {"title": "Paris", "text": "Paris is the capital..."}
#
# Designed to pipe into: target/release/stream-ingest

set -euo pipefail

INPUT="${1:-/dev/stdin}"
UA="ZETS-bot/1.0 (idan@chooz.co.il) educational AI research"
RATE_DELAY=0.12  # ~8 req/sec

if [ ! -e "$INPUT" ] && [ "$INPUT" != "/dev/stdin" ]; then
    echo "Usage: $0 <topics_file>" >&2
    echo "       cat topics.txt | $0 /dev/stdin" >&2
    exit 1
fi

count=0
failed=0
while IFS= read -r topic || [ -n "$topic" ]; do
    # Skip empty and comment lines
    [ -z "$topic" ] && continue
    [[ "$topic" =~ ^# ]] && continue

    # URL-encode spaces to underscores (Wikipedia convention)
    encoded="${topic// /_}"

    # Fetch and parse
    response=$(curl -sS --max-time 15 \
        "https://en.wikipedia.org/w/api.php?action=query&format=json&titles=${encoded}&prop=extracts&explaintext=true&exsectionformat=plain&redirects=1" \
        -A "$UA" 2>/dev/null) || {
        echo "[FAIL] $topic" >&2
        failed=$((failed + 1))
        sleep "$RATE_DELAY"
        continue
    }

    # Extract via python (already required on server)
    echo "$response" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    pages = d.get('query', {}).get('pages', {})
    for pid, p in pages.items():
        if pid == '-1':
            sys.stderr.write('[MISS] missing\n')
            sys.exit(1)
        text = p.get('extract', '')
        if len(text) < 100:
            sys.stderr.write('[THIN] ' + p.get('title', '') + '\n')
            sys.exit(1)
        # Emit a single JSONL line with ensure_ascii to keep lines compact
        print(json.dumps({'title': p.get('title', ''), 'text': text}, ensure_ascii=False))
        sys.exit(0)
    sys.exit(1)
except Exception as e:
    sys.stderr.write('[ERR] ' + str(e) + '\n')
    sys.exit(1)
" && count=$((count + 1)) || failed=$((failed + 1))

    sleep "$RATE_DELAY"

    # Progress to stderr (doesn't pollute the JSONL stream)
    if (( count > 0 && count % 25 == 0 )); then
        echo "[$count ok / $failed fail] $topic" >&2
    fi
done < "$INPUT"

echo "Done. Fetched $count articles, $failed failures." >&2
