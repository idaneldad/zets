"""
בדיקה הנדסית רצינית — האם יש מודל מתמטי מבוסס אותיות יווניות/עבריות
שמחזיר נתונים משמעותיים על משפטים/מילים?

מתודולוגיה:
1. נגדיר ~10 operators מתמטיים
2. נריץ על corpus של משפטים אמיתיים + random + shuffled
3. נבדוק אם יש הבדל סטטיסטי משמעותי

אם אין הבדל — האופרטור לא תופס משהו אמיתי. דיווח כן.
"""

import math
import random
import statistics
from collections import Counter

PHI = (1 + 5**0.5) / 2
PHI_INV = 1 / PHI
PI = math.pi

# ═════════════════════════════════════════════════════════════════
#  GEMATRIA
# ═════════════════════════════════════════════════════════════════

HEBREW_GEMATRIA = {
    'א':1,'ב':2,'ג':3,'ד':4,'ה':5,'ו':6,'ז':7,'ח':8,'ט':9,'י':10,
    'כ':20,'ל':30,'מ':40,'נ':50,'ס':60,'ע':70,'פ':80,'צ':90,'ק':100,
    'ר':200,'ש':300,'ת':400,
    'ך':20,'ם':40,'ן':50,'ף':80,'ץ':90,
}

def hebrew_gematria(text):
    return sum(HEBREW_GEMATRIA.get(c, 0) for c in text)


# ═════════════════════════════════════════════════════════════════
#  OPERATORS — לא משתמש בלולאות שיכולות להיתקע
# ═════════════════════════════════════════════════════════════════

def op_identity(text):
    return text

def op_reverse(text):
    """שזירה — היפוך סדר"""
    return text[::-1]

def op_at_bash(text):
    """את-בש — קבלי"""
    letters = 'אבגדהוזחטיכלמנסעפצקרשת'
    mapping = {l: letters[-(i+1)] for i, l in enumerate(letters)}
    # טיפול באותיות סופיות
    final_to_regular = {'ך':'כ','ם':'מ','ן':'נ','ף':'פ','ץ':'צ'}
    result = []
    for c in text:
        c2 = final_to_regular.get(c, c)
        result.append(mapping.get(c2, c))
    return ''.join(result)

def op_al_bam(text):
    """אל-בם — חצי מול חצי"""
    letters = 'אבגדהוזחטיכלמנסעפצקרשת'
    n = len(letters) // 2
    mapping = {}
    for i in range(n):
        mapping[letters[i]] = letters[i+n]
        mapping[letters[i+n]] = letters[i]
    final_to_regular = {'ך':'כ','ם':'מ','ן':'נ','ף':'פ','ץ':'צ'}
    result = []
    for c in text:
        c2 = final_to_regular.get(c, c)
        result.append(mapping.get(c2, c))
    return ''.join(result)

def op_phi_position(text):
    """סידור לפי permutation של golden-angle (Fibonacci-like)"""
    if not text: return ""
    n = len(text)
    # Generate permutation using multiplication by PHI mod n
    # This is a known shuffle technique
    indices = [(int(i * PHI) % n) for i in range(n)]
    # Remove duplicates while preserving order
    seen = set()
    unique_indices = []
    for idx in indices:
        if idx not in seen:
            seen.add(idx)
            unique_indices.append(idx)
    # Add missing indices
    for i in range(n):
        if i not in seen:
            unique_indices.append(i)
    return ''.join(text[i] for i in unique_indices[:n])

def op_phi_inv_position(text):
    """הפוך — כפל ב-1/φ"""
    if not text: return ""
    n = len(text)
    indices = [(int(i * PHI_INV * n) % n) for i in range(n)]
    seen = set()
    unique_indices = []
    for idx in indices:
        if idx not in seen:
            seen.add(idx)
            unique_indices.append(idx)
    for i in range(n):
        if i not in seen:
            unique_indices.append(i)
    return ''.join(text[i] for i in unique_indices[:n])

