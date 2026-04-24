#!/bin/bash
cd /home/dinio/zets
SNAPSHOT=${SNAPSHOT:-data/baseline/v4_fixed.zv4}
PORT=${PORT:-3148}
LOG=/tmp/zets_v4_server.log
if pgrep -f "zets_v4_server"; then
    echo "already running"
    exit 0
fi
nohup ./target/release/zets_v4_server "$SNAPSHOT" --port "$PORT" >"$LOG" 2>&1 &
echo "started PID=$! on port $PORT"
sleep 15
tail -5 "$LOG"
