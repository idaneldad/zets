"""
שיחת עומק עם ה-brain — 5 שלבים שבודקים התנהגויות שונות.
כל שלב חושף היבט אחר של ניהול המידע.
"""
import sys, os
sys.path.insert(0, os.path.dirname(__file__))
from brain_sim_v2 import KnowledgeBase, Brain, LexEntry, Article, Procedure, Episode, Skill
import json, time

# ───────────────────────────────────────────────────────────────
#  SETUP — teaching the brain a reasonable base of knowledge
# ───────────────────────────────────────────────────────────────
kb = KnowledgeBase()
brain = Brain(kb)

# Lexicon — vocabulary 3 שפות
words = [
    ("water",     {"he":"מים","ar":"ماء"}, ["liquid","aqua","fluid","H2O"], "noun"),
    ("fire",      {"he":"אש","ar":"نار"},  ["flame","blaze","combustion"],   "noun"),
    ("tea",       {"he":"תה","ar":"شاي"},  ["brew","infusion"],              "noun"),
    ("coffee",    {"he":"קפה","ar":"قهوة"},["brew","espresso","java"],       "noun"),
    ("boil",      {"he":"להרתיח"},          ["heat","bubble"],                "verb"),
    ("hot",       {"he":"חם"},              ["warm","heated","scalding"],     "adj"),
    ("cold",      {"he":"קר"},              ["chilly","freezing"],            "adj"),
    ("love",      {"he":"אהבה","ar":"حب"}, ["affection","adore","care"],     "noun"),
    ("war",       {"he":"מלחמה","ar":"حرب"},["conflict","battle","combat"],  "noun"),
    ("peace",     {"he":"שלום","ar":"سلام"},["harmony","tranquility"],       "noun"),
    ("human",     {"he":"אדם","ar":"إنسان"},["person","people"],             "noun"),
    ("machine",   {"he":"מכונה","ar":"آلة"}, ["device","apparatus"],          "noun"),
    ("learn",     {"he":"ללמוד"},           ["study","acquire"],              "verb"),
    ("teach",     {"he":"ללמד"},            ["instruct","educate"],           "verb"),
    ("memory",    {"he":"זיכרון"},          ["recollection","recall"],        "noun"),
    ("brain",     {"he":"מוח","ar":"دماغ"},["mind","cerebrum"],              "noun"),
    ("language",  {"he":"שפה","ar":"لغة"}, ["tongue","speech"],              "noun"),
    ("book",      {"he":"ספר","ar":"كتاب"},["volume","tome"],                "noun"),
    ("dream",     {"he":"חלום","ar":"حلم"},["vision","fantasy","reverie"],  "noun"),
    ("sleep",     {"he":"לישון"},           ["rest","slumber"],               "verb"),
]
for word, trans, syns, pos in words:
    kb.add_lexicon(LexEntry(word, word, "en", trans, syns, pos, "wiktionary"))

# Encyclopedia
kb.add_article(Article(
    "Water", "en",
    [
        {"name":"Intro", "sentences":["Water is a transparent, tasteless liquid.","Its chemical formula is H2O."]},
        {"name":"Chemistry", "sentences":["Water molecules form hydrogen bonds.","Boiling point is 100°C at sea level."]},
    ],
    {"water","h2o","liquid"}, "wikipedia"
))
kb.add_article(Article(
    "Tea", "en",
    [
        {"name":"Intro", "sentences":["Tea is a brewed beverage from the camellia plant.","It originated in China."]},
    ],
    {"tea","beverage","brew"}, "wikipedia"
))
kb.add_article(Article(
    "Brain", "en",
    [
        {"name":"Intro", "sentences":["The brain is the organ of thought.","Humans have ~86 billion neurons."]},
        {"name":"Memory", "sentences":["Hippocampus encodes episodic memories.","Neocortex stores semantic knowledge."]},
    ],
    {"brain","memory","neuron","human","mind"}, "wikipedia"
))
kb.add_article(Article(
    "Dream", "en",
    [
        {"name":"Intro", "sentences":["Dreams are experiences during sleep.","REM sleep is associated with vivid dreams."]},
    ],
    {"dream","sleep","rem"}, "wikipedia"
))

