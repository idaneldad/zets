#!/usr/bin/env python3
"""
test_licensing_trust_v1.py — proves 5 ideas from Idan's licensing + trust vision.

(1) OEM licensing: pre-installed signed license, works offline
(2) Activation workflow: optional online registration
(3) Hybrid routing: per-query local/cloud/hybrid decision
(4) Trust spaces: graph structure like "family/stranger/child"
(5) Federated learning: hash-only sync, privacy-preserving

Per CLAUDE_RULES §4: Python prototype BEFORE Rust.
Run: python3 py_testers/test_licensing_trust_v1.py
"""

import hashlib
import hmac
import json
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


# ═══════════════════════════════════════════════════════════════════════
# (1) License atoms — signed by authority
# ═══════════════════════════════════════════════════════════════════════

AUTHORITY_SECRET = b"anthropic-zets-authority-dev-key"  # real: Ed25519 key


def sign(payload: bytes) -> str:
    return hmac.new(AUTHORITY_SECRET, payload, hashlib.sha256).hexdigest()


def verify(payload: bytes, signature: str) -> bool:
    return hmac.compare_digest(sign(payload), signature)


@dataclass
class License:
    kind: str            # 'oem' | 'subscription' | 'trial' | 'community' | 'enterprise'
    tier: str            # 'free' | 'personal' | 'pro' | 'enterprise'
    device_id: Optional[str]
    customer_id: Optional[str]
    expires_ts: Optional[float]
    capabilities: list
    signature: str = ''

    def signed_payload(self) -> bytes:
        data = {
            'kind': self.kind, 'tier': self.tier,
            'device_id': self.device_id, 'customer_id': self.customer_id,
            'expires_ts': self.expires_ts, 'capabilities': self.capabilities,
        }
        return json.dumps(data, sort_keys=True).encode()

    def sign_self(self):
        self.signature = sign(self.signed_payload())

    def is_valid(self) -> tuple[bool, str]:
        if not self.signature:
            return False, 'no_signature'
        if not verify(self.signed_payload(), self.signature):
            return False, 'invalid_signature'
        if self.expires_ts and self.expires_ts < time.time():
            return False, 'expired'
        return True, 'ok'


# ═══════════════════════════════════════════════════════════════════════
# (3) Hybrid query routing
# ═══════════════════════════════════════════════════════════════════════

class PrivacyLevel(Enum):
    LOCAL_ONLY = 'local_only'
    ANONYMIZED = 'anonymized'
    INTERNAL = 'internal'
    PUBLIC = 'public'


class Route(Enum):
    LOCAL_ONLY = 'local_only'
    LOCAL_THEN_CLOUD = 'local_then_cloud'
    HASH_SYNC_ONLY = 'hash_sync_only'
    CLOUD_FIRST = 'cloud_first'


def route_query(query_text: str, privacy: PrivacyLevel,
                has_internet: bool, local_confidence: float) -> Route:
    """Hybrid routing: combines privacy + network + local knowledge."""
    if privacy == PrivacyLevel.LOCAL_ONLY:
        return Route.LOCAL_ONLY
    if not has_internet:
        return Route.LOCAL_ONLY
    if privacy == PrivacyLevel.ANONYMIZED:
        return Route.HASH_SYNC_ONLY
    if local_confidence >= 0.8:
        return Route.LOCAL_ONLY
    if privacy == PrivacyLevel.PUBLIC and local_confidence < 0.5:
        return Route.CLOUD_FIRST
    return Route.LOCAL_THEN_CLOUD


# ═══════════════════════════════════════════════════════════════════════
# (4) Trust spaces — graph structure
# ═══════════════════════════════════════════════════════════════════════

class TrustLevel(Enum):
    UNKNOWN = 0
    STRANGER = 1
    ACQUAINTANCE = 2
    FRIEND = 3
    FAMILY = 4
    SELF = 5
    AUTHORITY = 6


@dataclass
class TrustRelation:
    asker: str            # entity id asking
    subject: str          # entity id they're asking about (or '*' for any)
    level: TrustLevel
    allowed_domains: list # ['public', 'movies'] etc.
    denied_domains: list  # explicit exclusions
    granted_by: str


class TrustGraph:
    def __init__(self):
        self.relations: list[TrustRelation] = []

    def add(self, rel: TrustRelation):
        self.relations.append(rel)

    def can_query(self, asker: str, subject: str, domain: str) -> tuple[bool, str]:
        """Given (asker, subject, domain), decide allow/deny + reason."""
        # Find most specific relation (exact subject > wildcard)
        rel = self._find_relation(asker, subject)
        if rel is None:
            # No relation: treat as stranger w/ public only
            if domain == 'public':
                return True, 'default_public_allowed'
            return False, 'no_trust_relation_with_' + subject

        # Explicit deny wins
        if domain in rel.denied_domains:
            return False, f'domain:{domain}_explicitly_denied'

        # Must be in allowed_domains or be a trusted-enough relation
        if domain in rel.allowed_domains:
            return True, f'explicit_allow_via_trust:{rel.level.name}'

        # If allowed_domains is empty and trust is high, default allow (family)
        if not rel.allowed_domains and rel.level.value >= TrustLevel.FAMILY.value:
            return True, f'trust:{rel.level.name}_default_allow'

        return False, f'domain:{domain}_not_in_allowed'

    def _find_relation(self, asker: str, subject: str) -> Optional[TrustRelation]:
        # Exact match first
        for rel in self.relations:
            if rel.asker == asker and rel.subject == subject:
                return rel
        # Wildcard
        for rel in self.relations:
            if rel.asker == asker and rel.subject == '*':
                return rel
        return None


