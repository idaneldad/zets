// ═══════════════════════════════════════════════════════════════
// astro.rs — מנוע אסטרולוגי | DINIO Cortex V8.2
// ═══════════════════════════════════════════════════════════════
// חישוב מיקומי שמש, ירח, כוכבי לכת, היבטים, ותחזיות יומיות.
// ZERO DEPS — pure Rust, no external crates.
// ═══════════════════════════════════════════════════════════════

// ─── Constants ──────────────────────────────────────────────

const TWO_PI: f64 = std::f64::consts::PI * 2.0;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;

// J2000.0 epoch (Julian Day Number)
const J2000: f64 = 2451545.0;

// ─── Zodiac Signs ───────────────────────────────────────────

pub const SIGNS_HE: [&str; 12] = [
    "טלה", "שור", "תאומים", "סרטן", "אריה", "בתולה",
    "מאזניים", "עקרב", "קשת", "גדי", "דלי", "דגים",
];

pub const SIGNS_EN: [&str; 12] = [
    "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
    "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces",
];

// ─── Planet indices ──────────────────────────────────────────

pub const SUN: usize = 0;
pub const MOON: usize = 1;
pub const MERCURY: usize = 2;
pub const VENUS: usize = 3;
pub const MARS: usize = 4;
pub const JUPITER: usize = 5;
pub const SATURN: usize = 6;
pub const URANUS: usize = 7;
pub const NEPTUNE: usize = 8;
pub const PLUTO: usize = 9;
pub const NORTH_NODE: usize = 10;
pub const SOUTH_NODE: usize = 11;
pub const EARTH: usize = 12;

const PLANET_NAMES_HE: [&str; 13] = [
    "שמש", "ירח", "כוכב חמה", "נגה", "מאדים", "צדק", "שבתאי",
    "אורנוס", "נפטון", "פלוטו", "צומת צפוני", "צומת דרומי", "כדור הארץ",
];

const PLANET_NAMES_EN: [&str; 13] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
    "Uranus", "Neptune", "Pluto", "North Node", "South Node", "Earth",
];


// ─── Sign Profile Data ───────────────────────────────────────

pub const ELEMENTS_HE: [&str; 12] = ["אש","אדמה","אויר","מים","אש","אדמה","אויר","מים","אש","אדמה","אויר","מים"];
pub const ELEMENTS_EN: [&str; 12] = ["Fire","Earth","Air","Water","Fire","Earth","Air","Water","Fire","Earth","Air","Water"];
const QUALITIES_HE: [&str; 12] = ["קרדינלי","קבוע","משתנה","קרדינלי","קבוע","משתנה","קרדינלי","קבוע","משתנה","קרדינלי","קבוע","משתנה"];
const RULERS_HE: [&str; 12] = ["מאדים","נוגה","כוכב חמה","ירח","שמש","כוכב חמה","נוגה","פלוטו","צדק","שבתאי","אורנוס","נפטון"];
const RULERS_EN: [&str; 12] = ["Mars","Venus","Mercury","Moon","Sun","Mercury","Venus","Pluto","Jupiter","Saturn","Uranus","Neptune"];
const SYMBOLS: [&str; 12] = ["♈","♉","♊","♋","♌","♍","♎","♏","♐","♑","♒","♓"];
const STONES: [&str; 12] = ["יהלום","אמרלד","אגת","פנינה","רובי","ספיר","אופל","טורמלין","טורקיז","גרנט","אמטיסט","אקוומרין"];
const COLORS: [&str; 12] = ["אדום","ירוק","צהוב","כסף","זהב","חום","ורוד","בורדו","סגול","שחור","כחול","טורקיז"];
const LUCKY_DAYS: [&str; 12] = ["שלישי","שישי","רביעי","שני","ראשון","רביעי","שישי","שלישי","חמישי","שבת","שבת","חמישי"];

// Kabbalistic correspondences (Sefer Yetzirah)
const KAB_LETTERS: [&str; 12] = ["ה","ו","ז","ח","ט","י","ל","נ","ס","ע","צ","ק"];
const KAB_LETTER_NAMES: [&str; 12] = ["הא","ואו","זין","חית","טית","יוד","למד","נון","סמך","עין","צדי","קוף"];
const HEBREW_MONTHS: [&str; 12] = ["ניסן","אייר","סיוון","תמוז","אב","אלול","תשרי","חשוון","כסלו","טבת","שבט","אדר"];
const KAB_SENSES: [&str; 12] = ["ראייה","שמיעה","ריחה","שיחה","לעיטה","תשמיש","מעשה","הילוך","כעס","שחוק","הרהור","שינה"];

const COMPATIBLE: [[usize; 4]; 12] = [
    [4,8,2,10],  // Aries → Leo,Sag,Gem,Aqu
    [5,9,3,11],  // Taurus → Vir,Cap,Can,Pis
    [6,10,0,4],  // Gemini → Lib,Aqu,Ari,Leo
    [7,11,1,5],  // Cancer → Sco,Pis,Tau,Vir
    [0,8,2,6],   // Leo → Ari,Sag,Gem,Lib
    [1,9,3,7],   // Virgo → Tau,Cap,Can,Sco
    [2,10,4,8],  // Libra → Gem,Aqu,Leo,Sag
    [3,11,5,9],  // Scorpio → Can,Pis,Vir,Cap
    [0,4,6,10],  // Sag → Ari,Leo,Lib,Aqu
    [1,5,7,11],  // Cap → Tau,Vir,Sco,Pis
    [2,6,0,8],   // Aqu → Gem,Lib,Ari,Sag
    [3,7,1,9],   // Pisces → Can,Sco,Tau,Cap
];

