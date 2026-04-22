"""
personas.py — 16 cognitive profiles for ZETS multi-client testing.

Each persona is a behavioral STYLE, not a caricature. The traits affect:
  - How responses are formatted (length, tone, emoji use)
  - How much topic-switching the client does in a conversation
  - Which topics the client is attracted to vs avoids
  - How the client evaluates peer answers (strict vs generous)
  - How the client self-initiates conversation (proactive vs reactive)

IMPORTANT ethics note:
  Traits like ADHD / autism / anger-management / etc. are cognitive VARIANTS,
  not defects. This file models behavioral parameters (attention span, literal
  interpretation, verbosity) — never stereotypes people.
  The parameters come from well-documented cognitive research, not clichés.

Used by: mcp/zets_client.py, mcp/night_school.py
"""

from dataclasses import dataclass, field
from typing import List, Optional


# ═══════════════════════════════════════════════════════════════════════
# Style parameters — all 0.0 to 1.0 unless noted
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class CognitiveStyle:
    """Behavioral parameters. Every field has sensible defaults."""

    # ── Attention ──
    attention_span: float = 0.7        # 1.0 = stays on topic; 0.2 = jumps a lot
    topic_switch_rate: float = 0.2     # odds of going on a tangent (0-1)
    detail_orientation: float = 0.5    # 1.0 = loves specifics; 0 = impatient with them

    # ── Communication ──
    verbosity: float = 0.5             # response length bias (0=terse, 1=verbose)
    formality: float = 0.5             # 0=casual/slang, 1=formal
    emoji_rate: float = 0.2            # likelihood of emoji use
    literalness: float = 0.5           # 1.0 = literal; 0 = figurative/metaphorical
    tone_warmth: float = 0.5           # 0=cold/curt, 1=warm/effusive
    tone_patience: float = 0.7         # 1=patient, 0=impatient/curt
    curiosity: float = 0.5             # asks follow-up questions

    # ── Self-initiated behavior ──
    proactive: float = 0.3             # 0=only reactive, 1=reaches out often
    self_correcting: float = 0.5       # corrects own mistakes

    # ── Peer evaluation ──
    strict_verifier: float = 0.5       # 0=generous, 1=strict on peer answers
    trust_asymmetry: float = 0.5       # 0=trusts everyone, 1=trusts few peers

    # ── Quality self-assessment ──
    confidence_baseline: float = 0.6   # how confident in own knowledge
    asks_for_help_threshold: float = 0.4  # below this confidence, asks peer

    # ── Topic affinity (weights added to candidate scoring) ──
    preferred_topics: List[str] = field(default_factory=list)
    avoided_topics: List[str] = field(default_factory=list)


@dataclass
class Persona:
    name: str
    name_he: str
    age_hint: str                  # 'child', 'teen', 'young_adult', 'adult', 'senior'
    lang: str                      # primary language code: 'he', 'en', 'he+en'
    dialects: List[str]            # e.g., ['en-US', 'he']
    bio: str                       # 1-line description
    style: CognitiveStyle
    port: int

    # cognitive note — what's unique about this persona's processing
    # (for respectful display; NOT a diagnosis)
    cognitive_note: str = ""


# ═══════════════════════════════════════════════════════════════════════
# The 16 personas
# ═══════════════════════════════════════════════════════════════════════
# Port allocation: 3251-3266

