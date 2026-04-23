// ═══════════════════════════════════════════════════════════════
// numerology.rs — מנוע נומרולוגיה | DINIO Cortex V8.2
// ═══════════════════════════════════════════════════════════════
// Life Path, Expression, Soul Urge, Personality, Personal Year.
// ZERO DEPS — pure Rust, char_indices() for Hebrew safety.
// ═══════════════════════════════════════════════════════════════

// ─── Letter → Number maps ───────────────────────────────────

/// Pythagorean: A=1..Z=26 reduced mod 9 (with 11,22,33 as master numbers).
/// Maps both Latin (uppercase) and Hebrew letters to digits 1-9.
fn letter_value(c: char) -> Option<u32> {
    match c {
        // Latin A-Z
        'A' | 'a' => Some(1),
        'B' | 'b' => Some(2),
        'C' | 'c' => Some(3),
        'D' | 'd' => Some(4),
        'E' | 'e' => Some(5),
        'F' | 'f' => Some(6),
        'G' | 'g' => Some(7),
        'H' | 'h' => Some(8),
        'I' | 'i' => Some(9),
        'J' | 'j' => Some(1),
        'K' | 'k' => Some(2),
        'L' | 'l' => Some(3),
        'M' | 'm' => Some(4),
        'N' | 'n' => Some(5),
        'O' | 'o' => Some(6),
        'P' | 'p' => Some(7),
        'Q' | 'q' => Some(8),
        'R' | 'r' => Some(9),
        'S' | 's' => Some(1),
        'T' | 't' => Some(2),
        'U' | 'u' => Some(3),
        'V' | 'v' => Some(4),
        'W' | 'w' => Some(5),
        'X' | 'x' => Some(6),
        'Y' | 'y' => Some(7),
        'Z' | 'z' => Some(8),
        // Hebrew letters (Pythagorean-style mapping by position)
        'א' => Some(1),
        'ב' => Some(2),
        'ג' => Some(3),
        'ד' => Some(4),
        'ה' => Some(5),
        'ו' => Some(6),
        'ז' => Some(7),
        'ח' => Some(8),
        'ט' => Some(9),
        'י' => Some(1),
        'כ' | 'ך' => Some(2),
        'ל' => Some(3),
        'מ' | 'ם' => Some(4),
        'נ' | 'ן' => Some(5),
        'ס' => Some(6),
        'ע' => Some(7),
        'פ' | 'ף' => Some(8),
        'צ' | 'ץ' => Some(9),
        'ק' => Some(1),
        'ר' => Some(2),
        'ש' => Some(3),
        'ת' => Some(4),
        _ => None,
    }
}

/// Is the character a vowel (Latin or Hebrew "vowel letters")?
fn is_vowel(c: char) -> bool {
    matches!(c,
        'A' | 'E' | 'I' | 'O' | 'U' |
        'a' | 'e' | 'i' | 'o' | 'u' |
        // Hebrew vowel letters (matres lectionis)
        'א' | 'ה' | 'ו' | 'י'
    )
}

// ─── Reduction ──────────────────────────────────────────────

/// Reduce a number to a single digit, preserving Master Numbers 11, 22, 33.
pub fn reduce(mut n: u32) -> u32 {
    loop {
        if n <= 9 || n == 11 || n == 22 || n == 33 {
            return n;
        }
        let mut sum = 0u32;
        let mut tmp = n;
        while tmp > 0 {
            sum += tmp % 10;
            tmp /= 10;
        }
        n = sum;
    }
}

/// Sum all digits of a number (no master-number preservation).
fn digit_sum(mut n: u32) -> u32 {
    let mut s = 0u32;
    if n == 0 { return 0; }
    while n > 0 {
        s += n % 10;
        n /= 10;
    }
    s
}

// ─── Life Path ──────────────────────────────────────────────

/// Life Path Number: reduce day + reduce month + reduce year, then reduce total.
/// "DD/MM/YYYY" format.
pub fn life_path(birthday: &str) -> Option<u32> {
    let parts: Vec<&str> = birthday.split('/').collect();
    if parts.len() != 3 { return None; }
    let day:   u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let year:  u32 = parts[2].parse::<u32>().ok()?;

    if day == 0 || day > 31 || month == 0 || month > 12 { return None; }

    let r_day   = reduce(digit_sum(day));
    let r_month = reduce(digit_sum(month));

    // Year: sum all 4 digits, then reduce
    let year_sum = digit_sum(year / 1000)
        + digit_sum((year / 100) % 10)
        + digit_sum((year / 10) % 10)
        + digit_sum(year % 10);
    let r_year = reduce(year_sum);

    Some(reduce(r_day + r_month + r_year))
}

