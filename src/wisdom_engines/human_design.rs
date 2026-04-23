//! # Human Design Calculator (עיצוב אנושי)
//! BodyGraph from birth date/time. Based on I Ching gates + astro positions.
//! 9 centers, 64 gates, 36 channels, 5 types, 12 profiles.

/// The 5 Human Design Types
#[derive(Clone, Debug, PartialEq)]
pub enum HdType {
    Manifestor,        // ~9% — initiate, inform
    Generator,         // ~37% — respond, sacral authority
    ManifestingGenerator, // ~33% — multi-passionate, respond+initiate
    Projector,         // ~20% — guide, wait for invitation
    Reflector,         // ~1% — mirror, wait lunar cycle
}

impl HdType {
    pub fn strategy(&self) -> &'static str {
        match self {
            HdType::Manifestor => "inform before acting",
            HdType::Generator => "wait to respond",
            HdType::ManifestingGenerator => "wait to respond, then inform",
            HdType::Projector => "wait for invitation",
            HdType::Reflector => "wait 28 days (lunar cycle)",
        }
    }
    pub fn strategy_he(&self) -> &'static str {
        match self {
            HdType::Manifestor => "ליידע לפני פעולה",
            HdType::Generator => "להמתין לתגובה",
            HdType::ManifestingGenerator => "להמתין לתגובה ואז ליידע",
            HdType::Projector => "להמתין להזמנה",
            HdType::Reflector => "להמתין 28 יום (מחזור ירחי)",
        }
    }
    pub fn not_self_theme(&self) -> &'static str {
        match self {
            HdType::Manifestor => "anger",
            HdType::Generator | HdType::ManifestingGenerator => "frustration",
            HdType::Projector => "bitterness",
            HdType::Reflector => "disappointment",
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            HdType::Manifestor => "manifestor",
            HdType::Generator => "generator",
            HdType::ManifestingGenerator => "manifesting_generator",
            HdType::Projector => "projector",
            HdType::Reflector => "reflector",
        }
    }
}

/// 9 Centers of the BodyGraph
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Center {
    Head,       // Inspiration, pressure to think
    Ajna,       // Conceptualization, mental awareness
    Throat,     // Communication, manifestation
    G,          // Identity, direction, love
    Heart,      // Willpower, ego, material world
    Sacral,     // Life force, sexuality, work energy
    SolarPlexus,// Emotions, feelings, spirit
    Spleen,     // Intuition, health, survival instincts
    Root,       // Adrenaline, pressure, stress
}

impl Center {
    pub fn name_he(&self) -> &'static str {
        match self {
            Center::Head => "ראש",
            Center::Ajna => "אג'נה",
            Center::Throat => "גרון",
            Center::G => "זהות",
            Center::Heart => "לב",
            Center::Sacral => "סקרל",
            Center::SolarPlexus => "מקלעת_שמש",
            Center::Spleen => "טחול",
            Center::Root => "שורש",
        }
    }

    pub fn all() -> [Center; 9] {
        [Center::Head, Center::Ajna, Center::Throat, Center::G,
         Center::Heart, Center::Sacral, Center::SolarPlexus,
         Center::Spleen, Center::Root]
    }
}

/// Profile: combination of two line numbers (1-6)
#[derive(Clone, Debug)]
pub struct Profile {
    pub conscious: u8,   // 1-6 (personality/sun)
    pub unconscious: u8, // 1-6 (design/earth)
}

impl Profile {
    pub fn name(&self) -> String {
        let names = ["", "Investigator", "Hermit", "Martyr", "Opportunist", "Heretic", "Role Model"];
        format!("{}/{} ({}-{})", self.conscious, self.unconscious,
            names.get(self.conscious as usize).unwrap_or(&"?"),
            names.get(self.unconscious as usize).unwrap_or(&"?"))
    }
}

/// Gate: one of 64 I Ching hexagrams mapped to a center
#[derive(Clone, Debug)]
pub struct Gate {
    pub number: u8,       // 1-64
    pub center: Center,
    pub name: &'static str,
}

/// Channel: connection between two gates/centers
#[derive(Clone, Debug)]
pub struct Channel {
    pub gate1: u8,
    pub gate2: u8,
    pub name: &'static str,
}

/// Simplified BodyGraph result
#[derive(Clone, Debug)]
pub struct BodyGraph {
    pub hd_type: HdType,
    pub profile: Profile,
    pub defined_centers: Vec<Center>,
    pub undefined_centers: Vec<Center>,
    pub authority: &'static str,
    pub active_gates: Vec<u8>,
}