def op_interleave(text):
    """שזירה — חצי ראשון + חצי שני מתחלפים"""
    if len(text) < 2: return text
    mid = len(text) // 2
    first, second = text[:mid], text[mid:]
    result = []
    for i in range(max(len(first), len(second))):
        if i < len(first): result.append(first[i])
        if i < len(second): result.append(second[i])
    return ''.join(result)

def op_inverse_interleave(text):
    """שזירה הפוכה"""
    if len(text) < 2: return text
    odds = text[0::2]
    evens = text[1::2]
    return odds + evens

def op_golden_split(text):
    """חלוקה ביחס-φ + rotate"""
    if not text: return ""
    split = int(len(text) * PHI_INV)
    return text[split:] + text[:split]

def op_mirror_halves(text):
    """הפוך כל חצי בנפרד"""
    if len(text) < 2: return text
    mid = len(text) // 2
    return text[:mid][::-1] + text[mid:][::-1]


OPERATORS = {
    'identity':          op_identity,
    'reverse':           op_reverse,
    'at_bash':           op_at_bash,
    'al_bam':            op_al_bam,
    'phi_position':      op_phi_position,
    'phi_inv_position':  op_phi_inv_position,
    'interleave':        op_interleave,
    'inverse_interleave': op_inverse_interleave,
    'golden_split':      op_golden_split,
    'mirror_halves':     op_mirror_halves,
}


# ═════════════════════════════════════════════════════════════════
#  METRICS
# ═════════════════════════════════════════════════════════════════

def metric_entropy(text):
    if not text: return 0
    freq = Counter(text)
    total = len(text)
    return -sum((c/total) * math.log2(c/total) for c in freq.values() if c > 0)

def metric_letter_diversity(text):
    if not text: return 0
    return len(set(text)) / len(text)

def metric_pair_preservation(original, transformed):
    """% of adjacent pairs preserved"""
    if len(original) < 2: return 1.0
    orig_pairs = set(zip(original, original[1:]))
    trans_pairs = set(zip(transformed, transformed[1:]))
    if not orig_pairs: return 1.0
    return len(orig_pairs & trans_pairs) / len(orig_pairs)

def metric_gematria_ratio(original, transformed):
    g1 = hebrew_gematria(original)
    g2 = hebrew_gematria(transformed)
    if g1 == 0: return 0
    return g2 / g1


# ═════════════════════════════════════════════════════════════════
#  CORPUS
# ═════════════════════════════════════════════════════════════════

REAL_TEXTS = [
    "בראשיתבראאלהיםאתהשמיםואתהארץ",
    "שמעישראליהוהאלהינויהוהאחד",
    "ואהבתלרעךכמוך",
    "המלאךהגאלאתימכלרע",
    "כימציוןתצאתורהודבריהוהמירושלים",
    "הודוליהוהכיטוב",
    "אנייהוהרפאך",
    "גבריאלמיכאלרפאלאוריאל",
    "חסדגבורהתפארתנצחהודיסודמלכות",
    "ברוךאתהיהוה",
    "יברכךיהוהוישמרך",
    "שירהמעלותממעמקיםקראתיך",
    "מזמורלדודיהוהרעילאאחסר",
    "אנאבכחגדלתימינךתתירצרורה",
    "קדושקדושקדושיהוהצבאות",
    "איןכאלהינואיןכאדונינו",
    "חכמהבינהדעת",
    "אהבהגבורהתפארת",
    "אמתויציבונכוןוקים",
    "ברוךשאמרוהיההעולם",
    "מודהאנילפניך",
    "אדוןעולםאשרמלך",
    "אשרייושביביתך",
    "תהלהלדודארוממךאלוהיהמלך",
    "שבחיירושליםאתיהוה",
    "ישראלבטחביהוה",
    "הגומללחייביםטובותגמלניכלטוב",
    "לעולםיהאאדםיראשמיםבסתרובגלוי",
    "אתהאחדושמךאחד",
    "ברוךהמקוםברוךהואברוךשנתןתורה",
]

def random_text(length):
    letters = list(HEBREW_GEMATRIA.keys())[:22]
    return ''.join(random.choices(letters, k=length))

def shuffled_text(text):
    chars = list(text)
    random.shuffle(chars)
    return ''.join(chars)

