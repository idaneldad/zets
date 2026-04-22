#!/bin/bash
# ZETS deploy script — installs MCP + HTTP API systemd services
# and stages the zets-gui + nginx config.
#
# Run:  sudo bash /home/dinio/zets/mcp/deploy.sh
#
# Requires sudo (systemd + nginx). All other file ops are dinio-user.

set -e

echo "━━━ 1. Stop any running manual-start processes ━━━"
pkill -f "zets_mcp_server.py" 2>/dev/null || true
pkill -f "zets_http_api.py" 2>/dev/null || true
sleep 1

echo ""
echo "━━━ 2. Install systemd units ━━━"
cp /home/dinio/zets/mcp/zets-mcp.service /etc/systemd/system/zets-mcp.service
cp /home/dinio/zets/mcp/zets-http.service /etc/systemd/system/zets-http.service
systemctl daemon-reload
systemctl enable zets-mcp.service zets-http.service
systemctl start zets-mcp.service zets-http.service
sleep 2
systemctl --no-pager --quiet is-active zets-mcp.service && echo "  ✓ zets-mcp.service active"
systemctl --no-pager --quiet is-active zets-http.service && echo "  ✓ zets-http.service active"

echo ""
echo "━━━ 3. Verify ports ━━━"
ss -tlnp 2>/dev/null | grep -E ":3145|:3147"

echo ""
echo "━━━ 4. Deploy zets-gui (static) ━━━"
# The GUI is a single file — no build needed.
if [ -f /home/dinio/zets-gui/dist/index.html ]; then
  echo "  ✓ GUI staged at /home/dinio/zets-gui/dist/index.html"
  ls -lh /home/dinio/zets-gui/dist/
else
  echo "  ✗ GUI file missing!"
  exit 1
fi

echo ""
echo "━━━ 5. Install nginx config ━━━"
# Find existing ddev.chooz.co.il site config
SITE_CONF=""
for candidate in /etc/nginx/sites-available/ddev.chooz.co.il \
                 /etc/nginx/sites-available/ddev.chooz.co.il.conf \
                 /etc/nginx/conf.d/ddev.chooz.co.il.conf; do
  if [ -f "$candidate" ]; then SITE_CONF="$candidate"; break; fi
done

if [ -n "$SITE_CONF" ]; then
  echo "  Existing config found: $SITE_CONF"
  echo "  Backing up to: ${SITE_CONF}.bak.$(date +%Y%m%d_%H%M%S)"
  cp "$SITE_CONF" "${SITE_CONF}.bak.$(date +%Y%m%d_%H%M%S)"
  cp /home/dinio/zets/mcp/nginx-zets.conf "$SITE_CONF"
  echo "  ✓ Replaced with ZETS config"
else
  echo "  No existing config found — installing new"
  cp /home/dinio/zets/mcp/nginx-zets.conf /etc/nginx/sites-available/ddev.chooz.co.il
  ln -sf /etc/nginx/sites-available/ddev.chooz.co.il /etc/nginx/sites-enabled/ddev.chooz.co.il
fi

echo ""
echo "━━━ 6. Test + reload nginx ━━━"
if nginx -t 2>&1; then
  systemctl reload nginx
  echo "  ✓ Nginx reloaded"
else
  echo "  ✗ Nginx config test failed — NOT reloading"
  exit 1
fi

echo ""
echo "━━━ 7. Smoke test via public URL ━━━"
sleep 1
if curl -skf --max-time 5 https://ddev.chooz.co.il:3140/zets/api/health | head -c 200; then
  echo ""
  echo "  ✓ Public API responding"
else
  echo "  ✗ Public API not reachable"
fi

echo ""
echo "━━━ ZETS deployed ━━━"
echo "  GUI: https://ddev.chooz.co.il:3140/zets/"
echo "  MCP: https://ddev.chooz.co.il:3140/zets/mcp/sse"
echo "  API: https://ddev.chooz.co.il:3140/zets/api/"