// ─── Expression (Destiny) Number ────────────────────────────

/// Expression Number: sum of ALL letter values in the full name.
pub fn expression(name: &str) -> u32 {
    let sum: u32 = name.char_indices()
        .filter_map(|(_, c)| letter_value(c))
        .sum();
    reduce(sum)
}

// ─── Soul Urge (Heart's Desire) ─────────────────────────────

/// Soul Urge: sum of VOWEL letter values only.
pub fn soul_urge(name: &str) -> u32 {
    let sum: u32 = name.char_indices()
        .filter(|(_, c)| is_vowel(*c))
        .filter_map(|(_, c)| letter_value(c))
        .sum();
    reduce(sum)
}

// ─── Personality Number ─────────────────────────────────────

/// Personality Number: sum of CONSONANT letter values only.
pub fn personality(name: &str) -> u32 {
    let sum: u32 = name.char_indices()
        .filter(|(_, c)| letter_value(*c).is_some() && !is_vowel(*c))
        .filter_map(|(_, c)| letter_value(c))
        .sum();
    reduce(sum)
}

// ─── Personal Year ───────────────────────────────────────────

/// Personal Year: reduce(day + month + current_year_digits).
/// birthday: "DD/MM/YYYY", current_year: e.g. 2026
pub fn personal_year(birthday: &str, current_year: u32) -> Option<u32> {
    let parts: Vec<&str> = birthday.split('/').collect();
    if parts.len() != 3 { return None; }
    let day:   u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    if day == 0 || day > 31 || month == 0 || month > 12 { return None; }

    let r_day   = reduce(digit_sum(day));
    let r_month = reduce(digit_sum(month));

    let year_sum = digit_sum(current_year / 1000)
        + digit_sum((current_year / 100) % 10)
        + digit_sum((current_year / 10) % 10)
        + digit_sum(current_year % 10);
    let r_year = reduce(year_sum);

    Some(reduce(r_day + r_month + r_year))
}

// ─── Number meanings ─────────────────────────────────────────

pub fn meaning_he(n: u32) -> &'static str {
    match n {
        1  => "מנהיגות, עצמאות ויוזמה. אתה פורץ דרכים.",
        2  => "שיתוף פעולה, דיפלומטיה ורגישות. כוחך בשותפות.",
        3  => "ביטוי עצמי, יצירתיות ושמחה. כוחך בדיבור ואמנות.",
        4  => "יציבות, עבודה קשה וסדר. אתה בונה בסיסים מוצקים.",
        5  => "חירות, הרפתקה ושינוי. אתה מחפש ניסיון וחופש.",
        6  => "אחריות, אהבה ומשפחה. כוחך בטיפול ובריפוי.",
        7  => "חכמה, מחקר ורוחניות. אתה מחפש את האמת העמוקה.",
        8  => "כוח, שפע ועסקים. אתה בנוי להצלחה חומרית.",
        9  => "הומניזם, חמלה ושלמות. אתה נשמה שמגיעה לסיום מחזור.",
        11 => "מספר מאסטר: אינטואיציה גבוהה, השראה ורוחניות עמוקה.",
        22 => "מספר מאסטר: בניין, הגשמת חזון גדול בעולם המעשי.",
        33 => "מספר מאסטר: מורה מאסטר, ריפוי ואהבה ללא תנאי.",
        _  => "מספר ייחודי.",
    }
}

pub fn meaning_en(n: u32) -> &'static str {
    match n {
        1  => "Leadership, independence and initiative. You are a pioneer.",
        2  => "Cooperation, diplomacy and sensitivity. Strength in partnership.",
        3  => "Self-expression, creativity and joy. Power in speech and art.",
        4  => "Stability, hard work and order. You build solid foundations.",
        5  => "Freedom, adventure and change. You seek experience and liberty.",
        6  => "Responsibility, love and family. Strength in caring and healing.",
        7  => "Wisdom, research and spirituality. You seek deep truth.",
        8  => "Power, abundance and business. Built for material success.",
        9  => "Humanism, compassion and completion. A soul finishing a cycle.",
        11 => "Master number: high intuition, inspiration, deep spirituality.",
        22 => "Master number: builder, realizing a great vision in the practical world.",
        33 => "Master number: master teacher, healing and unconditional love.",
        _  => "Unique number.",
    }
}

// ─── Full reading ─────────────────────────────────────────────

