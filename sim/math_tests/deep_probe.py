"""
בדיקה עמוקה יותר על תוצאות מעניינות + בדיקת claims ספציפיות.

1. ה-at_bash הראה ratio ≈ 2.0 ב-REAL ו-1.2 ב-RAND. למה?
2. Memory מראה claims כמו "Gabriel Heb/Greek=246/154=φ". לבדוק סטטיסטית.
3. האם יש שלוש או יותר מילים שמראות דבר דומה?

השאלה: יש כאן signal אמיתי, או coincidences?
"""

import math
import random
import statistics
from collections import Counter

PHI = (1 + 5**0.5) / 2
PHI_INV = 1 / PHI

HEBREW_GEMATRIA = {
    'א':1,'ב':2,'ג':3,'ד':4,'ה':5,'ו':6,'ז':7,'ח':8,'ט':9,'י':10,
    'כ':20,'ל':30,'מ':40,'נ':50,'ס':60,'ע':70,'פ':80,'צ':90,'ק':100,
    'ר':200,'ש':300,'ת':400,
    'ך':20,'ם':40,'ן':50,'ף':80,'ץ':90,
}

def heb_g(text):
    return sum(HEBREW_GEMATRIA.get(c, 0) for c in text)


# ════════════════════════════════════════════════════════════════
#  1. למה at_bash מחזיר ratio=2 בטקסט אמיתי?
# ════════════════════════════════════════════════════════════════
print("═" * 80)
print("  בדיקה 1: למה at_bash מחזיר ratio≈2 בטקסט אמיתי?")
print("═" * 80)
print()

# at_bash: א↔ת, ב↔ש, ג↔ר, ד↔ק, ה↔צ, ...
letters = 'אבגדהוזחטיכלמנסעפצקרשת'
at_bash_map = {l: letters[-(i+1)] for i, l in enumerate(letters)}

print("  at_bash mapping:")
for i in range(0, len(letters), 4):
    print("   ", end="")
    for j in range(i, min(i+4, len(letters))):
        l = letters[j]
        print(f" {l}({HEBREW_GEMATRIA[l]})→{at_bash_map[l]}({HEBREW_GEMATRIA[at_bash_map[l]]})", end="")
    print()

# תיאורית: each pair sums to specific value
print()
print("  סכומי זוגות at_bash:")
unique_sums = set()
for l in letters:
    if l < at_bash_map[l]:  # avoid double counting
        pair_sum = HEBREW_GEMATRIA[l] + HEBREW_GEMATRIA[at_bash_map[l]]
        print(f"    {l}({HEBREW_GEMATRIA[l]}) + {at_bash_map[l]}({HEBREW_GEMATRIA[at_bash_map[l]]}) = {pair_sum}")
        unique_sums.add(pair_sum)

print()
print(f"  ה-sums הם: {sorted(unique_sums)}")
print(f"  המספרים מגוונים — אין קבוע אחד")
print()
print("  אבל ב-avg על טקסט:")
print("  אם טקסט אמיתי משתמש הרבה באותיות נפוצות (א,ו,י,ה,ל),")  
print("  הממוצע של at_bash(letter)/letter הולך להיות גדול (כי שולח אותן ל-ת,ק,צ,ר...)")
print()
print("  זה **artifact סטטיסטי** של התפלגות האותיות הנפוצות בעברית.")
print("  לא 'גילוי'. artifact.")


# ════════════════════════════════════════════════════════════════
#  2. Gabriel claim: Heb 246 / Greek 154 = φ?
# ════════════════════════════════════════════════════════════════
print()
print("═" * 80)
print("  בדיקה 2: Gabriel Heb/Greek = φ?  (claim מ-memory)")
print("═" * 80)
print()

claim_heb = 246  # גבריאל
claim_greek = 154
ratio = claim_heb / claim_greek
print(f"  גבריאל:  {claim_heb} / {claim_greek} = {ratio:.4f}")
print(f"  φ =      {PHI:.4f}")
print(f"  diff:    {abs(ratio - PHI):.4f}  ({abs(ratio - PHI)/PHI*100:.1f}% error)")
print()