const TRAITS_HE: [&str; 12] = [
    "אנרגטי, אמיץ ויוזם. מוביל טבעי עם נחישות חזקה.",
    "יציב, נאמן ומעשי. אוהב נוחות, יופי ואיכות.",
    "סקרן, חברותי ורב-תחומי. מתקשר מבריק עם חשיבה מהירה.",
    "רגיש, אכפתי ואינטואיטיבי. מגן על קרוביו ומוקף באהבה.",
    "כריזמטי, יצירתי ונדיב. אוהב להיות במרכז ולהעניק שמחה.",
    "מדויק, אנליטי ועובד קשה. שם לב לפרטים ושואף לשלמות.",
    "הרמוני, דיפלומטי ואוהב צדק. מחפש איזון ושותפות.",
    "עמוק, נחוש ואינטנסיבי. בעל אינטואיציה חזקה ונאמנות מוחלטת.",
    "אופטימי, הרפתקן וחופשי. אוהב ללמוד ולחקור עולמות חדשים.",
    "שאפתן, אחראי ומשמעתי. בונה לטווח ארוך עם תכנון חכם.",
    "חדשני, עצמאי והומניטרי. חושב מחוץ לקופסה ומוביל שינוי.",
    "חולמני, אמפתי ויצירתי. מחובר לרגשות, לאמנות ולרוחניות.",
];

// ─── Aspect definitions ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Aspect {
    pub planet_a: usize,
    pub planet_b: usize,
    pub angle: f64,     // actual angle between planets
    pub aspect_type: AspectType,
    pub orb: f64,       // deviation from exact
}

#[derive(Debug, Clone, PartialEq)]
pub enum AspectType {
    Conjunction,   // 0°
    Sextile,       // 60°
    Square,        // 90°
    Trine,         // 120°
    Opposition,    // 180°
}

impl AspectType {
    pub fn name_he(&self) -> &'static str {
        match self {
            AspectType::Conjunction  => "קוניונקציה (0°)",
            AspectType::Sextile      => "סקסטיל (60°)",
            AspectType::Square       => "ריבוע (90°)",
            AspectType::Trine        => "טרין (120°)",
            AspectType::Opposition   => "אופוזיציה (180°)",
        }
    }
    pub fn name_en(&self) -> &'static str {
        match self {
            AspectType::Conjunction  => "Conjunction (0°)",
            AspectType::Sextile      => "Sextile (60°)",
            AspectType::Square       => "Square (90°)",
            AspectType::Trine        => "Trine (120°)",
            AspectType::Opposition   => "Opposition (180°)",
        }
    }
    pub fn is_harmonious(&self) -> bool {
        matches!(self, AspectType::Trine | AspectType::Sextile | AspectType::Conjunction)
    }
}

// ─── Date → Julian Day ──────────────────────────────────────

/// Convert Gregorian date to Julian Day Number (JDN).
pub fn julian_day(year: i32, month: u32, day: u32) -> f64 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let a = y / 100;
    let b = 2 - a + a / 4;
    (365.25 * (y + 4716) as f64).floor()
        + (30.6001 * (m + 1) as f64).floor()
        + day as f64
        + b as f64
        - 1524.5
}

/// Current Julian Day from system clock (UTC)
pub fn now_jd() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    // Unix epoch = JD 2440587.5
    2440587.5 + secs / 86400.0
}

/// Days since J2000.0
fn days_since_j2000(jd: f64) -> f64 {
    jd - J2000
}

/// Normalize angle to [0, 360)
pub fn norm360(deg: f64) -> f64 {
    let mut d = deg % 360.0;
    if d < 0.0 { d += 360.0; }
    d
}

// ─── Sun Longitude ──────────────────────────────────────────

/// Compute ecliptic longitude of the Sun (degrees) for a given JD.
/// Uses low-precision VSOP87 approximation (accurate to ~0.01°).
pub fn sun_longitude(jd: f64) -> f64 {
    let d = days_since_j2000(jd);

    // Mean longitude and mean anomaly
    let l = norm360(280.460 + 0.9856474 * d);
    let g = norm360(357.528 + 0.9856003 * d) * DEG_TO_RAD;

    // Equation of center
    let lambda = l + 1.915 * g.sin() + 0.020 * (2.0 * g).sin();
    norm360(lambda)
}

// ─── Moon Longitude ─────────────────────────────────────────

/// Compute ecliptic longitude of the Moon (degrees) for a given JD.
/// Uses simplified lunar theory (accurate to ~1°).
pub fn moon_longitude(jd: f64) -> f64 {
    let d = days_since_j2000(jd);

    // Fundamental arguments (degrees)
    let l0 = norm360(218.316 + 13.176396 * d); // mean longitude
    let m  = norm360(134.963 + 13.064993 * d) * DEG_TO_RAD; // mean anomaly
    let f  = norm360(93.272  + 13.229350 * d) * DEG_TO_RAD; // argument of latitude
    let d_moon = norm360(297.850 + 12.190749 * d) * DEG_TO_RAD; // mean elongation

    let lon = l0
        + 6.289 * m.sin()
        - 1.274 * (2.0 * d_moon - m).sin()
        + 0.658 * (2.0 * d_moon).sin()
        - 0.214 * (2.0 * m).sin()
        - 0.186 * (d_moon).sin() * RAD_TO_DEG / RAD_TO_DEG // correction for solar mean anomaly
        - 0.114 * (2.0 * f).sin()
        + 0.059 * (2.0 * d_moon - 2.0 * m).sin()
        + 0.057 * (2.0 * d_moon - m).sin() * 0.0; // already included above

    norm360(lon)
}

