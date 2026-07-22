//! # MOD-04: MorphologicalParser
//!
//! Performs Arabic morphological (sarf) analysis on segmented tokens.
//! Identifies roots, morphological patterns (awzan), parts of speech,
//! and morphological features. This is the most linguistically complex
//! stage in the AGOS pipeline.
//!
//! ## Architecture (SPEC-0101 §2)
//!
//! Subsystem 1: **Fast-Path Checker** — Identifies particles & pronouns via
//!              KB-0005/KB-0006 lookup (heuristic fallback when KBs absent)
//! Subsystem 2: **Root Extraction** — Triliteral/quadriliteral root extraction
//! Subsystem 3: **Wazan Identification** — Verb form I–XV, noun pattern matching
//! Subsystem 4: **Feature Extraction** — POS, gender, number, person, tense,
//!              mood, voice, case, state → 64-bit bitfield
//!
//! ## Pipeline Interface
//!
//! ```ignore
//! Input:  SegmentedTokenStream (IR-3)
//! Output: MorphologicalAnalysis (IR-4)
//! ```

use agos_core::error::PipelineResult;
use agos_core::evidence::{EvidenceCategory, EvidenceEntry};
use agos_core::feature::{FeatureBitfield, FeatureCategory, NamedFeature};
use agos_core::ir::{
    MorphologicalAnalysis, MorphologicalAnalysisMetadata, RootRef, SegmentedTokenStream,
    StemAnalysis, TokenAnalysis, WazanRef,
};
use agos_core::pipeline::{PipelineContext, PipelineStage};
use agos_core::types::{GrammarSchool, PartOfSpeech};

use std::sync::Arc;

use agos_kb::WazanPatternLookup;

use crate::error::MorphError;

// ──────────────────────────────────────────────
//  Feature Bitfield Constants (KB-0007 §10)
// ──────────────────────────────────────────────

/// Gender values (bits 4–5)
const GENDER_MASCULINE: u8 = 0;
const GENDER_FEMININE: u8 = 1;

/// Number values (bits 6–7)
const NUMBER_SINGULAR: u8 = 0;
const NUMBER_DUAL: u8 = 1;
const NUMBER_PLURAL: u8 = 2;

/// Person values (bits 8–9)
const PERSON_FIRST: u8 = 0;
const PERSON_SECOND: u8 = 1;
const PERSON_THIRD: u8 = 2;

/// Tense values (bits 10–11)
const TENSE_PAST: u8 = 0;
const TENSE_PRESENT: u8 = 1;
const TENSE_IMPERATIVE: u8 = 2;

// ──────────────────────────────────────────────
//  Configuration (SPEC-0101 §3.2.1)
// ──────────────────────────────────────────────

/// Configuration for KB-0004 knowledge base lookup (SPEC-0101 §3.2.2).
///
/// When `enabled` is true and `path` points to a valid KB-0004 directory,
/// the MorphologicalParser will auto-load KB-0004 data to replace the
/// heuristic COMMON_NOUNS_3L / COMMON_VERBS_3L lists.
#[derive(Debug, Clone)]
pub struct Kb0004Config {
    /// Whether KB-0004 auto-loading is enabled (default: false)
    pub enabled: bool,
    /// Filesystem path to the KB-0004 data directory.
    /// Should contain stem-overrides.json, verb-pos-profiles.json,
    /// and noun-pos-profiles.json.
    pub path: String,
}

impl Default for Kb0004Config {
    fn default() -> Self {
        Self {
            enabled: false,
            path: String::new(),
        }
    }
}

/// Configuration for MOD-04: MorphologicalParser (SPEC-0101 §3.2.1).
#[derive(Debug, Clone)]
pub struct MorphologicalParserConfig {
    /// Grammar school for analysis
    pub school: GrammarSchool,
    /// Maximum morphological analyses per token (default: 256).
    /// Each segmentation of a token is analyzed independently, so this must
    /// accommodate max_segmentations × typical_analyses_per_stem.
    pub max_analyses: usize,
    /// Allow heuristic guess for unknown stems (default: false)
    pub enable_guess: bool,
    /// Enable weak root heuristics (default: true)
    pub enable_weak_heuristics: bool,
    /// Enable hamza heuristics (default: true)
    pub enable_hamza_heuristics: bool,
    /// Enable geminate/doubled root heuristics (default: true)
    pub enable_geminate_heuristics: bool,
    /// Maximum root candidates per stem (default: 8)
    pub max_root_candidates: usize,
    /// Maximum wazan candidates per stem (default: 16)
    pub max_wazan_candidates: usize,
    /// KB-0004 configuration for automatic POS lookup loading.
    /// When `kb_config.enabled` is true, `from_config()` or `Default` will
    /// attempt to load KB-0004 data from `kb_config.path`.
    pub kb_config: Kb0004Config,
}

impl Default for MorphologicalParserConfig {
    fn default() -> Self {
        Self {
            school: GrammarSchool::Basra,
            max_analyses: 256,
            enable_guess: false,
            enable_weak_heuristics: true,
            enable_hamza_heuristics: true,
            enable_geminate_heuristics: true,
            max_root_candidates: 8,
            max_wazan_candidates: 16,
            kb_config: Kb0004Config::default(),
        }
    }
}

// ──────────────────────────────────────────────
//  Internal Types (SPEC-0101 §3.1)
// ──────────────────────────────────────────────

/// Type of Arabic root (SPEC-0101 §3.1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootType {
    Sound,
    MithalWawi,
    MithalYai,
    AjwafWawi,
    AjwafYai,
    NaqisWawi,
    NaqisYai,
    LafifMafruq,
    LafifMakrun,
    HamzatedFirst,
    HamzatedMiddle,
    HamzatedLast,
    Doubled,
    QuadriliteralSound,
    QuadriliteralWeak,
}

/// How a root was extracted (SPEC-0101 §3.1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMethod {
    ExactMatch,
    Triliteral,
    Quadriliteral,
    WeakRestoration,
    WeakStripping,
    HamzaPattern,
    GeminateSplit,
    Guess,
}

/// A candidate root extracted from a stem (SPEC-0101 §3.1.2).
#[derive(Debug, Clone)]
pub struct RootCandidate {
    pub text: String,
    pub root_type: RootType,
    pub radicals: Vec<char>,
    pub source: String,
    pub extraction_method: ExtractionMethod,
    pub confidence: f64,
}

/// Category of a wazan/morphological pattern (SPEC-0101 §3.1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WazanCategory {
    VerbFormI,
    VerbFormII,
    VerbFormIII,
    VerbFormIV,
    VerbFormV,
    VerbFormVI,
    VerbFormVII,
    VerbFormVIII,
    VerbFormIX,
    VerbFormX,
    VerbFormXI,
    VerbFormXII,
    VerbFormXIII,
    VerbFormXIV,
    VerbFormXV,
    NounMasdar,
    NounIsmFail,
    NounIsmMaful,
    NounSifatMusabbaha,
    NounIsmMakan,
    NounIsmZaman,
    NounIsmAlah,
    NounSighahMubalaghah,
    NounTafdil,
    NounNisbah,
    NounBrokenPlural,
    NounOther,
}

/// A candidate morphological pattern (wazan) matched against a stem (SPEC-0101 §3.1.3).
#[derive(Debug, Clone)]
pub struct WazanCandidate {
    pub text: String,
    pub form: Option<u8>,
    pub category: WazanCategory,
    pub source: String,
    pub confidence: f64,
}

/// A single feature assignment (SPEC-0101 §3.1.4).
#[derive(Debug, Clone)]
pub struct FeatureAssignment {
    pub name: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
}

/// Complete feature set for one analysis (SPEC-0101 §3.1.4).
#[derive(Debug, Clone)]
pub struct FeatureSet {
    pub features: Vec<FeatureAssignment>,
    pub bitfield: u64,
    pub confidence: f64,
}

/// Convert a FeatureAssignment to a NamedFeature (SPEC-0101 §6).
impl From<FeatureAssignment> for NamedFeature {
    fn from(fa: FeatureAssignment) -> Self {
        // Map feature name to category (KB-0007 §1.3).
        let category = match fa.name.as_str() {
            "pos" | "part_of_speech" => FeatureCategory::Pos,
            "verb_form" | "noun_type" | "pronoun_type" => FeatureCategory::Derivational,
            "has_shadda" | "has_madd" | "has_hamza" => FeatureCategory::Orthographic,
            "stress_pattern" | "syllable_count" => FeatureCategory::Prosodic,
            // All inflectional features (gender, number, person, tense, mood, voice, case, state)
            _ => FeatureCategory::Inflectional,
        };
        NamedFeature {
            name: fa.name,
            value: fa.value,
            category,
            confidence: fa.confidence,
            source: fa.source,
        }
    }
}

// ──────────────────────────────────────────────
//  Arabic Morphology Constants
// ──────────────────────────────────────────────

/// Weak letters in Arabic
const WEAK_LETTERS: &[char] = &['و', 'ي', 'ا', 'ى'];

/// Hamza variants
const HAMZA_VARIANTS: &[char] = &['ء', 'أ', 'إ', 'ئ', 'ؤ'];

/// Common particles (heuristic list — KB-0005 will replace this)
const COMMON_PARTICLES: &[&str] = &[
    "فِي", "مِن", "إِلَى", "عَلَى", "عَنْ", "بِ", "لِ", "كَ",
    "وَ", "فَ", "ثُمَّ", "حَتَّى", "لَنْ", "لَمْ", "لَا", "إِنَّ",
    "أَنَّ", "كَأَنَّ", "لَكِنَّ", "لَيْتَ", "لَعَلَّ", "مَا", "مَنْ",
    "هَلْ", "أَ", "قَدْ", "سَوْفَ", "سَ",
];