random.seed(42)
RANDOM_TEXTS = [random_text(random.randint(15, 40)) for _ in range(30)]
SHUFFLED_TEXTS = [shuffled_text(t) for t in REAL_TEXTS]


# ═════════════════════════════════════════════════════════════════
#  WELCH'S T-TEST (approximate)
# ═════════════════════════════════════════════════════════════════

def welch_t(vals1, vals2):
    if len(vals1) < 2 or len(vals2) < 2: return None
    m1, m2 = statistics.mean(vals1), statistics.mean(vals2)
    sd1, sd2 = statistics.stdev(vals1), statistics.stdev(vals2)
    n1, n2 = len(vals1), len(vals2)
    
    pooled_se = math.sqrt((sd1**2/n1) + (sd2**2/n2))
    if pooled_se == 0:
        return {'t': float('inf') if m1 != m2 else 0, 'm1': m1, 'm2': m2, 'sd1': sd1, 'sd2': sd2}
    t = (m1 - m2) / pooled_se
    return {'t': t, 'm1': m1, 'm2': m2, 'sd1': sd1, 'sd2': sd2}

def interpret_t(t):
    abs_t = abs(t)
    if abs_t > 3.0:  return "< 0.01 (significant)", True
    if abs_t > 2.0:  return "< 0.05 (significant)", True
    if abs_t > 1.5:  return "~ 0.10 (marginal)",    False
    return                  "> 0.10 (not sig)",     False


# ═════════════════════════════════════════════════════════════════
#  MAIN
# ═════════════════════════════════════════════════════════════════

print("═" * 82)
print("  בדיקה הנדסית: האם operators מתמטיים מגלים מבנה במשפטים?")
print("═" * 82)
print()
print(f"  Corpus: {len(REAL_TEXTS)} REAL, {len(RANDOM_TEXTS)} RANDOM, {len(SHUFFLED_TEXTS)} SHUFFLED")
print(f"  Operators: {len(OPERATORS)}")
print(f"  Statistical test: Welch's t (approx), sig thresh |t| > 2.0")
print()

# ──────────────────────────────────────────────────────────────
#  TEST A: Entropy — REAL vs RANDOM
# ──────────────────────────────────────────────────────────────
print("═" * 82)
print("  TEST A: Entropy אחרי operator — REAL vs RANDOM")
print("═" * 82)
print()
print(f"  {'Operator':<20} {'REAL mean':<12} {'RANDOM mean':<12} {'t':<10} {'p':<25} {'Sig':<5}")
print(f"  {'─'*20} {'─'*12} {'─'*12} {'─'*10} {'─'*25} {'─'*5}")

sig_count_A = 0
total_A = 0
for op_name, op_fn in OPERATORS.items():
    real_entropies = [metric_entropy(op_fn(t)) for t in REAL_TEXTS]
    rand_entropies = [metric_entropy(op_fn(t)) for t in RANDOM_TEXTS]
    r = welch_t(real_entropies, rand_entropies)
    if r:
        p_str, sig = interpret_t(r['t'])
        total_A += 1
        if sig: sig_count_A += 1
        mark = "✓" if sig else " "
        print(f"  {op_name:<20} {r['m1']:<12.3f} {r['m2']:<12.3f} {r['t']:<10.2f} {p_str:<25} {mark}")

print()
print(f"  ממצא: {sig_count_A}/{total_A} מהoperators מבחינים entropy בין real ל-random")

# ──────────────────────────────────────────────────────────────
#  TEST B: Pair Preservation — REAL vs SHUFFLED (same letters, diff order)
# ──────────────────────────────────────────────────────────────
print()
print("═" * 82)
print("  TEST B: Pair Preservation — REAL vs SHUFFLED (אותן אותיות, סדר שונה)")
print("═" * 82)
print()
print("  שואל: האם operator משמר זוגות אותיות שכנות שונה בטקסט מסודר לעומת מבולבל?")
print()
print(f"  {'Operator':<20} {'REAL_preserve':<15} {'SHUF_preserve':<15} {'t':<10} {'p':<25} {'Sig':<5}")
print(f"  {'─'*20} {'─'*15} {'─'*15} {'─'*10} {'─'*25} {'─'*5}")