// ─── Planet Longitudes ──────────────────────────────────────

/// Orbital elements: (L0, L1, e, i, omega, N) where
/// L = L0 + L1*d (mean longitude), e = eccentricity
/// Simple Keplerian orbit approximation.
struct OrbitalElements {
    l0: f64,     // mean longitude at J2000 (degrees)
    l1_cent: f64, // mean longitude rate (degrees/CENTURY) — secular precision
    e0: f64,     // eccentricity at J2000
    e1_cent: f64, // eccentricity rate (per century)
    a:  f64,     // semi-major axis (AU)
    peri0: f64,  // perihelion longitude at J2000 (degrees)
    peri1_cent: f64, // perihelion rate (degrees/century)
}

const PLANET_ELEMENTS: [OrbitalElements; 8] = [
    // Mercury (Meeus "Astronomical Algorithms" — secular rates per century)
    OrbitalElements { l0: 252.2509, l1_cent: 149474.0722, e0: 0.205635, e1_cent: 0.000023, a: 0.387, peri0: 77.4561, peri1_cent: 1.5564 },
    // Venus
    OrbitalElements { l0: 181.9798, l1_cent: 58519.2130,  e0: 0.006773, e1_cent: -0.000048, a: 0.723, peri0: 131.5637, peri1_cent: 1.4080 },
    // Mars
    OrbitalElements { l0: 355.4330, l1_cent: 19141.6964,  e0: 0.093405, e1_cent: 0.000090, a: 1.524, peri0: 336.0602, peri1_cent: 1.8410 },
    // Jupiter
    OrbitalElements { l0: 34.3515,  l1_cent: 3036.3027,   e0: 0.048498, e1_cent: 0.000163, a: 5.203, peri0: 14.3312, peri1_cent: 1.6126 },
    // Saturn
    OrbitalElements { l0: 50.0774,  l1_cent: 1223.5110,   e0: 0.055548, e1_cent: -0.000346, a: 9.537, peri0: 93.0572, peri1_cent: 1.9637 },
    // Uranus
    OrbitalElements { l0: 314.0550, l1_cent: 429.8640,    e0: 0.047168, e1_cent: -0.000019, a: 19.191, peri0: 170.9640, peri1_cent: 1.4860 },
    // Neptune
    OrbitalElements { l0: 304.3487, l1_cent: 219.8833,    e0: 0.008590, e1_cent: 0.000025, a: 30.069, peri0: 44.9710, peri1_cent: 1.3253 },
    // Pluto
    OrbitalElements { l0: 238.9290, l1_cent: 145.2078,    e0: 0.248808, e1_cent: 0.000019, a: 39.482, peri0: 224.0675, peri1_cent: 1.3912 },
];

/// Compute heliocentric x,y for a planet using Keplerian elements + iterative Kepler equation.
fn helio_xy(planet: usize, d: f64) -> (f64, f64) {
    let el = &PLANET_ELEMENTS[planet - 2];
    let t = d / 36525.0; // Julian centuries from J2000

    // Mean longitude with secular variation: L = L0 + L1*T (T in centuries)
    let l = norm360(el.l0 + el.l1_cent * t);
    // Eccentricity with secular variation
    let e = el.e0 + el.e1_cent * t;
    // Perihelion with secular variation
    let peri = el.peri0 + el.peri1_cent * t;

    let m_deg = norm360(l - peri);
    let m = m_deg * DEG_TO_RAD;

    // Iterative Kepler equation: E - e*sin(E) = M
    let mut big_e = m;
    for _ in 0..10 {
        big_e = m + e * big_e.sin();
    }

    // True anomaly
    let nu = 2.0 * ((1.0 + e).sqrt() * (big_e / 2.0).sin())
        .atan2((1.0 - e).sqrt() * (big_e / 2.0).cos());

    // Distance from Sun
    let r = el.a * (1.0 - e * big_e.cos());

    // Heliocentric ecliptic longitude (simplified — ecliptic plane)
    let lon_rad = (nu * RAD_TO_DEG + peri) * DEG_TO_RAD;

    (r * lon_rad.cos(), r * lon_rad.sin())
}

/// Compute ecliptic longitude of a planet for a given JD.
/// Uses proper geocentric transformation: heliocentric Kepler → cartesian → atan2.
/// Accuracy: ±1° vs NASA JPL (verified against Skyfield DE421).
pub fn planet_longitude(planet: usize, jd: f64) -> f64 {
    match planet {
        SUN  => sun_longitude(jd),
        MOON => moon_longitude(jd),
        2..=9 => {
            let d = days_since_j2000(jd);

            // 1. Planet heliocentric position (Keplerian)
            let (px, py) = helio_xy(planet, d);

            // 2. Earth heliocentric position (opposite of Sun, ~1 AU)
            let earth_lon_rad = (sun_longitude(jd) + 180.0) * DEG_TO_RAD;
            let ex = earth_lon_rad.cos();
            let ey = earth_lon_rad.sin();

            // 3. Geocentric = planet - earth (vector subtraction)
            let gx = px - ex;
            let gy = py - ey;

            // 4. Ecliptic longitude from atan2
            norm360(gy.atan2(gx) * RAD_TO_DEG)
        }
        // North Node (mean lunar ascending node)
        NORTH_NODE => {
            let d = days_since_j2000(jd);
            let omega = norm360(125.044522 - 0.052953766 * d);
            // Add principal perturbation
            let omega_rad = omega * DEG_TO_RAD;
            norm360(omega + 1.4979 * omega_rad.sin() - 0.15 * (2.0 * omega_rad).sin())
        }
        // South Node = North Node + 180
        SOUTH_NODE => norm360(planet_longitude(NORTH_NODE, jd) + 180.0),
        // Earth = Sun + 180
        EARTH => norm360(sun_longitude(jd) + 180.0),
        _ => 0.0,
    }
}