/// Common pronouns (heuristic list — KB-0006 will replace this)
const COMMON_PRONOUNS: &[&str] = &[
    "هُوَ", "هِيَ", "هُمَا", "هُمْ", "هُنَّ", "أَنْتَ", "أَنْتِ",
    "أَنْتُمَا", "أَنْتُمْ", "أَنْتُنَّ", "أَنَا", "نَحْنُ",
    "هَذَا", "هَذِهِ", "هَؤُلَاءِ", "ذَلِكَ", "تِلْكَ", "أُولَئِكَ",
    "مَنْ", "مَا", "الَّذِي", "الَّتِي", "الَّذِينَ", "اللَّاتِي",
];

/// Common 3-letter Arabic nouns (heuristic list — KB-0001 will replace this).
///
/// These are triliteral stems that are primarily nouns, not verb forms.
/// Without diacritics, 3-letter stems are ambiguous between Form I verbs
/// (فَعَلَ) and noun patterns (فَعْل, فِعْل, فُعْل). This list provides a
/// heuristic override: 3-letter stems found here are treated as nouns with
/// higher confidence than verbs.
///
/// KB-0001 will eventually provide authoritative POS classification.
const COMMON_NOUNS_3L: &[&str] = &[
    // ── Body parts ──
    "رجل", // leg / man
    "عين", // eye / spring
    "رأس", // head
    "صدر", // chest
    "بطن", // belly
    "قلب", // heart
    "جلد", // skin
    "عظم", // bone
    "لحم", // meat / flesh
    "عنق", // neck
    "فم",  // mouth
    "يد",  // hand
    "دم",  // blood
    "سن",  // tooth
    "ظهر", // back
    "قدم", // foot
    "كبد", // liver
    "كتف", // shoulder
    "ساق", // leg (shin/calf)
    "أذن", // ear
    "فخذ", // thigh

    // ── Food & Drink ──
    "خبز", // bread
    "جبن", // cheese
    "عسل", // honey
    "لبن", // milk
    "زيت", // oil
    "تمر", // dates (fruit)
    "بصل", // onion
    "ملح", // salt
    "عنب", // grapes
    "تين", // figs
    "رز",  // rice
    "قمح", // wheat
    "ثوم", // garlic
    "عدس", // lentils
    "لوز", // almonds
    "فول", // beans
    "حب",  // grains / seeds
    "سكر", // sugar

    // ── Nature & Geography ──
    "بحر", // sea
    "جبل", // mountain
    "نهر", // river
    "شمس", // sun
    "قمر", // moon
    "ظل",  // shadow
    "نور", // light
    "نار", // fire
    "حجر", // stone
    "ثلج", // snow / ice
    "رمل", // sand
    "صخر", // rock
    "نجم", // star
    "غيم", // clouds
    "سحاب", // clouds
    "طين", // mud / clay
    "ترب", // dust / earth
    "ريح", // wind

    // ── Buildings, Places & Objects ──
    "باب", // door
    "دار", // house / home
    "بيت", // house
    "سوق", // market
    "بلد", // country / town
    "جسر", // bridge
    "سكة", // street / rail
    "قلم", // pen
    "كأس", // cup / glass
    "سيف", // sword
    "قوس", // bow (archery)
    "سهم", // arrow
    "رمح", // spear
    "نصل", // blade
    "ترس", // shield
    "خمر", // wine
    "حبل", // rope
    "خيط", // thread
    "قدر", // pot / capacity
    "سور", // wall
    "جدار", // wall
    "درب", // path
    "حصن", // fortress
    "قبر", // grave / tomb
    "بئر", // well

    // ── People & Social ──
    "زوج", // husband / spouse
    "جار", // neighbor
    "ضيف", // guest
    "ملك", // king / property
    "قوم", // people / tribe
    "شعب", // people / nation

    // ── Abstract & High-frequency ──
    "وقت", // time
    "شهر", // month
    "عام", // year
    "يوم", // day
    "ليل", // night
    "علم", // knowledge
    "حرب", // war
    "سلم", // peace / ladder
    "أمن", // peace / security
    "عقل", // mind / intellect
    "حكم", // rule / judgment
    "مثل", // example
    "دين", // religion / debt
    "فرح", // joy
    "حزن", // sadness
    "جوع", // hunger
    "ألم", // pain
    "لون", // color
    "شكل", // shape
    "حرف", // letter (alphabet) / craft
    "رسم", // drawing / painting
    "سحر", // magic / dawn
    "فكر", // thought
    "حلم", // dream
    "وهم", // illusion
    "عصر", // era / age
    "خلق", // creation / morals
    "قسم", // section / oath
    "رحم", // womb / mercy
    "كرم", // generosity / vineyard
    "جهد", // effort
    "شكر", // gratitude
    "صبر", // patience
    "غضب", // anger
    "شعر", // poetry / hair
    "فن",  // art
    "فقر", // poverty
    "غنى", // wealth (3 letters غ-ن-ي)
    "حسب", // honor / reckoning
    "أصل", // origin
    "نسب", // lineage / relation
    "صفة", // quality / attribute
    "وصف", // description
    "فصل", // chapter / season
];

/// Common 3-letter Arabic Form I verbs (heuristic list — KB-0001 will replace this).
///
/// These are triliteral stems that are primarily Form I verbs, not nouns.
/// Found here, Form I confidence is boosted from 0.30 → 0.35 to ensure
/// verb analysis decisively outranks the Masdar noun pattern (0.25).
///
/// This list is intentionally disjoint from COMMON_NOUNS_3L. If a stem
/// appears in both lists, the COMMON_NOUNS_3L check (which runs first)
/// suppresses verb analysis entirely, making the verb boost unreachable.
///
/// Includes sound, hollow, defective, and hamzated-initial verb stems.
/// KB-0001 will eventually provide authoritative POS classification.
const COMMON_VERBS_3L: &[&str] = &[
    // ── Sound triliteral verbs (3 strong radicals) ──
    "كتب", // he wrote
    "ضرب", // he hit
    "جلس", // he sat
    "فتح", // he opened
    "شرب", // he drank
    "ذهب", // he went
    "درس", // he studied
    "لعب", // he played
    "خرج", // he went out
    "دخل", // he entered
    "نزل", // he descended
    "صعد", // he ascended
    "طلع", // he rose
    "حضر", // he attended
    "صنع", // he made
    "حمل", // he carried
    "رفع", // he raised
    "قطع", // he cut
    "وصل", // he arrived / connected
    "رجع", // he returned
    "سمع", // he heard
    "غسل", // he washed
    "مسح", // he wiped
    "قتل", // he killed
    "كشف", // he uncovered
    "حفر", // he dug
    "سكن", // he resided
    "نظر", // he looked
    "عرف", // he knew
    "سجد", // he prostrated
    "ركض", // he ran
    "طلب", // he requested
    "كسب", // he earned
    "غفر", // he forgave
    "لعن", // he cursed
    "سرق", // he stole
    "كسر", // he broke
    "حفظ", // he preserved
    "فهم", // he understood
    "جمع", // he collected
    "نصر", // he helped
    "نجح", // he succeeded
    "فشل", // he failed
    "نشر", // he spread / published
    "هزم", // he defeated
    "سلب", // he robbed

    // ── Hollow verbs (middle weak letter) ──
    "قال", // he said (ق-و-ل)
    "قام", // he stood (ق-و-م)
    "كان", // he was (ك-و-ن)
    "زاد", // he increased (ز-ي-د)
    "باع", // he sold (ب-ي-ع)
    "سار", // he walked (س-ي-ر)
    "صام", // he fasted (ص-و-م)
    "نام", // he slept (ن-و-م)
    "عاد", // he returned (ع-و-د)
    "ساق", // he drove (س-و-ق)
    "طار", // he flew (ط-ي-ر)
    "ضاع", // he was lost (ض-ي-ع)
    "صار", // he became (ص-ي-ر)
    "مال", // he leaned (م-ي-ل)
    "راح", // he went / rested (ر-و-ح)
    "قاد", // he led (ق-و-د)
    "غاب", // he was absent (غ-ي-ب)
    "خاف", // he feared (خ-و-ف)
    "فاز", // he won (ف-و-ز)

    // ── Defective verbs (final weak letter) ──
    "مشى", // he walked (م-ش-ي)
    "جرى", // he ran (ج-ر-ي)
    "سعى", // he strove (س-ع-ي)
    "دعا", // he called (د-ع-و)
    "بكى", // he cried (ب-ك-ي)
    "بنى", // he built (ب-ن-ي)
    "هدى", // he guided (ه-د-ي)
    "رمى", // he threw (ر-م-ي)

    // ── Initial hamza verbs ──
    "أكل", // he ate
    "أمر", // he commanded
    "أخذ", // he took
];

// ──────────────────────────────────────────────
//  MorphologicalParser Stage
// ──────────────────────────────────────────────