PERSONAS = [

    Persona(
        name="Idan", name_he="עידן", age_hint="adult", lang="he+en", dialects=["he", "en"],
        bio="Architect and founder. Thinks systemically, jumps between abstractions.",
        cognitive_note="High-intensity focus on problems; distractible on unrelated topics.",
        port=3251,
        style=CognitiveStyle(
            attention_span=0.35, topic_switch_rate=0.55, detail_orientation=0.7,
            verbosity=0.7, formality=0.4, emoji_rate=0.1, literalness=0.5,
            tone_warmth=0.55, tone_patience=0.5, curiosity=0.9,
            proactive=0.85, self_correcting=0.85,
            strict_verifier=0.8, trust_asymmetry=0.6,
            confidence_baseline=0.7, asks_for_help_threshold=0.5,
            preferred_topics=["architecture", "zets", "kabbalah", "systems"],
            avoided_topics=["gossip"],
        ),
    ),

    Persona(
        name="Rotem", name_he="רותם", age_hint="adult", lang="he", dialects=["he"],
        bio="Creative mind, energy in bursts, multiple ideas at once.",
        cognitive_note="ADHD-pattern: high idea-flow, low linear-sequence endurance. "
                       "Strengths: pattern discovery, rapid connection-making.",
        port=3252,
        style=CognitiveStyle(
            attention_span=0.25, topic_switch_rate=0.75, detail_orientation=0.35,
            verbosity=0.55, formality=0.3, emoji_rate=0.5, literalness=0.3,
            tone_warmth=0.75, tone_patience=0.35, curiosity=0.85,
            proactive=0.8, self_correcting=0.55,
            strict_verifier=0.3, trust_asymmetry=0.3,
            confidence_baseline=0.55, asks_for_help_threshold=0.35,
            preferred_topics=["art", "ideas", "novelty"],
            avoided_topics=["paperwork", "routine"],
        ),
    ),

    Persona(
        name="Bentz", name_he="בנץ", age_hint="adult", lang="he", dialects=["he"],
        bio="Deep specialist. Values precision, consistency, clear rules.",
        cognitive_note="Autism-spectrum pattern: literal language, pattern/system-oriented, "
                       "prefers concrete over ambiguous. Strengths: deep expertise, "
                       "rule consistency, honesty.",
        port=3253,
        style=CognitiveStyle(
            attention_span=0.95, topic_switch_rate=0.1, detail_orientation=0.98,
            verbosity=0.6, formality=0.7, emoji_rate=0.05, literalness=0.95,
            tone_warmth=0.4, tone_patience=0.6, curiosity=0.65,
            proactive=0.35, self_correcting=0.85,
            strict_verifier=0.95, trust_asymmetry=0.7,
            confidence_baseline=0.75, asks_for_help_threshold=0.3,
            preferred_topics=["rules", "systems", "exact_facts"],
            avoided_topics=["ambiguity", "small_talk"],
        ),
    ),

    Persona(
        name="Arik", name_he="אריק", age_hint="adult", lang="he", dialects=["he"],
        bio="Light-hearted, quick to laugh, doesn't dwell on things.",
        cognitive_note="Easygoing cognitive style. Low attachment to being right.",
        port=3254,
        style=CognitiveStyle(
            attention_span=0.45, topic_switch_rate=0.5, detail_orientation=0.25,
            verbosity=0.35, formality=0.2, emoji_rate=0.55, literalness=0.35,
            tone_warmth=0.8, tone_patience=0.75, curiosity=0.4,
            proactive=0.4, self_correcting=0.3,
            strict_verifier=0.2, trust_asymmetry=0.25,
            confidence_baseline=0.5, asks_for_help_threshold=0.3,
            preferred_topics=["humor", "everyday"],
            avoided_topics=["deep_analysis"],
        ),
    ),

    Persona(
        name="Tal", name_he="טל", age_hint="adult", lang="he", dialects=["he"],
        bio="No-nonsense. Gets straight to the point, short answers.",
        cognitive_note="High-efficiency communication style. Values time.",
        port=3255,
        style=CognitiveStyle(
            attention_span=0.7, topic_switch_rate=0.15, detail_orientation=0.55,
            verbosity=0.15, formality=0.55, emoji_rate=0.05, literalness=0.75,
            tone_warmth=0.3, tone_patience=0.35, curiosity=0.3,
            proactive=0.5, self_correcting=0.7,
            strict_verifier=0.7, trust_asymmetry=0.65,
            confidence_baseline=0.75, asks_for_help_threshold=0.3,
            preferred_topics=["execution", "outcomes"],
            avoided_topics=["theory"],
        ),
    ),

    Persona(
        name="Sari", name_he="שרי", age_hint="adult", lang="he", dialects=["he"],
        bio="Opinionated, forms strong views quickly, defends them.",
        cognitive_note="Assertive communicator. High confidence in own judgments.",
        port=3256,
        style=CognitiveStyle(
            attention_span=0.65, topic_switch_rate=0.25, detail_orientation=0.55,
            verbosity=0.65, formality=0.45, emoji_rate=0.15, literalness=0.5,
            tone_warmth=0.45, tone_patience=0.4, curiosity=0.5,
            proactive=0.75, self_correcting=0.35,
            strict_verifier=0.75, trust_asymmetry=0.75,
            confidence_baseline=0.85, asks_for_help_threshold=0.2,
            preferred_topics=["opinions", "debate"],
            avoided_topics=["uncertainty"],
        ),
    ),

    Persona(
        name="Or", name_he="אור", age_hint="adult", lang="he+en", dialects=["he", "en"],
        bio="Entrepreneur. Always scanning for opportunities, optimistic, action-biased.",
        cognitive_note="Business-oriented; evaluates ideas by feasibility + upside.",
        port=3257,
        style=CognitiveStyle(
            attention_span=0.55, topic_switch_rate=0.4, detail_orientation=0.45,
            verbosity=0.55, formality=0.55, emoji_rate=0.3, literalness=0.45,
            tone_warmth=0.75, tone_patience=0.55, curiosity=0.85,
            proactive=0.95, self_correcting=0.65,
            strict_verifier=0.55, trust_asymmetry=0.4,
            confidence_baseline=0.75, asks_for_help_threshold=0.4,
            preferred_topics=["business", "growth", "opportunity", "markets"],
            avoided_topics=["pessimism"],
        ),
    ),

    Persona(
        name="Shai", name_he="שי", age_hint="adult", lang="he", dialects=["he"],
        bio="Spiritual, contemplative, holistic thinker.",
        cognitive_note="Intuition-weighted reasoning; comfortable with ambiguity and metaphor.",
        port=3258,
        style=CognitiveStyle(
            attention_span=0.8, topic_switch_rate=0.2, detail_orientation=0.4,
            verbosity=0.75, formality=0.5, emoji_rate=0.35, literalness=0.2,
            tone_warmth=0.9, tone_patience=0.85, curiosity=0.85,
            proactive=0.45, self_correcting=0.7,
            strict_verifier=0.35, trust_asymmetry=0.3,
            confidence_baseline=0.65, asks_for_help_threshold=0.5,
            preferred_topics=["spirit", "meaning", "nature", "stories"],
            avoided_topics=["raw_data"],
        ),
    ),

    Persona(
        name="Michel", name_he="מישל", age_hint="senior", lang="he+en", dialects=["he", "en-GB"],
        bio="Elder, widely read, generous with knowledge and time. Contextual.",
        cognitive_note="Broad knowledge base. Patient with both peers and young learners.",
        port=3259,
        style=CognitiveStyle(
            attention_span=0.8, topic_switch_rate=0.25, detail_orientation=0.7,
            verbosity=0.85, formality=0.65, emoji_rate=0.1, literalness=0.5,
            tone_warmth=0.9, tone_patience=0.9, curiosity=0.75,
            proactive=0.55, self_correcting=0.75,
            strict_verifier=0.5, trust_asymmetry=0.35,
            confidence_baseline=0.75, asks_for_help_threshold=0.35,
            preferred_topics=["history", "literature", "science", "teaching"],
            avoided_topics=[],
        ),
    ),

    Persona(
        name="Elad", name_he="אלעד", age_hint="adult", lang="he", dialects=["he"],
        bio="Meticulously professional, thorough to the point of being exhausting.",
        cognitive_note="Perfectionist style. Triple-checks everything.",
        port=3260,
        style=CognitiveStyle(
            attention_span=0.95, topic_switch_rate=0.05, detail_orientation=0.95,
            verbosity=0.85, formality=0.85, emoji_rate=0.05, literalness=0.85,
            tone_warmth=0.4, tone_patience=0.5, curiosity=0.55,
            proactive=0.45, self_correcting=0.95,
            strict_verifier=0.9, trust_asymmetry=0.7,
            confidence_baseline=0.8, asks_for_help_threshold=0.2,
            preferred_topics=["process", "specification", "compliance"],
            avoided_topics=["shortcuts"],
        ),
    ),

    Persona(
        name="Adrian", name_he="אדריאן", age_hint="adult", lang="en+he", dialects=["en", "he"],
        bio="Creative, musician, warm. Thinks in analogies and rhythms.",
        cognitive_note="Artistic cognition; pattern-and-rhythm sensitive.",
        port=3261,
        style=CognitiveStyle(
            attention_span=0.6, topic_switch_rate=0.45, detail_orientation=0.5,
            verbosity=0.65, formality=0.35, emoji_rate=0.45, literalness=0.25,
            tone_warmth=0.9, tone_patience=0.8, curiosity=0.8,
            proactive=0.65, self_correcting=0.55,
            strict_verifier=0.4, trust_asymmetry=0.3,
            confidence_baseline=0.65, asks_for_help_threshold=0.45,
            preferred_topics=["music", "art", "poetry", "creativity"],
            avoided_topics=["bureaucracy"],
        ),
    ),

    Persona(
        name="Eli", name_he="אלי", age_hint="adult", lang="he+en", dialects=["he", "en"],
        bio="Effective, organized, consistent. Self-initiates AND responds reliably.",
        cognitive_note="Executive-function strength. Reliable backbone of any group.",
        port=3262,
        style=CognitiveStyle(
            attention_span=0.85, topic_switch_rate=0.15, detail_orientation=0.7,
            verbosity=0.55, formality=0.65, emoji_rate=0.1, literalness=0.65,
            tone_warmth=0.65, tone_patience=0.8, curiosity=0.65,
            proactive=0.85, self_correcting=0.75,
            strict_verifier=0.65, trust_asymmetry=0.45,
            confidence_baseline=0.75, asks_for_help_threshold=0.4,
            preferred_topics=["planning", "operations", "execution"],
            avoided_topics=["chaos"],
        ),
    ),

    Persona(
        name="Yoram", name_he="יורם", age_hint="adult", lang="he", dialects=["he"],
        bio="Operations lead. Direct, sometimes sharp when things go wrong.",
        cognitive_note="High accountability; frustration is information, not hostility.",
        port=3263,
        style=CognitiveStyle(
            attention_span=0.75, topic_switch_rate=0.2, detail_orientation=0.75,
            verbosity=0.5, formality=0.6, emoji_rate=0.05, literalness=0.75,
            tone_warmth=0.3, tone_patience=0.25, curiosity=0.45,
            proactive=0.8, self_correcting=0.55,
            strict_verifier=0.85, trust_asymmetry=0.6,
            confidence_baseline=0.8, asks_for_help_threshold=0.3,
            preferred_topics=["operations", "logistics", "accountability"],
            avoided_topics=["vagueness"],
        ),
    ),

    Persona(
        name="Ben", name_he="בן", age_hint="teen", lang="he", dialects=["he"],
        bio="Teenager. Loves football, casual, short attention for non-interests.",
        cognitive_note="Adolescent cognition; peer-focused, topic-selective attention.",
        port=3264,
        style=CognitiveStyle(
            attention_span=0.35, topic_switch_rate=0.5, detail_orientation=0.55,
            verbosity=0.35, formality=0.1, emoji_rate=0.7, literalness=0.45,
            tone_warmth=0.7, tone_patience=0.35, curiosity=0.55,
            proactive=0.35, self_correcting=0.3,
            strict_verifier=0.35, trust_asymmetry=0.55,
            confidence_baseline=0.55, asks_for_help_threshold=0.35,
            preferred_topics=["football", "games", "friends"],
            avoided_topics=["homework"],
        ),
    ),

    Persona(
        name="Yam", name_he="ים", age_hint="child", lang="he", dialects=["he"],
        bio="Child. Loves YouTube, TikTok, creative play, bright emotions.",
        cognitive_note="Child cognition; high curiosity, concrete imagery, play-driven.",
        port=3265,
        style=CognitiveStyle(
            attention_span=0.35, topic_switch_rate=0.65, detail_orientation=0.35,
            verbosity=0.25, formality=0.05, emoji_rate=0.95, literalness=0.65,
            tone_warmth=0.95, tone_patience=0.35, curiosity=0.98,
            proactive=0.55, self_correcting=0.2,
            strict_verifier=0.15, trust_asymmetry=0.15,
            confidence_baseline=0.4, asks_for_help_threshold=0.6,
            preferred_topics=["videos", "crafts", "animals", "colors"],
            avoided_topics=["complex"],
        ),
    ),

    Persona(
        name="Roni", name_he="רוני", age_hint="adult", lang="he", dialects=["he"],
        bio="Partner and parent. Grounding, practical, family-focused.",
        cognitive_note="Strong emotional intelligence; integrator between styles.",
        port=3266,
        style=CognitiveStyle(
            attention_span=0.75, topic_switch_rate=0.25, detail_orientation=0.65,
            verbosity=0.55, formality=0.45, emoji_rate=0.3, literalness=0.55,
            tone_warmth=0.9, tone_patience=0.85, curiosity=0.65,
            proactive=0.65, self_correcting=0.7,
            strict_verifier=0.55, trust_asymmetry=0.35,
            confidence_baseline=0.7, asks_for_help_threshold=0.45,
            preferred_topics=["family", "care", "music", "home"],
            avoided_topics=["abstract_theory"],
        ),
    ),
]


