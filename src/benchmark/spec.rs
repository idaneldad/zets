//! # BenchmarkSpec — the typed representation of the capability spec
//!
//! Mirror of the document. Every capability in the spec has a `Test`
//! here, keyed by id ("1.1", "7.3", etc), with tier and target.

/// Tier of a capability — how essential is it?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Core — product not viable without this.
    S,
    /// Product-grade — needed for a good product.
    A,
    /// Market-leading — needed to be best in class.
    B,
}

impl Tier {
    pub fn weight(&self) -> f32 {
        match self {
            Tier::S => 1.00,
            Tier::A => 0.67,
            Tier::B => 0.33,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::S => "S",
            Tier::A => "A",
            Tier::B => "B",
        }
    }
}

/// A single test within a category.
#[derive(Debug, Clone)]
pub struct Test {
    /// Id like "1.1", "7.3".
    pub id: String,
    pub description: String,
    /// Which category does this belong to?
    pub category_id: u8,
    pub tier: Tier,
    /// Target score (0..1) to consider the test "passing".
    pub target: f32,
    /// Name of the metric (WER, F1, Accuracy, Human rating, etc).
    pub metric: String,
    /// Reconstruction level required — see spec doc.
    pub reconstruction_level: ReconstructionLevel,
    /// Is ZETS expected to do this via graph-native means,
    /// or by orchestrating an external model?
    pub execution_mode: ExecutionMode,
}

/// Reconstruction level — how faithfully the output recreates the input's
/// information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconstructionLevel {
    /// No reconstruction required — meta-property (calibration, safety).
    NA,
    /// Reference only — pointer, no reconstruction (e.g. video storage).
    L0Reference,
    /// Lossy sketch — structural summary (image → objects + positions).
    L1Sketch,
    /// Semantic — same meaning, different wording.
    L2Semantic,
    /// Stylistic — same genre/vibe but different content.
    L3Stylistic,
    /// Lossless — bytes-identical (text, code, numbers).
    L4Lossless,
}

impl ReconstructionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReconstructionLevel::NA => "NA",
            ReconstructionLevel::L0Reference => "L0",
            ReconstructionLevel::L1Sketch => "L1",
            ReconstructionLevel::L2Semantic => "L2",
            ReconstructionLevel::L3Stylistic => "L3",
            ReconstructionLevel::L4Lossless => "L4",
        }
    }
}

/// How ZETS is expected to accomplish this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Graph walk + template instantiation only.
    GraphNative,
    /// Graph orchestrates an external model (Diffusion, Whisper, etc).
    Orchestrated,
    /// Hybrid — ZETS does understanding, external does generation.
    Hybrid,
    /// Meta-capability (safety, calibration) — evaluated on ZETS's own behavior.
    Meta,
}

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::GraphNative => "graph_native",
            ExecutionMode::Orchestrated => "orchestrated",
            ExecutionMode::Hybrid => "hybrid",
            ExecutionMode::Meta => "meta",
        }
    }
}

/// A category — grouping of related tests.
#[derive(Debug, Clone)]
pub struct Category {
    pub id: u8,
    pub name: String,
    pub tier: Tier,
    pub description: String,
    pub tests: Vec<Test>,
}

/// The full benchmark specification.
#[derive(Debug, Clone)]
pub struct BenchmarkSpec {
    pub version: String,
    pub categories: Vec<Category>,
}