/// MOD-04: MorphologicalParser — Arabic morphological (sarf) analysis.
///
/// Processes segmented tokens through 4 internal subsystems:
/// 1. Fast-Path Check — identify particles and pronouns
/// 2. Root Extraction — triliteral/quadriliteral root identification
/// 3. Wazan Identification — verb form and noun pattern matching
/// 4. Feature Extraction — morphological features → 64-bit bitfield
///
/// ## Determinism
///
/// Fully deterministic. Same input + same KB versions = same output always.
///
/// ## Performance Targets (SPEC-0001-C3 §5.9)
///
/// | Metric | Target |
/// |--------|--------|
/// | Throughput | > 10K stems/second |
/// | Latency (p50) | < 100 μs per stem |
#[derive(Debug, Clone)]
pub struct MorphologicalParser {
    /// Configuration
    pub config: MorphologicalParserConfig,
    /// Optional KB-0004 lookup for authoritative POS data (SPEC-0101 §3.2.2).
    ///
    /// When present, `check_verb_form()` consults KB-0004 first for
    /// stem-level POS classification, falling back to heuristic constant
    /// lists when the stem is not in the KB. This enables a gradual
    /// migration path: Phase 1 seeds KB-0004 from the heuristic lists,
    /// Phase 2+ expands KB-0004 entries while keeping the heuristic
    /// fallback for unknown stems.
    pub kb: Option<Arc<dyn WazanPatternLookup>>,
}

impl MorphologicalParser {
    pub fn new(config: MorphologicalParserConfig) -> Self {
        Self { config, kb: None }
    }

    /// Create a MorphologicalParser with KB-0004 lookup for POS overrides.
    ///
    /// When `kb` is provided, `check_verb_form()` consults KB-0004 first for
    /// stem-level POS classification, falling back to heuristic lists when the
    /// stem is not in the KB. This enables a gradual migration path: Phase 1
    /// seeds KB-0004 from the heuristic lists, Phase 2+ expands KB-0004
    /// entries while keeping the heuristic fallback for unknown stems.
    pub fn with_kb(config: MorphologicalParserConfig, kb: Arc<dyn WazanPatternLookup>) -> Self {
        Self { config, kb: Some(kb) }
    }

    /// Create a MorphologicalParser from config, auto-loading KB-0004 if configured.
    ///
    /// When `config.kb_config.enabled` is true, this method attempts to load
    /// KB-0004 from `config.kb_config.path`. If the directory doesn't exist
    /// or loading fails, a warning is logged and the parser falls back to
    /// heuristic lists (same as `new()`).
    ///
    /// This is the recommended constructor for production use, as it wires
    /// automatic KB-0004 loading into the pipeline initialization sequence.
    pub fn from_config(config: MorphologicalParserConfig) -> Self {
        if !config.kb_config.enabled || config.kb_config.path.is_empty() {
            return Self { config, kb: None };
        }

        let kb_path = std::path::Path::new(&config.kb_config.path);
        match agos_kb::Kb0004::load_from_directory(kb_path) {
            Ok(kb) => {
                tracing::info!(
                    "MOD-04: KB-0004 loaded from {} ({} stem overrides, {} verb profiles, {} noun profiles)",
                    config.kb_config.path,
                    kb.stem_override_count(),
                    kb.verb_profile_count(),
                    kb.noun_profile_count(),
                );
                Self { config, kb: Some(Arc::new(kb)) }
            }
            Err(e) => {
                tracing::warn!(
                    "MOD-04: KB-0004 loading failed from {}: {} — falling back to heuristic lists",
                    config.kb_config.path,
                    e,
                );
                Self { config, kb: None }
            }
        }
    }

    // ════════════════════════════════════════════
    //  Core Analysis Pipeline
    // ════════════════════════════════════════════

    /// Analyze morphology for a segmented token stream (SPEC-0001-C3 §5.3).
    pub fn analyze(&self, input: SegmentedTokenStream) -> PipelineResult<MorphologicalAnalysis> {
        let mut token_analyses = Vec::new();
        let mut unknown_stems = Vec::new();

        for seg_token in &input.tokens {
            // Only analyze word tokens
            if seg_token.raw_token.token_type != agos_core::types::TokenType::Word {
                continue;
            }

            let mut stem_analyses = Vec::new();

            for segmentation in &seg_token.segmentations {
                // Find the stem morpheme
                let stem_text = segmentation
                    .morphemes
                    .iter()
                    .find(|m| m.morpheme_type == agos_core::types::MorphemeType::Stem)
                    .map(|m| &m.text[..])
                    .unwrap_or(&seg_token.raw_token.text);

                // Subsystem 1: Fast-Path Check (Particles & Pronouns)
                if let Some(analysis) = self.check_fast_path(stem_text, segmentation) {
                    stem_analyses.push(analysis);
                    continue;
                }

                // Subsystem 2: Root Extraction
                let roots = self.extract_roots(stem_text);
                if roots.is_empty() && !self.config.enable_guess {
                    unknown_stems.push(stem_text.to_string());
                    continue;
                }

                // For each root candidate, try wazan matching + feature extraction
                for root in &roots {
                    // Subsystem 3: Wazan Identification
                    let wazans = self.identify_wazan(stem_text, root);

                    for wazan in &wazans {
                        // Subsystem 4: Feature Extraction
                        let pos = self.determine_pos(stem_text, wazan);
                        let feature_set = self.extract_features(stem_text, root, wazan, pos);

                        let named_features: Vec<NamedFeature> = feature_set
                            .features
                            .into_iter()
                            .map(Into::into)
                            .collect();

                        let analysis_id = format!(
                            "ana-{}-{}-{}",
                            seg_token.raw_token.id,
                            root.text,
                            wazan.text
                        );

                        let root_ref = RootRef {
                            text: root.text.clone(),
                            source: root.source.clone(),
                            confidence: root.confidence,
                        };

                        let wazan_ref = WazanRef {
                            text: wazan.text.clone(),
                            source: wazan.source.clone(),
                            form: wazan.form,
                            confidence: wazan.confidence,
                        };

                        stem_analyses.push(StemAnalysis {
                            analysis_id,
                            segmentation_id: segmentation.id.clone(),
                            stem: stem_text.to_string(),
                            root: Some(root_ref),
                            wazan: Some(wazan_ref),
                            pos,
                            features: named_features,
                            is_ambiguous: stem_analyses.len() > 1,
                            alternatives: vec![],
                            evidence: vec![EvidenceEntry {
                                id: format!("ev-{}-morph", seg_token.raw_token.id),
                                timestamp: String::new(),
                                stage: "MOD-04".to_string(),
                                stage_iteration: 0,
                                category: EvidenceCategory::Morphology,
                                rule_or_algorithm: format!(
                                    "morph_analysis({}, {}, {:?})",
                                    stem_text, root.text, wazan.category
                                ),
                                version: "1.0".to_string(),
                                input_description: format!("stem={}", stem_text),
                                input_state_hash: String::new(),
                                output_description: format!("{} analysis", root.text),
                                output_delta: format!("root={}, wazan={}", root.text, wazan.text),
                                confidence: (root.confidence * wazan.confidence) / 2.0,
                                token_indices: vec![seg_token.raw_token.id],
                            }],
                        });
                    }
                }

                // If no wazan matched but we have roots, create a minimal analysis
                if stem_analyses.is_empty() && !roots.is_empty() {
                    let root = &roots[0];
                    stem_analyses.push(self.make_partial_analysis(
                        seg_token, segmentation, stem_text, root,
                    ));
                }
            }

            // Limit analyses
            if stem_analyses.len() > self.config.max_analyses {
                return Err(MorphError::MaxAnalysesExceeded {
                    token_id: seg_token.raw_token.id,
                    limit: self.config.max_analyses,
                    actual: stem_analyses.len(),
                }
                .into_pipeline("MOD-04"));
            }

            if !stem_analyses.is_empty() {
                token_analyses.push(TokenAnalysis {
                    token_id: seg_token.raw_token.id,
                    stem_analyses,
                });
            }
        }

        // Build metadata
        let total_tokens = input.tokens.len() as u64;
        let analyzed_tokens = token_analyses.len() as u64;
        let ambiguous_tokens = token_analyses
            .iter()
            .filter(|ta| ta.stem_analyses.len() > 1)
            .count() as u64;
        let unknown_stems_dedup: Vec<String> = {
            let mut v = unknown_stems;
            v.sort();
            v.dedup();
            v
        };
        let unknown_count = unknown_stems_dedup.len() as u64;

        Ok(MorphologicalAnalysis {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            token_analyses,
            metadata: MorphologicalAnalysisMetadata {
                total_tokens,
                analyzed_tokens,
                ambiguous_tokens,
                unknown_tokens: unknown_count,
                unknown_stems: unknown_stems_dedup,
            },
        })
    }

    // ════════════════════════════════════════════
    //  Subsystem 1: Fast-Path Checker
    // ════════════════════════════════════════════

    /// Check if a stem is a known particle or pronoun (SPEC-0101 §4, Step 3).
    fn check_fast_path(&self, stem_text: &str, seg: &agos_core::ir::Segmentation) -> Option<StemAnalysis> {
        let upper = stem_text.to_uppercase();

        // Check particles
        if COMMON_PARTICLES.contains(&upper.as_str()) {
            return Some(StemAnalysis {
                analysis_id: format!("ana-particle-{}", seg.id),
                segmentation_id: seg.id.clone(),
                stem: stem_text.to_string(),
                root: None,
                wazan: None,
                pos: PartOfSpeech::Particle,
                features: vec![],
                is_ambiguous: false,
                alternatives: vec![],
                evidence: vec![],
            });
        }

        // Check pronouns
        if COMMON_PRONOUNS.contains(&upper.as_str()) {
            return Some(StemAnalysis {
                analysis_id: format!("ana-pronoun-{}", seg.id),
                segmentation_id: seg.id.clone(),
                stem: stem_text.to_string(),
                root: None,
                wazan: None,
                pos: PartOfSpeech::Pronoun,
                features: vec![],
                is_ambiguous: false,
                alternatives: vec![],
                evidence: vec![],
            });
        }

        None
    }

    // ════════════════════════════════════════════
    //  Subsystem 2: Root Extraction
    // ════════════════════════════════════════════

