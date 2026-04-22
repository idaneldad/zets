#!/usr/bin/env python3
"""
multi_lang_wiki.py — download + stream-parse Wikipedia dumps for many languages.

Priority: SMALL languages first (fast wins), expand as disk allows.
Polite: max 2 concurrent downloads. Streams parse during download wait.
Safe: STOP file, disk monitoring, max-articles cap per language.

Output per language:
  data/wikipedia_dumps/<lang>-latest-pages-articles.xml.bz2  (raw dump)
  data/wikipedia_dumps/<lang>_parsed.jsonl.gz                (parsed articles, gzipped)
  data/wikipedia_dumps/<lang>_progress.json                   (resume state)

After parse completes, the raw .bz2 is DELETED to save disk.

Usage:
  python3 multi_lang_wiki.py            # run to completion (or STOP)
  touch STOP                             # graceful stop
"""

import bz2
import gzip
import json
import os
import re
import shutil
import signal
import sys
import time
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime
from pathlib import Path
from queue import Queue
from threading import Thread, Lock

HERE = Path(os.path.dirname(os.path.abspath(__file__)))
STOP_FILE = HERE / 'STOP'
DUMP_DIR = Path('/home/dinio/zets/data/wikipedia_dumps')
DUMP_DIR.mkdir(parents=True, exist_ok=True)
LOG_DIR = HERE / 'logs' / 'multilang'
LOG_DIR.mkdir(parents=True, exist_ok=True)
STATUS_FILE = HERE / 'multilang_status.json'

USER_AGENT = 'ZETS-Learner/0.1 (+https://github.com/idaneldad/zets; contact: idan@chooz.co.il)'

# Safety limits
MIN_DISK_GB = 15.0
MAX_ARTICLES_PER_LANG = 500_000
MAX_PARALLEL_DOWNLOADS = 2
ARTICLE_TEXT_CAP = 20_000   # chars per article (truncate long ones)

# ═══════════════════════════════════════════════════════════════════════
# Language catalog — approx sizes as of 2025/2026
# (lang_code, est_mb_compressed, est_articles, display_name)
# ═══════════════════════════════════════════════════════════════════════

LANGUAGES = [
    # Tier 1: TINY (< 100 MB) — quick wins, diverse
    ('sw',   60,    80_000, 'Swahili'),
    ('yi',   40,    15_000, 'Yiddish'),
    ('la',  120,   170_000, 'Latin'),
    ('eo',  200,   340_000, 'Esperanto'),
    ('gl',  200,   200_000, 'Galician'),
    ('eu',  200,   430_000, 'Basque'),
    ('cy',  150,   280_000, 'Welsh'),
    ('hy',  250,   300_000, 'Armenian'),
    ('hi',  400,   160_000, 'Hindi'),
    ('bn',  300,   130_000, 'Bengali'),
    ('ta',  350,   160_000, 'Tamil'),
    ('ur',  200,   190_000, 'Urdu'),
    ('el',  350,   230_000, 'Greek'),
    ('th',  300,   170_000, 'Thai'),
    ('bg',  400,   300_000, 'Bulgarian'),
    ('sk',  400,   250_000, 'Slovak'),
    ('sl',  300,   200_000, 'Slovenian'),
    ('hr',  400,   200_000, 'Croatian'),
    ('sr',  500,   670_000, 'Serbian'),
    ('lt',  300,   210_000, 'Lithuanian'),
    ('lv',  250,   120_000, 'Latvian'),
    ('et',  250,   240_000, 'Estonian'),

    # Tier 2: SMALL (300-600 MB)
    ('da',  400,   300_000, 'Danish'),
    ('no',  550,   620_000, 'Norwegian'),
    ('fi',  700,   580_000, 'Finnish'),
    ('hu',  600,   530_000, 'Hungarian'),
    ('ro',  550,   470_000, 'Romanian'),
    ('cs',  700,   550_000, 'Czech'),
    ('id',  400,   680_000, 'Indonesian'),
    ('fa',  400,   980_000, 'Persian'),
    ('vi',  600, 1_300_000, 'Vietnamese'),

    # Tier 3: MEDIUM (600MB-1.5GB) — strategic languages
    ('he',  800,   360_000, 'Hebrew'),
    ('tr',  800,   580_000, 'Turkish'),
    ('ko',  800,   650_000, 'Korean'),
    ('uk',  700, 1_300_000, 'Ukrainian'),
    ('ar', 1500, 1_200_000, 'Arabic'),

    # Tier 4: LARGE (1.5GB-3GB) — last in queue
    ('sv', 1400, 2_600_000, 'Swedish'),
    ('nl', 1800, 2_100_000, 'Dutch'),
    ('pl', 2000, 1_600_000, 'Polish'),
    ('pt', 2500, 1_100_000, 'Portuguese'),

    # Tier 5: HUGE (3GB+) — only if all above succeed
    ('zh', 3000, 1_400_000, 'Chinese'),
    ('it', 4000, 1_900_000, 'Italian'),
    ('ru', 4000, 2_000_000, 'Russian'),
    ('ja', 5000, 1_400_000, 'Japanese'),
    ('es', 5000, 2_000_000, 'Spanish'),
    ('fr', 6000, 2_700_000, 'French'),
    ('de', 9000, 2_900_000, 'German'),
    ('en', 22000, 6_900_000, 'English'),
]