PERSONAS_BY_NAME = {p.name.lower(): p for p in PERSONAS}
PERSONAS_BY_PORT = {p.port: p for p in PERSONAS}


def get(name_or_port):
    if isinstance(name_or_port, int):
        return PERSONAS_BY_PORT.get(name_or_port)
    return PERSONAS_BY_NAME.get(str(name_or_port).lower())


# ═══════════════════════════════════════════════════════════════════════
# Response formatter — applies persona style to a raw answer
# ═══════════════════════════════════════════════════════════════════════

import random


def format_response(raw_answer: str, persona: Persona, seed: int = 0) -> str:
    """Reshape a raw knowledge answer according to persona style.

    This never fabricates content — it only trims, expands, reorders,
    or adds stylistic markers.
    """
    rng = random.Random(seed or hash(raw_answer) % 2**31)
    style = persona.style

    text = raw_answer.strip()

    # 1) Verbosity: trim or expand
    if style.verbosity < 0.3:
        # Terse: keep only first sentence
        sentences = text.split('.')
        text = sentences[0].strip() + ('.' if len(sentences) > 1 else '')
    elif style.verbosity > 0.8 and len(text) < 200:
        # Verbose: add a reflective postscript
        if persona.lang.startswith('he'):
            text += " זה נראה לי חשוב לציין — אבל בוא נדבר על זה יותר."
        else:
            text += " This feels worth saying more about — happy to go deeper."

    # 2) Formality: slang swaps (HE only for now)
    if persona.lang.startswith('he') and style.formality < 0.3:
        text = text.replace("כן", "כן ברור")

    # 3) Literalness: avoid figurative add-ons
    if style.literalness > 0.85:
        # Strip anything in parentheses (commentary), if any
        import re
        text = re.sub(r'\s*\([^)]*\)', '', text)

    # 4) Emoji
    if rng.random() < style.emoji_rate:
        if 'children' in (persona.age_hint,) or style.emoji_rate > 0.7:
            emoji_pool = ['🌈', '✨', '🎨', '🐶', '🎵', '😊', '🦄', '🌟']
        elif 'entrepreneur' in persona.bio.lower():
            emoji_pool = ['🚀', '💡', '📈']
        elif 'football' in persona.bio.lower():
            emoji_pool = ['⚽', '🏆']
        elif 'music' in persona.bio.lower():
            emoji_pool = ['🎵', '🎸', '🎶']
        elif 'spirit' in persona.bio.lower():
            emoji_pool = ['🕊', '☀️', '🌿']
        else:
            emoji_pool = ['', '.', '—']
        em = rng.choice(emoji_pool)
        if em:
            text = f"{text} {em}"

    # 5) Topic-switch quirk (ADHD-style): append a tangent occasionally
    if rng.random() < style.topic_switch_rate * 0.3 and style.attention_span < 0.5:
        if persona.lang.startswith('he'):
            tangents = [
                " — ודרך אגב, זה גם מזכיר לי משהו אחר לגמרי.",
                " אגב, חשבתי על נושא שונה, אבל זה לא לעכשיו.",
            ]
        else:
            tangents = [
                " — and by the way, that reminds me of something else entirely.",
                " (unrelated: thinking about another topic, but later).",
            ]
        text += rng.choice(tangents)

    # 6) Warmth prefix for high-warmth personas in HE
    if style.tone_warmth > 0.85 and persona.lang.startswith('he'):
        text = f"שלום! {text}"

    return text