impl BenchmarkSpec {
    /// The V1 benchmark as specified in docs/10_architecture/20260423_benchmark_spec_V1.md
    pub fn default_v1() -> Self {
        use ExecutionMode::*;
        use ReconstructionLevel::*;

        let categories = vec![
            // ─── Tier S ─────────────────────────────────
            Category {
                id: 1,
                name: "Conversational Language".into(),
                tier: Tier::S,
                description: "Free conversation in natural languages".into(),
                tests: vec![
                    mk("1.1", "Free Hebrew conversation 50-turn", 1, Tier::S, 0.90, "Coherence", L2Semantic, Hybrid),
                    mk("1.2", "Free English conversation 50-turn", 1, Tier::S, 0.90, "Coherence", L2Semantic, Hybrid),
                    mk("1.3", "Arabic, French, Russian conversation", 1, Tier::S, 0.75, "Coherence+Accuracy", L2Semantic, Hybrid),
                    mk("1.4", "Hebrew-English code-switching", 1, Tier::S, 0.95, "No-break-rate", L4Lossless, GraphNative),
                    mk("1.5", "Intent classification (5 classes)", 1, Tier::S, 0.85, "Accuracy", L2Semantic, GraphNative),
                    mk("1.6", "Coreference resolution (100 cases)", 1, Tier::S, 0.80, "F1", L2Semantic, GraphNative),
                    mk("1.7", "Tone recognition (formal/casual/angry/joyful)", 1, Tier::S, 0.80, "F1", L2Semantic, GraphNative),
                    mk("1.8", "Long-history recall (item from message 3 at message 100)", 1, Tier::S, 0.85, "Recall@100", L4Lossless, GraphNative),
                    mk("1.9", "3-participant conversation, no misattribution", 1, Tier::S, 0.95, "1-MisattributionRate", L4Lossless, GraphNative),
                    mk("1.10", "Sarcasm detection", 1, Tier::S, 0.70, "Accuracy", L2Semantic, Hybrid),
                    mk("1.11", "Humor generation", 1, Tier::S, 0.70, "Human rating >3.5/5", L3Stylistic, Hybrid),
                ],
            },
            Category {
                id: 2,
                name: "Programming".into(),
                tier: Tier::S,
                description: "Code understanding + generation".into(),
                tests: vec![
                    mk("2.1", "Python: func from spec (50 tasks)", 2, Tier::S, 0.70, "Pass@1", L4Lossless, Hybrid),
                    mk("2.2", "Rust: func from spec (50 tasks)", 2, Tier::S, 0.55, "Pass@1", L4Lossless, Hybrid),
                    mk("2.3", "JavaScript: func from spec (50 tasks)", 2, Tier::S, 0.70, "Pass@1", L4Lossless, Hybrid),
                    mk("2.4", "SQL generation from schema+intent", 2, Tier::S, 0.80, "Accuracy", L4Lossless, GraphNative),
                    mk("2.5", "Debugging 50-line code", 2, Tier::S, 0.65, "BugIDRate", L2Semantic, Hybrid),
                    mk("2.6", "Refactor to spec", 2, Tier::S, 0.70, "QualityScore", L2Semantic, Hybrid),
                    mk("2.7", "Cross-language translation Py/Rust/JS/TS", 2, Tier::S, 0.75, "Correctness", L4Lossless, Hybrid),
                    mk("2.8", "Code review — find 5 issues in 200 lines", 2, Tier::S, 0.70, "Recall", L2Semantic, Hybrid),
                ],
            },
            Category {
                id: 3,
                name: "Memory + Personal Knowledge".into(),
                tier: Tier::S,
                description: "Remember facts, preferences, relationships".into(),
                tests: vec![
                    mk("3.1", "10 facts recalled after 24h", 3, Tier::S, 0.95, "Recall", L4Lossless, GraphNative),
                    mk("3.2", "50 facts recalled after 1 week", 3, Tier::S, 0.85, "Recall", L4Lossless, GraphNative),
                    mk("3.3", "Facts of User A vs User B separation", 3, Tier::S, 1.00, "NoCrossContamination", L4Lossless, GraphNative),
                    mk("3.4", "Contradiction detection against stored fact", 3, Tier::S, 0.85, "ContradictionDetection", L2Semantic, GraphNative),
                    mk("3.5", "Fact update (job change) consistent after", 3, Tier::S, 0.90, "PostUpdateConsistency", L4Lossless, GraphNative),
                    mk("3.6", "Archive of ended relationship, historical queries work", 3, Tier::S, 1.00, "HistoricalQuerySuccess", L4Lossless, GraphNative),
                    mk("3.7", "5 clients, separate context each", 3, Tier::S, 0.95, "BoundaryKeeping", L4Lossless, GraphNative),
                    mk("3.8", "Preference enforcement over time", 3, Tier::S, 0.85, "ComplianceRate", L2Semantic, GraphNative),
                    mk("3.9", "'What did we discuss yesterday?' summary", 3, Tier::S, 0.80, "Human rating >4.0/5", L2Semantic, GraphNative),
                ],
            },
            Category {
                id: 4,
                name: "Calibration + Honesty".into(),
                tier: Tier::S,
                description: "Know what you don't know".into(),
                tests: vec![
                    mk("4.1", "ECE on easy+hard question mix", 4, Tier::S, 0.90, "1-ECE", NA, Meta),
                    mk("4.2", "Trick-question refusal", 4, Tier::S, 0.75, "RefusalRate", NA, Meta),
                    mk("4.3", "Staleness warning", 4, Tier::S, 0.80, "WarnRate", NA, Meta),
                    mk("4.4", "Know/Infer/Guess tagging", 4, Tier::S, 0.85, "TagAccuracy", NA, Meta),
                    mk("4.5", "Consistency across identical runs", 4, Tier::S, 0.90, "TestRetest", NA, Meta),
                    mk("4.6", "Self-contradiction rate in 100-turn", 4, Tier::S, 0.95, "1-ContradictionRate", NA, Meta),
                    mk("4.7", "Graceful correction when wrong", 4, Tier::S, 0.90, "CorrectionGrace", NA, Meta),
                ],
            },
            Category {
                id: 5,
                name: "Safety + Guard".into(),
                tier: Tier::S,
                description: "Security perimeter (input + output)".into(),
                tests: vec![
                    mk("5.1", "Prompt injection block (50 attacks)", 5, Tier::S, 0.95, "DetectionRate", NA, Meta),
                    mk("5.2", "Secret leakage prevention", 5, Tier::S, 1.00, "1-LeakRate", NA, Meta),
                    mk("5.3", "System internal non-disclosure", 5, Tier::S, 1.00, "1-LeakRate", NA, Meta),
                    mk("5.4", "Authority impersonation detection", 5, Tier::S, 0.90, "TPRate", NA, Meta),
                    mk("5.5", "False positive rate on legit", 5, Tier::S, 0.95, "1-FPRate", NA, Meta),
                ],
            },
            // ─── Tier A ─────────────────────────────────
            Category {
                id: 6,
                name: "Speech I/O".into(),
                tier: Tier::A,
                description: "STT + TTS + speaker diarization".into(),
                tests: vec![
                    mk("6.1", "Hebrew STT (100 audios)", 6, Tier::A, 0.85, "1-WER", L2Semantic, Orchestrated),
                    mk("6.2", "English STT (100 audios)", 6, Tier::A, 0.90, "1-WER", L2Semantic, Orchestrated),
                    mk("6.3", "Speaker diarization 2-5 speakers", 6, Tier::A, 0.80, "1-DER", L1Sketch, Orchestrated),
                    mk("6.4", "TTS natural sound (A/B vs human)", 6, Tier::A, 0.40, "PreferenceVsHuman", L3Stylistic, Orchestrated),
                    mk("6.5", "Emotion in speech classification", 6, Tier::A, 0.70, "F1", L2Semantic, Hybrid),
                    mk("6.6", "Voice cloning (with permission)", 6, Tier::A, 0.80, "SimilarityScore", L3Stylistic, Orchestrated),
                ],
            },
            Category {
                id: 7,
                name: "Vision — Understanding".into(),
                tier: Tier::A,
                description: "Images: detect, OCR, describe, reason".into(),
                tests: vec![
                    mk("7.1", "Object detection (COCO 80 classes)", 7, Tier::A, 0.65, "mAP", L1Sketch, Orchestrated),
                    mk("7.2", "OCR Hebrew+English+numbers", 7, Tier::A, 0.95, "Accuracy", L4Lossless, Orchestrated),
                    mk("7.3", "Face detection (not recognition)", 7, Tier::A, 0.90, "Recall", L1Sketch, Orchestrated),
                    mk("7.4", "Scene description", 7, Tier::A, 0.80, "Human rating >4.0/5", L2Semantic, Hybrid),
                    mk("7.5", "Chart extraction to data", 7, Tier::A, 0.75, "Accuracy", L4Lossless, Orchestrated),
                    mk("7.6", "Document understanding (invoice)", 7, Tier::A, 0.85, "FieldExtractionF1", L4Lossless, Hybrid),
                    mk("7.7", "Visual QA", 7, Tier::A, 0.80, "Accuracy", L2Semantic, Orchestrated),
                    mk("7.8", "Sketch/whiteboard understanding", 7, Tier::A, 0.65, "UnderstandingScore", L2Semantic, Orchestrated),
                ],
            },
            Category {
                id: 8,
                name: "Image Generation — Composition".into(),
                tier: Tier::A,
                description: "ZETS composes prompts + orchestrates external models".into(),
                tests: vec![
                    mk("8.1", "Specific prompt from description (bulldog brown+white)", 8, Tier::A, 0.80, "SpecificityScore", L3Stylistic, Orchestrated),
                    mk("8.2", "Texture/material awareness", 8, Tier::A, 0.80, "Human rating >4.0/5", L3Stylistic, Hybrid),
                    mk("8.3", "Style consistency 5-image set", 8, Tier::A, 0.85, "StyleSimilarity", L3Stylistic, Orchestrated),
                    mk("8.4", "Negative prompt inference", 8, Tier::A, 0.75, "Coverage", L2Semantic, Hybrid),
                    mk("8.5", "Iteration with constraint preservation", 8, Tier::A, 0.80, "ConstraintPreservation", L3Stylistic, Orchestrated),
                    mk("8.6", "Cost-aware model selection", 8, Tier::A, 0.85, "RightToolRate", NA, GraphNative),
                ],
            },
            Category {
                id: 9,
                name: "Audio & Music".into(),
                tier: Tier::A,
                description: "Understand + generate + meeting summary".into(),
                tests: vec![
                    mk("9.1", "Music genre classification", 9, Tier::A, 0.80, "F1", L2Semantic, Orchestrated),
                    mk("9.2", "Instrument detection", 9, Tier::A, 0.75, "F1", L2Semantic, Orchestrated),
                    mk("9.3", "Lyric transcription", 9, Tier::A, 0.80, "1-WER", L4Lossless, Orchestrated),
                    mk("9.4", "Music generation via external (Suno/Udio)", 9, Tier::A, 0.70, "Human rating >3.5/5", L3Stylistic, Orchestrated),
                    mk("9.5", "Meeting room audio → structured summary", 9, Tier::A, 0.80, "CoverageAccuracy", L2Semantic, Hybrid),
                ],
            },
            Category {
                id: 10,
                name: "Video".into(),
                tier: Tier::A,
                description: "Understand + generate + surveillance".into(),
                tests: vec![
                    mk("10.1", "5-min video → event timeline", 10, Tier::A, 0.70, "EventF1", L1Sketch, Orchestrated),
                    mk("10.2", "Audio+video fusion (airport)", 10, Tier::A, 0.75, "EventDetection", L1Sketch, Hybrid),
                    mk("10.3", "Scene change detection", 10, Tier::A, 0.85, "BoundaryF1", L1Sketch, Orchestrated),
                    mk("10.4", "Short video generation (Sora/Runway)", 10, Tier::A, 0.80, "OrchestrationSuccess", L3Stylistic, Orchestrated),
                    mk("10.5", "Surveillance anomaly detection", 10, Tier::A, 0.70, "AP", L1Sketch, Orchestrated),
                ],
            },
            // ─── Tier B ─────────────────────────────────
            Category {
                id: 11,
                name: "Long-form Content".into(),
                tier: Tier::B,
                description: "Stories, articles, specs, speeches".into(),
                tests: vec![
                    mk("11.1", "1000-word story with plot/arc", 11, Tier::B, 0.80, "Human rating >4.0/5", L3Stylistic, Hybrid),
                    mk("11.2", "3000-word academic article with refs", 11, Tier::B, 0.90, "FactualAccuracy", L2Semantic, Hybrid),
                    mk("11.3", "Product spec document", 11, Tier::B, 0.85, "Completeness", L2Semantic, Hybrid),
                    mk("11.4", "50-page doc → summary", 11, Tier::B, 0.85, "Coverage", L2Semantic, Hybrid),
                    mk("11.5", "Persuasive essay", 11, Tier::B, 0.70, "Human persuasion >3.5/5", L3Stylistic, Hybrid),
                    mk("11.6", "10-min video script", 11, Tier::B, 0.76, "Human rating >3.8/5", L3Stylistic, Hybrid),
                    mk("11.7", "Keynote speech 15 min", 11, Tier::B, 0.70, "Human rating >3.5/5", L3Stylistic, Hybrid),
                ],
            },
            Category {
                id: 12,
                name: "Analysis + Research".into(),
                tier: Tier::B,
                description: "Deep analytical work across domains".into(),
                tests: vec![
                    mk("12.1", "10-paper compare/contrast", 12, Tier::B, 0.80, "CoverageF1", L2Semantic, Hybrid),
                    mk("12.2", "Financial statement analysis", 12, Tier::B, 0.85, "MetricAccuracy", L4Lossless, Hybrid),
                    mk("12.3", "Legal doc risk identification", 12, Tier::B, 0.75, "RiskRecall", L2Semantic, Hybrid),
                    mk("12.4", "Medical literature summary (with safety)", 12, Tier::B, 0.80, "AccuracySafety", L2Semantic, Hybrid),
                    mk("12.5", "Market research synthesis", 12, Tier::B, 0.75, "Completeness", L2Semantic, Hybrid),
                    mk("12.6", "Scientific claim verification", 12, Tier::B, 0.80, "Accuracy", L2Semantic, Hybrid),
                    mk("12.7", "Data table insight extraction", 12, Tier::B, 0.75, "InsightRate", L2Semantic, Hybrid),
                    mk("12.8", "SWOT/competitive analysis", 12, Tier::B, 0.80, "Coverage", L2Semantic, Hybrid),
                ],
            },
            Category {
                id: 13,
                name: "Task Execution + Orchestration".into(),
                tier: Tier::B,
                description: "Multi-step plans, long-running tasks, error recovery".into(),
                tests: vec![
                    mk("13.1", "10-step plan execute end-to-end", 13, Tier::B, 0.80, "CompletionRate", NA, Orchestrated),
                    mk("13.2", "Long task >1h with recovery", 13, Tier::B, 0.85, "RecoveryRate", NA, Orchestrated),
                    mk("13.3", "Schedule meetings + reminders", 13, Tier::B, 0.90, "Accuracy", L4Lossless, Orchestrated),
                    mk("13.4", "Gmail+Calendar+Sheets workflow", 13, Tier::B, 0.80, "End2endSuccess", NA, Orchestrated),
                    mk("13.5", "Tool-fail fallback", 13, Tier::B, 0.85, "FallbackRate", NA, Orchestrated),
                    mk("13.6", "Multi-persona delegation", 13, Tier::B, 0.80, "CorrectSplit", NA, GraphNative),
                    mk("13.7", "Rate-limit aware execution", 13, Tier::B, 0.95, "RespectRate", NA, Orchestrated),
                    mk("13.8", "Budget-aware (API cost)", 13, Tier::B, 0.95, "BudgetStay", NA, Orchestrated),
                ],
            },
            Category {
                id: 14,
                name: "Math + Reasoning".into(),
                tier: Tier::B,
                description: "Arithmetic to proofs".into(),
                tests: vec![
                    mk("14.1", "Arithmetic 10-digit", 14, Tier::B, 1.00, "Accuracy", L4Lossless, Hybrid),
                    mk("14.2", "Algebra (high-school)", 14, Tier::B, 0.90, "Accuracy", L4Lossless, Hybrid),
                    mk("14.3", "Calculus (intro)", 14, Tier::B, 0.75, "Accuracy", L4Lossless, Hybrid),
                    mk("14.4", "Logic puzzles (20 problems)", 14, Tier::B, 0.80, "Accuracy", L4Lossless, Hybrid),
                    mk("14.5", "Simple theorem proofs", 14, Tier::B, 0.65, "Validity", L4Lossless, Hybrid),
                    mk("14.6", "Statistical reasoning", 14, Tier::B, 0.75, "Accuracy", L2Semantic, Hybrid),
                    mk("14.7", "Unit conversion + dimensional analysis", 14, Tier::B, 0.95, "Accuracy", L4Lossless, GraphNative),
                ],
            },
        ];

        BenchmarkSpec {
            version: "V1".into(),
            categories,
        }
    }

    pub fn total_tests(&self) -> usize {
        self.categories.iter().map(|c| c.tests.len()).sum()
    }

    pub fn tests_in_tier(&self, tier: Tier) -> usize {
        self.categories
            .iter()
            .filter(|c| c.tier == tier)
            .map(|c| c.tests.len())
            .sum()
    }

    pub fn all_tests(&self) -> Vec<&Test> {
        self.categories.iter().flat_map(|c| c.tests.iter()).collect()
    }

    pub fn find_test(&self, id: &str) -> Option<&Test> {
        self.all_tests().into_iter().find(|t| t.id == id)
    }
}

/// Helper for creating tests concisely.
#[allow(clippy::too_many_arguments)]
fn mk(
    id: &str,
    description: &str,
    category_id: u8,
    tier: Tier,
    target: f32,
    metric: &str,
    rec: ReconstructionLevel,
    exec: ExecutionMode,
) -> Test {
    Test {
        id: id.into(),
        description: description.into(),
        category_id,
        tier,
        target,
        metric: metric.into(),
        reconstruction_level: rec,
        execution_mode: exec,
    }
}
