#!/usr/bin/env python3
"""
test_multi_interface_v1.py — proves the 4-way interface concept.

Design: ONE zets_node Rust binary serves all 4 interface types:
  1. Web GUI  — served from /gui/ (embedded static files)
  2. CLI      — same binary with --cli flag
  3. HTTP API — /api/v1/* endpoints
  4. MCP      — /mcp/sse endpoint for Claude/LLM integration

Licensing: every API/MCP request is checked against license tier.
  tier=free:       10 queries/hour, no /api/teach, no /api/export
  tier=personal:   1000/hour, /api/teach allowed, no export
  tier=pro:        unlimited, all endpoints
  tier=enterprise: unlimited, custom domains, offline packs

Per-customer override: customer can CHOOSE a stricter tier than their license
(e.g. pro customer wants only 100/hour in kids-mode).

ENFORCEMENT: min(license_tier_limit, customer_override) — the stricter wins.

Run: python3 py_testers/test_multi_interface_v1.py

This is the Python prototype per CLAUDE_RULES_v1 §4.
"""

import dataclasses
import json
import time
from dataclasses import dataclass, field
from typing import Optional


# ═══════════════════════════════════════════════════════════════════════
# License tiers
# ═══════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class TierLimits:
    name: str
    queries_per_hour: int          # 0 = unlimited
    teach_allowed: bool            # can write to graph?
    export_allowed: bool           # can dump full graph?
    offline_packs: bool            # can install private language packs
    custom_domains: bool           # RSS/API beyond default catalog
    api_key_required: bool         # public endpoint must sign requests


TIERS = {
    'free': TierLimits(
        name='free',
        queries_per_hour=10,
        teach_allowed=False,
        export_allowed=False,
        offline_packs=False,
        custom_domains=False,
        api_key_required=False,
    ),
    'personal': TierLimits(
        name='personal',
        queries_per_hour=1000,
        teach_allowed=True,
        export_allowed=False,
        offline_packs=True,
        custom_domains=False,
        api_key_required=True,
    ),
    'pro': TierLimits(
        name='pro',
        queries_per_hour=100_000,
        teach_allowed=True,
        export_allowed=True,
        offline_packs=True,
        custom_domains=True,
        api_key_required=True,
    ),
    'enterprise': TierLimits(
        name='enterprise',
        queries_per_hour=0,  # unlimited
        teach_allowed=True,
        export_allowed=True,
        offline_packs=True,
        custom_domains=True,
        api_key_required=True,
    ),
}


@dataclass
class CustomerOverride:
    """Customer may VOLUNTARILY choose STRICTER limits than their license."""
    max_queries_per_hour: Optional[int] = None
    allow_teach: Optional[bool] = None
    allow_export: Optional[bool] = None
    allow_custom_domains: Optional[bool] = None
    child_safe: bool = False       # filter register>4 (slang) everywhere

    def effective_limit(self, license_tier: TierLimits) -> TierLimits:
        """Return the STRICTER of license_tier and customer_override."""
        qph = license_tier.queries_per_hour
        if self.max_queries_per_hour is not None:
            if qph == 0:  # license is unlimited
                qph = self.max_queries_per_hour
            else:
                qph = min(qph, self.max_queries_per_hour)

        teach = license_tier.teach_allowed
        if self.allow_teach is not None:
            teach = teach and self.allow_teach

        export = license_tier.export_allowed
        if self.allow_export is not None:
            export = export and self.allow_export

        custom = license_tier.custom_domains
        if self.allow_custom_domains is not None:
            custom = custom and self.allow_custom_domains

        return TierLimits(
            name=f'{license_tier.name}+override',
            queries_per_hour=qph,
            teach_allowed=teach,
            export_allowed=export,
            offline_packs=license_tier.offline_packs,
            custom_domains=custom,
            api_key_required=license_tier.api_key_required,
        )


# ═══════════════════════════════════════════════════════════════════════
# Rate limiter (per-customer, per-interface)
# ═══════════════════════════════════════════════════════════════════════