sig_count_B = 0
total_B = 0
for op_name, op_fn in OPERATORS.items():
    if op_name == 'identity': continue
    real_pres = [metric_pair_preservation(t, op_fn(t)) for t in REAL_TEXTS]
    shuf_pres = [metric_pair_preservation(t, op_fn(t)) for t in SHUFFLED_TEXTS]
    r = welch_t(real_pres, shuf_pres)
    if r:
        p_str, sig = interpret_t(r['t'])
        total_B += 1
        if sig: sig_count_B += 1
        mark = "✓" if sig else " "
        print(f"  {op_name:<20} {r['m1']:<15.3f} {r['m2']:<15.3f} {r['t']:<10.2f} {p_str:<25} {mark}")

print()
print(f"  ממצא: {sig_count_B}/{total_B} מהoperators מבחינים real מ-shuffled ב-pair_preservation")
print("  חשוב: זה היה הטסט הקריטי. REAL vs SHUFFLED בודק האם יש משמעות לסדר.")

# ──────────────────────────────────────────────────────────────
#  TEST C: Gematria Ratio — האם יש יחסי φ/π?
# ──────────────────────────────────────────────────────────────
print()
print("═" * 82)
print("  TEST C: Gematria Ratio — האם operators יוצרים יחסים מיוחדים?")
print("═" * 82)
print()
print("  בוחן: ratio = gematria(transformed) / gematria(original)")
print("  שואל: האם ה-ratio הממוצע קרוב ל-φ, π, או ערך מעניין אחר?")
print()

golden_targets = {
    'φ (1.618)':   PHI,
    '1/φ (0.618)': PHI_INV,
    'π (3.14)':    PI,
    '1/π (0.318)': 1/PI,
    '2':           2.0,
    '1/2':         0.5,
    '1 (no change)': 1.0,
}

print(f"  {'Operator':<20} {'REAL ratio':<12} {'closest':<15} {'dist':<8} {'RAND ratio':<12} {'closest':<15} {'dist'}")
print(f"  {'─'*20} {'─'*12} {'─'*15} {'─'*8} {'─'*12} {'─'*15} {'─'*6}")

for op_name, op_fn in OPERATORS.items():
    real_ratios = [metric_gematria_ratio(t, op_fn(t)) for t in REAL_TEXTS if hebrew_gematria(t) > 0]
    rand_ratios = [metric_gematria_ratio(t, op_fn(t)) for t in RANDOM_TEXTS if hebrew_gematria(t) > 0]
    
    if not real_ratios or not rand_ratios: continue
    
    m_real = statistics.mean(real_ratios)
    m_rand = statistics.mean(rand_ratios)
    
    real_closest = min(golden_targets.items(), key=lambda kv: abs(m_real - kv[1]))
    rand_closest = min(golden_targets.items(), key=lambda kv: abs(m_rand - kv[1]))
    
    print(f"  {op_name:<20} {m_real:<12.3f} {real_closest[0]:<15} {abs(m_real-real_closest[1]):<8.3f} {m_rand:<12.3f} {rand_closest[0]:<15} {abs(m_rand-rand_closest[1]):.3f}")

# ──────────────────────────────────────────────────────────────
#  TEST D: הקריטי — האם יחסי φ/π **ספציפיים לטקסט אמיתי** יותר מרנדומלי?
# ──────────────────────────────────────────────────────────────
print()
print("═" * 82)
print("  TEST D: האם יחסי φ/π **ספציפיים** לטקסט אמיתי?")
print("═" * 82)
print()
print("  שואל: האם |ratio - φ| קטן יותר בטקסט אמיתי מאשר ברנדומלי?")
print()
print(f"  {'Operator':<20} {'|real-φ|':<12} {'|rand-φ|':<12} {'t':<10} {'p':<20} {'Sig':<5}")
print(f"  {'─'*20} {'─'*12} {'─'*12} {'─'*10} {'─'*20} {'─'*5}")

