//! # Installation Profile (פרופיל התקנה)
//! Same brain, different goals. A fridge Cortex and a server Cortex
//! share the same code but have different curiosity, resources, and priorities.
//!
//! Like a human: a doctor and a chef have the same brain —
//! but focus their learning and attention differently.

use std::collections::HashMap;

/// Installation type determines Cortex's behavior, goals, and resource usage
#[derive(Clone, Debug, PartialEq)]
pub enum InstallationType {
    Server,      // Full brain — learn everything, unlimited resources
    Phone,       // Personal assistant — learn about THIS user
    Business,    // Domain-specific — learn market, products, customers
    Appliance,   // Minimal — learn ONE domain (fridge, car, etc.)
    Medical,     // Critical — learn with extreme caution, save lives
    Education,   // Teaching — learn to explain, adapt to student
    Security,    // Guard — learn threats, monitor, alert
}

impl InstallationType {
    pub fn name_he(&self) -> &'static str {
        match self {
            InstallationType::Server    => "שרת_מלא",
            InstallationType::Phone     => "טלפון_אישי",
            InstallationType::Business  => "עסקי",
            InstallationType::Appliance => "מכשיר",
            InstallationType::Medical   => "רפואי",
            InstallationType::Education => "חינוכי",
            InstallationType::Security  => "אבטחה",
        }
    }
}

/// What Cortex is curious about in this installation
#[derive(Clone, Debug)]
pub struct CuriosityProfile {
    /// Primary learning domains (high priority)
    pub primary_domains: Vec<String>,
    /// Secondary domains (learn when idle)
    pub secondary_domains: Vec<String>,
    /// Excluded domains (never learn — save resources)
    pub excluded_domains: Vec<String>,
    /// Max concepts to hold in RAM
    pub max_concepts: usize,
    /// Max facts in graph
    pub max_facts: usize,
    /// Learning rate: how aggressively to seek new knowledge
    pub learning_rate: f32,     // 0.0=passive, 1.0=aggressive
    /// Exploration vs exploitation balance
    pub exploration: f32,       // 0.0=only use known, 1.0=always explore new
}

/// Resource budget for this installation
#[derive(Clone, Debug)]
pub struct ResourceBudget {
    pub max_ram_mb: usize,
    pub max_storage_mb: usize,
    pub max_cpu_percent: u8,
    pub can_access_internet: bool,
    pub can_sync_from_server: bool,
    pub sync_interval_secs: u64,
    pub nightmode_enabled: bool,
}

/// The full installation profile
#[derive(Clone, Debug)]
pub struct InstallationProfile {
    pub installation_type: InstallationType,
    pub instance_id: String,        // unique per installation
    pub instance_name: String,      // "Idan's phone", "CHOOZ server"
    pub curiosity: CuriosityProfile,
    pub resources: ResourceBudget,
    pub authority_default: String,   // default authority level for users
    pub language: String,            // primary language
    pub personality_preset: String,  // "warm", "professional", "minimal"
}

impl InstallationProfile {
    /// Server: full brain, learn everything
    pub fn server(name: &str) -> Self {
        InstallationProfile {
            installation_type: InstallationType::Server,
            instance_id: format!("srv_{}", name),
            instance_name: name.into(),
            curiosity: CuriosityProfile {
                primary_domains: vec![
                    "science".into(), "history".into(), "philosophy".into(),
                    "torah".into(), "kabbalah".into(), "mathematics".into(),
                    "technology".into(), "economics".into(), "psychology".into(),
                ],
                secondary_domains: vec![
                    "art".into(), "literature".into(), "music".into(),
                    "geography".into(), "law".into(), "medicine".into(),
                ],
                excluded_domains: vec![],
                max_concepts: 10_000_000,
                max_facts: 100_000_000,
                learning_rate: 0.9,
                exploration: 0.7,
            },
            resources: ResourceBudget {
                max_ram_mb: 40_000,
                max_storage_mb: 500_000,
                max_cpu_percent: 90,
                can_access_internet: true,
                can_sync_from_server: false, // IS the server
                sync_interval_secs: 0,
                nightmode_enabled: true,
            },
            authority_default: "owner".into(),
            language: "he".into(),
            personality_preset: "wise_warm".into(),
        }
    }