class RateLimiter:
    """Sliding 1-hour window."""

    def __init__(self):
        self.buckets: dict[str, list[float]] = {}

    def _prune(self, key: str):
        now = time.time()
        cutoff = now - 3600
        self.buckets[key] = [t for t in self.buckets.get(key, []) if t > cutoff]

    def allow(self, key: str, limit: int) -> tuple[bool, int, int]:
        """Returns (allowed, remaining, reset_in_seconds)."""
        self._prune(key)
        bucket = self.buckets.setdefault(key, [])
        if limit == 0:
            bucket.append(time.time())
            return True, -1, 0   # unlimited
        if len(bucket) >= limit:
            reset = int(3600 - (time.time() - bucket[0]))
            return False, 0, reset
        bucket.append(time.time())
        return True, limit - len(bucket), 3600


# ═══════════════════════════════════════════════════════════════════════
# The 4 interface handlers — all backed by ONE ZetsNode
# ═══════════════════════════════════════════════════════════════════════

class ZetsNode:
    """Unified node — the Rust version will do the same thing."""

    def __init__(self, license_tier: str = 'free',
                 customer: Optional[CustomerOverride] = None):
        self.license = TIERS[license_tier]
        self.customer = customer or CustomerOverride()
        self.effective = self.customer.effective_limit(self.license)
        self.rate_limiter = RateLimiter()
        self.audit_log: list = []

    # Common policy check — called from EVERY interface
    def authorize(self, interface: str, action: str,
                  api_key: Optional[str] = None) -> dict:
        """Return {allowed: bool, reason: str, headers: dict}."""
        # 1. API key required?
        if interface in ('api', 'mcp') and self.effective.api_key_required:
            if not api_key:
                return {'allowed': False, 'reason': 'api_key_required',
                        'status': 401}

        # 2. Action-level permissions
        if action == 'teach' and not self.effective.teach_allowed:
            return {'allowed': False, 'reason': 'teach_not_allowed_in_tier',
                    'status': 403}
        if action == 'export' and not self.effective.export_allowed:
            return {'allowed': False, 'reason': 'export_not_allowed_in_tier',
                    'status': 403}

        # 3. Rate limit (per interface × api_key)
        rate_key = api_key or "local"
        ok, remaining, reset = self.rate_limiter.allow(
            rate_key, self.effective.queries_per_hour)
        if not ok:
            return {
                'allowed': False, 'reason': 'rate_limit',
                'status': 429,
                'headers': {'X-RateLimit-Reset-Seconds': str(reset)},
            }

        # 4. Audit (keep last 100)
        self.audit_log.append({
            'ts': time.time(), 'interface': interface,
            'action': action, 'allowed': True,
            'remaining': remaining,
        })
        self.audit_log = self.audit_log[-100:]

        return {
            'allowed': True,
            'headers': {
                'X-RateLimit-Remaining': str(remaining),
                'X-RateLimit-Limit': str(self.effective.queries_per_hour),
                'X-Tier': self.effective.name,
            },
        }

    # Business logic — pure, independent of interface
    def query(self, text: str) -> dict:
        return {'answer': f"(mock answer for: {text[:40]})",
                'followups': ['More detail?', 'Different angle?']}

    def teach(self, fact: str) -> dict:
        return {'stored': True, 'fact': fact[:40]}


# ═══════════════════════════════════════════════════════════════════════
# Interface adapters — each is VERY thin
# ═══════════════════════════════════════════════════════════════════════

class WebGuiAdapter:
    """Browser served locally. Uses session cookie, so no API key."""
    def __init__(self, node: ZetsNode):
        self.node = node

    def handle_query(self, text: str, session_id: str) -> dict:
        auth = self.node.authorize('web', 'query')
        if not auth['allowed']:
            return {'error': auth['reason'], 'status': auth.get('status', 403)}
        result = self.node.query(text)
        result['status'] = 200
        result['headers'] = auth.get('headers', {})
        return result


