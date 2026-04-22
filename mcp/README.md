# ZETS MCP + HTTP API

Python services that bridge ZETS cognitive kernel to AI agents and web UIs.

## Components

- **`zets_mcp_server.py`** — MCP (Model Context Protocol) SSE server on port 3145.
  Exposes ZETS tools (`zets_health`, `zets_benchmark`, `zets_verify`,
  `zets_ingest_text`, `zets_distill`, `zets_explain`, `zets_measure_moats`,
  `zets_tests`) + generic server-ops tools (`shell_run`, `file_read`,
  `file_write`, `git_status`). This is what Claude connects to.

- **`zets_http_api.py`** — Minimal REST API on port 3147 for the zets-gui web
  chat. Endpoints: `/api/health`, `/api/snapshots`, `/api/query`, `/api/ingest`.

- **`nginx-zets.conf`** — Nginx config for `ddev.chooz.co.il:3140`:
  - `/zets/` → static GUI (`/home/dinio/zets-gui/dist/`)
  - `/zets/mcp/` → MCP SSE (port 3145)
  - `/zets/api/` → HTTP API (port 3147)

- **`zets-mcp.service` / `zets-http.service`** — systemd units.

- **`deploy.sh`** — one-shot installer (requires `sudo`).

## Deploy

```bash
sudo bash /home/dinio/zets/mcp/deploy.sh
```

This will:
1. Stop any manually-started instances.
2. Install + enable + start both systemd services.
3. Back up any existing ddev.chooz.co.il nginx config and install the ZETS one.
4. Test + reload nginx.
5. Smoke-test public endpoints.

## Manual start (for dev)

```bash
# MCP on port 3145
python3 /home/dinio/zets/mcp/zets_mcp_server.py

# HTTP API on port 3147 (separate terminal)
python3 /home/dinio/zets/mcp/zets_http_api.py
```

## Architecture

ZETS itself is a stateless Rust CLI. These Python wrappers add:

- **Persistence of process** — the Rust binaries can be called on demand
  without each call paying the 2-second snapshot-load cost (a future
  Phase 8 Rust HTTP server will eliminate this via in-process snapshot cache).

- **Network boundary** — Claude / browser clients never exec on the server;
  they call defined tools through a constrained interface.

- **Subprocess dispatch** — each MCP/API tool spawns the appropriate
  Rust binary (`benchmark-runner`, `ingest-corpus`, etc.) and returns its
  structured output.

## Public URLs (after deploy)

- Chat:    `https://ddev.chooz.co.il:3140/zets/`
- MCP SSE: `https://ddev.chooz.co.il:3140/zets/mcp/sse`
- REST:    `https://ddev.chooz.co.il:3140/zets/api/{health,snapshots,query,ingest}`

## Replaces

This replaces the legacy `/home/dinio/cortex-v7/gateway/mcp_server.py` and
the cortex-v7-gui. Once verified working, cortex-v7 and lev can be fully
archived + removed from disk.