    /// Phone: personal assistant
    pub fn phone(user_name: &str) -> Self {
        InstallationProfile {
            installation_type: InstallationType::Phone,
            instance_id: format!("phone_{}", user_name),
            instance_name: format!("{}_{}", user_name, "phone"),
            curiosity: CuriosityProfile {
                primary_domains: vec![
                    "user_preferences".into(), "schedule".into(),
                    "contacts".into(), "habits".into(), "health".into(),
                ],
                secondary_domains: vec![
                    "news".into(), "weather".into(), "navigation".into(),
                ],
                excluded_domains: vec![
                    "deep_science".into(), "academic".into(),
                ],
                max_concepts: 50_000,
                max_facts: 500_000,
                learning_rate: 0.5,
                exploration: 0.3,
            },
            resources: ResourceBudget {
                max_ram_mb: 512,
                max_storage_mb: 2_000,
                max_cpu_percent: 30,
                can_access_internet: true,
                can_sync_from_server: true,
                sync_interval_secs: 3600, // sync hourly
                nightmode_enabled: false,  // phone sleeps
            },
            authority_default: "owner".into(),
            language: "he".into(),
            personality_preset: "friendly_helpful".into(),
        }
    }

    /// Business: domain-specific (e.g., CHOOZ)
    pub fn business(company: &str, domain: &str) -> Self {
        InstallationProfile {
            installation_type: InstallationType::Business,
            instance_id: format!("biz_{}", company),
            instance_name: format!("{} Cortex", company),
            curiosity: CuriosityProfile {
                primary_domains: vec![
                    domain.into(), "pricing".into(), "inventory".into(),
                    "customers".into(), "competitors".into(), "market".into(),
                ],
                secondary_domains: vec![
                    "logistics".into(), "marketing".into(), "legal".into(),
                ],
                excluded_domains: vec![
                    "kabbalah".into(), "astrology".into(), "music".into(),
                ],
                max_concepts: 500_000,
                max_facts: 5_000_000,
                learning_rate: 0.7,
                exploration: 0.4,
            },
            resources: ResourceBudget {
                max_ram_mb: 4_000,
                max_storage_mb: 50_000,
                max_cpu_percent: 50,
                can_access_internet: true,
                can_sync_from_server: true,
                sync_interval_secs: 600, // sync every 10 min
                nightmode_enabled: true,
            },
            authority_default: "delegated".into(),
            language: "he".into(),
            personality_preset: "professional".into(),
        }
    }

    /// Appliance: minimal, focused (fridge, car, etc.)
    pub fn appliance(device: &str, domain: &str) -> Self {
        InstallationProfile {
            installation_type: InstallationType::Appliance,
            instance_id: format!("dev_{}", device),
            instance_name: format!("{} Cortex", device),
            curiosity: CuriosityProfile {
                primary_domains: vec![domain.into()],
                secondary_domains: vec![],
                excluded_domains: vec![ // exclude almost everything
                    "history".into(), "philosophy".into(), "kabbalah".into(),
                    "astrology".into(), "finance".into(), "music".into(),
                    "art".into(), "literature".into(), "politics".into(),
                ],
                max_concepts: 10_000,
                max_facts: 100_000,
                learning_rate: 0.3,
                exploration: 0.1,
            },
            resources: ResourceBudget {
                max_ram_mb: 128,
                max_storage_mb: 500,
                max_cpu_percent: 15,
                can_access_internet: false,
                can_sync_from_server: true,
                sync_interval_secs: 86400, // sync daily
                nightmode_enabled: false,
            },
            authority_default: "public".into(),
            language: "he".into(),
            personality_preset: "minimal_efficient".into(),
        }
    }

    /// Medical: critical, cautious
    pub fn medical(facility: &str) -> Self {
        InstallationProfile {
            installation_type: InstallationType::Medical,
            instance_id: format!("med_{}", facility),
            instance_name: format!("{} Medical Cortex", facility),
            curiosity: CuriosityProfile {
                primary_domains: vec![
                    "medicine".into(), "pharmacology".into(), "anatomy".into(),
                    "diagnostics".into(), "drug_interactions".into(),
                ],
                secondary_domains: vec![
                    "nutrition".into(), "psychology".into(), "genetics".into(),
                ],
                excluded_domains: vec![
                    "kabbalah".into(), "astrology".into(), "finance".into(),
                ],
                max_concepts: 1_000_000,
                max_facts: 10_000_000,
                learning_rate: 0.8,   // learn aggressively BUT verify
                exploration: 0.2,     // conservative — stick to verified
            },
            resources: ResourceBudget {
                max_ram_mb: 8_000,
                max_storage_mb: 100_000,
                max_cpu_percent: 60,
                can_access_internet: true,
                can_sync_from_server: true,
                sync_interval_secs: 300,
                nightmode_enabled: true,
            },
            authority_default: "delegated".into(),
            language: "he".into(),
            personality_preset: "precise_cautious".into(),
        }
    }