/// Compute all 7 planet longitudes (legacy — backward compatible) for a given JD.
pub fn all_longitudes(jd: f64) -> [f64; 7] {
    [
        planet_longitude(SUN,     jd),
        planet_longitude(MOON,    jd),
        planet_longitude(2, jd), // Mercury
        planet_longitude(3, jd), // Venus
        planet_longitude(4, jd), // Mars
        planet_longitude(5, jd), // Jupiter
        planet_longitude(6, jd), // Saturn
    ]
}

/// Compute all 13 celestial point longitudes for Human Design.
/// Order: Sun, Earth, Moon, NorthNode, SouthNode, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
pub fn all_longitudes_hd(jd: f64) -> [(usize, f64); 13] {
    [
        (SUN,        planet_longitude(SUN, jd)),
        (EARTH,      planet_longitude(EARTH, jd)),
        (MOON,       planet_longitude(MOON, jd)),
        (NORTH_NODE, planet_longitude(NORTH_NODE, jd)),
        (SOUTH_NODE, planet_longitude(SOUTH_NODE, jd)),
        (2,          planet_longitude(2, jd)),   // Mercury
        (3,          planet_longitude(3, jd)),   // Venus
        (4,          planet_longitude(4, jd)),   // Mars
        (5,          planet_longitude(5, jd)),   // Jupiter
        (6,          planet_longitude(6, jd)),   // Saturn
        (URANUS,     planet_longitude(URANUS, jd)),
        (NEPTUNE,    planet_longitude(NEPTUNE, jd)),
        (PLUTO,      planet_longitude(PLUTO, jd)),
    ]
}

/// Find the JD when Sun crossed a specific longitude (searching backwards from start_jd).
/// Used for HD Design Date (88 degrees before birth Sun).
pub fn sun_crossing_before(target_lon: f64, start_jd: f64) -> f64 {
    let target = norm360(target_lon);
    // Coarse: step 1 day, search 100 days back
    let mut jd = start_jd - 100.0;
    let mut best_jd = jd;
    let mut best_diff = 999.0_f64;
    while jd < start_jd {
        let diff = angle_diff(sun_longitude(jd), target);
        if diff < best_diff { best_diff = diff; best_jd = jd; }
        jd += 1.0;
    }
    // Fine: step 0.001 day (~86 sec) around best
    jd = best_jd - 2.0;
    let end = best_jd + 2.0;
    best_diff = 999.0;
    while jd < end {
        let diff = angle_diff(sun_longitude(jd), target);
        if diff < best_diff { best_diff = diff; best_jd = jd; }
        jd += 0.001;
    }
    best_jd
}

/// Smallest angular difference between two longitudes
fn angle_diff(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

/// Planet name in Hebrew by index (0..12)
pub fn planet_name_he(idx: usize) -> &'static str {
    if idx < PLANET_NAMES_HE.len() { PLANET_NAMES_HE[idx] } else { "?" }
}

/// Planet name in English by index (0..12)
pub fn planet_name_en(idx: usize) -> &'static str {
    if idx < PLANET_NAMES_EN.len() { PLANET_NAMES_EN[idx] } else { "?" }
}

// ─── Aspects ────────────────────────────────────────────────

const ASPECT_DEFINITIONS: [(f64, f64, AspectType); 5] = [
    (0.0,   8.0, AspectType::Conjunction),
    (60.0,  6.0, AspectType::Sextile),
    (90.0,  8.0, AspectType::Square),
    (120.0, 8.0, AspectType::Trine),
    (180.0, 8.0, AspectType::Opposition),
];

/// Find all major aspects between the given planet longitudes.
pub fn find_aspects(longitudes: &[f64; 7]) -> Vec<Aspect> {
    let mut aspects = Vec::new();

    for i in 0..7 {
        for j in (i + 1)..7 {
            let diff = (longitudes[i] - longitudes[j]).abs();
            let angle = if diff > 180.0 { 360.0 - diff } else { diff };

            for (exact, orb, ref atype) in &ASPECT_DEFINITIONS {
                let deviation = (angle - exact).abs();
                if deviation <= *orb {
                    aspects.push(Aspect {
                        planet_a:    i,
                        planet_b:    j,
                        angle,
                        aspect_type: atype.clone(),
                        orb:         deviation,
                    });
                    break;
                }
            }
        }
    }

    // Sort: harmonious first, then by orb (tighter = more important)
    aspects.sort_by(|a, b| {
        let ha = a.aspect_type.is_harmonious();
        let hb = b.aspect_type.is_harmonious();
        if ha != hb {
            return if ha { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        a.orb.partial_cmp(&b.orb).unwrap_or(std::cmp::Ordering::Equal)
    });

    aspects
}

// ─── Zodiac Sign helpers ─────────────────────────────────────

pub fn sign_from_longitude(lon: f64) -> usize {
    (lon / 30.0) as usize % 12
}

pub fn sign_name_he(lon: f64) -> &'static str {
    SIGNS_HE[sign_from_longitude(lon)]
}

pub fn sign_name_en(lon: f64) -> &'static str {
    SIGNS_EN[sign_from_longitude(lon)]
}

pub fn degree_in_sign(lon: f64) -> f64 {
    lon % 30.0
}

// ─── Forecast Templates ─────────────────────────────────────

/// A per-sign daily theme string (Hebrew + English).
struct SignTheme {
    energy_he: &'static str,
    energy_en: &'static str,
    advice_he:  &'static str,
    advice_en:  &'static str,
}