# ═══════════════════════════════════════════════════════════════════════
# Shared state
# ═══════════════════════════════════════════════════════════════════════

state_lock = Lock()
STATE = {
    'started_ts': time.time(),
    'current_phase': 'init',
    'languages': {
        code: {
            'code': code, 'name': name, 'est_mb': mb,
            'status': 'pending',  # pending / downloading / parsing / done / failed / skipped
            'downloaded_mb': 0,
            'articles_parsed': 0,
            'articles_written': 0,
            'error': None,
            'started_ts': None,
            'completed_ts': None,
        }
        for code, mb, _, name in LANGUAGES
    },
}


def save_state():
    with state_lock:
        STATE['updated_ts'] = time.time()
        STATE['uptime_hours'] = (time.time() - STATE['started_ts']) / 3600
        STATE['summary'] = _summary()
        STATUS_FILE.write_text(json.dumps(STATE, ensure_ascii=False, indent=1))


def _summary() -> dict:
    langs = STATE['languages'].values()
    return {
        'total_languages': len(STATE['languages']),
        'done': sum(1 for v in langs if v['status'] == 'done'),
        'downloading': sum(1 for v in langs if v['status'] == 'downloading'),
        'parsing': sum(1 for v in langs if v['status'] == 'parsing'),
        'failed': sum(1 for v in langs if v['status'] == 'failed'),
        'skipped': sum(1 for v in langs if v['status'] == 'skipped'),
        'pending': sum(1 for v in langs if v['status'] == 'pending'),
        'total_articles_written': sum(v['articles_written'] for v in langs),
    }


def log(msg: str):
    line = f"[{datetime.now().strftime('%H:%M:%S')}] {msg}"
    print(line, flush=True)
    with open(LOG_DIR / 'run.log', 'a', encoding='utf-8') as f:
        f.write(line + '\n')


def disk_free_gb() -> float:
    return shutil.disk_usage('/home/dinio/zets').free / (1024 ** 3)


def should_stop() -> tuple[bool, str]:
    if STOP_FILE.exists():
        return True, 'STOP_FILE'
    df = disk_free_gb()
    if df < MIN_DISK_GB:
        return True, f'DISK_LOW_{df:.1f}GB'
    return False, ''


# ═══════════════════════════════════════════════════════════════════════
# Download a single language dump
# ═══════════════════════════════════════════════════════════════════════

def download_lang(lang_code: str) -> Path | None:
    url = f'https://dumps.wikimedia.org/{lang_code}wiki/latest/{lang_code}wiki-latest-pages-articles.xml.bz2'
    local = DUMP_DIR / f'{lang_code}wiki-latest-pages-articles.xml.bz2'
    tmp = local.with_suffix('.bz2.part')

    if local.exists():
        log(f"  [{lang_code}] dump exists, skipping download ({local.stat().st_size/1024/1024:.0f} MB)")
        return local

    with state_lock:
        STATE['languages'][lang_code]['status'] = 'downloading'
        STATE['languages'][lang_code]['started_ts'] = time.time()

    try:
        req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
        with urllib.request.urlopen(req, timeout=120) as r:
            total = int(r.headers.get('Content-Length', 0))
            log(f"  [{lang_code}] downloading {total/1024/1024:.0f} MB from {url}")
            written = 0
            last_progress_ts = time.time()
            with open(tmp, 'wb') as f:
                while True:
                    stop, reason = should_stop()
                    if stop:
                        log(f"  [{lang_code}] download aborted: {reason}")
                        tmp.unlink(missing_ok=True)
                        return None
                    chunk = r.read(512 * 1024)
                    if not chunk:
                        break
                    f.write(chunk)
                    written += len(chunk)
                    if time.time() - last_progress_ts > 15:
                        pct = 100 * written / max(total, 1)
                        log(f"  [{lang_code}] download {pct:.1f}% ({written/1024/1024:.0f}/{total/1024/1024:.0f} MB)")
                        with state_lock:
                            STATE['languages'][lang_code]['downloaded_mb'] = written / 1024 / 1024
                        save_state()
                        last_progress_ts = time.time()
        tmp.rename(local)
        size_mb = local.stat().st_size / 1024 / 1024
        log(f"  [{lang_code}] downloaded ✓ {size_mb:.0f} MB")
        with state_lock:
            STATE['languages'][lang_code]['downloaded_mb'] = size_mb
        return local
    except Exception as e:
        log(f"  [{lang_code}] download FAILED: {e}")
        tmp.unlink(missing_ok=True)
        with state_lock:
            STATE['languages'][lang_code]['status'] = 'failed'
            STATE['languages'][lang_code]['error'] = f'download: {e}'[:200]
        return None