    /// Should this installation learn about a given topic?
    pub fn should_learn(&self, topic: &str) -> LearningDecision {
        let topic_lower = topic.to_lowercase();

        // Check excluded first
        for exc in &self.curiosity.excluded_domains {
            if topic_lower.contains(&exc.to_lowercase()) {
                return LearningDecision::Skip {
                    reason: format!("domain '{}' excluded in {} profile", exc, self.installation_type.name_he()),
                };
            }
        }

        // Check primary
        for pri in &self.curiosity.primary_domains {
            if topic_lower.contains(&pri.to_lowercase()) {
                return LearningDecision::LearnNow {
                    priority: 8,
                    reason: format!("primary domain: {}", pri),
                };
            }
        }

        // Check secondary
        for sec in &self.curiosity.secondary_domains {
            if topic_lower.contains(&sec.to_lowercase()) {
                return LearningDecision::LearnLater {
                    priority: 4,
                    reason: format!("secondary domain: {}", sec),
                };
            }
        }

        // Unknown domain — depends on exploration setting
        if self.curiosity.exploration > 0.5 {
            LearningDecision::LearnLater {
                priority: 3,
                reason: "unknown domain, high exploration setting".into(),
            }
        } else {
            LearningDecision::Skip {
                reason: "unknown domain, low exploration setting".into(),
            }
        }
    }

    /// Check if resource budget allows an operation
    pub fn can_afford(&self, ram_mb: usize, storage_mb: usize) -> bool {
        ram_mb <= self.resources.max_ram_mb && storage_mb <= self.resources.max_storage_mb
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"type":"{}","name":"{}","language":"{}","personality":"{}","max_concepts":{},"learning_rate":{:.1},"exploration":{:.1},"ram_mb":{},"internet":{}}}"#,
            self.installation_type.name_he(), self.instance_name,
            self.language, self.personality_preset,
            self.curiosity.max_concepts, self.curiosity.learning_rate,
            self.curiosity.exploration, self.resources.max_ram_mb,
            self.resources.can_access_internet
        )
    }
}

#[derive(Clone, Debug)]
pub enum LearningDecision {
    LearnNow { priority: u8, reason: String },
    LearnLater { priority: u8, reason: String },
    Skip { reason: String },
}

impl LearningDecision {
    pub fn should_learn(&self) -> bool {
        !matches!(self, LearningDecision::Skip { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_learns_everything() {
        let srv = InstallationProfile::server("dinio");
        assert!(srv.should_learn("quantum physics").should_learn());
        assert!(srv.should_learn("kabbalah zohar").should_learn());
        assert!(srv.should_learn("random_topic").should_learn()); // high exploration
    }

    #[test]
    fn test_phone_focused() {
        let phone = InstallationProfile::phone("idan");
        assert!(phone.should_learn("user_preferences").should_learn());
        assert!(phone.should_learn("schedule meeting").should_learn());
        assert!(!phone.should_learn("deep_science quantum").should_learn());
    }

    #[test]
    fn test_fridge_minimal() {
        let fridge = InstallationProfile::appliance("fridge", "food");
        assert!(fridge.should_learn("food safety").should_learn());
        assert!(!fridge.should_learn("philosophy ethics").should_learn());
        assert!(!fridge.should_learn("kabbalah zohar").should_learn());
        assert!(!fridge.should_learn("random topic").should_learn()); // low exploration
    }

    #[test]
    fn test_medical_cautious() {
        let med = InstallationProfile::medical("hospital");
        assert!(med.should_learn("medicine diagnosis").should_learn());
        assert!(med.should_learn("drug_interactions aspirin").should_learn());
        assert!(!med.should_learn("astrology chart").should_learn());
        assert!(med.curiosity.exploration < 0.3); // conservative
    }

    #[test]
    fn test_business_chooz() {
        let biz = InstallationProfile::business("CHOOZ", "promotional_products");
        assert!(biz.should_learn("pricing strategy").should_learn());
        assert!(biz.should_learn("customers retention").should_learn());
        assert!(!biz.should_learn("kabbalah meditation").should_learn());
    }

    #[test]
    fn test_resource_budget() {
        let fridge = InstallationProfile::appliance("fridge", "food");
        assert!(!fridge.can_afford(256, 100)); // 256MB > 128MB limit
        assert!(fridge.can_afford(64, 100));
        
        let server = InstallationProfile::server("main");
        assert!(server.can_afford(30_000, 400_000));
    }

    #[test]
    fn test_different_personalities() {
        assert_eq!(InstallationProfile::server("x").personality_preset, "wise_warm");
        assert_eq!(InstallationProfile::phone("x").personality_preset, "friendly_helpful");
        assert_eq!(InstallationProfile::business("x","y").personality_preset, "professional");
        assert_eq!(InstallationProfile::appliance("x","y").personality_preset, "minimal_efficient");
        assert_eq!(InstallationProfile::medical("x").personality_preset, "precise_cautious");
    }
}