#[derive(Debug)]
pub struct NumerologyReading {
    pub name:           String,
    pub birthday:       String,
    pub life_path:      u32,
    pub expression:     u32,
    pub soul_urge:      u32,
    pub personality:    u32,
    pub personal_year:  u32,
}

pub fn full_reading(name: &str, birthday: &str, current_year: u32) -> Option<NumerologyReading> {
    Some(NumerologyReading {
        name:          name.to_string(),
        birthday:      birthday.to_string(),
        life_path:     life_path(birthday)?,
        expression:    expression(name),
        soul_urge:     soul_urge(name),
        personality:   personality(name),
        personal_year: personal_year(birthday, current_year)?,
    })
}

pub fn reading_to_json(r: &NumerologyReading) -> String {
    format!(
        r#"{{"name":"{}","birthday":"{}","life_path":{},"life_path_meaning_he":"{}","life_path_meaning_en":"{}","expression":{},"expression_meaning_he":"{}","expression_meaning_en":"{}","soul_urge":{},"soul_urge_meaning_he":"{}","soul_urge_meaning_en":"{}","personality":{},"personality_meaning_he":"{}","personality_meaning_en":"{}","personal_year":{},"personal_year_meaning_he":"{}","personal_year_meaning_en":"{}"}}"#,
        r.name.replace('"', "'"),
        r.birthday,
        r.life_path,   meaning_he(r.life_path).replace('"', "'"),   meaning_en(r.life_path).replace('"', "'"),
        r.expression,  meaning_he(r.expression).replace('"', "'"),  meaning_en(r.expression).replace('"', "'"),
        r.soul_urge,   meaning_he(r.soul_urge).replace('"', "'"),   meaning_en(r.soul_urge).replace('"', "'"),
        r.personality, meaning_he(r.personality).replace('"', "'"), meaning_en(r.personality).replace('"', "'"),
        r.personal_year, meaning_he(r.personal_year).replace('"', "'"), meaning_en(r.personal_year).replace('"', "'"),
    )
}

// ─── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_master_numbers() {
        assert_eq!(reduce(11), 11);
        assert_eq!(reduce(22), 22);
        assert_eq!(reduce(33), 33);
        assert_eq!(reduce(10), 1);
        assert_eq!(reduce(29), 11); // 2+9=11
        assert_eq!(reduce(38), 11); // 3+8=11
        assert_eq!(reduce(9),  9);
    }

    #[test]
    fn test_life_path() {
        // Example: 15/03/1990 → 1+5=6, 3, 1+9+9+0=19→10→1 → 6+3+1=10→1
        let lp = life_path("15/03/1990").unwrap();
        assert!(lp >= 1 && lp <= 33, "life path out of range: {}", lp);
    }

    #[test]
    fn test_expression_non_zero() {
        // Any name with letters should have expression > 0
        let e = expression("דניאל");
        assert!(e >= 1 && e <= 33, "expression: {}", e);
        let e2 = expression("Daniel");
        assert!(e2 >= 1 && e2 <= 33, "expression en: {}", e2);
    }

    #[test]
    fn test_soul_urge_vowels_only() {
        // "AEI" = 1+5+9=15 → 6
        assert_eq!(soul_urge("AEI"), 6);
        // consonants only — soul urge = 0 → reduce(0) = 0
        assert_eq!(soul_urge("BCD"), 0);
    }

    #[test]
    fn test_personality_consonants() {
        // "BCF" = 2+3+6=11 (master)
        assert_eq!(personality("BCF"), 11);
    }

    #[test]
    fn test_personal_year() {
        let py = personal_year("15/03/1990", 2026).unwrap();
        assert!(py >= 1 && py <= 33, "personal year: {}", py);
    }

    #[test]
    fn test_full_reading() {
        let r = full_reading("שרה כהן", "01/01/1985", 2026).unwrap();
        assert!(r.life_path >= 1);
        assert!(r.expression >= 1);
    }

    #[test]
    fn test_hebrew_char_indices() {
        // Verify char_indices works correctly for Hebrew
        let name = "שרה";
        let vals: Vec<u32> = name.char_indices()
            .filter_map(|(_, c)| letter_value(c))
            .collect();
        assert_eq!(vals.len(), 3, "Hebrew name should have 3 letter values");
    }

    #[test]
    fn test_invalid_birthday() {
        assert!(life_path("bad").is_none());
        assert!(life_path("32/01/2000").is_none());
        assert!(life_path("01/13/2000").is_none());
    }
}