# ═══════════════════════════════════════════════════════════════════════
# Parse a language dump (stream)
# ═══════════════════════════════════════════════════════════════════════

WIKITEXT_CLEANUPS = [
    (re.compile(r'\{\{[^{}]*\}\}'), ''),
    (re.compile(r'\{\|.*?\|\}', re.DOTALL), ''),
    (re.compile(r'<ref[^>]*>.*?</ref>', re.DOTALL), ''),
    (re.compile(r'<ref[^>]*/>'), ''),
    (re.compile(r'<[^>]+>'), ''),
    (re.compile(r'\[\[[^\]|]+\|([^\]]+)\]\]'), r'\1'),
    (re.compile(r'\[\[([^\]]+)\]\]'), r'\1'),
    (re.compile(r'\[https?://\S+\s+([^\]]+)\]'), r'\1'),
    (re.compile(r'\[https?://\S+\]'), ''),
    (re.compile(r"'''([^']+)'''"), r'\1'),
    (re.compile(r"''([^']+)''"), r'\1'),
    (re.compile(r'={2,}\s*([^=]+?)\s*={2,}'), r'\1'),
    (re.compile(r'\n{3,}'), '\n\n'),
    (re.compile(r'[ \t]+'), ' '),
]


def clean_wikitext(text: str) -> str:
    if not text:
        return ''
    # Run template removal twice for nested templates
    for _ in range(2):
        text = re.sub(r'\{\{[^{}]*\}\}', '', text)
    for pattern, replacement in WIKITEXT_CLEANUPS:
        text = pattern.sub(replacement, text)
    return text.strip()


def parse_lang(lang_code: str, dump_path: Path) -> bool:
    out_path = DUMP_DIR / f'{lang_code}_parsed.jsonl.gz'
    if out_path.exists() and out_path.stat().st_size > 1000:
        log(f"  [{lang_code}] parsed JSONL exists, skipping parse")
        with state_lock:
            STATE['languages'][lang_code]['status'] = 'done'
        return True

    with state_lock:
        STATE['languages'][lang_code]['status'] = 'parsing'

    ns_uri = 'http://www.mediawiki.org/xml/export-0.11/'
    NS_PAGE = f'{{{ns_uri}}}page'
    NS_TITLE = f'{{{ns_uri}}}title'
    NS_TEXT = f'{{{ns_uri}}}text'
    NS_NS = f'{{{ns_uri}}}ns'
    NS_REDIRECT = f'{{{ns_uri}}}redirect'
    NS_REVISION = f'{{{ns_uri}}}revision'

    articles_seen = 0
    articles_written = 0
    t0 = time.time()
    last_progress = t0

    try:
        with bz2.open(dump_path, 'rb') as raw:
            with gzip.open(out_path, 'wt', encoding='utf-8') as out:
                try:
                    context = ET.iterparse(raw, events=('end',))
                    for _, elem in context:
                        if elem.tag != NS_PAGE:
                            continue
                        articles_seen += 1

                        # Safety
                        if articles_seen % 500 == 0:
                            stop, reason = should_stop()
                            if stop:
                                log(f"  [{lang_code}] parse halted: {reason}")
                                break
                        if articles_written >= MAX_ARTICLES_PER_LANG:
                            log(f"  [{lang_code}] reached max_articles {MAX_ARTICLES_PER_LANG}")
                            break

                        title_el = elem.find(NS_TITLE)
                        text_el = elem.find(f'{NS_REVISION}/{NS_TEXT}')
                        ns_el = elem.find(NS_NS)
                        redirect_el = elem.find(NS_REDIRECT)

                        if redirect_el is not None:
                            elem.clear(); continue
                        if ns_el is not None and ns_el.text != '0':
                            elem.clear(); continue
                        title = title_el.text if title_el is not None else None
                        raw_text = text_el.text if text_el is not None else None
                        if not title or not raw_text:
                            elem.clear(); continue

                        clean = clean_wikitext(raw_text)
                        if len(clean) < 200:
                            elem.clear(); continue

                        out.write(json.dumps({
                            'title': title,
                            'lang': lang_code,
                            'source': f'wikipedia_{lang_code}',
                            'source_tier': 'B',
                            'prior_confidence': 0.80,
                            'text': clean[:ARTICLE_TEXT_CAP],
                            'text_length_original': len(clean),
                        }, ensure_ascii=False) + '\n')
                        articles_written += 1

                        if time.time() - last_progress > 30:
                            rate = articles_written / max(time.time() - t0, 0.1)
                            log(f"  [{lang_code}] parsed {articles_written:,} articles, rate {rate:.0f}/s")
                            with state_lock:
                                STATE['languages'][lang_code]['articles_parsed'] = articles_seen
                                STATE['languages'][lang_code]['articles_written'] = articles_written
                            save_state()
                            last_progress = time.time()

                        elem.clear()
                except ET.ParseError as e:
                    log(f"  [{lang_code}] XML parse error at article {articles_seen}: {e}")
    except Exception as e:
        log(f"  [{lang_code}] parse error: {e}")
        with state_lock:
            STATE['languages'][lang_code]['status'] = 'failed'
            STATE['languages'][lang_code]['error'] = f'parse: {e}'[:200]
        return False

    elapsed = time.time() - t0
    log(f"  [{lang_code}] parsed ✓ {articles_written:,} articles in {elapsed:.0f}s "
        f"(output: {out_path.stat().st_size/1024/1024:.0f} MB gzipped)")
    with state_lock:
        STATE['languages'][lang_code]['articles_parsed'] = articles_seen
        STATE['languages'][lang_code]['articles_written'] = articles_written
        STATE['languages'][lang_code]['status'] = 'done'
        STATE['languages'][lang_code]['completed_ts'] = time.time()
    save_state()

    # Delete raw dump to save disk
    try:
        dump_path.unlink()
        log(f"  [{lang_code}] raw dump deleted (saved ~{dump_path.stat().st_size/1024/1024 if dump_path.exists() else 0:.0f} MB)")
    except Exception:
        pass

    return True