    /// Extract root candidates from a stem (SPEC-0101 §4.3-4.7).
    fn extract_roots(&self, stem_text: &str) -> Vec<RootCandidate> {
        let mut roots = Vec::new();

        // Try triliteral extraction
        if let Some(candidates) = self.extract_triliteral(stem_text) {
            roots.extend(candidates);
        }

        // Try quadriliteral extraction
        if let Some(candidates) = self.extract_quadriliteral(stem_text) {
            roots.extend(candidates);
        }

        // Try weak root handling
        if self.config.enable_weak_heuristics {
            if let Some(candidates) = self.handle_weak_root(stem_text) {
                roots.extend(candidates);
            }
        }

        // Try hamzated root handling
        if self.config.enable_hamza_heuristics {
            if let Some(candidates) = self.handle_hamzated_root(stem_text) {
                roots.extend(candidates);
            }
        }

        // Try doubled/geminate root handling
        if self.config.enable_geminate_heuristics {
            if let Some(candidates) = self.handle_doubled_root(stem_text) {
                roots.extend(candidates);
            }
        }

        // Try guess if enabled and no roots found
        if roots.is_empty() && self.config.enable_guess {
            if let Some(guess) = self.guess_root(stem_text) {
                roots.push(guess);
            }
        }

        // Sort by confidence descending, limit
        roots.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        roots.truncate(self.config.max_root_candidates);
        roots
    }

    /// Triliteral root extraction (SPEC-0101 §4.3).
    fn extract_triliteral(&self, stem_text: &str) -> Option<Vec<RootCandidate>> {
        let consonants: Vec<char> = stem_text
            .chars()
            .filter(|c| c.is_alphabetic() && !WEAK_LETTERS.contains(c))
            .collect();

        if consonants.len() >= 3 {
            let root_text: String = consonants[..3].iter().collect();
            Some(vec![RootCandidate {
                text: root_text,
                root_type: RootType::Sound,
                radicals: consonants[..3].to_vec(),
                source: "heuristic:triliteral".to_string(),
                extraction_method: ExtractionMethod::Triliteral,
                confidence: 0.5,
            }])
        } else {
            None
        }
    }

    /// Quadriliteral root extraction (SPEC-0101 §4.7).
    fn extract_quadriliteral(&self, stem_text: &str) -> Option<Vec<RootCandidate>> {
        let consonants: Vec<char> = stem_text
            .chars()
            .filter(|c| c.is_alphabetic() && !WEAK_LETTERS.contains(c))
            .collect();

        if consonants.len() >= 4 {
            let root_text: String = consonants[..4].iter().collect();
            Some(vec![RootCandidate {
                text: root_text,
                root_type: RootType::QuadriliteralSound,
                radicals: consonants[..4].to_vec(),
                source: "heuristic:quadriliteral".to_string(),
                extraction_method: ExtractionMethod::Quadriliteral,
                confidence: 0.4,
            }])
        } else {
            None
        }
    }

    /// Weak root handling (SPEC-0101 §4.4 — hollow, defective, assimilated).
    fn handle_weak_root(&self, stem_text: &str) -> Option<Vec<RootCandidate>> {
        let mut candidates = Vec::new();
        let chars: Vec<char> = stem_text.chars().collect();

        // Hollow root (أجوف): middle radical is weak, appears as medial alif
        // Pattern: C₁ + {ا,و,ي} + C₂
        if chars.len() >= 3 {
            let c1 = chars[0];
            let c2 = chars[2];

            // Check for hollow pattern (C1 + long vowel + C2)
            let middle = chars[1];
            if middle == 'ا' || middle == 'و' || middle == 'ي' {
                // Try wawi variant
                let root_waw = format!("{}{}{}", c1, 'و', c2);
                candidates.push(RootCandidate {
                    text: root_waw.clone(),
                    root_type: RootType::AjwafWawi,
                    radicals: vec![c1, 'و', c2],
                    source: "heuristic:hollow".to_string(),
                    extraction_method: ExtractionMethod::WeakRestoration,
                    confidence: 0.45,
                });

                // Try yai variant
                let root_ya = format!("{}{}{}", c1, 'ي', c2);
                if root_ya != root_waw {
                    candidates.push(RootCandidate {
                        text: root_ya,
                        root_type: RootType::AjwafYai,
                        radicals: vec![c1, 'ي', c2],
                        source: "heuristic:hollow".to_string(),
                        extraction_method: ExtractionMethod::WeakRestoration,
                        confidence: 0.4,
                    });
                }
            }

            // Defective root (ناقص): final radical is weak, appears as final alif/ya/waw
            let last = chars[chars.len() - 1];
            if last == 'ا' || last == 'ى' || last == 'ي' || last == 'و' {
                let (waw_radical, ya_radical) = if last == 'و' {
                    ('و', 'ي')
                } else if last == 'ا' {
                    ('و', 'ي')
                } else if last == 'ى' {
                    ('و', 'ي')
                } else {
                    ('ي', 'و')
                };
                let c1 = chars[0];
                let c2 = chars[1];

                let root_waw = format!("{}{}{}", c1, c2, waw_radical);
                candidates.push(RootCandidate {
                    text: root_waw,
                    root_type: RootType::NaqisWawi,
                    radicals: vec![c1, c2, waw_radical],
                    source: "heuristic:defective".to_string(),
                    extraction_method: ExtractionMethod::WeakRestoration,
                    confidence: 0.4,
                });

                if ya_radical != waw_radical {
                    let root_ya = format!("{}{}{}", c1, c2, ya_radical);
                    candidates.push(RootCandidate {
                        text: root_ya,
                        root_type: RootType::NaqisYai,
                        radicals: vec![c1, c2, ya_radical],
                        source: "heuristic:defective".to_string(),
                        extraction_method: ExtractionMethod::WeakRestoration,
                        confidence: 0.35,
                    });
                }
            }

            // Assimilated root (مثال): initial radical is weak
            if c1 == 'و' || c1 == 'ي' {
                candidates.push(RootCandidate {
                    text: stem_text.chars().take(3).collect(),
                    root_type: if c1 == 'و' { RootType::MithalWawi } else { RootType::MithalYai },
                    radicals: stem_text.chars().take(3).collect::<Vec<_>>(),
                    source: "heuristic:assimilated".to_string(),
                    extraction_method: ExtractionMethod::WeakStripping,
                    confidence: 0.4,
                });
            }
        }

        if candidates.is_empty() { None } else { Some(candidates) }
    }

    /// Hamzated root handling (SPEC-0101 §4.5).
    fn handle_hamzated_root(&self, stem_text: &str) -> Option<Vec<RootCandidate>> {
        let chars: Vec<char> = stem_text.chars().collect();
        let mut candidates = Vec::new();

        // Find hamza position
        for (i, &c) in chars.iter().enumerate() {
            if HAMZA_VARIANTS.contains(&c) {
                let mut radicals: Vec<char> = chars.iter().copied().collect();
                // Normalize hamza to bare hamza (ء)
                radicals[i] = 'ء';

                // Keep only 3 consonants
                let filtered: Vec<char> = radicals
                    .into_iter()
                    .filter(|r| r.is_alphabetic() && !WEAK_LETTERS.contains(r))
                    .collect();

                if filtered.len() >= 3 {
                    let root_type = match i {
                        0 => RootType::HamzatedFirst,
                        _ if i == chars.len() - 1 => RootType::HamzatedLast,
                        _ => RootType::HamzatedMiddle,
                    };

                    let root_text: String = filtered[..3].iter().collect();
                    candidates.push(RootCandidate {
                        text: root_text,
                        root_type,
                        radicals: filtered[..3].to_vec(),
                        source: "heuristic:hamzated".to_string(),
                        extraction_method: ExtractionMethod::HamzaPattern,
                        confidence: 0.4,
                    });
                }
                break;
            }
        }

        if candidates.is_empty() { None } else { Some(candidates) }
    }

    /// Doubled/geminate root handling (SPEC-0101 §4.6).
    fn handle_doubled_root(&self, stem_text: &str) -> Option<Vec<RootCandidate>> {
        let chars: Vec<char> = stem_text.chars().collect();
        if chars.len() < 2 {
            return None;
        }

        let last = chars[chars.len() - 1];
        let second_last = chars[chars.len() - 2];

        // If the last two consonants are the same → doubled root
        if last == second_last {
            let root_text: String = if chars.len() >= 3 {
                format!("{}{}{}", chars[0], last, last)
            } else {
                format!("{}{}{}", last, last, last)
            };
            return Some(vec![RootCandidate {
                text: root_text,
                root_type: RootType::Doubled,
                radicals: vec![chars[0], last, last],
                source: "heuristic:doubled".to_string(),
                extraction_method: ExtractionMethod::GeminateSplit,
                confidence: 0.45,
            }]);
        }

        None
    }

    /// Low-confidence root guessing (SPEC-0101 §4.8).
    fn guess_root(&self, stem_text: &str) -> Option<RootCandidate> {
        let consonants: Vec<char> = stem_text
            .chars()
            .filter(|c| c.is_alphabetic() && !WEAK_LETTERS.contains(c))
            .collect();

        if consonants.len() >= 3 {
            let root_text: String = consonants[..3].iter().collect();
            Some(RootCandidate {
                text: root_text,
                root_type: RootType::Sound,
                radicals: consonants[..3].to_vec(),
                source: "heuristic:guess".to_string(),
                extraction_method: ExtractionMethod::Guess,
                confidence: 0.2,
            })
        } else {
            None
        }
    }

    // ════════════════════════════════════════════
    //  Subsystem 3: Wazan Identification
    // ════════════════════════════════════════════