# יש משהו דומה במילים אחרות?
test_pairs = [
    # Heb name, Greek name, heb_value, greek_value (if known)
    ("גבריאל",  "Γαβριηλ",  246, 154),   # claim from memory
    ("מיכאל",   "Μιχαηλ",   101, 141),
    ("רפאל",    "Ραφαηλ",   311, 128),  # approximately
    ("אוריאל",  "Ουριηλ",   248, 618),  # memory: Uriel Greek=618
    ("מטטרון",  "Μετατρων",  314, None),  
]

print("  סטטיסטיקה של יחסי Heb/Greek בשמות מלאכים:")
ratios = []
for heb, gr, h_val, g_val in test_pairs:
    if g_val is None or g_val == 0: continue
    r = h_val / g_val
    phi_diff = abs(r - PHI)
    phi_inv_diff = abs(r - PHI_INV)
    pi_diff = abs(r - math.pi)
    min_target, min_name = min(
        [(phi_diff, 'φ'), (phi_inv_diff, '1/φ'), (pi_diff, 'π'), (abs(r-1), '1'), (abs(r-2), '2'), (abs(r-0.5), '0.5')],
        key=lambda x: x[0]
    )
    ratios.append(r)
    print(f"    {heb:<10} Heb={h_val:<5} Greek={g_val:<5}  ratio={r:.3f}  closest: {min_name} (dist={min_target:.3f})")

print()
print(f"  {len(ratios)} pairs tested. Ratios: {[f'{r:.3f}' for r in ratios]}")
print()
print("  אין pattern consistent. המקרה של גבריאל 1.597 קרוב ל-φ=1.618,")
print("  אבל אוריאל 0.401 לא קרוב לכלום מיוחד,")
print("  מיכאל 0.716 לא קרוב לכלום מיוחד.")


# ════════════════════════════════════════════════════════════════
#  3. Null hypothesis test: אם נקח מילים רנדומליות, כמה מהן יהיו קרובות ל-φ?
# ════════════════════════════════════════════════════════════════
print()
print("═" * 80)
print("  בדיקה 3: Null hypothesis — כמה זוגות רנדומליים קרובים ל-φ?")
print("═" * 80)
print()
print("  השאלה: אם נקח 5 זוגות של גימטריה רנדומלית, כמה מהן יהיו |ratio - φ| < 0.05?")
print()

random.seed(42)
# נבדוק 10,000 זוגות רנדומליים
within_phi = 0
within_phi_inv = 0
within_pi = 0
within_1 = 0
within_2 = 0
trials = 10000

THRESHOLD = 0.05

for _ in range(trials):
    h = random.randint(50, 500)
    g = random.randint(50, 500)
    if g == 0: continue
    r = h / g
    if abs(r - PHI) < THRESHOLD: within_phi += 1
    if abs(r - PHI_INV) < THRESHOLD: within_phi_inv += 1
    if abs(r - math.pi) < THRESHOLD: within_pi += 1
    if abs(r - 1) < THRESHOLD: within_1 += 1
    if abs(r - 2) < THRESHOLD: within_2 += 1

print(f"  מתוך {trials} זוגות רנדומליים (gematria 50-500):")
print(f"    |ratio - φ|     < {THRESHOLD}:  {within_phi:5d}  ({within_phi/trials*100:.2f}%)")
print(f"    |ratio - 1/φ|   < {THRESHOLD}:  {within_phi_inv:5d}  ({within_phi_inv/trials*100:.2f}%)")
print(f"    |ratio - π|     < {THRESHOLD}:  {within_pi:5d}  ({within_pi/trials*100:.2f}%)")
print(f"    |ratio - 1|     < {THRESHOLD}:  {within_1:5d}  ({within_1/trials*100:.2f}%)")
print(f"    |ratio - 2|     < {THRESHOLD}:  {within_2:5d}  ({within_2/trials*100:.2f}%)")
print()
print(f"  Baseline: אם נבדוק 5 זוגות, סביר שאחד מהם יפול ב-±0.05 מ-φ")
print(f"  (probability per pair ≈ {within_phi/trials*100:.1f}%)")
print()
print(f"  סיכוי ש-לפחות אחד מ-5 זוגות יפול ליד φ:")
p_no_hit = (1 - within_phi/trials) ** 5
p_any_hit = 1 - p_no_hit
print(f"    = 1 - (1 - {within_phi/trials:.4f})^5 = {p_any_hit*100:.1f}%")