# ═══════════════════════════════════════════════════════════════════════
# Worker — handles one language end-to-end
# ═══════════════════════════════════════════════════════════════════════

def process_lang(lang_code: str):
    stop, reason = should_stop()
    if stop:
        with state_lock:
            STATE['languages'][lang_code]['status'] = 'skipped'
            STATE['languages'][lang_code]['error'] = reason
        return

    try:
        dump_path = download_lang(lang_code)
        if not dump_path:
            return

        parse_lang(lang_code, dump_path)
    except Exception as e:
        log(f"  [{lang_code}] ERROR in worker: {e}")
        with state_lock:
            STATE['languages'][lang_code]['status'] = 'failed'
            STATE['languages'][lang_code]['error'] = str(e)[:200]
    finally:
        save_state()


# ═══════════════════════════════════════════════════════════════════════
# Main orchestrator
# ═══════════════════════════════════════════════════════════════════════

def main():
    log(f"━━━ multi_lang_wiki starting ━━━")
    log(f"  Disk free: {disk_free_gb():.1f} GB")
    log(f"  Languages in catalog: {len(LANGUAGES)}")
    log(f"  Max concurrent downloads: {MAX_PARALLEL_DOWNLOADS}")
    log(f"  Stop: touch {STOP_FILE}")

    # Graceful signal handling
    shutdown = {'flag': False}
    def handler(signum, frame):
        shutdown['flag'] = True
    signal.signal(signal.SIGTERM, handler)
    signal.signal(signal.SIGINT, handler)

    save_state()

    # Process languages in order (small→large), N workers at a time
    queue: Queue = Queue()
    for code, _, _, _ in LANGUAGES:
        queue.put(code)

    workers = []

    def worker_loop():
        while True:
            try:
                code = queue.get_nowait()
            except Exception:
                return
            stop, reason = should_stop()
            if stop or shutdown['flag']:
                with state_lock:
                    STATE['languages'][code]['status'] = 'skipped'
                    STATE['languages'][code]['error'] = reason or 'shutdown'
                save_state()
                continue
            process_lang(code)

    # Start N workers
    for _ in range(MAX_PARALLEL_DOWNLOADS):
        t = Thread(target=worker_loop, daemon=True)
        t.start()
        workers.append(t)

    # Wait for completion, checking STOP periodically
    while any(t.is_alive() for t in workers):
        time.sleep(30)
        save_state()

    log(f"━━━ multi_lang_wiki done ━━━")
    save_state()
    summary = STATE['summary']
    log(f"  Summary: {json.dumps(summary)}")


if __name__ == '__main__':
    main()