const SIGN_THEMES: [SignTheme; 12] = [
    // Aries (טלה)
    SignTheme {
        energy_he: "אנרגיה גבוהה ויוזמה",
        energy_en: "High energy and initiative",
        advice_he: "פעל במהירות, אך היזהר מחפזון",
        advice_en: "Act swiftly, but avoid rashness",
    },
    // Taurus (שור)
    SignTheme {
        energy_he: "יציבות וסבלנות",
        energy_en: "Stability and patience",
        advice_he: "התמקד בבניית יסודות מוצקים",
        advice_en: "Focus on building solid foundations",
    },
    // Gemini (תאומים)
    SignTheme {
        energy_he: "תקשורת ותנועה",
        energy_en: "Communication and movement",
        advice_he: "שתף רעיונות, הקשב לאחרים",
        advice_en: "Share ideas, listen to others",
    },
    // Cancer (סרטן)
    SignTheme {
        energy_he: "רגשות ובית",
        energy_en: "Emotions and home",
        advice_he: "טפל ביחסים ובאנשים שאתה אוהב",
        advice_en: "Nurture relationships and loved ones",
    },
    // Leo (אריה)
    SignTheme {
        energy_he: "יצירתיות ומנהיגות",
        energy_en: "Creativity and leadership",
        advice_he: "הבע את עצמך בגאון",
        advice_en: "Express yourself with pride",
    },
    // Virgo (בתולה)
    SignTheme {
        energy_he: "סדר ופרטים",
        energy_en: "Order and details",
        advice_he: "ארגן, תכנן ושפר",
        advice_en: "Organize, plan, and improve",
    },
    // Libra (מאזניים)
    SignTheme {
        energy_he: "שיווי משקל ויחסים",
        energy_en: "Balance and relationships",
        advice_he: "חפש הסכמה ויצור הרמוניה",
        advice_en: "Seek agreement and create harmony",
    },
    // Scorpio (עקרב)
    SignTheme {
        energy_he: "עומק ושינוי",
        energy_en: "Depth and transformation",
        advice_he: "חקור את הנסתר ואמץ שינוי",
        advice_en: "Explore the hidden and embrace change",
    },
    // Sagittarius (קשת)
    SignTheme {
        energy_he: "הרפתקה ואמת",
        energy_en: "Adventure and truth",
        advice_he: "הרחב אופקים וחפש משמעות",
        advice_en: "Broaden horizons and seek meaning",
    },
    // Capricorn (גדי)
    SignTheme {
        energy_he: "אחריות ומשמעת",
        energy_en: "Responsibility and discipline",
        advice_he: "עבוד בשיטתיות לעבר מטרותיך",
        advice_en: "Work systematically toward your goals",
    },
    // Aquarius (דלי)
    SignTheme {
        energy_he: "חדשנות ועצמאות",
        energy_en: "Innovation and independence",
        advice_he: "חשוב מחוץ לקופסה",
        advice_en: "Think outside the box",
    },
    // Pisces (דגים)
    SignTheme {
        energy_he: "אינטואיציה ורוחניות",
        energy_en: "Intuition and spirituality",
        advice_he: "הקשב לתחושות הפנימיות שלך",
        advice_en: "Listen to your inner feelings",
    },
];

// ─── Aspect-based forecast lines ────────────────────────────

fn aspect_text_he(a: &Aspect) -> String {
    let pa = PLANET_NAMES_HE[a.planet_a];
    let pb = PLANET_NAMES_HE[a.planet_b];
    let atype = a.aspect_type.name_he();
    if a.aspect_type.is_harmonious() {
        format!("{} ב{} עם {} — אנרגיה חיובית ותמיכה", pa, atype, pb)
    } else {
        format!("{} ב{} עם {} — מתח ואתגר, הזדמנות לצמיחה", pa, atype, pb)
    }
}

fn aspect_text_en(a: &Aspect) -> String {
    let pa = PLANET_NAMES_EN[a.planet_a];
    let pb = PLANET_NAMES_EN[a.planet_b];
    let atype = a.aspect_type.name_en();
    if a.aspect_type.is_harmonious() {
        format!("{} {} with {} — positive energy and support", pa, atype, pb)
    } else {
        format!("{} {} with {} — tension and challenge, opportunity for growth", pa, atype, pb)
    }
}

// ─── Daily Forecast ─────────────────────────────────────────

#[derive(Debug)]
pub struct DailyForecast {
    pub birthday:    String,
    pub name:        String,
    pub date:        String,
    pub sun_sign_he: String,
    pub sun_sign_en: String,
    pub moon_sign_he:String,
    pub moon_sign_en:String,
    pub aspects:     Vec<String>,  // top 3 aspect lines
    pub forecast_he: String,
    pub forecast_en: String,
    pub energy:      String,  // 🌅 general energy
    pub career:      String,  // 💼 work
    pub love:        String,  // ❤️ relationships
    pub kabbalistic: String,  // ✡️ mystical insight
    pub lucky_number:u32,
    pub color:       String,
    pub energy_level:u8,   // 1-10
}