# Procedures
kb.add_procedure(Procedure(
    "make_tea",
    ["Boil water","Place tea leaves in cup","Pour hot water","Steep 3 minutes","Serve"],
    ["water","tea","heat_source","cup"],
    "Hot flavored liquid ready to drink",
    "manual"
))
kb.add_procedure(Procedure(
    "make_coffee",
    ["Grind beans","Heat water to 93°C","Pour over grounds","Wait for extraction","Serve"],
    ["water","coffee","heat_source"],
    "Hot coffee ready",
    "manual"
))

print("═" * 85)
print("  SETUP — Brain has been taught:")
print(f"    {len(kb.lexicon)} words | {len(kb.encyclopedia)} articles | {len(kb.procedures)} procedures")
print("═" * 85)


# ───────────────────────────────────────────────────────────────
#  שלב 1 — שאלה מזהה, ידע זמין
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  STAGE 1 — שאלה פשוטה שהמוח יודע לענות עליה")
print("█"*85)
q = "what is water?"
print(f"\n  🗣 USER: {q}")
r = brain.process(q)
print(f"\n  🤖 BRAIN responds: {r['response']}")
print(f"\n  📊 Regions activated: {len(r['regions_active'])}")
for n, lvl in sorted(r['regions_active'].items(), key=lambda x: -x[1])[:5]:
    bar = "█" * int(lvl * 15)
    print(f"      {n:<16} {lvl:.2f} {bar}")
print(f"\n  🔎 Queries made:  {r['queries_made']}")
print(f"  💭 Working mem:   {r['working_memory']}")
print(f"\n  ━ ניתוח: המוח הפעיל semantic_gateway + hippocampus.")
print(f"    query flow: lexical → wernicke → semantic_gw → KB lookup")
print(f"    הידע קיים, התשובה הוחזרה. אין skill עדיין כי זה ראשון.")


# ───────────────────────────────────────────────────────────────
#  שלב 2 — שאלה שדורשת שני מושגים (cross-concept)
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  STAGE 2 — שאלה קומפוזיטית: brain + memory")
print("█"*85)
q = "how does the brain store memory?"
print(f"\n  🗣 USER: {q}")
r = brain.process(q)
print(f"\n  🤖 BRAIN: {r['response']}")
print(f"  🔎 Queries: {r['queries_made']}")
print(f"  💭 Working mem: {r['working_memory']}")
print(f"\n  ━ ניתוח: המוח מצא 3 lemmas ('brain','store','memory')")
print(f"    lexicon נתן תרגומים, encyclopedia נתן article 'Brain'")
print(f"    MA כרגע: מחזיק שני concepts בעבודה — ({r['working_memory']})")
print(f"    אבל המוח לא מחבר אותם semantically — הוא רק אוסף.")
print(f"    זה חולשה ברורה — יש הגעה אך לא הבנה.")


# ───────────────────────────────────────────────────────────────
#  שלב 3 — שאלה שדורשת cross-lingual (שפה שהוא לא מכיר)
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  STAGE 3 — מילה בשפה לא מוכרת (ערבית): 'ماء' (water)")
print("█"*85)
q = "ماء is important"
print(f"\n  🗣 USER: {q}")
r = brain.process(q)
print(f"\n  🤖 BRAIN: {r['response']}")
print(f"  🔎 Queries: {r['queries_made']}")
print(f"\n  ━ ניתוח קריטי:")
print(f"    המוח ניסה lookup של 'ماء' ב-'en' — לא מצא.")
print(f"    כי ה-lookup הנוכחי טיפש — הוא מחפש ישיר במקום להפעיל reverse translation.")
print(f"    כאן נכשל. צריך שהמוח יחפש: האם 'ماء' מופיעה כ-translation של משהו?")
print(f"    זה גילוי חשוב — ה-brain צריך לולאה של attempts על lookups שנכשלים.")

# להדגמה — נעשה reverse lookup ידני כדי להראות את הפוטנציאל
print(f"\n  🔧 FIX demonstration — reverse lookup manually:")
found = None
for key, entry in kb.lexicon.items():
    if "ماء" in entry.translations.values():
        found = entry
        break