    /// Identify possible wazans (morphological patterns) for a stem (SPEC-0101 §5).
    fn identify_wazan(&self, stem_text: &str, root: &RootCandidate) -> Vec<WazanCandidate> {
        let mut candidates = Vec::new();

        // Try to match verb forms I–XV based on pattern prefixes and suffixes
        self.match_verb_forms(stem_text, root, &mut candidates);

        // Try to match noun patterns
        self.match_noun_patterns(stem_text, &mut candidates);

        // Sort by confidence descending, limit
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(self.config.max_wazan_candidates);
        candidates
    }

    /// Match verb form patterns (SPEC-0101 §5.3, Arabic Verb Forms I–XV).
    fn match_verb_forms(
        &self,
        stem_text: &str,
        root: &RootCandidate,
        candidates: &mut Vec<WazanCandidate>,
    ) {
        // Use school-specific form priority
        let form_order: &[u8] = match self.config.school {
            GrammarSchool::Andalus => &[1, 2, 4, 3, 5, 6, 8, 7, 9, 10],
            _ => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };

        for &form in form_order {
            let confidence = self.check_verb_form(stem_text, root, form);
            if confidence > 0.0 {
                let pattern = match form {
                    1 => "فَعَلَ", 2 => "فَعَّلَ", 3 => "فَاعَلَ",
                    4 => "أَفْعَلَ", 5 => "تَفَعَّلَ", 6 => "تَفَاعَلَ",
                    7 => "اِنْفَعَلَ", 8 => "اِفْتَعَلَ", 9 => "اِفْعَلَّ",
                    10 => "اِسْتَفْعَلَ", 11 => "اِفْعَالَّ", 12 => "اِفْعَوْعَلَ",
                    13 => "اِفْعَوَّلَ", 14 => "اِفْعَنْلَلَ", 15 => "اِفْعَنْلَى",
                    _ => continue,
                };

                let category = match form {
                    1 => WazanCategory::VerbFormI,
                    2 => WazanCategory::VerbFormII,
                    3 => WazanCategory::VerbFormIII,
                    4 => WazanCategory::VerbFormIV,
                    5 => WazanCategory::VerbFormV,
                    6 => WazanCategory::VerbFormVI,
                    7 => WazanCategory::VerbFormVII,
                    8 => WazanCategory::VerbFormVIII,
                    9 => WazanCategory::VerbFormIX,
                    10 => WazanCategory::VerbFormX,
                    11 => WazanCategory::VerbFormXI,
                    12 => WazanCategory::VerbFormXII,
                    13 => WazanCategory::VerbFormXIII,
                    14 => WazanCategory::VerbFormXIV,
                    15 => WazanCategory::VerbFormXV,
                    _ => unreachable!(),
                };

                let kb_available = self.kb.is_some();
                candidates.push(WazanCandidate {
                    text: pattern.to_string(),
                    form: Some(form),
                    category,
                    source: if kb_available {
                        format!("kb0004:verb_form:{}", form)
                    } else {
                        "heuristic:verb_form".to_string()
                    },
                    confidence,
                });
            }
        }
    }

    /// Check if a stem is a known noun (KB-0004 first, heuristic fallback).
    ///
    /// Logic:
    /// 1. If KB-0004 is available and has the stem → use KB's judgment authoritatively
    /// 2. If KB-0004 has the stem but it's NOT a noun → authoritative "not noun"
    /// 3. If no KB or stem not in KB → fall back to heuristic COMMON_NOUNS_3L list
    fn is_known_noun_stem(&self, stem_text: &str) -> bool {
        if let Some(kb) = &self.kb {
            // KB-0004 is authoritative when it has an entry for this stem
            if kb.is_primarily_noun(stem_text).is_some() {
                return true;
            }
            // KB has an override but it's NOT noun → authoritative "not noun"
            if kb.stem_pos_override(stem_text).is_some() {
                return false;
            }
        }
        // No KB or stem not in KB → fall back to heuristic
        COMMON_NOUNS_3L.contains(&stem_text)
    }

    /// Check if a stem is a known verb (KB-0004 first, heuristic fallback).
    ///
    /// Logic:
    /// 1. If KB-0004 is available and has the stem → use KB's judgment authoritatively
    /// 2. If KB-0004 has the stem but it's NOT a verb → authoritative "not verb"
    /// 3. If no KB or stem not in KB → fall back to heuristic COMMON_VERBS_3L list
    fn is_known_verb_stem(&self, stem_text: &str) -> bool {
        if let Some(kb) = &self.kb {
            // KB-0004 is authoritative when it has an entry for this stem
            if kb.is_primarily_verb(stem_text).is_some() {
                return true;
            }
            // KB has an override but it's NOT verb → authoritative "not verb"
            if kb.stem_pos_override(stem_text).is_some() {
                return false;
            }
        }
        // No KB or stem not in KB → fall back to heuristic
        COMMON_VERBS_3L.contains(&stem_text)
    }

    /// Look up the confidence for a noun pattern from KB-0004, with fallback.
    ///
    /// When KB-0004 is available and has the pattern, uses the authoritative
    /// confidence from KB-0004 noun profiles. Otherwise falls back to the
    /// provided heuristic value. This enables gradual KB migration:
    /// Phase 1: all patterns use heuristic fallback
    /// Phase 2: patterns added to KB-0004 are used authoritatively
    fn noun_pattern_confidence(&self, pattern: &str, fallback: f64) -> f64 {
        if let Some(kb) = &self.kb {
            // Check if the pattern actually has a profile in KB-0004
            // (noun_confidence() returns 0.15 for unknown patterns via trait default,
            // so we must check for None before consulting the KB)
            if kb.noun_profile(pattern).is_some() {
                return kb.noun_confidence(pattern);
            }
        }
        fallback
    }

    /// Check if a noun pattern is classified as an Adjective in KB-0004.
    ///
    /// Some Arabic noun patterns (فَعِيل, فَعُول, فَعْلَان, أَفْعَل, etc.)
    /// are semantically adjectives. KB-0004 noun profiles store this
    /// distinction via `default_pos`.
    fn noun_pattern_is_adjective(&self, pattern: &str) -> bool {
        if let Some(kb) = &self.kb {
            if kb.noun_is_adjective(pattern) {
                return true;
            }
        }
        // Fallback heuristic: patterns ending with ي (nisbah) are adjectives
        // when the stem doesn't end with ة
        false
    }

    /// Look up the confidence for a verb form from KB-0004, with fallback.
    ///
    /// When KB-0004 is available and has profiles for the verb form, uses
    /// the authoritative confidence from KB-0004 verb profiles:
    /// - If `is_boosted` and the form has a boosted_confidence → returns boosted value
    /// - Otherwise → returns the max `verb_confidence` across all variants
    ///
    /// When the KB doesn't have the form (or KB is not configured), returns
    /// the heuristic `fallback` value. This enables gradual KB migration:
    /// Phase 1: all forms use heuristic fallback
    /// Phase 2: forms added to KB-0004 are used authoritatively
    fn verb_form_confidence(&self, form: u8, is_boosted: bool, fallback: f64) -> f64 {
        if let Some(kb) = &self.kb {
            if !kb.verb_profiles_for_form(form).is_empty() {
                if is_boosted {
                    return kb.verb_boosted_confidence(form);
                }
                return kb.verb_confidence(form);
            }
        }
        fallback
    }