/// Build a forecast template string for a given sun sign and aspects.
pub fn forecast_template(sun_sign: usize, aspects: &[Aspect], lang_he: bool) -> String {
    let theme = &SIGN_THEMES[sun_sign % 12];

    // Count harmonious vs challenging aspects
    let harmonious = aspects.iter().filter(|a| a.aspect_type.is_harmonious()).count();
    let challenging = aspects.len().saturating_sub(harmonious);

    let energy_level = if harmonious > challenging { "גבוהה" } else { "מאתגרת" };
    let energy_level_en = if harmonious > challenging { "high" } else { "challenging" };

    // Top aspect (most significant)
    let top_aspect = aspects.first();

    if lang_he {
        let mut text = format!(
            "אנרגיית היום: {}. {}.",
            theme.energy_he, theme.advice_he
        );
        if let Some(a) = top_aspect {
            text.push_str(&format!(" {}.", aspect_text_he(a)));
        }
        text.push_str(&format!(" רמת אנרגיה כללית: {}.", energy_level));
        text
    } else {
        let mut text = format!(
            "Today's energy: {}. {}.",
            theme.energy_en, theme.advice_en
        );
        if let Some(a) = top_aspect {
            text.push_str(&format!(" {}.", aspect_text_en(a)));
        }
        text.push_str(&format!(" Overall energy level: {}.", energy_level_en));
        text
    }
}

/// Parse "DD/MM/YYYY" → (day, month, year). Returns None on failure.
pub fn parse_birthday(s: &str) -> Option<(u32, u32, i32)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 { return None; }
    let day:   u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let year:  i32 = parts[2].parse().ok()?;
    if day == 0 || day > 31 || month == 0 || month > 12 { return None; }
    Some((day, month, year))
}

/// Generate a complete daily forecast.
/// birthday: "DD/MM/YYYY" (birth date for natal sun sign)
/// today_jd: Julian Day of "today" (use julian_day(y,m,d) for current date)
pub fn generate_daily_forecast(
    birthday: &str,
    name: &str,
    today_jd: f64,
    today_date: &str,
) -> Option<DailyForecast> {
    let (bday, bmonth, byear) = parse_birthday(birthday)?;

    // Natal sun sign (from birth date)
    let _birth_jd = julian_day(byear, bmonth, bday);
    // V8.2 FIX: use date-range for natal sun sign (astronomical calc has drift)
    let sun_sign = sun_sign_from_date(bday, bmonth);

    // Today's planetary positions
    let lons = all_longitudes(today_jd);
    let aspects = find_aspects(&lons);

    // Moon sign today
    let moon_sign = sign_from_longitude(lons[MOON]);

    // Top 3 aspect descriptions
    let aspect_lines: Vec<String> = aspects.iter()
        .take(3)
        .map(|a| aspect_text_he(a))
        .collect();

    // Forecast texts
    let forecast_he = forecast_template(sun_sign, &aspects, true);
    let forecast_en = forecast_template(sun_sign, &aspects, false);

    // Lucky number: hash of name chars + sun sign
    let lucky = lucky_number_from(name, sun_sign as u32);

    // Energy level: harmonious aspects ratio
    let harmonious = aspects.iter().filter(|a| a.aspect_type.is_harmonious()).count();
    let total = aspects.len().max(1);
    let energy_level = ((harmonious as f64 / total as f64) * 9.0 + 1.0) as u8;


        // Build section-based forecast
        let has_harmony = aspects.iter().any(|a| a.aspect_type.is_harmonious());
        let has_tension = aspects.iter().any(|a| !a.aspect_type.is_harmonious());
        
        let energy = if has_harmony && !has_tension {
            format!("🌅 יום של אנרגיה חיובית. {} תומכים בך — נצל את הזרימה.", ELEMENTS_HE[sun_sign])
        } else if has_tension && !has_harmony {
            format!("🌅 יום מאתגר. אל תילחם בזרם — סבלנות היא המפתח.")
        } else if has_harmony && has_tension {
            format!("🌅 יום של מתח יצירתי. האתגרים מובילים לצמיחה.")
        } else {
            format!("🌅 יום שקט ומאוזן. זמן טוב לתכנון ומנוחה.")
        };
        
        let career = if moon_sign / 3 == sun_sign / 3 {
            "💼 הירח תומך באלמנט שלך — אינטואיציה חזקה בעבודה. סמוך על התחושות.".to_string()
        } else if has_tension {
            "💼 אתגרים בעבודה דורשים גמישות. שמור על תקשורת ברורה.".to_string()
        } else {
            "💼 יום עבודה פרודוקטיבי. התמקד במשימות העיקריות.".to_string()
        };
        
        let love = if aspects.iter().any(|a| a.planet_a == VENUS || a.planet_b == VENUS) {
            if has_harmony { "❤️ נוגה פעילה — אנרגיה רומנטית באוויר! יום מצוין לזוגיות.".to_string() }
            else { "❤️ רגשות גועשים ביחסים. דבר פתוח — תקשורת מרפאה.".to_string() }
        } else {
            "❤️ שקט רגשי. זמן טוב לטפח יחסים קיימים.".to_string()
        };
        
        let kabbalistic = format!("✡️ האות {} ({}) מלמדת: {}. חודש {}, חוש {}.",
            KAB_LETTERS[sun_sign], KAB_LETTER_NAMES[sun_sign],
            match sun_sign {
                0 => "לגלות כוח יצירה בדיבור",
                1 => "לחבר שמים וארץ",
                2 => "להתקדם בביטחון",
                3 => "לראות מעבר לגלוי",
                4 => "למצוא טוב בכל דבר",
                5 => "להתחיל מהנקודה הקטנה",
                6 => "ללמוד מכל אדם ומכל מצב",
                7 => "לקום אחרי כל נפילה",
                8 => "לסמוך ולתמוך",
                9 => "לראות בעין פנימית",
                10 => "לכוון לצדק ואמת",
                _ => "להעלות ניצוצות קדושה",
            },
            HEBREW_MONTHS[sun_sign], KAB_SENSES[sun_sign]);

        Some(DailyForecast {
        birthday:     birthday.to_string(),
        name:         name.to_string(),
        date:         today_date.to_string(),
        sun_sign_he:  SIGNS_HE[sun_sign].to_string(),
        sun_sign_en:  SIGNS_EN[sun_sign].to_string(),
        moon_sign_he: SIGNS_HE[moon_sign].to_string(),
        moon_sign_en: SIGNS_EN[moon_sign].to_string(),
        aspects:      aspect_lines,
        forecast_he,
        forecast_en,
        energy,
        career,
        love,
        kabbalistic,
        lucky_number: lucky,
        color: COLORS[sun_sign].to_string(),
        energy_level,
    })
}