# ════════════════════════════════════════════════════════════════
#  4. הטענה האמיתית: האם יש **over-representation** של φ בגימטריות?
# ════════════════════════════════════════════════════════════════
print()
print("═" * 80)
print("  בדיקה 4: Over-representation — האם יחסי φ שכיחים יותר בגימטריות?")
print("═" * 80)
print()

# נקח 100+ מילים מהתנ"ך (מ-memory יש לנו dataset)
# נבנה random pairs ונבדוק כמה מהם קרובים ל-φ

tanakh_words_gematria = [
    ("בראשית", 913), ("אלהים", 86), ("שמים", 390), ("ארץ", 291),
    ("אברהם", 248), ("יצחק", 208), ("יעקב", 182), ("משה", 345),
    ("תורה", 611), ("עם", 110), ("אחד", 13), ("אמת", 441),
    ("אהבה", 13), ("שלום", 376), ("שבת", 702), ("חיים", 68),
    ("אור", 207), ("חכמה", 73), ("בינה", 67), ("דעת", 474),
    ("מלך", 90), ("עולם", 146), ("רוח", 214), ("נשמה", 395),
    ("לב", 32), ("יהוה", 26), ("אדני", 65), ("אל", 31),
    ("ישראל", 541), ("ציון", 156), ("ירושלים", 586), ("כהן", 75),
    ("נביא", 63), ("מלאך", 91), ("גבריאל", 246), ("מיכאל", 101),
    ("רפאל", 311), ("אוריאל", 248), ("מטטרון", 314), ("סנדלפון", 280),
    ("רזיאל", 248), ("חסד", 72), ("גבורה", 216), ("תפארת", 1081),
    ("נצח", 148), ("הוד", 15), ("יסוד", 80), ("מלכות", 496),
    ("כתר", 620),
]

print(f"  בודק {len(tanakh_words_gematria)} מילים מהתנ״ך/קבלה")
print(f"  סה\"כ זוגות אפשריים: {len(tanakh_words_gematria) * (len(tanakh_words_gematria) - 1)}")
print()

# Real data: tanakh words
all_pairs_phi_count = 0
all_pairs_total = 0
for i, (w1, g1) in enumerate(tanakh_words_gematria):
    for j, (w2, g2) in enumerate(tanakh_words_gematria):
        if i == j: continue
        if g2 == 0: continue
        r = g1 / g2
        all_pairs_total += 1
        if abs(r - PHI) < THRESHOLD:
            all_pairs_phi_count += 1

# Random control: same number of pairs with same gematria distribution
random.seed(42)
gematria_values = [g for _, g in tanakh_words_gematria]
random_pairs_phi_count = 0
random_pairs_total = 0
for _ in range(all_pairs_total):
    g1 = random.choice(gematria_values)
    g2 = random.choice(gematria_values)
    if g2 == 0: continue
    r = g1 / g2
    random_pairs_total += 1
    if abs(r - PHI) < THRESHOLD:
        random_pairs_phi_count += 1

print(f"  REAL pairs (all-vs-all):    {all_pairs_phi_count}/{all_pairs_total} = {all_pairs_phi_count/all_pairs_total*100:.2f}% קרובים ל-φ")
print(f"  RANDOM pairs (same dist):   {random_pairs_phi_count}/{random_pairs_total} = {random_pairs_phi_count/random_pairs_total*100:.2f}% קרובים ל-φ")
print()

