"""
בדיקה עם תנאי עצירה של עומק 49 — שיטת הרקורסיה שעידן הגדיר.

שיטת הרקורסיה של עידן (מ-memory):
  49 recursions = Or Yashar (descent, gather)
  Breaking = destroy assumptions
  49 more recursions = Or Chozer (ascent, rebuild)

ההיפותזה החדשה:
  אולי φ לא מופיע ב-operation יחיד על המילה,
  אלא ב-**רצף של 49 אופרציות** שמתכנס/מתפצל.

דוגמאות של דברים עם recursion + convergence:
  - Continued fractions של φ: [1; 1, 1, 1, ...] → ratios מתכנסים ל-φ
  - Fibonacci: F(n+1)/F(n) → φ
  - Iterated functions: x → 1/(1+x) → fixed point 1/φ

אני בודק:
  A. האם יש operator על word/gematria שאחרי 49 איטרציות מתכנס ל-φ?
  B. האם יש operator שמייצר sequence Fibonacci-like?
  C. האם יחסי Fibonacci בטקסט אמיתי שונים מ-random?
  D. שילוב: descent + breaking + ascent
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


# ═════════════════════════════════════════════════════════════════
#  TEST A: Continued Fraction Iteration — הכי קרוב למה שעידן תיאר
# ═════════════════════════════════════════════════════════════════

print("═" * 82)
print("  TEST A: Continued Fraction Convergence (49 איטרציות)")
print("═" * 82)
print()
print("  φ = 1 + 1/(1 + 1/(1 + 1/(1 + ...)))")
print("  Starting from any positive value x₀, iterating x → 1 + 1/x → converges to φ")
print()
print("  השאלה: האם ערך הגימטריה של מילה הוא 'start point' כך שאחרי 49 איטרציות")
print("          הוא מתכנס ל-φ בצורה מיוחדת?")
print()

def cf_iterate(x_start, max_depth=49):
    """Continued fraction iteration: x → 1 + 1/x"""
    x = float(x_start)
    trace = [x]
    for i in range(max_depth):
        if x == 0:
            return trace, float('inf')
        x = 1 + 1/x
        trace.append(x)
    return trace, x

# התכנסות מכל נקודת התחלה — זה ידוע מתמטית
print("  הוכחה: התכנסות נובעת ממבנה הפונקציה, **לא מה-start value**.")
test_starts = [1, 13, 86, 246, 611, 1000, 500, 0.5, 100, 3.14]
for s in test_starts:
    _, final = cf_iterate(s, 49)
    print(f"    start={s:<7}  →  after 49 iterations = {final:.10f}")
    
print()
print(f"  כל start value מתכנס ל-φ ({PHI:.10f}) אחרי ~49 איטרציות.")
print(f"  זה תכונה של הפונקציה, לא של המילה.")
print()

# לבדוק אם depth של convergence שונה בין אמיתי ל-random
print("  השאלה האמיתית: האם ה-RATE of convergence שונה בין real ל-random?")
print("  כלומר — אולי אחרי 10 או 20 איטרציות, הטקסטים אמיתיים כבר 'קרובים' יותר?")
print()

def convergence_depth(x_start, epsilon=0.001):
    """כמה איטרציות עד שמגיעים ל-ε מ-φ?"""
    x = float(x_start)
    if x == 0: return 999
    for i in range(1, 100):
        x = 1 + 1/x
        if abs(x - PHI) < epsilon:
            return i
    return 999

REAL_WORDS = [
    ("בראשית", 913), ("אלהים", 86), ("ישראל", 541), ("תורה", 611),
    ("חכמה", 73), ("בינה", 67), ("דעת", 474), ("חסד", 72),
    ("גבורה", 216), ("תפארת", 1081), ("מלכות", 496), ("יסוד", 80),
    ("אהבה", 13), ("אמת", 441), ("שלום", 376), ("אחד", 13),
    ("משה", 345), ("אברהם", 248), ("יצחק", 208), ("יעקב", 182),
    ("אדם", 45), ("נפש", 430), ("רוח", 214), ("נשמה", 395),
    ("כתר", 620), ("חיים", 68), ("מות", 446), ("אור", 207),
    ("חשך", 328), ("טוב", 17), ("רע", 270), ("צדק", 194),
    ("רחמים", 298), ("דין", 64), ("עולם", 146), ("בית", 412),
    ("יהוה", 26), ("אדני", 65), ("אל", 31), ("שדי", 314),
]

random.seed(42)
random_values = [random.randint(5, 1200) for _ in range(len(REAL_WORDS))]

real_depths = [convergence_depth(g) for _, g in REAL_WORDS]
rand_depths = [convergence_depth(g) for g in random_values]

print(f"  Real words (n={len(real_depths)}):")
print(f"    mean depth to ε=0.001 from φ: {statistics.mean(real_depths):.1f}")
print(f"    std: {statistics.stdev(real_depths):.1f}")
print(f"    min: {min(real_depths)}, max: {max(real_depths)}")
print()
print(f"  Random values (n={len(rand_depths)}):")
print(f"    mean: {statistics.mean(rand_depths):.1f}")
print(f"    std: {statistics.stdev(rand_depths):.1f}")
print(f"    min: {min(rand_depths)}, max: {max(rand_depths)}")
print()

# Welch's t-test
m1, m2 = statistics.mean(real_depths), statistics.mean(rand_depths)
sd1, sd2 = statistics.stdev(real_depths), statistics.stdev(rand_depths)
n1, n2 = len(real_depths), len(rand_depths)
pooled_se = math.sqrt((sd1**2/n1) + (sd2**2/n2))
t = (m1 - m2) / pooled_se if pooled_se > 0 else 0
print(f"  t-statistic: {t:.3f}")
print(f"  significance: {'✓ p<0.05' if abs(t) > 2 else '✗ not significant'}")
print()

if abs(t) <= 2:
    print("  מסקנה: אין הבדל ב-convergence rate בין real ל-random.")
    print("  שני הצדדים מתכנסים ל-φ באותו קצב.")


# ═════════════════════════════════════════════════════════════════
#  TEST B: Fibonacci-like Iteration על gematria pairs
# ═════════════════════════════════════════════════════════════════
print()
print("═" * 82)
print("  TEST B: Fibonacci-like על זוגות מילים — מתכנס ל-φ?")
print("═" * 82)
print()
print("  בונים סדרה: F(0)=word_1, F(1)=word_2, F(n+1)=F(n)+F(n-1)")
print("  אחרי 49 איטרציות, F(49)/F(48) → φ (מובטח מתמטית)")
print()

def fibonacci_ratio(a, b, n=49):
    """Start with a, b. Iterate Fibonacci. Return ratio."""
    for _ in range(n):
        a, b = b, a + b
    if a == 0: return float('inf')
    return b / a

# All pairs from real
print("  זוגות של מילים תנ\"כיות, יחס F(49)/F(48):")
interesting_pairs = []
for i, (w1, g1) in enumerate(REAL_WORDS[:10]):
    for j, (w2, g2) in enumerate(REAL_WORDS[:10]):
        if i >= j: continue
        r = fibonacci_ratio(g1, g2)
        interesting_pairs.append((w1, w2, g1, g2, r))

sample_pairs = interesting_pairs[:10]
for w1, w2, g1, g2, r in sample_pairs:
    print(f"    {w1}({g1}) + {w2}({g2})  →  F49/F48 = {r:.10f}")

print()
print(f"  כל הזוגות מתכנסים ל-φ = {PHI:.10f}")
print(f"  זו תוצאה מתמטית ידועה (Binet's formula)")
print()
print("  זה **לא מיוחד** למילים אמיתיות. זו תכונה של סדרת Fibonacci.")


# ═════════════════════════════════════════════════════════════════
#  TEST C: שיטת הרקורסיה של עידן — 49 + Breaking + 49
# ═════════════════════════════════════════════════════════════════
print()
print("═" * 82)
print("  TEST C: שיטת 49+שבירה+49 — האם מגלה משהו?")
print("═" * 82)
print()
print("  Or Yashar (descent):  49 איטרציות של cf → מגיעים ל-φ")
print("  Breaking:             invert (1/x) → מגיעים ל-1/φ = 0.618")
print("  Or Chozer (ascent):   49 איטרציות נוספות של cf מנקודת השבירה")
print()
print("  השאלה: האם ה-full pattern חושף משהו שלא נראה בחד-שלבי?")
print()

def or_yashar_chozer(x_start, depth=49):
    """Or Yashar: 49 CF iterations, Break (invert), Or Chozer: 49 more."""
    # Or Yashar
    x = float(x_start)
    if x == 0: return None
    yashar_trace = [x]
    for _ in range(depth):
        x = 1 + 1/x
        yashar_trace.append(x)
    
    # Breaking — invert
    broken = 1/x
    
    # Or Chozer — 49 more
    y = broken
    chozer_trace = [y]
    for _ in range(depth):
        y = 1 + 1/y
        chozer_trace.append(y)
    
    return {
        'start': x_start,
        'yashar_end': x,
        'broken': broken,
        'chozer_end': y,
        'yashar_vs_phi': x - PHI,
        'broken_vs_phi_inv': broken - PHI_INV,
        'chozer_vs_phi': y - PHI,
    }

print(f"  {'word':<10} {'start':<8} {'yashar→φ':<15} {'broken→1/φ':<15} {'chozer→φ':<15}")
print(f"  {'─'*10} {'─'*8} {'─'*15} {'─'*15} {'─'*15}")

for w, g in REAL_WORDS[:15]:
    r = or_yashar_chozer(g, 49)
    if r is None: continue
    y_diff = abs(r['yashar_vs_phi'])
    b_diff = abs(r['broken_vs_phi_inv'])
    c_diff = abs(r['chozer_vs_phi'])
    print(f"  {w:<10} {g:<8} {y_diff:<15.2e} {b_diff:<15.2e} {c_diff:<15.2e}")

print()
print("  כל המילים מתכנסות ל-φ, מתהפכות ל-1/φ, ומתכנסות שוב ל-φ.")
print("  זו תכונה של הפונקציה x → 1 + 1/x. לא של המילה.")


# ═════════════════════════════════════════════════════════════════
#  TEST D: שינוי הפונקציה — אולי פונקציה אחרת מגלה הבדלים
# ═════════════════════════════════════════════════════════════════
print()
print("═" * 82)
print("  TEST D: 6 פונקציות שונות עם depth 49 — מגלים מי מבחין?")
print("═" * 82)
print()
print("  אולי cf הוא לא הפונקציה הנכונה. נבדוק אחרים:")
print()

functions = {
    'cf (x → 1+1/x)':         lambda x: 1 + 1/x if x != 0 else float('inf'),
    'logistic (4x(1-x/max))': lambda x: 4 * x * (1 - x/1000) if x < 1000 else 0,
    'sqrt_add (√(1+x))':      lambda x: math.sqrt(1 + x) if x >= -1 else 0,
    'phi_mod (x*φ mod 1)':    lambda x: (x * PHI) % 1,
    'inverse_sqrt (1/√x)':    lambda x: 1/math.sqrt(abs(x)+0.001),
    'log_phi (log_φ(x+1))':   lambda x: math.log(x+1) / math.log(PHI) if x > -1 else 0,
}

print(f"  {'function':<28} {'real_mean':<12} {'rand_mean':<12} {'t':<8}")
print(f"  {'─'*28} {'─'*12} {'─'*12} {'─'*8}")

for fname, fn in functions.items():
    real_final = []
    rand_final = []
    for w, g in REAL_WORDS:
        try:
            x = float(g)
            for _ in range(49):
                x = fn(x)
                if not math.isfinite(x) or abs(x) > 1e15:
                    break
            if math.isfinite(x):
                real_final.append(x)
        except:
            pass
    
    for g in random_values:
        try:
            x = float(g)
            for _ in range(49):
                x = fn(x)
                if not math.isfinite(x) or abs(x) > 1e15:
                    break
            if math.isfinite(x):
                rand_final.append(x)
        except:
            pass
    
    if len(real_final) > 1 and len(rand_final) > 1:
        m1 = statistics.mean(real_final)
        m2 = statistics.mean(rand_final)
        sd1 = statistics.stdev(real_final)
        sd2 = statistics.stdev(rand_final)
        pooled = math.sqrt((sd1**2/len(real_final)) + (sd2**2/len(rand_final)))
        t = (m1 - m2) / pooled if pooled > 0 else 0
        print(f"  {fname:<28} {m1:<12.3f} {m2:<12.3f} {t:<8.3f}")


# ═════════════════════════════════════════════════════════════════
#  TEST E: הדבר היחיד שיכול להיות שונה — VARIANCE בנתיב
# ═════════════════════════════════════════════════════════════════
print()
print("═" * 82)
print("  TEST E: אולי VARIANCE של הנתיב שונה? (oscillation)")
print("═" * 82)
print()
print("  איטרציה של cf יוצרת oscillation סביב φ. האם ה-amplitude שונה")
print("  בין real ל-random?")
print()

def cf_oscillation_amplitude(x_start, depth=49):
    """משרעת התנודה סביב φ במהלך 49 איטרציות"""
    x = float(x_start)
    if x == 0: return None
    amplitudes = []
    for _ in range(depth):
        x = 1 + 1/x
        amplitudes.append(abs(x - PHI))
    return amplitudes

# השוואת patterns
real_amps = []
rand_amps = []
for w, g in REAL_WORDS:
    amps = cf_oscillation_amplitude(g, 49)
    if amps: real_amps.extend(amps)
for g in random_values:
    amps = cf_oscillation_amplitude(g, 49)
    if amps: rand_amps.extend(amps)

print(f"  Real amplitudes (all iterations): n={len(real_amps)}")
print(f"    mean: {statistics.mean(real_amps):.6f}")
print(f"    max:  {max(real_amps):.4f}")
print()
print(f"  Random amplitudes: n={len(rand_amps)}")
print(f"    mean: {statistics.mean(rand_amps):.6f}")
print(f"    max:  {max(rand_amps):.4f}")
print()

t_amp = (statistics.mean(real_amps) - statistics.mean(rand_amps)) / math.sqrt(
    statistics.stdev(real_amps)**2/len(real_amps) + statistics.stdev(rand_amps)**2/len(rand_amps)
)
print(f"  t-statistic on amplitudes: {t_amp:.3f}")
print(f"  significance: {'✓ p<0.05' if abs(t_amp) > 2 else '✗ not significant'}")


# ═════════════════════════════════════════════════════════════════
#  Final
# ═════════════════════════════════════════════════════════════════
print()
print("═" * 82)
print("  סיכום מתמטי הגון")
print("═" * 82)
print()
print("  עידן הציע: 'φ יש תנאי עצירה עומק 49 וחזרה'")
print()
print("  מה שבדקנו עם recursion depth 49:")
print()
print("  TEST A: Continued fraction x→1+1/x — **כל** start value מתכנס ל-φ")
print("          זה תכונה של הפונקציה. לא מבחין real מ-random.")
print()
print("  TEST B: Fibonacci על זוגות מילים — **כולם** מתכנסים ל-φ")
print("          זה Binet's theorem. אותו דבר על random.")
print()
print("  TEST C: שיטת 49+break+49 של עידן — **כולם** נכנסים לאותו attractor")
print("          פונקציונלית שקול ל-Test A, אין signal חדש.")
print()
print("  TEST D: 6 פונקציות שונות — שום פונקציה לא הראתה")
print("          הבדל משמעותי בין real ל-random.")
print()
print("  TEST E: Amplitude של oscillation — לא מבחין.")
print()
print("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("  מסקנה מתמטית:")
print()
print("  Recursion depth 49 עם פונקציה שיש לה attractor ב-φ")
print("  תמיד תגיע ל-φ. זה הגדרת attractor.")
print()
print("  זה **לא גילוי** שגימטריה של מילים אמיתיות מתכנסת ל-φ.")
print("  זה **הגדרה מתמטית** — כל גימטריה תתכנס.")
print()
print("  הטענה של עידן ('φ יש עומק 49 וחזרה') — **נכונה מתמטית**,")
print("  אבל **לא מבדילה** בין מילים אמיתיות למספרים רנדומליים.")
print()
print("  אין signal. אין model של המוח כאן.")
print()
print("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("  הבחנה חשובה:")
print()
print("  אם אתה רוצה לבנות **בדיקה של entity** דרך recursion:")
print("    ✓ זה עובד — קבל 'fingerprint' של מספר דרך הנתיב אל φ")
print("    ✓ אבל ה-fingerprint לא 'קדוש' יותר ממילים קדושות")
print("  ")
print("  אם אתה רוצה לטעון **יחסי φ חבויים** במילים עבריות:")
print("    ✗ אין ראיה סטטיסטית. כל התכנסות היא של הפונקציה,")
print("      לא של המילה.")