if found:
    print(f"    Found: 'ماء' is translation of '{found.word}' → meaning {found.translations}")
    print(f"    Had the brain tried reverse lookup → it would know!")


# ───────────────────────────────────────────────────────────────
#  שלב 4 — חזרה 4 פעמים על אותה שאלה — skill emergence
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  STAGE 4 — חזרה על שאלת water 4 פעמים → צפוי skill emergence")
print("█"*85)
for i in range(4):
    brain.process("what is water?")
print(f"\n  אחרי 4 repetitions: episodes = {len(kb.episodes)}")

# consolidate (sleep)
print(f"\n  🌙 Running CONSOLIDATE (sleep cycle)...")
result = kb.consolidate()
print(f"    ✓ new skills emerged: {result['new_skills']}")
print(f"    ✓ skills in KB: {list(kb.skills.keys())}")
if kb.skills:
    first_skill = list(kb.skills.values())[0]
    print(f"    ✓ confidence: {first_skill.confidence:.2f} ({first_skill.success_count}/{first_skill.attempt_count})")

# query again — skill kicks in
print(f"\n  🗣 USER (5th time): what is water?")
r = brain.process("what is water?")
print(f"  🔎 Queries made:  {r['queries_made']}")
print(f"  ━ שים לב: עכשיו יש 'skill:...' ב-queries לפני lex/art")
print(f"    זה ה-Hebbian — המוח זיהה pattern חוזר, קצר מסלולים")
# show Hebbian weights
print(f"\n  Hebbian connection weights (>0.5 means strengthened):")
for (a, b), w in sorted(brain.connections.items(), key=lambda x: -x[1])[:5]:
    if w > 0.5:
        bar = "▓" * int((w - 0.5) * 40)
        print(f"    {a:>16} → {b:<16}  {w:.3f} {bar}")


# ───────────────────────────────────────────────────────────────
#  שלב 5 — שאלה שאין לה תשובה — האם המוח יודע שהוא לא יודע?
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  STAGE 5 — שאלה על מושג שלא מולמד: 'what is quasar?'")
print("█"*85)
q = "what is quasar?"
print(f"\n  🗣 USER: {q}")
r = brain.process(q)
print(f"\n  🤖 BRAIN: {r['response']}")
print(f"  🔎 Queries made: {r['queries_made']}")
print(f"\n  ━ ניתוח: המוח lookup-ed 'quasar' ב-lexicon — לא מצא.")
print(f"    אבל הוא ניסה רק 'quasar' בלי synonyms, בלי spreading, בלי reverse.")
print(f"    החולשה: המוח מצהיר 'I don't know' אחרי query אחד טריוויאלי.")
print(f"    במוח אנושי זה לא מה שקורה — default_mode נכנס, associations לעבודה,")
print(f"    הוא מנסה 'זה נשמע כמו...' או 'אולי ב-...'.")
print(f"    כאן נחשף שחסר לנו: Default Mode כ-fallback חיפוש רחב.")


# ───────────────────────────────────────────────────────────────
#  סיכום — מה קרה בכל שלב, מה חזק ומה חלש
# ───────────────────────────────────────────────────────────────
print("\n\n" + "█"*85)
print("  FINAL — מצב הbrain וה-KB אחרי 5 שלבים")
print("█"*85)
stats = kb.stats()
print(f"\n  KB stats:")
for k, v in stats.items():
    print(f"    {k}: {v}")

print(f"\n  Episodes (last 4):")
for ep in kb.episodes[-4:]:
    print(f"    [{ep.outcome:<12}] {ep.input[:40]:<40} → {ep.queries_made[:3]}")

print(f"\n  Hebbian connections that got stronger:")
baseline = 0.5
strengthened = [(k, w) for k, w in brain.connections.items() if w > baseline]
if strengthened:
    for (a, b), w in sorted(strengthened, key=lambda x: -x[1]):
        print(f"    {a:>16} → {b:<16}  0.500 → {w:.3f}")
else:
    print("    (none yet — need more repeated successful patterns)")