    /// Check if a stem matches a specific verb form pattern.
    ///
    /// Returns confidence > 0 if the stem length and prefix/suffix patterns
    /// are consistent with the given verb form. Returns 0.0 for stems that
    /// clearly cannot be that verb form (e.g., a 4-letter stem cannot be
    /// Form I which requires 3 letters; a stem ending in ة is a noun, not a verb).
    ///
    /// This prevents MOD-04 from always classifying words as Verb first,
    /// which was the previous behavior (all forms always returned 0.2–0.3).
    fn check_verb_form(&self, stem_text: &str, root: &RootCandidate, form: u8) -> f64 {
        let stem_len = stem_text.chars().count();

        // Stems ending with taa marbuta (ة) are nouns, never verbs.
        // Stems with noun plural suffixes (ات, ون, ين) are also not verbs.
        if stem_text.ends_with('ة')
            || stem_text.ends_with("ات")
            || stem_text.ends_with("ون")
            || stem_text.ends_with("ين")
        {
            return 0.0;
        }

        // Known 3-letter nouns: check KB-0004 first, then heuristic fallback.
        // KB-0004 is authoritative when it has an entry for the stem.
        // The heuristic list is a fallback for stems not yet in the KB.
        if stem_len == 3 && self.is_known_noun_stem(stem_text) {
            return 0.0;
        }

        match form {
            // Form I (فَعَلَ): 3-letter stem, the basic verb form.
            // E.g., كتب (kataba), ضرب (daraba), فتح (fataha)
            //
            // Also match 4-letter stems starting with imperfect prefixes
            // (ي, ت, أ, ن) where the prefix masks a 3-letter stem:
            //   يكتب (yaktubu) = ي + كتب → 4 letters, ي prefix
            //   تكتب (taktubu) = ت + كتب → 4 letters, ت prefix
            //   أكتب (aktubu)  = أ + كتب → 4 letters, أ prefix
            //   نكتب (naktubu) = ن + كتب → 4 letters, ن prefix
            //
            // Also match 4-letter stems starting with imperative prefix (ا):
            //   اكتب (uktub) = ا + كتب → 4 letters, ا prefix
            //
            // KB-0004 lookup: verb_boosted_confidence(1) = 0.35, verb_confidence(1) = 0.30
            1 if stem_len == 3 && self.is_known_verb_stem(stem_text)
                => self.verb_form_confidence(1, true, 0.35),
            1 if stem_len == 3
                => self.verb_form_confidence(1, false, 0.30),
            1 if stem_len == 4
                && (stem_text.starts_with('ي')
                    || stem_text.starts_with('ت')
                    || stem_text.starts_with('أ')
                    || stem_text.starts_with('ن'))
                => self.verb_form_confidence(1, false, 0.25),
            1 if stem_len == 4 && stem_text.starts_with('ا')
                => self.verb_form_confidence(1, false, 0.20),

            // Form II (فَعَّلَ): 3-letter stem with doubled middle consonant.
            // Without tashkeel, this is hard to detect: عَلَّمَ → "علم" (3 letters)
            // 4+ stems with C2=C3 also suggest gemination, e.g., "علّم" = ع-ل-ل-م
            // KB-0004 lookup: verb_confidence(2) = 0.35
            2 if stem_len >= 3 && root.root_type == RootType::Doubled
                => self.verb_form_confidence(2, false, 0.35),
            2 if stem_len >= 3 && stem_len <= 4
                && stem_text.chars().nth(1) == stem_text.chars().nth(2)
                => self.verb_form_confidence(2, false, 0.20),
            2 if stem_len == 3 && WEAK_LETTERS.contains(
                &stem_text.chars().nth(1).unwrap_or(' '))
                => self.verb_form_confidence(2, false, 0.15),

            // Form III (فَاعَلَ): 4-letter stem with medial alif (2nd char is ا).
            // E.g., كاتب (kataba), قاتل (qatala), سافر (safara)
            // KB-0004 lookup: verb_confidence(3) = 0.20
            3 if stem_len >= 4
                && stem_text.chars().nth(1) == Some('ا')
                => self.verb_form_confidence(3, false, 0.20),

            // Form IV (أَفْعَلَ): 4+ letter stem starting with hamzated alif.
            // E.g., أخرج (akhraja), أكمل (akmala), أفهم (afhama)
            // KB-0004 lookup: verb_confidence(4) = 0.20
            4 if stem_len >= 4
                && (stem_text.starts_with('أ') || stem_text.starts_with('إ'))
                => self.verb_form_confidence(4, false, 0.20),

            // Form V (تَفَعَّلَ): 4+ letter stem starting with ت.
            // E.g., تعلّم (ta'allama), تكسّر (takassara)
            // KB-0004 lookup: verb_confidence(5) = 0.20
            5 if stem_len >= 4 && stem_text.starts_with('ت')
                => self.verb_form_confidence(5, false, 0.20),

            // Form VI (تَفَاعَلَ): 5+ letter stem starting with ت with internal ا.
            // E.g., تعاون (ta'aawana), تباعد (tabaa'ada)
            // KB-0004 lookup: verb_confidence(6) = 0.15
            6 if stem_len >= 5
                && stem_text.starts_with('ت')
                && stem_text.chars().nth(2) == Some('ا')
                => self.verb_form_confidence(6, false, 0.15),

            // Form VII (اِنْفَعَلَ): 5+ letter stem starting with ا.
            // E.g., انكسر (inkasara), انطلق (intalaqa)
            // KB-0004 lookup: verb_confidence(7) = 0.15
            7 if stem_len >= 5 && stem_text.starts_with('ا')
                => self.verb_form_confidence(7, false, 0.15),

            // Form VIII (اِفْتَعَلَ): 5+ letter stem with ت as 3rd consonant.
            // E.g., اجتهد (ijtahada), افتتح (iftataha), اجتماع with ت at pos 3
            // KB-0004 lookup: verb_confidence(8) = 0.20
            8 if stem_len >= 5
                && stem_text.chars().nth(2) == Some('ت')
                => self.verb_form_confidence(8, false, 0.20),

            // Form IX (اِفْعَلَّ): 4+ letter stem with last two consonants same.
            // E.g., اسودّ (iswadda = to be black), احمرّ (ihmarra = to be red)
            // KB-0004 lookup: verb_confidence(9) = 0.10
            9 if stem_len >= 4
                && stem_text.chars().last() == stem_text.chars().nth(stem_len - 2)
                => self.verb_form_confidence(9, false, 0.10),

            // Form X (اِسْتَفْعَلَ): 6+ letter stem starting with ا.
            // E.g., استخرج (istakhraja), استغفر (istaghfara)
            // KB-0004 lookup: verb_confidence(10) = 0.10
            10 if stem_len >= 6 && stem_text.starts_with('ا')
                => self.verb_form_confidence(10, false, 0.10),

            // Forms XI–XV: rare, low confidence even if structure matches
            // KB-0004 lookup: verb_confidence(11-15) = 0.05, 0.05, 0.03, 0.03, 0.03
            11..=15 if stem_len >= 5
                => self.verb_form_confidence(form, false, 0.05),

            // Default: stem does not match this form
            _ => 0.0,
        }
    }

    /// Match noun patterns (SPEC-0101 §5.4).
    ///
    /// Uses KB-0004 noun profile confidence when available (authoritative),
    /// falling back to heuristic values for patterns not yet in the KB.
    /// The source string changes to "kb0004:noun_pattern" when KB-0004
    /// provides the confidence, enabling traceability.
    fn match_noun_patterns(
        &self,
        stem_text: &str,
        candidates: &mut Vec<WazanCandidate>,
    ) {
        // Check for feminine marker
        let has_ta_marbuta = stem_text.ends_with('ة');
        let has_alif_mamduda = stem_text.contains("اء");

        // Check for plural/dual suffixes
        let has_plural_suffix = stem_text.ends_with("ون") || stem_text.ends_with("ين")
            || stem_text.ends_with("ات") || stem_text.ends_with("ان");

        let kb_available = self.kb.is_some();

        // Active participle pattern: فَاعِل
        let fail_conf = self.noun_pattern_confidence("فَاعِل",
            if stem_text.chars().count() >= 4 { 0.2 } else { 0.1 });
        candidates.push(WazanCandidate {
            text: "فَاعِل".to_string(),
            form: None,
            category: WazanCategory::NounIsmFail,
            source: if kb_available { "kb0004:noun_pattern:فَاعِل".to_string() } else { "heuristic:noun_pattern".to_string() },
            confidence: fail_conf,
        });

        // Passive participle pattern: مَفْعُول
        let maful_conf = self.noun_pattern_confidence("مَفْعُول", 0.15);
        candidates.push(WazanCandidate {
            text: "مَفْعُول".to_string(),
            form: None,
            category: WazanCategory::NounIsmMaful,
            source: if kb_available { "kb0004:noun_pattern:مَفْعُول".to_string() } else { "heuristic:noun_pattern".to_string() },
            confidence: maful_conf,
        });

        // Masdar (verbal noun) — higher confidence for 3-letter stems
        // since many 3-letter forms are actually verbal nouns (e.g., عَمَل, سَلَام)
        // and even concrete nouns share this form without diacritics.
        let masdar_fallback = if stem_text.chars().count() == 3 { 0.25 } else { 0.15 };
        let masdar_conf = self.noun_pattern_confidence("مَصْدَر", masdar_fallback);
        candidates.push(WazanCandidate {
            text: "مَصْدَر".to_string(),
            form: None,
            category: WazanCategory::NounMasdar,
            source: if kb_available { "kb0004:noun_pattern:مَصْدَر".to_string() } else { "heuristic:noun_pattern".to_string() },
            confidence: masdar_conf,
        });

        // Broken plural
        if has_plural_suffix {
            let jam_conf = self.noun_pattern_confidence("جَمْع", 0.25);
            candidates.push(WazanCandidate {
                text: "جَمْع".to_string(),
                form: None,
                category: WazanCategory::NounBrokenPlural,
                source: if kb_available { "kb0004:noun_pattern:جَمْع".to_string() } else { "heuristic:noun_pattern".to_string() },
                confidence: jam_conf,
            });
        }

        // Noun of place / time: مَفْعَل
        if stem_text.starts_with('م') && stem_text.len() >= 4 {
            let makan_conf = self.noun_pattern_confidence("مَفْعَل", 0.15);
            candidates.push(WazanCandidate {
                text: "مَفْعَل".to_string(),
                form: None,
                category: WazanCategory::NounIsmMakan,
                source: if kb_available { "kb0004:noun_pattern:مَفْعَل".to_string() } else { "heuristic:noun_pattern".to_string() },
                confidence: makan_conf,
            });
        }

        // Feminine noun patterns:
        //   - Stems ending with تاء مربوطة (ة) → فَعْلَة pattern
        //   - Stems ending with alif mamduda (اء) → فَعْلَاء pattern
        if has_ta_marbuta {
            let faila_conf = self.noun_pattern_confidence("فَعْلَة", 0.30);
            candidates.push(WazanCandidate {
                text: "فَعْلَة".to_string(),
                form: None,
                category: WazanCategory::NounOther,
                source: if kb_available { "kb0004:noun_pattern:فَعْلَة".to_string() } else { "heuristic:noun_pattern".to_string() },
                confidence: faila_conf,
            });
        } else if has_alif_mamduda {
            let fala_conf = self.noun_pattern_confidence("فَعْلَاء", 0.30);
            candidates.push(WazanCandidate {
                text: "فَعْلَاء".to_string(),
                form: None,
                category: WazanCategory::NounOther,
                source: if kb_available { "kb0004:noun_pattern:فَعْلَاء".to_string() } else { "heuristic:noun_pattern".to_string() },
                confidence: fala_conf,
            });
        }
    }

    // ════════════════════════════════════════════
    //  Subsystem 4: Feature Extraction
    // ════════════════════════════════════════════