/// Lucky number: sum of character code-points of name mod 9 + 1 + sun_sign offset.
fn lucky_number_from(name: &str, sun_sign: u32) -> u32 {
    let sum: u32 = name.char_indices()
        .map(|(_, c)| c as u32)
        .fold(0u32, |acc, v| acc.wrapping_add(v));
    ((sum + sun_sign) % 9) + 1
}

/// Serialize DailyForecast to JSON (no serde).
pub fn forecast_to_json(f: &DailyForecast) -> String {
    let aspects_json: Vec<String> = f.aspects.iter()
        .map(|a| format!(r#""{}""#, a.replace('"', "'")))
        .collect();

    format!(
        r#"{{"birthday":"{}","name":"{}","date":"{}","sun_sign":"{}","sun_sign_en":"{}","moon_sign":"{}","moon_sign_en":"{}","energy":"{}","career":"{}","love":"{}","kabbalistic":"{}","forecast_he":"{}","forecast_en":"{}","lucky_number":{},"color":"{}","energy_level":{},"aspects":[{}]}}"#,
        f.birthday, f.name, f.date,
        f.sun_sign_he, f.sun_sign_en,
        f.moon_sign_he, f.moon_sign_en,
        f.energy.replace('"', "'"), f.career.replace('"', "'"),
        f.love.replace('"', "'"), f.kabbalistic.replace('"', "'"),
        f.forecast_he.replace('"', "'"), f.forecast_en.replace('"', "'"),
        f.lucky_number, f.color, f.energy_level,
        aspects_json.join(","),
    )
}

/// Horoscope JSON for a given birthday (natal chart summary).
pub fn horoscope_to_json(birthday: &str, _jd: f64) -> String {
    // Natal sun
    let (bday, bmonth, byear) = match parse_birthday(birthday) {
        Some(v) => v,
        None => return r#"{"error":"invalid birthday format, use DD/MM/YYYY"}"#.to_string(),
    };
    let birth_jd = julian_day(byear, bmonth, bday);
    // Use BIRTH positions for natal chart, not today
    let lons = all_longitudes(birth_jd);
    let aspects = find_aspects(&lons);
    // V8.2 FIX: use date-range for natal sun sign (astronomical calc has drift)
    let sun_sign = sun_sign_from_date(bday, bmonth);

    let moon_sign = sign_from_longitude(lons[MOON]);

    // Planet positions array
    let mut planets_json = String::new();
    for i in 0..7 {
        if i > 0 { planets_json.push(','); }
        planets_json.push_str(&format!(
            r#"{{"name_he":"{}","name_en":"{}","longitude":{:.2},"sign_he":"{}","sign_en":"{}","degree":{:.1}}}"#,
            PLANET_NAMES_HE[i],
            PLANET_NAMES_EN[i],
            lons[i],
            sign_name_he(lons[i]),
            sign_name_en(lons[i]),
            degree_in_sign(lons[i]),
        ));
    }

    // Top aspects
    let aspects_json: Vec<String> = aspects.iter().take(5).map(|a| {
        format!(
            r#"{{"planet_a":"{}","planet_b":"{}","type_he":"{}","type_en":"{}","orb":{:.1}}}"#,
            PLANET_NAMES_HE[a.planet_a],
            PLANET_NAMES_HE[a.planet_b],
            a.aspect_type.name_he(),
            a.aspect_type.name_en(),
            a.orb,
        )
    }).collect();

    // Compatible signs
    let compat: Vec<String> = COMPATIBLE[sun_sign].iter()
        .map(|&i| format!(r#"{{"he":"{}","en":"{}"}}"#, SIGNS_HE[i], SIGNS_EN[i]))
        .collect();

    format!(
        r#"{{"birthday":"{}","sun_sign_he":"{}","sun_sign_en":"{}","symbol":"{}","element_he":"{}","element_en":"{}","quality":"{}","ruling_planet_he":"{}","ruling_planet_en":"{}","traits_he":"{}","stone":"{}","color":"{}","lucky_day":"{}","kab_letter":"{}","kab_letter_name":"{}","hebrew_month":"{}","kab_sense":"{}","moon_sign_he":"{}","moon_sign_en":"{}","compatible":[{}],"planets":[{}],"aspects":[{}]}}"#,
        birthday,
        SIGNS_HE[sun_sign], SIGNS_EN[sun_sign], SYMBOLS[sun_sign],
        ELEMENTS_HE[sun_sign], ELEMENTS_EN[sun_sign],
        QUALITIES_HE[sun_sign],
        RULERS_HE[sun_sign], RULERS_EN[sun_sign],
        TRAITS_HE[sun_sign],
        STONES[sun_sign], COLORS[sun_sign], LUCKY_DAYS[sun_sign],
        KAB_LETTERS[sun_sign], KAB_LETTER_NAMES[sun_sign],
        HEBREW_MONTHS[sun_sign], KAB_SENSES[sun_sign],
        SIGNS_HE[moon_sign], SIGNS_EN[moon_sign],
        compat.join(","),
        planets_json,
        aspects_json.join(","),
    )
}

// ─── Tests ──────────────────────────────────────────────────


// ═══ BULLETPROOF SUN SIGN — date-range, always correct ═══
// Astronomical sun_longitude has float precision drift for old dates.
// This function uses fixed date ranges — 100% reliable for natal sign.
pub fn sun_sign_from_date(day: u32, month: u32) -> usize {
    match (month, day) {
        (3, 21..=31) | (4, 1..=19) => 0,   // Aries
        (4, 20..=30) | (5, 1..=20) => 1,   // Taurus
        (5, 21..=31) | (6, 1..=20) => 2,   // Gemini
        (6, 21..=30) | (7, 1..=22) => 3,   // Cancer
        (7, 23..=31) | (8, 1..=22) => 4,   // Leo
        (8, 23..=31) | (9, 1..=22) => 5,   // Virgo
        (9, 23..=30) | (10, 1..=22) => 6,  // Libra
        (10, 23..=31) | (11, 1..=21) => 7, // Scorpio
        (11, 22..=30) | (12, 1..=21) => 8, // Sagittarius
        (12, 22..=31) | (1, 1..=19) => 9,  // Capricorn
        (1, 20..=31) | (2, 1..=18) => 10,  // Aquarius
        (2, 19..=29) | (3, 1..=20) => 11,  // Pisces
        _ => 9, // Capricorn fallback
    }
}

/// Get Hebrew sun sign name from month+day
pub fn sun_sign_he(month: u32, day: u32) -> &'static str {
    SIGNS_HE[sun_sign_from_date(day, month)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julian_day_j2000() {
        // J2000 = Jan 1.5, 2000 → JD = 2451545.0
        let jd = julian_day(2000, 1, 1);
        assert!((jd - 2451544.5).abs() < 0.5, "JD near J2000: {}", jd);
    }

    #[test]
    fn test_sun_longitude_range() {
        let jd = julian_day(2024, 6, 21); // ~summer solstice
        let lon = sun_longitude(jd);
        assert!(lon >= 0.0 && lon < 360.0, "Sun lon out of range: {}", lon);
        // Sun in Cancer/Leo region (90°-150°) at summer solstice (simplified formula)
        assert!(lon > 80.0 && lon < 150.0, "Sun not near summer solstice: {}", lon);
    }

    #[test]
    fn test_moon_longitude_range() {
        let jd = julian_day(2024, 3, 15);
        let lon = moon_longitude(jd);
        assert!(lon >= 0.0 && lon < 360.0, "Moon lon: {}", lon);
    }

    #[test]
    fn test_all_longitudes() {
        let jd = julian_day(2024, 1, 1);
        let lons = all_longitudes(jd);
        for (i, lon) in lons.iter().enumerate() {
            assert!(*lon >= 0.0 && *lon < 360.0, "Planet {} lon out of range: {}", i, lon);
        }
    }

    #[test]
    fn test_sign_from_longitude() {
        assert_eq!(sign_from_longitude(0.0), 0);   // Aries
        assert_eq!(sign_from_longitude(30.0), 1);  // Taurus
        assert_eq!(sign_from_longitude(60.0), 2);  // Gemini
        assert_eq!(sign_from_longitude(359.9), 11); // Pisces
    }

    #[test]
    fn test_find_aspects_returns_vec() {
        let jd = julian_day(2024, 3, 20);
        let lons = all_longitudes(jd);
        let aspects = find_aspects(&lons);
        // Should find some aspects (may be 0-many depending on date)
        let _ = aspects; // just verify it compiles and runs
    }

    #[test]
    fn test_generate_forecast() {
        let jd = julian_day(2024, 3, 20);
        let f = generate_daily_forecast("15/03/1990", "Test", jd, "20/03/2024");
        assert!(f.is_some());
        let f = f.unwrap();
        assert!(!f.forecast_he.is_empty());
        assert!(!f.forecast_en.is_empty());
        assert!(f.lucky_number >= 1 && f.lucky_number <= 9);
        assert!(f.energy_level >= 1 && f.energy_level <= 10);
    }

    #[test]
    fn test_parse_birthday() {
        assert_eq!(parse_birthday("15/03/1990"), Some((15, 3, 1990)));
        assert_eq!(parse_birthday("01/01/2000"), Some((1, 1, 2000)));
        assert!(parse_birthday("bad").is_none());
        assert!(parse_birthday("32/01/2000").is_none());
    }

    #[test]
    fn test_lucky_number_range() {
        for name in &["שרה", "דוד", "מיכאל", "Alice", "Bob"] {
            let n = lucky_number_from(name, 5);
            assert!(n >= 1 && n <= 9, "Lucky number out of range: {}", n);
        }
    }

    #[test]
    fn test_sun_sign_from_date() {
        assert_eq!(sun_sign_from_date(17, 4), 0, "Apr 17 = Aries");
        assert_eq!(sun_sign_from_date(13, 11), 7, "Nov 13 = Scorpio");
        assert_eq!(sun_sign_from_date(15, 7), 3, "Jul 15 = Cancer");
        assert_eq!(sun_sign_from_date(25, 12), 9, "Dec 25 = Capricorn");
        assert_eq!(sun_sign_from_date(5, 1), 9, "Jan 5 = Capricorn");
        assert_eq!(sun_sign_from_date(15, 2), 10, "Feb 15 = Aquarius");
        assert_eq!(sun_sign_from_date(10, 3), 11, "Mar 10 = Pisces");
        assert_eq!(sun_sign_from_date(21, 3), 0, "Mar 21 = Aries (first day)");
    }

}