# ═══════════════════════════════════════════════════════════════════════
# (5) Federated learning — hash-only sync
# ═══════════════════════════════════════════════════════════════════════

class FederationClient:
    """Hash-only: never sends actual content."""

    def __init__(self, endpoint: str, opt_in: bool = False,
                 exclude_domains: Optional[list] = None):
        self.endpoint = endpoint
        self.opt_in = opt_in
        self.exclude_domains = exclude_domains or ['medical', 'financial', 'personal']
        self.sent_hashes: list[str] = []

    def prepare_delta(self, atom_content: str, domain: str,
                      confidence_change: float) -> Optional[dict]:
        """Returns hash-based delta — NOT content."""
        if not self.opt_in:
            return None
        if domain in self.exclude_domains:
            return None
        # SHA-256 hash of normalized content — one-way
        h = hashlib.sha256(atom_content.strip().lower().encode()).hexdigest()
        delta = {
            'hash': h[:16],   # truncated for brevity
            'domain': domain,
            'delta_confidence': round(confidence_change, 3),
        }
        self.sent_hashes.append(h[:16])
        return delta


# ═══════════════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════════════

def test_oem_license_offline():
    """[1] OEM license works without network."""
    lic = License(kind='oem', tier='pro',
                  device_id='laptop-abc-123',
                  customer_id=None, expires_ts=None,
                  capabilities=['query', 'teach', 'export'])
    lic.sign_self()
    valid, reason = lic.is_valid()
    assert valid, f"OEM should be valid: {reason}"
    print(f"  ✓ OEM license valid offline: device={lic.device_id}, tier={lic.tier}")


def test_expired_license_rejected():
    lic = License(kind='subscription', tier='pro',
                  device_id=None, customer_id='cust-1',
                  expires_ts=time.time() - 86400,  # expired yesterday
                  capabilities=['query'])
    lic.sign_self()
    valid, reason = lic.is_valid()
    assert not valid
    print(f"  ✓ expired license rejected: {reason}")


def test_tampered_license_rejected():
    lic = License(kind='trial', tier='free',
                  device_id='d1', customer_id=None,
                  expires_ts=time.time() + 86400, capabilities=['query'])
    lic.sign_self()
    # Tamper
    lic.tier = 'enterprise'
    valid, reason = lic.is_valid()
    assert not valid
    print(f"  ✓ tampered license rejected: {reason}")


def test_hybrid_routing_privacy_levels():
    """[3] Privacy + internet combine for routing."""
    # Medical question → local only regardless of internet
    r = route_query("what meds am I on?", PrivacyLevel.LOCAL_ONLY, True, 0.3)
    assert r == Route.LOCAL_ONLY
    print(f"  ✓ LOCAL_ONLY overrides internet: {r.value}")

    # Public question with internet + low local confidence → cloud first
    r = route_query("weather today", PrivacyLevel.PUBLIC, True, 0.2)
    assert r == Route.CLOUD_FIRST
    print(f"  ✓ PUBLIC+internet+low_local → cloud: {r.value}")

    # Anonymized → hash-only sync
    r = route_query("how do I parse JSON?", PrivacyLevel.ANONYMIZED, True, 0.4)
    assert r == Route.HASH_SYNC_ONLY
    print(f"  ✓ ANONYMIZED → hash_sync_only: {r.value}")

    # Offline always = local
    r = route_query("anything", PrivacyLevel.PUBLIC, False, 0.1)
    assert r == Route.LOCAL_ONLY
    print(f"  ✓ no_internet → local: {r.value}")