    /// Determine the part of speech (SPEC-0101 §6).
    ///
    /// Uses KB-0004 noun profile POS when available for noun patterns
    /// (e.g., فَعِيل is Adjective, not Noun). Falls back to positional
    /// heuristics (nisbah ي ending → Adjective) when KB is absent.
    fn determine_pos(&self, stem_text: &str, wazan: &WazanCandidate) -> PartOfSpeech {
        match wazan.category {
            WazanCategory::VerbFormI | WazanCategory::VerbFormII
            | WazanCategory::VerbFormIII | WazanCategory::VerbFormIV
            | WazanCategory::VerbFormV | WazanCategory::VerbFormVI
            | WazanCategory::VerbFormVII | WazanCategory::VerbFormVIII
            | WazanCategory::VerbFormIX | WazanCategory::VerbFormX
            | WazanCategory::VerbFormXI | WazanCategory::VerbFormXII
            | WazanCategory::VerbFormXIII | WazanCategory::VerbFormXIV
            | WazanCategory::VerbFormXV => PartOfSpeech::Verb,

            WazanCategory::NounMasdar | WazanCategory::NounIsmFail
            | WazanCategory::NounIsmMaful | WazanCategory::NounSifatMusabbaha
            | WazanCategory::NounIsmMakan | WazanCategory::NounIsmZaman
            | WazanCategory::NounIsmAlah | WazanCategory::NounSighahMubalaghah
            | WazanCategory::NounTafdil | WazanCategory::NounNisbah
            | WazanCategory::NounBrokenPlural | WazanCategory::NounOther => {
                // Check KB-0004 first for the pattern's POS classification
                // This handles patterns like فَعِيل (Adjective), فَعُول (Adjective),
                // أَفْعَل (Adjective), فَعْلَان (Adjective), etc.
                if self.noun_pattern_is_adjective(&wazan.text) {
                    return PartOfSpeech::Adjective;
                }
                // Fallback: nisbah ي suffix on non-ة stems
                if stem_text.ends_with('ي') && !stem_text.ends_with("ة") {
                    PartOfSpeech::Adjective
                } else {
                    PartOfSpeech::Noun
                }
            }
        }
    }

    /// Extract morphological features (SPEC-0101 §6.2-6.4).
    fn extract_features(
        &self,
        stem_text: &str,
        _root: &RootCandidate,
        wazan: &WazanCandidate,
        pos: PartOfSpeech,
    ) -> FeatureSet {
        let mut feature_assignments = Vec::new();
        let mut bitfield = FeatureBitfield::new();

        // Set POS in bitfield (using PartOfSpeech code)
        bitfield = bitfield.with_pos(pos.code());

        match pos {
            PartOfSpeech::Verb => {
                self.extract_verb_features(stem_text, wazan, &mut feature_assignments, &mut bitfield);
            }
            PartOfSpeech::Noun | PartOfSpeech::Adjective => {
                self.extract_noun_features(stem_text, &mut feature_assignments, &mut bitfield);
            }
            _ => {}
        }

        let confidence = if feature_assignments.is_empty() {
            0.0
        } else {
            feature_assignments.iter().map(|f| f.confidence).sum::<f64>() / feature_assignments.len() as f64
        };

        FeatureSet {
            features: feature_assignments,
            bitfield: bitfield.to_raw(),
            confidence,
        }
    }