sig_count_D = 0
total_D = 0
for op_name, op_fn in OPERATORS.items():
    real_ratios = [metric_gematria_ratio(t, op_fn(t)) for t in REAL_TEXTS if hebrew_gematria(t) > 0]
    rand_ratios = [metric_gematria_ratio(t, op_fn(t)) for t in RANDOM_TEXTS if hebrew_gematria(t) > 0]
    
    if not real_ratios or not rand_ratios: continue
    
    real_dist_from_phi = [abs(r - PHI) for r in real_ratios]
    rand_dist_from_phi = [abs(r - PHI) for r in rand_ratios]
    
    result = welch_t(real_dist_from_phi, rand_dist_from_phi)
    if result:
        p_str, sig = interpret_t(result['t'])
        total_D += 1
        if sig: sig_count_D += 1
        mark = "✓" if sig else " "
        print(f"  {op_name:<20} {result['m1']:<12.3f} {result['m2']:<12.3f} {result['t']:<10.2f} {p_str:<20} {mark}")

print()
print(f"  ממצא: {sig_count_D}/{total_D} מראים קרבה שונה ל-φ בין real לrandom")

# ──────────────────────────────────────────────────────────────
#  Summary
# ──────────────────────────────────────────────────────────────
print()
print("═" * 82)
print("  סיכום סופי — דיווח כנה")
print("═" * 82)
print()
print(f"  TEST A: {sig_count_A}/{total_A} operators מבחינים REAL מ-RANDOM ב-entropy")
print(f"  TEST B: {sig_count_B}/{total_B} operators מבחינים REAL מ-SHUFFLED ב-pair_preservation")
print(f"  TEST D: {sig_count_D}/{total_D} operators מראים 'קרבה ל-φ' ספציפית לREAL")
print()

if sig_count_A > total_A / 2:
    print("  ✓ Entropy בטקסט אמיתי שונה מ-random (צפוי — אותיות לא מתפלגות אחיד)")
else:
    print("  ✗ אין הבדל משמעותי ב-entropy — זה הפתעה")

if sig_count_B > total_B / 2:
    print("  ✓ Pair_preservation מבחין בין טקסט עם סדר למבולבל (צפוי)")
else:
    print("  ✗ אין הבדל ב-pair preservation — operators לא תופסים 'סדר משמעותי'")

if sig_count_D > 0:
    print(f"  ⚠ {sig_count_D} operators מראים קרבה ל-φ ספציפית. צריך לבחון!")
    print("    (יכול להיות: (א) אמיתי, (ב) artifact של התפלגות אותיות, (ג) מקרי)")
else:
    print("  ✗ שום operator לא מראה קרבה ל-φ שספציפית לטקסט אמיתי")
    print("    → אם יש יחסי φ — הם נובעים מההתפלגות של אותיות עבריות, לא ממשמעות")

print()
print("═" * 82)
print("  תובנה עיקרית")
print("═" * 82)
print()
print("  השאלה הייתה: 'האם יש מודל מתמטי על המוח, שימוש ב-operators כמו שזירה + פאי'?")
print()
print("  הבדיקה בדקה את ההיפותזה שמודלים מתמטיים מגלים משהו מיוחד בטקסט אמיתי.")
print("  התוצאה המדויקת:")
print(f"    - operators שמבחינים טקסט מסודר ממבולבל:  ~{sig_count_B} מתוך {total_B} (pair preservation)")
print(f"    - operators שמראים יחסי φ 'נבחרים':        {sig_count_D} מתוך {total_D}")
print()
print("  מסקנה הגונה:")
print("    - פונקציות שזירה/reversal **מבחינות** טקסט מסודר מ-random (טריוויאלי)")
print("    - אבל אין הוכחה שהן 'מודל של המוח' או 'חוקי φ נסתרים'")
print("    - כל תוצאה של יחסי φ/π — צריך לבדוק אם היא artifact סטטיסטי של")
print("      התפלגות האותיות, לא של המשמעות")
print()
print("  המלצה:")
print("    - מודל מתמטי של המוח = 10+ operators שמגלים **דבר אחד קונסיסטנטי**")
print("    - לא שזירה לבד, לא פאי לבד")  
print("    - עד שאין דבר כזה → לא לקרוא לזה 'מודל'")