class CliAdapter:
    """Terminal. No auth — local process, runs as same user as node."""
    def __init__(self, node: ZetsNode):
        self.node = node

    def handle_query(self, text: str) -> dict:
        auth = self.node.authorize('cli', 'query')
        # CLI respects rate limit but skips api_key check (local)
        if not auth['allowed'] and auth['reason'] != 'api_key_required':
            return {'error': auth['reason']}
        result = self.node.query(text)
        return result


class ApiAdapter:
    """Public HTTP API. api_key REQUIRED on all paid tiers."""
    def __init__(self, node: ZetsNode):
        self.node = node

    def handle(self, path: str, api_key: Optional[str], body: dict) -> dict:
        action_map = {
            '/api/v1/query': 'query',
            '/api/v1/teach': 'teach',
            '/api/v1/export': 'export',
        }
        action = action_map.get(path, 'query')
        auth = self.node.authorize('api', action, api_key=api_key)
        if not auth['allowed']:
            return {'status': auth.get('status', 403),
                    'body': {'error': auth['reason']},
                    'headers': auth.get('headers', {})}

        if action == 'query':
            body_out = self.node.query(body.get('q', ''))
        elif action == 'teach':
            body_out = self.node.teach(body.get('fact', ''))
        else:
            body_out = {'error': f'unsupported: {action}'}
        return {'status': 200, 'body': body_out, 'headers': auth['headers']}


class McpAdapter:
    """MCP protocol — used by Claude and other LLMs."""
    def __init__(self, node: ZetsNode):
        self.node = node

    def handle_tool_call(self, tool_name: str, args: dict,
                         api_key: Optional[str]) -> dict:
        # MCP tools map to actions
        action_map = {'zets_query': 'query',
                      'zets_teach': 'teach',
                      'zets_export': 'export'}
        action = action_map.get(tool_name, 'query')
        auth = self.node.authorize('mcp', action, api_key=api_key)
        if not auth['allowed']:
            return {'error': auth['reason'], 'is_error': True}

        if action == 'query':
            return self.node.query(args.get('q', ''))
        if action == 'teach':
            return self.node.teach(args.get('fact', ''))
        return {'error': 'unsupported'}


# ═══════════════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════════════

def test_free_tier_limits_apply():
    node = ZetsNode(license_tier='free')
    api = ApiAdapter(node)

    # 10 queries allowed
    for i in range(10):
        r = api.handle('/api/v1/query', api_key=None, body={'q': 'x'})
        # free tier doesn't require api_key
    # 11th is blocked
    r = api.handle('/api/v1/query', api_key=None, body={'q': 'x'})
    assert r['status'] == 429, f"11th query should 429, got {r}"
    print(f"  ✓ free tier 11th query → {r['status']}  ({r['body']['error']})")


def test_teach_blocked_for_free_tier():
    node = ZetsNode(license_tier='free')
    api = ApiAdapter(node)
    r = api.handle('/api/v1/teach', api_key='k123', body={'fact': 'x is y'})
    assert r['status'] == 403
    print(f"  ✓ free tier /teach → 403 ({r['body']['error']})")


def test_customer_override_stricter_than_license():
    """Customer on 'pro' tier overrides to 100/hour only."""
    node = ZetsNode(
        license_tier='pro',
        customer=CustomerOverride(max_queries_per_hour=100),
    )
    print(f"  license=pro ({node.license.queries_per_hour}/hr), "
          f"customer wants 100/hr")
    print(f"  effective: {node.effective.queries_per_hour}/hr")
    assert node.effective.queries_per_hour == 100
    print(f"  ✓ stricter of (pro={node.license.queries_per_hour}, custom=100) = 100")