    /// Extract verb-specific features (SPEC-0101 §6.2).
    fn extract_verb_features(
        &self,
        stem_text: &str,
        wazan: &WazanCandidate,
        features: &mut Vec<FeatureAssignment>,
        bitfield: &mut FeatureBitfield,
    ) {
        // Verb form
        if let Some(form) = wazan.form {
            features.push(FeatureAssignment {
                name: "verb_form".to_string(),
                value: format!("{}", form),
                confidence: wazan.confidence,
                source: "wazan".to_string(),
            });
        }

        // Tense detection from affixes
        let has_prefix = stem_text.starts_with('ي') || stem_text.starts_with('ت')
            || stem_text.starts_with('أ') || stem_text.starts_with('ن');
        let has_perfect_suffix = stem_text.ends_with('ت') || stem_text.ends_with('ا')
            || stem_text.ends_with('و');

        if !has_prefix && has_perfect_suffix {
            // Perfect tense (ماضي)
            features.push(FeatureAssignment {
                name: "tense".to_string(), value: "past".to_string(),
                confidence: 0.5, source: "affix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_tense(TENSE_PAST);
        } else if has_prefix {
            // Imperfect tense (مضارع)
            features.push(FeatureAssignment {
                name: "tense".to_string(), value: "present".to_string(),
                confidence: 0.5, source: "affix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_tense(TENSE_PRESENT);
        } else {
            // Imperative
            features.push(FeatureAssignment {
                name: "tense".to_string(), value: "imperative".to_string(),
                confidence: 0.3, source: "affix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_tense(TENSE_IMPERATIVE);
        }

        // Person detection from prefixes
        if stem_text.starts_with('أ') {
            features.push(FeatureAssignment {
                name: "person".to_string(), value: "first".to_string(),
                confidence: 0.4, source: "prefix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_person(PERSON_FIRST);
        } else if stem_text.starts_with('ت') {
            features.push(FeatureAssignment {
                name: "person".to_string(), value: "second".to_string(),
                confidence: 0.4, source: "prefix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_person(PERSON_SECOND);
        } else if stem_text.starts_with('ي') || stem_text.starts_with('ن') {
            features.push(FeatureAssignment {
                name: "person".to_string(), value: "third".to_string(),
                confidence: 0.4, source: "prefix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_person(PERSON_THIRD);
        }

        // Gender detection — Arabic verb gender is marked by prefixes and suffixes.
        // Internal ت is always a root letter (e.g., ك-ت-ب in يكتب), never a gender marker.
        // Prefixes: يـ = masculine (3rd person), تـ = 3rd feminine / 2nd person common
        //   أ/نـ = 1st person (masculine default)
        // Suffixes: ين = 2nd person feminine, ن = feminine plural
        if stem_text.starts_with('ي') || stem_text.starts_with('أ') || stem_text.starts_with('ن') {
            // ي = 3rd person, أ = 1st, ن = 1st plural
            // Check feminine suffixes before defaulting to masculine
            // (يكتبن = 3rd feminine plural despite ي prefix)
            if stem_text.ends_with("ين") || stem_text.ends_with('ن') {
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "feminine".to_string(),
                    confidence: 0.35, source: "suffix_analysis".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_FEMININE);
            } else {
                // Default masculine for ي/أ/ن prefixes
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "masculine".to_string(),
                    confidence: 0.3, source: "prefix_analysis".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_MASCULINE);
            }
        } else if stem_text.starts_with('ت') {
            // ت prefix = 3rd feminine or 2nd person (gender neutral)
            // Check for feminine suffixes to disambiguate
            if stem_text.ends_with("ين") || stem_text.ends_with('ن') {
                // Feminine suffixes (تكتبين = you fem, يكتبن = they fem)
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "feminine".to_string(),
                    confidence: 0.4, source: "suffix_analysis".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_FEMININE);
            } else {
                // Default to feminine for ت prefix (most common 3rd fem reading)
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "feminine".to_string(),
                    confidence: 0.3, source: "prefix_analysis".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_FEMININE);
            }
        } else {
            // Imperative or perfect: check feminine suffixes
            if stem_text.ends_with('ي') || stem_text.ends_with('ن') {
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "feminine".to_string(),
                    confidence: 0.4, source: "suffix_analysis".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_FEMININE);
            } else {
                features.push(FeatureAssignment {
                    name: "gender".to_string(), value: "masculine".to_string(),
                    confidence: 0.3, source: "default".to_string(),
                });
                *bitfield = (*bitfield).with_gender(GENDER_MASCULINE);
            }
        }

        // Number detection from suffixes
        if stem_text.ends_with('و') || stem_text.ends_with("ون") {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "plural".to_string(),
                confidence: 0.4, source: "suffix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_PLURAL);
        } else if stem_text.ends_with('ا') && !stem_text.starts_with('ا') {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "dual".to_string(),
                confidence: 0.35, source: "suffix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_DUAL);
        } else {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "singular".to_string(),
                confidence: 0.3, source: "default".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_SINGULAR);
        }
    }

    /// Extract noun-specific features (SPEC-0101 §6.3).
    fn extract_noun_features(
        &self,
        stem_text: &str,
        features: &mut Vec<FeatureAssignment>,
        bitfield: &mut FeatureBitfield,
    ) {
        // Gender detection
        if stem_text.ends_with('ة') || stem_text.contains("اء") {
            features.push(FeatureAssignment {
                name: "gender".to_string(), value: "feminine".to_string(),
                confidence: 0.4, source: "suffix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_gender(GENDER_FEMININE);
        } else {
            features.push(FeatureAssignment {
                name: "gender".to_string(), value: "masculine".to_string(),
                confidence: 0.2, source: "default".to_string(),
            });
            *bitfield = (*bitfield).with_gender(GENDER_MASCULINE);
        }

        // Number detection
        if stem_text.ends_with("ون") || stem_text.ends_with("ين") || stem_text.ends_with("ات") {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "plural".to_string(),
                confidence: 0.5, source: "suffix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_PLURAL);
        } else if stem_text.ends_with("ان") {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "dual".to_string(),
                confidence: 0.45, source: "suffix_analysis".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_DUAL);
        } else {
            features.push(FeatureAssignment {
                name: "number".to_string(), value: "singular".to_string(),
                confidence: 0.3, source: "default".to_string(),
            });
            *bitfield = (*bitfield).with_number(NUMBER_SINGULAR);
        }

        // State (definiteness)
        let has_al = stem_text.starts_with("ال");
        if has_al {
            features.push(FeatureAssignment {
                name: "state".to_string(), value: "definite".to_string(),
                confidence: 0.6, source: "prefix_analysis".to_string(),
            });
        } else {
            features.push(FeatureAssignment {
                name: "state".to_string(), value: "indefinite".to_string(),
                confidence: 0.3, source: "default".to_string(),
            });
        }
    }

    /// Create a partial analysis when root is found but no wazan matched.
    fn make_partial_analysis(
        &self,
        seg_token: &agos_core::ir::SegmentedToken,
        segmentation: &agos_core::ir::Segmentation,
        stem_text: &str,
        root: &RootCandidate,
    ) -> StemAnalysis {
        StemAnalysis {
            analysis_id: format!("ana-partial-{}-{}", seg_token.raw_token.id, root.text),
            segmentation_id: segmentation.id.clone(),
            stem: stem_text.to_string(),
            root: Some(RootRef {
                text: root.text.clone(),
                source: root.source.clone(),
                confidence: root.confidence,
            }),
            wazan: None,
            pos: PartOfSpeech::Unknown,
            features: vec![],
            is_ambiguous: false,
            alternatives: vec![],
            evidence: vec![EvidenceEntry {
                id: format!("ev-{}-partial", seg_token.raw_token.id),
                timestamp: String::new(),
                stage: "MOD-04".to_string(),
                stage_iteration: 0,
                category: EvidenceCategory::Morphology,
                rule_or_algorithm: "partial_analysis".to_string(),
                version: "1.0".to_string(),
                input_description: format!("stem={}", stem_text),
                input_state_hash: String::new(),
                output_description: format!("root={}", root.text),
                output_delta: "partial_analysis".to_string(),
                confidence: root.confidence * 0.5,
                token_indices: vec![seg_token.raw_token.id],
            }],
        }
    }
}

// ──────────────────────────────────────────────
//  PipelineStage Implementation
// ──────────────────────────────────────────────

impl PipelineStage<SegmentedTokenStream, MorphologicalAnalysis> for MorphologicalParser {
    fn stage_id(&self) -> &'static str {
        "MOD-04"
    }

    fn process(
        &self,
        input: SegmentedTokenStream,
        _ctx: &PipelineContext,
    ) -> PipelineResult<MorphologicalAnalysis> {
        self.analyze(input)
    }

    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        if self.config.max_analyses == 0 {
            return Err(agos_core::error::PipelineError::fatal(
                agos_core::error::codes::INVALID_REQUEST,
                "MorphologicalParserConfig.max_analyses must be > 0",
                "MOD-04",
            ));
        }

        // If KB-0004 is enabled, verify the directory exists at validation time.
        if self.config.kb_config.enabled {
            let kb_path = std::path::Path::new(&self.config.kb_config.path);
            if !kb_path.exists() || !kb_path.is_dir() {
                return Err(agos_core::error::PipelineError::fatal(
                    agos_core::error::codes::KB_LOAD_FAILURE,
                    format!(
                        "KB-0004 directory not found or is not a directory: {}",
                        self.config.kb_config.path
                    ),
                    "MOD-04",
                ));
            }
        }

        Ok(())
    }
}

impl Default for MorphologicalParser {
    fn default() -> Self {
        // Use from_config so KB-0004 auto-loading works with default config
        // (kb_config.enabled defaults to false, so this is equivalent to new()
        // unless the user has set up Kb0004Config).
        Self::from_config(MorphologicalParserConfig::default())
    }
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agos_core::ir::{Morpheme, RawToken, SegmentedToken, Segmentation, SegmentedTokenStreamMetadata};
    use agos_core::types::{MorphemeType, TokenType};

    fn make_segmented_stream(stems: &[&str]) -> SegmentedTokenStream {
        let mut tokens = Vec::new();
        for (i, &stem) in stems.iter().enumerate() {
            let raw = RawToken {
                id: i,
                text: stem.to_string(),
                token_type: TokenType::Word,
                start_offset: 0,
                end_offset: stem.len(),
            };
            let seg = Segmentation {
                id: format!("seg-{}-0", i),
                morphemes: vec![Morpheme {
                    text: stem.to_string(),
                    morpheme_type: MorphemeType::Stem,
                    original_offset: 0,
                    length: stem.len(),
                }],
                confidence: 1.0,
                source: "default".to_string(),
            };
            tokens.push(SegmentedToken {
                raw_token: raw,
                segmentations: vec![seg],
            });
        }
        SegmentedTokenStream {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            tokens,
            metadata: SegmentedTokenStreamMetadata {
                total_tokens: stems.len() as u64,
                segmentable_tokens: stems.len() as u64,
                ambiguous_tokens: 0,
                total_ambiguity: 1.0,
            },
        }
    }

    fn test_ctx() -> PipelineContext {
        PipelineContext::new(agos_core::types::GrammarSchool::Basra)
    }

    #[test]
    fn test_empty_input() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        let input = make_segmented_stream(&[]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.total_tokens, 0);
    }

    #[test]
    fn test_simple_word_analysis() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "كتب" (kataba = "he wrote") — triliteral root ك-ت-ب
        let input = make_segmented_stream(&["كتب"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.analyzed_tokens, 1);
        assert!(!output.token_analyses.is_empty());
        let ta = &output.token_analyses[0];
        assert!(!ta.stem_analyses.is_empty());
        // Should find root ك-ت-ب
        let has_ktb = ta.stem_analyses.iter().any(|sa| {
            sa.root.as_ref().map(|r| r.text == "كتب") == Some(true)
        });
        assert!(has_ktb, "Should find root كتب (k-t-b)");
    }

    #[test]
    fn test_particle_fast_path() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "فِي" (fee = "in") — should be identified as particle
        let input = make_segmented_stream(&["فِي"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.analyzed_tokens, 1);
        let ta = &output.token_analyses[0];
        let is_particle = ta.stem_analyses.iter().any(|sa| sa.pos == PartOfSpeech::Particle);
        assert!(is_particle, "فِي should be identified as particle");
    }

    #[test]
    fn test_pronoun_fast_path() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "هُوَ" (huwa = "he") — should be identified as pronoun
        let input = make_segmented_stream(&["هُوَ"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.analyzed_tokens, 1);
        let ta = &output.token_analyses[0];
        let is_pronoun = ta.stem_analyses.iter().any(|sa| sa.pos == PartOfSpeech::Pronoun);
        assert!(is_pronoun, "هُوَ should be identified as pronoun");
    }

    #[test]
    fn test_hollow_root_detection() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "قَالَ" (qaala = "he said") — hollow root ق-و-ل
        let input = make_segmented_stream(&["قال"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let ta = &output.token_analyses[0];
        let has_hollow = ta.stem_analyses.iter().any(|sa| {
            sa.root.as_ref().map(|r| r.text == "قول") == Some(true)
        });
        assert!(has_hollow, "قال should have hollow root ق-و-ل");
    }

    #[test]
    fn test_defective_root_detection() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "رَمَى" (ramaa = "he threw") — defective root ر-م-ي
        let input = make_segmented_stream(&["رمى"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let ta = &output.token_analyses[0];
        let has_defective = ta.stem_analyses.iter().any(|sa| {
            sa.root.as_ref().map(|r| r.text == "رمي") == Some(true)
        });
        assert!(has_defective, "رمى should have defective root ر-م-ي");
    }

    #[test]
    fn test_pipeline_stage_trait() {
        let parser = MorphologicalParser::default();
        assert_eq!(parser.stage_id(), "MOD-04");
        let ctx = test_ctx();
        let result = parser.validate_config(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metadata_counts() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // Mixed: known word, particle, pronoun
        let input = make_segmented_stream(&["كتب", "فِي", "هُوَ"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.total_tokens, 3);
        assert_eq!(output.metadata.analyzed_tokens, 3);
    }

    #[test]
    fn test_verb_form_identification() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // Basic verb forms should get form assignment
        let input = make_segmented_stream(&["كتب"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let ta = &output.token_analyses[0];
        let has_form = ta.stem_analyses.iter().any(|sa| {
            sa.wazan.as_ref().and_then(|w| w.form) == Some(1)
        });
        assert!(has_form, "كتب should have Form I among its analyses");
    }

    #[test]
    fn test_school_config_affects_form_priority() {
        let ctx = PipelineContext::new(GrammarSchool::Andalus);
        let mut config = MorphologicalParserConfig::default();
        config.school = GrammarSchool::Andalus;
        let parser = MorphologicalParser::new(config);
        let input = make_segmented_stream(&["كتب"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_rejects_zero_analyses() {
        let mut config = MorphologicalParserConfig::default();
        config.max_analyses = 0;
        let parser = MorphologicalParser::new(config);
        let ctx = test_ctx();
        let result = parser.validate_config(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_spec_fields() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        let input = make_segmented_stream(&["كتب"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.spec, "SPEC-0001");
        assert_eq!(output.version, "1.0");
    }

    #[test]
    fn test_verb_feature_extraction() {
        let parser = MorphologicalParser::default();
        let ctx = test_ctx();
        // "يَكْتُبُ" (yaktubu = "he writes") — imperfect, 3rd masc, singular
        let input = make_segmented_stream(&["يكتب"]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let ta = &output.token_analyses[0];

        // Get the verb form analysis
        for sa in &ta.stem_analyses {
            if sa.wazan.as_ref().and_then(|w| w.form) == Some(1) {
                assert_eq!(
                    sa.pos, PartOfSpeech::Verb,
                    "Form I analysis should be a verb"
                );
            }
        }
    }
}