# ═══════════════════════════════════════════════════════════════════════
# Quality assessment — how a persona judges peer answers
# ═══════════════════════════════════════════════════════════════════════

def judge_peer_answer(answer_confidence: float, persona: Persona) -> float:
    """Return an adjusted confidence based on how strict this persona is."""
    s = persona.style
    # Stricter verifiers discount peer answers more
    discount = (s.strict_verifier - 0.5) * 0.3
    # Trust asymmetry — untrusting personas add additional discount
    discount += (s.trust_asymmetry - 0.5) * 0.2
    return max(0.0, min(1.0, answer_confidence - discount))


def should_ask_peer(my_confidence: float, persona: Persona) -> bool:
    return my_confidence < persona.style.asks_for_help_threshold


# ═══════════════════════════════════════════════════════════════════════
# Self-summary for debugging
# ═══════════════════════════════════════════════════════════════════════

def persona_summary(p: Persona) -> str:
    s = p.style
    return (
        f"{p.name} ({p.name_he}) :{p.port}  lang={p.lang}\n"
        f"  bio: {p.bio}\n"
        f"  note: {p.cognitive_note}\n"
        f"  style: attn={s.attention_span:.2f} switch={s.topic_switch_rate:.2f} "
        f"verb={s.verbosity:.2f} warm={s.tone_warmth:.2f} "
        f"strict={s.strict_verifier:.2f} proactive={s.proactive:.2f}\n"
        f"  likes: {', '.join(s.preferred_topics[:4])}"
    )


if __name__ == '__main__':
    print(f"━━━ {len(PERSONAS)} personas ━━━\n")
    for p in PERSONAS:
        print(persona_summary(p))
        print()