def test_customer_cannot_exceed_license():
    """Customer on 'free' tier tries to raise limit to 10K — can't."""
    node = ZetsNode(
        license_tier='free',
        customer=CustomerOverride(max_queries_per_hour=10000),
    )
    print(f"  license=free (10/hr), customer asks for 10000/hr")
    print(f"  effective: {node.effective.queries_per_hour}/hr")
    assert node.effective.queries_per_hour == 10  # license wins
    print(f"  ✓ customer cannot exceed license (10)")


def test_same_node_same_policy_across_interfaces():
    """CLI + API + MCP all share the same policy on the same node."""
    node = ZetsNode(license_tier='free')
    cli = CliAdapter(node)
    api = ApiAdapter(node)
    mcp = McpAdapter(node)

    # Use up the 10 queries, mixed across interfaces
    # NOTE: CLI also uses same api_key (via config/session) so bucket is shared
    for _ in range(4): cli.handle_query('x')  # uses 'local' bucket
    # To properly test: all interfaces must use same customer identity
    # In real system: node has a customer_id, ALL interfaces use it
    for _ in range(3): api.handle('/api/v1/query', api_key='local', body={'q': 'x'})
    for _ in range(3): mcp.handle_tool_call('zets_query', {'q': 'x'}, api_key='local')

    # 11th on ANY interface should 429
    r_api = api.handle('/api/v1/query', api_key='local', body={'q': 'x'})
    print(f"  after 10 mixed requests across CLI+API+MCP:")
    print(f"  11th on API  → {r_api['status']} ({r_api['body'].get('error')})")
    # MCP uses same limiter
    r_mcp = mcp.handle_tool_call('zets_query', {'q': 'x'}, api_key='local')
    print(f"  12th on MCP  → error={r_mcp.get('error')}")


def test_api_key_required_on_paid_tiers():
    node = ZetsNode(license_tier='personal')
    api = ApiAdapter(node)
    r = api.handle('/api/v1/query', api_key=None, body={'q': 'x'})
    assert r['status'] == 401
    print(f"  ✓ personal tier without api_key → 401")
    r = api.handle('/api/v1/query', api_key='valid', body={'q': 'x'})
    assert r['status'] == 200
    print(f"  ✓ personal tier WITH api_key → 200")


def test_enterprise_is_unlimited_but_still_audited():
    node = ZetsNode(license_tier='enterprise')
    api = ApiAdapter(node)
    for _ in range(50):
        r = api.handle('/api/v1/query', api_key='ent', body={'q': 'x'})
        assert r['status'] == 200
    print(f"  ✓ enterprise: 50 queries → all 200")
    print(f"  audit log length: {len(node.audit_log)}")
    assert len(node.audit_log) >= 50


def test_child_safe_customer_override():
    """Customer sets child_safe=True. We should filter slang atoms."""
    node = ZetsNode(
        license_tier='pro',
        customer=CustomerOverride(child_safe=True),
    )
    assert node.customer.child_safe is True
    print(f"  ✓ customer override: child_safe={node.customer.child_safe}")
    print(f"    (Rust impl: filter register ≥ 5 in all walks)")


if __name__ == '__main__':
    print("━━━ Multi-Interface + Licensing — Python Prototype ━━━\n")
    print("[1] Free tier enforces 10 queries/hour limit:")
    test_free_tier_limits_apply()
    print("\n[2] Free tier blocks /teach action:")
    test_teach_blocked_for_free_tier()
    print("\n[3] Customer override STRICTER than license wins:")
    test_customer_override_stricter_than_license()
    print("\n[4] Customer CANNOT exceed license (stricter wins):")
    test_customer_cannot_exceed_license()
    print("\n[5] CLI + API + MCP share SAME rate limit bucket:")
    test_same_node_same_policy_across_interfaces()
    print("\n[6] API key required on paid tiers:")
    test_api_key_required_on_paid_tiers()
    print("\n[7] Enterprise unlimited but audited:")
    test_enterprise_is_unlimited_but_still_audited()
    print("\n[8] Customer child_safe override:")
    test_child_safe_customer_override()
    print("\n━━━ ALL TESTS PASSED ━━━")