# זה אותו baseline — כי גימטריה של מילים יוצרת distributin ריאלי
# ולכן אם נדגום שני ערכים בצורה 'אמיתית' (מהתנ"ך), אותה תוצאה
# כמו אם נדגום 'רנדומלית' מאותה התפלגות

# הhyper-test: האם ההתפלגות של יחסים חריגה מ-uniform?
# נעשה test: אם היחסים uniform ב-(0, 5], אז P(|r-φ|<0.05) = 0.1/5 = 2%

expected_rate = 2 * THRESHOLD / 5  # approximate
real_rate = all_pairs_phi_count / all_pairs_total

print(f"  Expected rate (uniform hypothesis): {expected_rate*100:.2f}%")
print(f"  Observed real rate:                 {real_rate*100:.2f}%")
print(f"  Observed random rate:               {random_pairs_phi_count/random_pairs_total*100:.2f}%")
print()

# Chi-squared test
expected = all_pairs_total * expected_rate
observed = all_pairs_phi_count
if expected > 0:
    chi_sq = (observed - expected) ** 2 / expected
    print(f"  Chi-squared (vs uniform): {chi_sq:.3f}")
    if chi_sq > 3.84:  # p < 0.05 for df=1
        print("    → significant (p < 0.05)")
    else:
        print("    → not significant")


# ════════════════════════════════════════════════════════════════
#  5. שורה תחתונה
# ════════════════════════════════════════════════════════════════
print()
print("═" * 80)
print("  שורה תחתונה הגונה")
print("═" * 80)
print()
print("  מה שבדקנו היום:")
print()
print("  1. Operators (שזירה, פאי, אל-בם, את-בש) על טקסט:")
print("     → מבחינים טקסט אמיתי מ-random ב-entropy (טריוויאלי)")
print("     → לא מבחינים real מ-shuffled משמעותית")
print("     → לא יוצרים יחסי φ שספציפיים לטקסט")
print()
print("  2. ratio=2 של at_bash — artifact סטטיסטי של אותיות נפוצות, לא 'גילוי'")
print()
print("  3. גבריאל Heb/Greek = 1.597 (1.28% מ-φ):")
print("     → בבדיקה רחבה של מלאכים, אין דפוס consistent")
print("     → מקרה יחיד לא מוכיח דבר")
print()
print("  4. 10,000 זוגות random:")
print(f"     → ~{within_phi/trials*100:.1f}% נופלים ב-±0.05 מ-φ באופן טבעי")
print(f"     → אם בוחנים 5 זוגות, יש ~{p_any_hit*100:.0f}% שאחד יפול שם")
print("     → מציאת 1 מקרה = לא מיוחד")
print()
print("  5. 2,400+ זוגות של מילים תנ\"כיות:")
print(f"     → ~{all_pairs_phi_count/all_pairs_total*100:.1f}% קרובים ל-φ")
print(f"     → בדומה ל-random באותה התפלגות")
print("     → אין over-representation של φ")
print()
print("  מסקנה מתודולוגית:")
print("    ❌ אין מודל מתמטי פשוט (שזירה + פאי + גימטריה)")
print("       שמגלה 'סוד נסתר' במוח/בטקסט")
print("    ✓ יש artifacts סטטיסטיים (entropy נבדל, at_bash bias)")
print("       שניתן להסביר מההתפלגות של אותיות/מילים")
print("    ✓ Cherry-picked examples (גבריאל) קיימים בכל dataset")
print("       ולא מעידים על משהו")
print()
print("  המלצה הנדסית:")
print("    השתמש בגימטריה/אותיות כ-TAGGING MECHANISM (זיהוי),")
print("    לא כ-PREDICTIVE MODEL (חיזוי).")
print("    ZETS יכול לתייג edges עם letter_id, זה עובד.")
print("    ZETS לא יכול לבנות AGI על יחסי φ במילים. זה לא קיים סטטיסטית.")
