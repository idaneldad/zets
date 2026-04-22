#!/bin/bash
# autonomous_master.sh — launches ALL autonomous learning processes in background.
#
# Runs:
#   1. night_learner.py         — RSS polling every 30 min
#   2. multi_lang_wiki.py       — Wikipedia dumps (small→large)
#
# Both are safe: STOP file support, disk monitoring, polite fetching.
#
# Usage:
#   bash autonomous_master.sh start    # launches everything
#   bash autonomous_master.sh status   # live status
#   bash autonomous_master.sh stop     # graceful shutdown (STOP file)
#   bash autonomous_master.sh kill     # hard kill

set -e

ZETS=/home/dinio/zets
AUTO=$ZETS/mcp/autonomous
LOGS=$AUTO/logs
PIDFILE=$LOGS/master.pid

mkdir -p $LOGS $LOGS/night $LOGS/multilang

cmd=${1:-status}

case "$cmd" in
  start)
    # Remove any stale STOP
    rm -f $AUTO/STOP

    # 1. Start night_learner (RSS)
    if pgrep -f "night_learner.py" > /dev/null; then
      echo "✓ night_learner already running (PID: $(pgrep -f night_learner.py))"
    else
      cd $ZETS
      nohup nice -n 10 python3 $AUTO/night_learner.py \
        > $LOGS/night_stdout.log 2>&1 &
      echo $! > $LOGS/night.pid
      echo "✓ night_learner started: PID $!"
    fi

    sleep 2

    # 2. Start multi_lang_wiki (Wikipedia dumps)
    if pgrep -f "multi_lang_wiki.py" > /dev/null; then
      echo "✓ multi_lang_wiki already running (PID: $(pgrep -f multi_lang_wiki.py))"
    else
      cd $ZETS
      nohup nice -n 15 python3 $AUTO/multi_lang_wiki.py \
        > $LOGS/multilang_stdout.log 2>&1 &
      echo $! > $LOGS/multilang.pid
      echo "✓ multi_lang_wiki started: PID $!"
    fi

    echo ""
    echo "━━━ Autonomous learning is running ━━━"
    echo ""
    echo "  Morning report:  python3 $AUTO/morning_report.py"
    echo "  Live status:     bash $0 status"
    echo "  Graceful stop:   bash $0 stop"
    echo "  Hard kill:       bash $0 kill"
    echo ""
    echo "  Disk: $(df -h $ZETS | tail -1 | awk '{print $4}') free"
    echo ""
    echo "  Safety:"
    echo "    - STOP file stops everything cleanly"
    echo "    - disk < 15 GB → auto-stop"
    echo "    - max 500K articles per language"
    echo "    - robots.txt + User-Agent + rate limits enforced"
    ;;

  status)
    echo "━━━ Autonomous Master Status ━━━"
    echo ""
    echo "Processes:"
    if pgrep -af "night_learner.py" | grep -v grep; then :; else echo "  night_learner    DOWN"; fi
    if pgrep -af "multi_lang_wiki.py" | grep -v grep; then :; else echo "  multi_lang_wiki  DOWN"; fi
    echo ""
    if [ -f "$AUTO/status.json" ]; then
      echo "Night learner (RSS):"
      python3 -c "
import json
s = json.load(open('$AUTO/status.json'))
print(f'  uptime:  {s.get(\"uptime_hours\",0):.2f}h')
print(f'  cycles:  {s.get(\"cycles_completed\",0)}')
print(f'  novel:   {s.get(\"total_items_novel\",0):,}')
print(f'  corrob:  {s.get(\"total_items_corroborated\",0):,}')
print(f'  status:  {s.get(\"stop_reason\") or \"running\"}')
"
    fi
    echo ""
    if [ -f "$AUTO/multilang_status.json" ]; then
      echo "Multilang Wiki:"
      python3 -c "
import json
s = json.load(open('$AUTO/multilang_status.json'))
summ = s.get('summary', {})
print(f'  uptime:  {s.get(\"uptime_hours\",0):.2f}h')
print(f'  done:    {summ.get(\"done\",0)}/{summ.get(\"total_languages\",0)} languages')
print(f'  active:  downloading={summ.get(\"downloading\",0)}  parsing={summ.get(\"parsing\",0)}')
print(f'  failed:  {summ.get(\"failed\",0)}')
print(f'  total articles written: {summ.get(\"total_articles_written\",0):,}')
"
    fi
    echo ""
    echo "Disk: $(df -h $ZETS | tail -1 | awk '{print $4}') free"
    ;;

  stop)
    touch $AUTO/STOP
    echo "✓ STOP file created. Processes will exit cleanly on next safety check (up to 1 min)."
    ;;

  kill)
    pkill -f "night_learner.py" 2>/dev/null && echo "✓ night_learner killed" || echo "night_learner not running"
    pkill -f "multi_lang_wiki.py" 2>/dev/null && echo "✓ multi_lang_wiki killed" || echo "multi_lang_wiki not running"
    rm -f $AUTO/STOP $LOGS/night.pid $LOGS/multilang.pid
    ;;

  report)
    python3 $AUTO/morning_report.py
    ;;

  *)
    echo "Usage: $0 [start|status|stop|kill|report]"
    exit 1
    ;;
esac