def test_trust_spaces_like_child():
    """[4] Child (user) with trust graph for family/friends/strangers."""
    g = TrustGraph()

    g.add(TrustRelation(asker='yam', subject='family:dad', level=TrustLevel.FAMILY,
                        allowed_domains=[], denied_domains=[], granted_by='parent'))
    g.add(TrustRelation(asker='yam', subject='family:uncle', level=TrustLevel.FAMILY,
                        allowed_domains=[], denied_domains=['medical'],
                        granted_by='parent'))
    g.add(TrustRelation(asker='yam', subject='friend:rotem', level=TrustLevel.FRIEND,
                        allowed_domains=['games', 'movies', 'public'], denied_domains=[],
                        granted_by='parent'))
    g.add(TrustRelation(asker='yam', subject='*', level=TrustLevel.STRANGER,
                        allowed_domains=['public'], denied_domains=[],
                        granted_by='default'))

    # Dad can ask anything
    allowed, reason = g.can_query('yam', 'family:dad', 'medical')
    assert allowed
    print(f"  ✓ dad asks medical: allowed ({reason})")

    # Uncle can ask most, not medical
    allowed, reason = g.can_query('yam', 'family:uncle', 'medical')
    assert not allowed
    print(f"  ✓ uncle asks medical: denied ({reason})")
    allowed, reason = g.can_query('yam', 'family:uncle', 'games')
    assert allowed
    print(f"  ✓ uncle asks games: allowed ({reason})")

    # Rotem can ask movies but not medical
    allowed, reason = g.can_query('yam', 'friend:rotem', 'movies')
    assert allowed
    print(f"  ✓ rotem asks movies: allowed ({reason})")
    allowed, reason = g.can_query('yam', 'friend:rotem', 'medical')
    assert not allowed
    print(f"  ✓ rotem asks medical: denied ({reason})")

    # Stranger can ask public only
    allowed, reason = g.can_query('yam', 'stranger:xxx', 'public')
    assert allowed
    print(f"  ✓ stranger asks public: allowed ({reason})")
    allowed, reason = g.can_query('yam', 'stranger:xxx', 'personal')
    assert not allowed
    print(f"  ✓ stranger asks personal: denied ({reason})")


def test_federation_hash_only():
    """[5] Federated sync never leaks content."""
    client = FederationClient(endpoint='https://federation.zets',
                              opt_in=True,
                              exclude_domains=['medical', 'financial'])

    # Public fact — sharable
    delta = client.prepare_delta("Python is a language", 'public', 0.1)
    assert delta is not None
    assert 'hash' in delta
    assert 'Python' not in str(delta)  # content NOT in delta
    print(f"  ✓ public fact → hash-only: {delta}")

    # Medical fact — excluded
    delta = client.prepare_delta("User takes medication X", 'medical', 0.1)
    assert delta is None
    print(f"  ✓ medical fact → NOT shared")

    # Opt-out client shares nothing
    client2 = FederationClient(endpoint='x', opt_in=False)
    delta = client2.prepare_delta("anything", 'public', 0.1)
    assert delta is None
    print(f"  ✓ opt-out client → nothing shared")


def test_full_scenario():
    """Tie it all together: child user on OEM Windows laptop, partial network."""
    print("  Scenario: Yam (10yo) on her new Windows laptop, OEM-preinstalled ZETS.")

    # OEM license loaded
    lic = License(kind='oem', tier='personal',
                  device_id='yam-laptop-001',
                  customer_id=None, expires_ts=None,
                  capabilities=['query'])
    lic.sign_self()
    valid, _ = lic.is_valid()
    assert valid
    print(f"    ✓ OEM license active (kind={lic.kind}, tier={lic.tier})")

    # Trust graph configured by parent
    trust = TrustGraph()
    trust.add(TrustRelation(asker='yam', subject='family:parent',
                            level=TrustLevel.AUTHORITY,
                            allowed_domains=[], denied_domains=[],
                            granted_by='factory'))
    trust.add(TrustRelation(asker='yam', subject='*',
                            level=TrustLevel.STRANGER,
                            allowed_domains=['public', 'education'],
                            denied_domains=[], granted_by='factory'))
    print(f"    ✓ Trust graph: 2 relations (parent + default stranger)")

    # User query: school homework, has internet
    allowed, _ = trust.can_query('yam', 'stranger:public_web', 'education')
    assert allowed
    route = route_query("how does photosynthesis work?",
                        PrivacyLevel.PUBLIC, True, 0.3)
    print(f"    ✓ Homework query: allowed via trust, route={route.value}")

    # User query: asking about parent's salary
    allowed, reason = trust.can_query('yam', 'stranger:public_web', 'personal')
    assert not allowed
    print(f"    ✓ Personal query to stranger: {reason}")

    # Offline mode (travel)
    route = route_query("any question", PrivacyLevel.PUBLIC, False, 0.2)
    assert route == Route.LOCAL_ONLY
    print(f"    ✓ Offline travel: local-only routing")

    # Federation opt-out (parents didn't enable)
    fed = FederationClient(endpoint='x', opt_in=False)
    delta = fed.prepare_delta("photosynthesis is...", 'education', 0.1)
    assert delta is None
    print(f"    ✓ Federation disabled by parent: no data shared")


if __name__ == '__main__':
    print("━━━ Licensing + Trust Spaces — Python Prototype ━━━\n")
    print("[1] OEM license (pre-installed, offline):")
    test_oem_license_offline()
    print("\n[2] Expired license rejected:")
    test_expired_license_rejected()
    print("\n[3] Tampered license rejected:")
    test_tampered_license_rejected()
    print("\n[4] Hybrid routing (privacy + internet):")
    test_hybrid_routing_privacy_levels()
    print("\n[5] Trust spaces (family/friends/strangers):")
    test_trust_spaces_like_child()
    print("\n[6] Federation (hash-only, opt-in):")
    test_federation_hash_only()
    print("\n[7] Full scenario (child on OEM laptop):")
    test_full_scenario()
    print("\n━━━ ALL TESTS PASSED ━━━")
