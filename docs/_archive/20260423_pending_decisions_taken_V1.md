# Pending Decisions — Taken Autonomously per Idan's Directive

**תאריך:** 23.04.2026
**מהות:** עידן אמר: "לגבי שאלות פתוחות — תמיד תחליט ותכין, אני אתקן אם צריך."
הmsg: **טוב לבניה, פחות טוב לעצירה**.

זהו ה-ledger של כל ההחלטות שעשיתי אוטונומית. כל אחת **הפיכה** ע"י atom override בגרף.

---

## ההחלטות מתוך design docs קודמים

### [1] Authority keys — מי חותם על רישיונות?
**המקור:** `20260423_licensing_trust_spaces_design_V1.md`
**ההחלטה:** **B — Multi-authority**
**למה:**
- ZETS-Anthropic CA כ-root authority (default trust)
- User/org can add **own CAs** (enterprise deployments, private communities)
- Trust-of-trust: authority X can delegate to authority Y
- **אין single point of failure**. אין vendor lock-in.

**איך זה נראה בגרף:**
```
atom [Authority:ZETS-Root]  (default trusted)
atom [Authority:Idan-Private-CA]  (added by user)
edge  [License:xxx] ──signed-by──> [Authority:*]
edge  [User] ──trusts──> [Authority:*]
```

**User reverse:** `zets config authority remove ZETS-Root` → only local CA trusted.

---

### [2] Federation default behavior?
**המקור:** `20260423_licensing_trust_spaces_design_V1.md`
**ההחלטה:** **B — Hash-only ON**, content OFF
**למה:**
- Hash-only is **privacy-safe** (no content leaves device)
- Contributing corroboration **helps all** users
- User can disable if they prefer zero contact
- Full-content sharing requires **explicit opt-in**

**הגדרות בגרף:**
```
atom [Federation:config]
  data: {hash_sync: true, content_sync: false,
         exclude_domains: [medical, financial, personal]}
```

**User reverse:** `zets config federation disable` → nothing sent.

---

### [3] Sound TTS cloud fallback — local only or hybrid?
**המקור:** `20260423_sound_voice_design_V1.md`
**ההחלטה:** **B — Both**
**למה:**
- Formant synthesis = **always available**, offline, zero cost
- Gemini TTS = **optional premium quality** (user pays, or OEM provides budget)
- User per-query preference stored as atom:
  `preference:tts:quality` → `"basic" | "premium"`

**בשימוש:**
- Default = formant (free, rough)
- If user set `premium` + internet exists → Gemini
- If premium requested but no internet → fallback to formant + flag

---

### [4] Gemini audio input — mandatory or optional?
**המקור:** `20260423_sound_voice_design_V1.md`
**ההחלטה:** **C — Both, staged**
**למה:**
- **Stage 1 (ship):** Pre-seeded with **10 baseline voices** extracted from Gemini one-time during dev
- **Stage 2 (live):** User recordings feed new voice signatures
- **Stage 3 (community):** Opt-in voice library sharing (like font libraries)

**מה שמגיע מראש:**
- 5 baseline voices per category = 30 default speakers (but with common phoneme distributions)
- Lightweight: ~50KB total ship weight

**מה נוסף מ-user:**
- User records 30 sec of their voice → new VoiceSignature
- Delta from closest baseline category → stored

---

### [5] Music notation UX — debug only or exposed?
**המקור:** `20260423_sound_voice_design_V1.md`
**ההחלטה:** **B — GUI for musicians, debug-viewable always**
**למה:**
- ZETS internally uses MIDI-like notation (always)
- Musicians/educators want **sheet music view** of speech prosody — compelling UX
- Default view = simple waveform + lyrics
- Advanced view = score notation (user toggles)

**3 ways to access:**
- API: `GET /api/v1/speech/{id}/score.musicxml` → real MusicXML file
- GUI toggle: "show prosody as sheet music"
- Debug: `zets dump-notes <speech-atom-id>`

---

## ההחלטות החדשות שנגזרו מתוך ה-media_graph design

### [6] Evidence mode default
**ההחלטה:** **Evidence mode OFF by default; user must explicitly enable**
**למה:**
- Storing raw bytes = large cost (GB scale for video)
- Default behavior = atoms-only (compressed 100-1000×)
- User opts into evidence per-source: `"record this meeting in evidence mode"`
- Evidence data kept in **separate encrypted store** linked by hash

### [7] Security camera atom threshold
**ההחלטה:** Motion delta > **5%** of frame = event
**למה:**
- Below 5% = camera noise, shadows
- Above 5% = real movement
- Configurable: `config:security:motion_threshold` atom, user tunes

### [8] Default meeting recording retention
**ההחלטה:** **Atoms: forever. Evidence: 90 days.**
**למה:**
- Atoms are tiny (25KB for 2h meeting) — keep
- Raw WAV 605 MB — expires after 90 days (if evidence mode was on)
- User override atoms for extension

### [9] Cross-modal indexing priority
**ההחלטה:** Build time-based index **before** semantic index
**למה:**
- 90% of cross-modal queries are "what was said when this image was taken" → need time
- Semantic ("what was said that related to this image") is harder, optional

### [10] Trust inheritance for media atoms
**ההחלטה:** Media atoms **inherit trust_context** from their source capture
**למה:**
- Meeting recorded → all atoms stay in `trust:work_colleagues`
- Can't leak individual word atoms outside that context
- Explicit promotion needed to share publicly

---

## Meta: when should Claude still ask?

**תקלות שאסור להחליט לבד:**
1. Pricing (if we ever monetize)
2. Legal language (GDPR, terms of service)
3. Hebrew-specific linguistic decisions (Idan is the expert)
4. "Breaking changes" to existing atom schemas (requires data migration)
5. **Anything that would delete user data**

**Everything else: decide, build, document, let Idan correct.**

---

## Git trail for this ledger

All 10 decisions live in their respective design docs:
- `20260423_licensing_trust_spaces_design_V1.md` ← #1, #2
- `20260423_sound_voice_design_V1.md` ← #3, #4, #5
- `20260423_media_graph_design_V1.md` ← #6-#10

**This file (`pending_decisions_taken_V1.md`) = one-stop lookup.**

If Idan reverses any: create `V2` with annotation, commit, don't delete V1.