impl BodyGraph {
    /// Determine type from defined centers
    pub fn determine_type(defined: &[Center]) -> HdType {
        let has_sacral = defined.contains(&Center::Sacral);
        let has_throat = defined.contains(&Center::Throat);
        let has_motor_to_throat = defined.contains(&Center::Heart) ||
            defined.contains(&Center::SolarPlexus) ||
            defined.contains(&Center::Root);

        if defined.is_empty() || defined.len() <= 1 {
            return HdType::Reflector;
        }

        if has_sacral && has_throat && has_motor_to_throat {
            HdType::ManifestingGenerator
        } else if has_sacral {
            HdType::Generator
        } else if has_throat && has_motor_to_throat {
            HdType::Manifestor
        } else {
            HdType::Projector
        }
    }

    /// Determine inner authority from defined centers
    pub fn determine_authority(defined: &[Center]) -> &'static str {
        if defined.contains(&Center::SolarPlexus) { "emotional" }
        else if defined.contains(&Center::Sacral) { "sacral" }
        else if defined.contains(&Center::Spleen) { "splenic" }
        else if defined.contains(&Center::Heart) { "ego" }
        else if defined.contains(&Center::G) { "self-projected" }
        else if defined.contains(&Center::Ajna) { "mental" }
        else { "lunar" } // reflector
    }

    /// Calculate from seed (deterministic from birth timestamp)
    pub fn from_seed(seed: u64) -> Self {
        let mut s = seed;
        let lcg = |s: &mut u64| -> u64 {
            *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            *s >> 33
        };

        // Determine defined centers (typically 4-6 out of 9)
        let all_centers = Center::all();
        let mut defined = Vec::new();
        let mut undefined = Vec::new();
        for &center in &all_centers {
            if lcg(&mut s) % 3 != 0 { // ~67% chance defined
                defined.push(center);
            } else {
                undefined.push(center);
            }
        }

        // Profile
        let conscious = (lcg(&mut s) % 6 + 1) as u8;
        let unconscious = (lcg(&mut s) % 6 + 1) as u8;

        // Active gates (typically 13-26 out of 64)
        let mut gates = Vec::new();
        for g in 1..=64u8 {
            if lcg(&mut s) % 4 == 0 { gates.push(g); } // ~25% chance
        }

        let hd_type = Self::determine_type(&defined);
        let authority = Self::determine_authority(&defined);

        BodyGraph {
            hd_type, profile: Profile { conscious, unconscious },
            defined_centers: defined, undefined_centers: undefined,
            authority, active_gates: gates,
        }
    }

    pub fn to_json(&self) -> String {
        let def: Vec<&str> = self.defined_centers.iter().map(|c| c.name_he()).collect();
        let undef: Vec<&str> = self.undefined_centers.iter().map(|c| c.name_he()).collect();
        format!(
            r#"{{"type":"{}","strategy":"{}","authority":"{}","profile":"{}","defined":[{}],"undefined":[{}],"gates":{}}}"#,
            self.hd_type.as_str(), self.hd_type.strategy_he(), self.authority,
            self.profile.name(),
            def.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(","),
            undef.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(","),
            self.active_gates.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_type_generator() {
        let defined = vec![Center::Sacral, Center::Root, Center::Spleen];
        assert_eq!(BodyGraph::determine_type(&defined), HdType::Generator);
    }
    #[test] fn test_type_projector() {
        let defined = vec![Center::Ajna, Center::Head, Center::G];
        assert_eq!(BodyGraph::determine_type(&defined), HdType::Projector);
    }
    #[test] fn test_type_manifestor() {
        let defined = vec![Center::Throat, Center::Heart];
        assert_eq!(BodyGraph::determine_type(&defined), HdType::Manifestor);
    }
    #[test] fn test_type_reflector() {
        assert_eq!(BodyGraph::determine_type(&[]), HdType::Reflector);
    }
    #[test] fn test_authority_emotional() {
        let defined = vec![Center::Sacral, Center::SolarPlexus];
        assert_eq!(BodyGraph::determine_authority(&defined), "emotional");
    }
    #[test] fn test_from_seed_deterministic() {
        let bg1 = BodyGraph::from_seed(19770404);
        let bg2 = BodyGraph::from_seed(19770404);
        assert_eq!(bg1.hd_type, bg2.hd_type);
        assert_eq!(bg1.profile.conscious, bg2.profile.conscious);
    }
    #[test] fn test_profile_name() {
        let p = Profile { conscious: 3, unconscious: 5 };
        assert!(p.name().contains("3/5"));
        assert!(p.name().contains("Martyr"));
    }
}
