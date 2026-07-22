//! # MOD-05: SyntaxParser
//!
//! Performs Arabic syntactic (nahw) analysis on morphologically-analyzed tokens.
//! Determines sentence structure, constituent relationships, and i'rab
//! (grammatical roles) according to the configured grammar school.
//!
//! ## Architecture (SPEC-0001-C3 §6)
//!
//! 1. **Sentence Segmentation** — Group tokens into sentence boundaries
//! 2. **Sentence Type Identification** — Verbal (jumlah fi'liyyah) vs
//!    Nominal (jumlah ismiyyah) vs Conditional (jumlah shartiyyah)
//! 3. **Verbal Sentence Parsing** — Fi'l → Fa'il → Maf'ul constructions
//! 4. **Nominal Sentence Parsing** — Mubtada' → Khabar constructions
//! 5. **Construction Detection** — Idafa, Na'at, Harf Jarr, etc.
//! 6. **Ambiguity Handling** — Multiple parse trees from morphological ambiguity
//!
//! ## Pipeline Interface
//!
//! ```ignore
//! Input:  MorphologicalAnalysis (IR-4)
//! Output: SyntaxTree (IR-5)
//! ```

use std::collections::HashMap;

use agos_core::error::PipelineResult;
use agos_core::ir::{
    Constituent, MorphologicalAnalysis, ParseTree, SyntaxTree, SyntaxTreeMetadata, TokenAnalysis,
};
use agos_core::pipeline::{PipelineContext, PipelineStage};
use agos_core::types::{NodeType, PartOfSpeech, SentenceType, SyntacticRole};

use crate::config::SyntaxParserConfig;
use crate::error::SyntaxError;

// ──────────────────────────────────────────────
//  Internal Types
// ──────────────────────────────────────────────

/// A sentence identified within the token stream.
#[derive(Debug, Clone)]
struct Sentence {
    /// Token indices (from the original analysis) that belong to this sentence
    pub token_ids: Vec<usize>,
    /// Start index within the token stream
    pub start: usize,
    /// End index (exclusive)
    #[allow(dead_code)]
    pub end: usize,
}

/// Result of parsing a single sentence.
#[derive(Debug, Clone)]
struct ParsedSentence {
    /// The parse tree constituent
    pub constituent: Constituent,
    /// Sentence type
    pub sentence_type: SentenceType,
    /// Confidence (0.0 to 1.0)
    pub confidence: f64,
}

// ──────────────────────────────────────────────
//  Sentence Boundary Punctuation
// ──────────────────────────────────────────────

// Reserved for future sentence boundary detection from the original token stream.
// const SENTENCE_BOUNDARY_PUNCT: &[char] = &['.', '!', '؟', ';'];
// const CLAUSE_CONJUNCTIONS: &[&str] = &["وَ", "فَ", "ثُمَّ", "حَتَّى", "لَكِنَّ", "بَلْ"];

// ──────────────────────────────────────────────
//  SyntaxParser Stage
// ──────────────────────────────────────────────

/// MOD-05: SyntaxParser — Arabic syntactic (nahw) parsing.
///
/// Processes morphologically-analyzed tokens through 6 internal subsystems:
/// 1. Sentence Segmentation
/// 2. Sentence Type Identification
/// 3. Verbal Sentence Parsing
/// 4. Nominal Sentence Parsing
/// 5. Construction Detection
/// 6. Ambiguity Handling
///
/// ## Determinism
///
/// Fully deterministic. Same input = same parse trees always.
///
/// ## Performance Targets (SPEC-0001-C3 §6.9)
///
/// | Metric | Target |
/// |--------|--------|
/// | Throughput | > 1K sentences/second |
/// | Latency (p50) | < 1 ms per sentence |
#[derive(Debug, Clone)]
pub struct SyntaxParser {
    pub config: SyntaxParserConfig,
}

impl SyntaxParser {
    pub fn new(config: SyntaxParserConfig) -> Self {
        Self { config }
    }

    // ════════════════════════════════════════════
    //  Core Parse Pipeline
    // ════════════════════════════════════════════

    /// Parse syntax for a morphologically-analyzed token stream (SPEC-0001-C3 §6.3).
    pub fn parse(&self, input: MorphologicalAnalysis) -> PipelineResult<SyntaxTree> {
        // Step 1: Sentence Segmentation
        let sentences = self.segment_sentences(&input);

        if sentences.is_empty() {
            // No sentences — return an empty parse
            return Ok(SyntaxTree {
                spec: "SPEC-0001".to_string(),
                version: "1.0".to_string(),
                trees: vec![],
                metadata: SyntaxTreeMetadata {
                    sentence_count: 0,
                    tokens_parsed: 0,
                    ambiguity_count: 0,
                    parse_time_ms: 0.0,
                },
            });
        }

        // Check max sentence length
        for sentence in &sentences {
            if sentence.token_ids.len() > self.config.max_sentence_length {
                return Err(SyntaxError::SentenceTooLong {
                    token_count: sentence.token_ids.len(),
                    max_length: self.config.max_sentence_length,
                }
                .into_pipeline("MOD-05"));
            }
        }

        // Step 2-5: Parse each sentence
        let mut trees = Vec::new();
        let mut total_ambiguity = 0u64;

        for sentence in &sentences {
            let parsed = self.parse_sentence(&input, sentence);
            for tree in parsed {
                trees.push(ParseTree {
                    id: format!("tree-{}-{}", sentence.start, trees.len()),
                    tree_type: tree.sentence_type,
                    root: tree.constituent,
                    confidence: tree.confidence,
                    source: format!("agos-{}", self.config.school.as_str()),
                });
                if tree.confidence < 0.7 {
                    total_ambiguity += 1;
                }
            }
        }

        // Check max parse trees
        if trees.len() > self.config.max_parse_trees {
            // Keep only the highest-confidence trees
            trees.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            trees.truncate(self.config.max_parse_trees);
        }

        // Step 7: Return
        let tokens_parsed = sentences.iter().map(|s| s.token_ids.len() as u64).sum();

        Ok(SyntaxTree {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            trees,
            metadata: SyntaxTreeMetadata {
                sentence_count: sentences.len() as u64,
                tokens_parsed,
                ambiguity_count: total_ambiguity,
                parse_time_ms: 0.0,
            },
        })
    }

    // ════════════════════════════════════════════
    //  Subsystem 1: Sentence Segmentation
    // ════════════════════════════════════════════

    /// Group tokens into sentence boundaries (SPEC-0001-C3 §6.3, Step 1).
    fn segment_sentences(&self, input: &MorphologicalAnalysis) -> Vec<Sentence> {
        // Build a map of token_id → TokenAnalysis for quick lookup
        let token_analysis_map: HashMap<usize, &TokenAnalysis> = input
            .token_analyses
            .iter()
            .map(|ta| (ta.token_id, ta))
            .collect();

        // We need the full ordered list of token IDs from the input
        // The MorphologicalAnalysis only has token_analyses (word tokens).
        // We reconstruct token order from token_ids in token_analyses.
        let mut all_token_ids: Vec<usize> = token_analysis_map.keys().copied().collect();
        all_token_ids.sort();

        if all_token_ids.is_empty() {
            return vec![];
        }

        // Find sentence boundaries
        let mut sentences = Vec::new();
        let mut start = 0usize;

        for i in 0..all_token_ids.len() {
            let token_id = all_token_ids[i];

            // Check if this token is a sentence boundary marker
            let is_boundary = self.is_sentence_boundary(token_id, &token_analysis_map);

            if is_boundary {
                let end = i + 1;
                if end > start {
                    let slice: Vec<usize> = all_token_ids[start..end].to_vec();
                    sentences.push(Sentence {
                        token_ids: slice,
                        start,
                        end,
                    });
                }
                start = end;
            }
        }

        // Handle remaining tokens as the last sentence
        if start < all_token_ids.len() {
            sentences.push(Sentence {
                token_ids: all_token_ids[start..].to_vec(),
                start,
                end: all_token_ids.len(),
            });
        }

        // If no boundaries found, treat entire input as one sentence
        if sentences.is_empty() && !all_token_ids.is_empty() {
            let end = all_token_ids.len();
            sentences.push(Sentence {
                token_ids: all_token_ids,
                start: 0,
                end,
            });
        }

        sentences
    }

    /// Check if a token is a sentence boundary marker.
    fn is_sentence_boundary(
        &self,
        token_id: usize,
        analysis_map: &HashMap<usize, &TokenAnalysis>,
    ) -> bool {
        // We need the raw token text. We derive it from stem_analyses.stem.
        // If the token has no analysis, it might be punctuation skipped by MOD-04.
        // For now, we use a heuristic: tokens not in analysis_map are non-word
        // tokens (punctuation, whitespace) and may be boundaries.
        if !analysis_map.contains_key(&token_id) {
            // Non-word token — could be punctuation
            // In the current pipeline, MOD-04 skips non-word tokens,
            // so we rely on sentence boundary logic within analyzed tokens.
            return false;
        }

        false
    }

    // ════════════════════════════════════════════
    //  Subsystem 2: Sentence Type & Parse
    // ════════════════════════════════════════════

    /// Parse a single sentence, returning all possible parse trees (ambiguity).
    fn parse_sentence(&self, input: &MorphologicalAnalysis, sentence: &Sentence) -> Vec<ParsedSentence> {
        let token_ids = &sentence.token_ids;
        if token_ids.is_empty() {
            return vec![];
        }

        // Build analysis map
        let analysis_map: HashMap<usize, &TokenAnalysis> = input
            .token_analyses
            .iter()
            .map(|ta| (ta.token_id, ta))
            .collect();

        // Get POS and feature info for each token in the sentence
        let token_info: Vec<(usize, Vec<TokenParseInfo>)> = token_ids
            .iter()
            .map(|&id| {
                let infos = if let Some(ta) = analysis_map.get(&id) {
                    ta.stem_analyses
                        .iter()
                        .map(|sa| TokenParseInfo {
                            pos: sa.pos,
                            stem: sa.stem.clone(),
                            root: sa.root.as_ref().map(|r| r.text.clone()),
                            features: sa.features.iter().map(|f| f.name.clone()).collect(),
                        })
                        .collect()
                } else {
                    vec![TokenParseInfo {
                        pos: PartOfSpeech::Unknown,
                        stem: String::new(),
                        root: None,
                        features: vec![],
                    }]
                };
                (id, infos)
            })
            .collect();

        if token_info.is_empty() {
            return vec![];
        }

        // Determine sentence type from the first word's POS
        let first_pos = token_info
            .first()
            .and_then(|(_, infos)| infos.first())
            .map(|info| info.pos)
            .unwrap_or(PartOfSpeech::Unknown);

        // Identify sentence type (Step 3)
        let sentence_type = match first_pos {
            PartOfSpeech::Verb => SentenceType::JumlahFiliyyah,
            PartOfSpeech::Particle => {
                // Check if it's a conditional or prepositional particle
                SentenceType::JumlahShartiyyah
            }
            PartOfSpeech::Noun
            | PartOfSpeech::Pronoun
            | PartOfSpeech::Adjective
            | PartOfSpeech::ProperNoun => SentenceType::JumlahIsmiyyah,
            _ => SentenceType::Unknown,
        };

        // Parse based on sentence type
        match sentence_type {
            SentenceType::JumlahFiliyyah => {
                self.parse_verbal_sentence(token_ids, &token_info)
            }
            SentenceType::JumlahIsmiyyah => {
                self.parse_nominal_sentence(token_ids, &token_info)
            }
            _ => {
                // Default/fallback: try both parses
                let mut results = self.parse_verbal_sentence(token_ids, &token_info);
                results.extend(self.parse_nominal_sentence(token_ids, &token_info));
                if results.is_empty() && self.config.enable_partial_parse {
                    let partial = self.make_partial_parse(token_ids, &token_info);
                    results.push((partial, 0.2));
                }
                results
            }
        }
        .into_iter()
        .map(|(constituent, confidence)| ParsedSentence {
            constituent,
            sentence_type,
            confidence,
        })
        .collect()
    }

    // ════════════════════════════════════════════
    //  Subsystem 3: Verbal Sentence Parsing
    // ════════════════════════════════════════════

    /// Parse a verbal sentence: Fi'l → Fa'il → Maf'ul (SPEC-0001-C3 §6.3, Step 4).
    fn parse_verbal_sentence(
        &self,
        token_ids: &[usize],
        token_info: &[(usize, Vec<TokenParseInfo>)],
    ) -> Vec<(Constituent, f64)> {
        let mut results = Vec::new();

        // For each possible morphological analysis combination, build a parse
        // In this heuristic implementation, we take the first analysis per token.

        // Find the verb (fi'l)
        let verb_idx = token_info.iter().position(|(_, infos)| {
            infos.iter().any(|info| info.pos == PartOfSpeech::Verb)
        });

        let Some(verb_pos) = verb_idx else {
            // No verb found, try partial parse
            if self.config.enable_partial_parse {
                return vec![(self.make_partial_parse(token_ids, token_info), 0.3)];
            }
            return vec![];
        };

        // Build constituent tree
        let mut children = Vec::new();
        let mut confidence_sum = 0.0f64;
        let mut confidence_count = 0u32;

        // Verb constituent
        let verb_token_ids = vec![token_ids[verb_pos]];
        children.push(self.make_word_constituent(
            SyntacticRole::FiL,
            &verb_token_ids,
            self.make_features(token_info, verb_pos, "verb"),
        ));
        confidence_sum += 0.8;
        confidence_count += 1;

        // Find subject (fa'il): the noun/pronoun after the verb in nominative
        let subject_candidates: Vec<usize> = (verb_pos + 1..token_info.len())
            .filter(|&i| {
                token_info[i]
                    .1
                    .iter()
                    .any(|info| matches!(info.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::ProperNoun | PartOfSpeech::Adjective))
            })
            .collect();

        if let Some(&subj_idx) = subject_candidates.first() {
            let subj_token_ids = vec![token_ids[subj_idx]];
            children.push(self.make_word_constituent(
                SyntacticRole::Fail,
                &subj_token_ids,
                self.make_features(token_info, subj_idx, "subject"),
            ));
            confidence_sum += 0.7;
            confidence_count += 1;

            // Find objects (maf'ul bihi): nouns/pronouns after the subject
            let object_candidates: Vec<usize> = (subj_idx + 1..token_info.len())
                .filter(|&i| {
                    token_info[i]
                        .1
                        .iter()
                        .any(|info| matches!(info.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::ProperNoun))
                })
                .collect();

            for &obj_idx in object_candidates.iter().take(2) {
                // Max 2 objects for simplicity
                let obj_token_ids = vec![token_ids[obj_idx]];
                children.push(self.make_word_constituent(
                    SyntacticRole::MafulBiHi,
                    &obj_token_ids,
                    self.make_features(token_info, obj_idx, "object"),
                ));
                confidence_sum += 0.5;
                confidence_count += 1;
            }
        }

        let confidence = if confidence_count > 0 {
            confidence_sum / confidence_count as f64
        } else {
            0.5
        };

        let root = Constituent {
            node_type: NodeType::Clause,
            role: SyntacticRole::FiL,
            token_ids: token_ids.to_vec(),
            children,
            features: HashMap::from([("sentence_type".to_string(), "jumlah_fi'liyyah".to_string())]),
            implicit: false,
        };

        results.push((root, confidence));
        results
    }

    // ════════════════════════════════════════════
    //  Subsystem 4: Nominal Sentence Parsing
    // ════════════════════════════════════════════

    /// Parse a nominal sentence: Mubtada' → Khabar (SPEC-0001-C3 §6.3, Step 5).
    fn parse_nominal_sentence(
        &self,
        token_ids: &[usize],
        token_info: &[(usize, Vec<TokenParseInfo>)],
    ) -> Vec<(Constituent, f64)> {
        let mut results = Vec::new();

        if token_ids.len() < 2 {
            // Single word — could be mubtada' without khabar (incomplete)
            let root = self.make_word_constituent(
                SyntacticRole::Mubtada,
                token_ids,
                self.make_features(token_info, 0, "mubtada"),
            );
            return vec![(
                Constituent {
                    node_type: NodeType::Clause,
                    role: SyntacticRole::Mubtada,
                    token_ids: token_ids.to_vec(),
                    children: vec![root],
                    features: HashMap::from([(
                        "sentence_type".to_string(),
                        "jumlah_ismiyyah".to_string(),
                    )]),
                    implicit: false,
                },
                0.4,
            )];
        }

        // Find mubtada' (topic): first noun/pronoun in nominative/definite
        let mubtada_idx = token_info.iter().position(|(_, infos)| {
            infos
                .iter()
                .any(|info| matches!(info.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::ProperNoun | PartOfSpeech::Adjective))
        });

        let Some(&mub_idx) = mubtada_idx.as_ref() else {
            if self.config.enable_partial_parse {
                return vec![(self.make_partial_parse(token_ids, token_info), 0.2)];
            }
            return vec![];
        };

        let mut children = Vec::new();
        let mut confidence_sum = 0.0f64;
        let mut confidence_count = 0u32;

        // Mubtada' constituent
        let mub_token_ids = vec![token_ids[mub_idx]];
        children.push(self.make_word_constituent(
            SyntacticRole::Mubtada,
            &mub_token_ids,
            self.make_features(token_info, mub_idx, "mubtada"),
        ));
        confidence_sum += 0.7;
        confidence_count += 1;

        // Khabar (comment): everything after mubtada'
        if mub_idx + 1 < token_info.len() {
            let khabar_token_ids: Vec<usize> = token_ids[mub_idx + 1..].to_vec();

            // Check if khabar starts with a preposition
            let khabar_children = self.parse_khabar_phrase(
                &khabar_token_ids,
                &token_info[mub_idx + 1..],
            );

            let khabar_role = if khabar_children
                .first()
                .map(|c| c.role == SyntacticRole::HarfJarr)
                .unwrap_or(false)
            {
                SyntacticRole::Majrur
            } else {
                SyntacticRole::Khabar
            };

            children.push(Constituent {
                node_type: NodeType::Phrase,
                role: khabar_role,
                token_ids: khabar_token_ids.clone(),
                children: khabar_children,
                features: self.make_features_slice(token_info, mub_idx + 1, "khabar"),
                implicit: false,
            });
            confidence_sum += 0.6;
            confidence_count += 1;
        }

        let confidence = if confidence_count > 0 {
            confidence_sum / confidence_count as f64
        } else {
            0.3
        };

        let root = Constituent {
            node_type: NodeType::Clause,
            role: SyntacticRole::Mubtada,
            token_ids: token_ids.to_vec(),
            children,
            features: HashMap::from([(
                "sentence_type".to_string(),
                "jumlah_ismiyyah".to_string(),
            )]),
            implicit: false,
        };

        results.push((root, confidence));
        results
    }

    // ════════════════════════════════════════════
    //  Subsystem 5: Construction Detection
    // ════════════════════════════════════════════

    /// Parse a khabar phrase, detecting prepositions, idafa, etc.
    fn parse_khabar_phrase(
        &self,
        token_ids: &[usize],
        token_info: &[(usize, Vec<TokenParseInfo>)],
    ) -> Vec<Constituent> {
        let mut children = Vec::new();

        for (i, &tid) in token_ids.iter().enumerate() {
            let infos = &token_info[i].1;

            // Get the stem text for the first info to check for prepositions
            let stem = infos.first().map(|info| info.stem.as_str()).unwrap_or("");

            // Check for preposition clitics like بِ, لِ, كَ
            let is_preposition = matches!(
                infos.first().map(|info| info.pos),
                Some(PartOfSpeech::Preposition | PartOfSpeech::Particle)
            ) || stem == "بِ"
                || stem == "لِ"
                || stem == "كَ"
                || stem == "فِي"
                || stem == "مِن"
                || stem == "إِلَى"
                || stem == "عَلَى"
                || stem == "عَنْ";

            if is_preposition {
                children.push(self.make_word_constituent(
                    SyntacticRole::HarfJarr,
                    &[tid],
                    HashMap::new(),
                ));
            } else {
                // Default: treat as a noun (majrur if preceded by preposition,
                // or part of idafa)
                children.push(self.make_word_constituent(
                    SyntacticRole::Majrur,
                    &[tid],
                    HashMap::new(),
                ));
            }
        }

        children
    }

    // ════════════════════════════════════════════
    //  Helper Methods
    // ════════════════════════════════════════════

    /// Make features map for a single token.
    fn make_features(
        &self,
        token_info: &[(usize, Vec<TokenParseInfo>)],
        index: usize,
        _label: &str,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some((_, infos)) = token_info.get(index) {
            if let Some(info) = infos.first() {
                map.insert("pos".to_string(), format!("{:?}", info.pos));
                if let Some(root) = &info.root {
                    map.insert("root".to_string(), root.clone());
                }
            }
        }
        map
    }

    /// Make features map for a slice of tokens.
    fn make_features_slice(
        &self,
        token_info: &[(usize, Vec<TokenParseInfo>)],
        start: usize,
        _label: &str,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        // Collect POS of all tokens in the slice
        let pos_list: Vec<String> = token_info[start..]
            .iter()
            .flat_map(|(_, infos)| infos.first().map(|info| format!("{:?}", info.pos)))
            .collect();
        if !pos_list.is_empty() {
            map.insert("pos_sequence".to_string(), pos_list.join(","));
        }
        map
    }

    /// Create a word-level constituent.
    fn make_word_constituent(
        &self,
        role: SyntacticRole,
        token_ids: &[usize],
        features: HashMap<String, String>,
    ) -> Constituent {
        Constituent {
            node_type: NodeType::Word,
            role,
            token_ids: token_ids.to_vec(),
            children: vec![],
            features,
            implicit: false,
        }
    }

    /// Create a partial (low-confidence) parse when full parsing fails.
    fn make_partial_parse(
        &self,
        token_ids: &[usize],
        token_info: &[(usize, Vec<TokenParseInfo>)],
    ) -> Constituent {
        let children: Vec<Constituent> = token_ids
            .iter()
            .enumerate()
            .map(|(i, &tid)| {
                let pos = token_info
                    .get(i)
                    .and_then(|(_, infos)| infos.first())
                    .map(|info| info.pos)
                    .unwrap_or(PartOfSpeech::Unknown);

                let role = match pos {
                    PartOfSpeech::Verb => SyntacticRole::FiL,
                    PartOfSpeech::Particle | PartOfSpeech::Preposition => SyntacticRole::HarfJarr,
                    PartOfSpeech::Conjunction => SyntacticRole::Idafa,
                    _ => SyntacticRole::Unknown,
                };

                self.make_word_constituent(role, &[tid], HashMap::new())
            })
            .collect();

        Constituent {
            node_type: NodeType::Clause,
            role: SyntacticRole::Unknown,
            token_ids: token_ids.to_vec(),
            children,
            features: HashMap::from([("partial".to_string(), "true".to_string())]),
            implicit: false,
        }
    }
}

// ──────────────────────────────────────────────
//  Token Parse Info (internal helper)
// ──────────────────────────────────────────────

/// Lightweight parse information extracted from StemAnalysis.
#[derive(Debug, Clone)]
struct TokenParseInfo {
    pos: PartOfSpeech,
    stem: String,
    root: Option<String>,
    #[allow(dead_code)]
    features: Vec<String>,
}

// ──────────────────────────────────────────────
//  PipelineStage Implementation
// ──────────────────────────────────────────────

impl PipelineStage<MorphologicalAnalysis, SyntaxTree> for SyntaxParser {
    fn stage_id(&self) -> &'static str {
        "MOD-05"
    }

    fn process(
        &self,
        input: MorphologicalAnalysis,
        _ctx: &PipelineContext,
    ) -> PipelineResult<SyntaxTree> {
        self.parse(input)
    }

    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        if self.config.max_parse_trees == 0 {
            return Err(agos_core::error::PipelineError::fatal(
                agos_core::error::codes::INVALID_REQUEST,
                "SyntaxParserConfig.max_parse_trees must be > 0",
                "MOD-05",
            ));
        }
        Ok(())
    }
}

impl Default for SyntaxParser {
    fn default() -> Self {
        Self::new(SyntaxParserConfig::default())
    }
}

// ═══════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use agos_core::types::GrammarSchool;
    use agos_core::ir::{RootRef, StemAnalysis};

    /// Helper: create a minimal StemAnalysis.
    fn make_stem(pos: PartOfSpeech, stem_text: &str, root: Option<&str>) -> StemAnalysis {
        StemAnalysis {
            analysis_id: format!("test-{}-{}", stem_text, pos.code()),
            segmentation_id: "seg-default".to_string(),
            stem: stem_text.to_string(),
            root: root.map(|r| RootRef {
                text: r.to_string(),
                source: "test".to_string(),
                confidence: 0.8,
            }),
            wazan: None,
            pos,
            features: vec![],
            is_ambiguous: false,
            alternatives: vec![],
            evidence: vec![],
        }
    }

    /// Helper: create a MorphologicalAnalysis from POS-tagged tokens.
    fn make_morphology(tokens: &[(usize, PartOfSpeech, &str)]) -> MorphologicalAnalysis {
        let token_analyses: Vec<TokenAnalysis> = tokens
            .iter()
            .map(|&(id, pos, stem)| TokenAnalysis {
                token_id: id,
                stem_analyses: vec![make_stem(pos, stem, None)],
            })
            .collect();

        MorphologicalAnalysis {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            token_analyses,
            metadata: agos_core::ir::MorphologicalAnalysisMetadata {
                total_tokens: tokens.len() as u64,
                analyzed_tokens: tokens.len() as u64,
                ambiguous_tokens: 0,
                unknown_tokens: 0,
                unknown_stems: vec![],
            },
        }
    }

    fn test_ctx() -> PipelineContext {
        PipelineContext::new(GrammarSchool::Basra)
    }

    // ──────────────────────────────────────────
    //  Basic Tests
    // ──────────────────────────────────────────

    #[test]
    fn test_empty_input() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        let input = make_morphology(&[]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 0);
        assert!(output.trees.is_empty());
    }

    #[test]
    fn test_single_word_verb() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Single verb \"كتب\" (kataba = he wrote)
        let input = make_morphology(&[(0, PartOfSpeech::Verb, "كتب")]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        assert!(!output.trees.is_empty());
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahFiliyyah);
        // Check that the root has a verb child
        let has_fil = tree.root.children.iter().any(|c| c.role == SyntacticRole::FiL);
        assert!(has_fil, "Verbal sentence should have a Fi'l constituent");
    }

    #[test]
    fn test_verbal_sentence_verb_subject() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // \"كَتَبَ زَيْدٌ\" (kataba Zaydun = \"Zayd wrote\") — verb + subject
        let input = make_morphology(&[
            (0, PartOfSpeech::Verb, "كتب"),
            (1, PartOfSpeech::Noun, "زيد"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahFiliyyah);
        let has_fil = tree.root.children.iter().any(|c| c.role == SyntacticRole::FiL);
        let has_fail = tree.root.children.iter().any(|c| c.role == SyntacticRole::Fail);
        assert!(has_fil, "Should have Fi'l (verb)");
        assert!(has_fail, "Should have Fa'il (subject)");
    }

    #[test]
    fn test_nominal_sentence_mubtada_khabar() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // \"زَيْدٌ قَائِمٌ\" (Zaydun qa'imun = \"Zayd is standing\") — topic + comment
        let input = make_morphology(&[
            (0, PartOfSpeech::Noun, "زيد"),
            (1, PartOfSpeech::Adjective, "قائم"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahIsmiyyah);
        let has_mubtada = tree.root.children.iter().any(|c| c.role == SyntacticRole::Mubtada);
        let has_khabar = tree.root.children.iter().any(|c| {
            c.role == SyntacticRole::Khabar || c.role == SyntacticRole::Majrur
        });
        assert!(has_mubtada, "Should have Mubtada' (topic)");
        assert!(has_khabar, "Should have Khabar (comment)");
    }

    // ──────────────────────────────────────────
    //  Pipeline Stage Tests
    // ──────────────────────────────────────────

    #[test]
    fn test_stage_id() {
        let parser = SyntaxParser::default();
        assert_eq!(parser.stage_id(), "MOD-05");
    }

    #[test]
    fn test_validate_config() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        let result = parser.validate_config(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_rejects_zero_trees() {
        let mut config = SyntaxParserConfig::default();
        config.max_parse_trees = 0;
        let parser = SyntaxParser::new(config);
        let ctx = test_ctx();
        let result = parser.validate_config(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_spec_fields() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        let input = make_morphology(&[(0, PartOfSpeech::Verb, "كتب")]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.spec, "SPEC-0001");
        assert_eq!(output.version, "1.0");
    }

    // ──────────────────────────────────────────
    //  Sentence Segmentation Tests
    // ──────────────────────────────────────────

    #[test]
    fn test_multi_sentence_segmentation() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Two verb-only \"sentences\" (would be separate sentences with punctuation,
        // but MOD-04 skips punctuation tokens, so we just have two verbs)
        let input = make_morphology(&[
            (0, PartOfSpeech::Verb, "كتب"),
            (2, PartOfSpeech::Verb, "قرأ"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should treat as one combined sentence since no boundary markers
        assert_eq!(output.metadata.sentence_count, 1);
    }

    // ──────────────────────────────────────────
    //  Verb-Object Detection Tests
    // ──────────────────────────────────────────

    #[test]
    fn test_verb_subject_object() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // \"ضَرَبَ زَيْدٌ عَمْرًا\" (daraba Zaydun 'Amran = \"Zayd hit 'Amr\")
        let input = make_morphology(&[
            (0, PartOfSpeech::Verb, "ضرب"),
            (1, PartOfSpeech::Noun, "زيد"),
            (2, PartOfSpeech::Noun, "عمرو"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let tree = &output.trees[0];
        let roles: Vec<SyntacticRole> = tree.root.children.iter().map(|c| c.role).collect();
        assert!(roles.contains(&SyntacticRole::FiL), "Should have verb");
        assert!(roles.contains(&SyntacticRole::Fail), "Should have subject");
        assert!(
            roles.contains(&SyntacticRole::MafulBiHi),
            "Should have object"
        );
    }

    // ──────────────────────────────────────────
    //  Pronoun as Subject Test
    // ──────────────────────────────────────────

    #[test]
    fn test_pronoun_as_subject() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // \"هُوَ كَتَبَ\" (huwa kataba = \"he wrote\") — pronoun + verb
        let input = make_morphology(&[
            (0, PartOfSpeech::Pronoun, "هو"),
            (1, PartOfSpeech::Verb, "كتب"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        // Starts with pronoun → parsed as nominal (ismiyyah)
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahIsmiyyah);
    }

    // ──────────────────────────────────────────
    //  Preposition Handling Test
    // ──────────────────────────────────────────

    #[test]
    fn test_preposition_in_khabar() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // \"زَيْدٌ فِي الدَّارِ\" (Zaydun fi al-dari = \"Zayd is in the house\")
        // We use Particle for preposition since the fast-path classifies
        // prepositions as Particles
        let input = make_morphology(&[
            (0, PartOfSpeech::Noun, "زيد"),
            (1, PartOfSpeech::Particle, "في"),
            (2, PartOfSpeech::Noun, "الدار"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let tree = &output.trees[0];
        // Should be nominal (ismiyyah)
        assert_eq!(tree.tree_type, SentenceType::JumlahIsmiyyah);
    }

    // ──────────────────────────────────────────
    //  Edge Case: Single Noun
    // ──────────────────────────────────────────

    #[test]
    fn test_single_noun() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Single noun \"زيد\" (Zayd)
        let input = make_morphology(&[(0, PartOfSpeech::Noun, "زيد")]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahIsmiyyah);
        // Should have mubtada'
        let has_mubtada = tree.root.children.iter().any(|c| c.role == SyntacticRole::Mubtada);
        assert!(has_mubtada, "Single noun should be parsed as mubtada'");
    }

    // ──────────────────────────────────────────
    //  Proper Noun Test
    // ──────────────────────────────────────────

    #[test]
    fn test_proper_noun_as_subject() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Verb + proper noun
        let input = make_morphology(&[
            (0, PartOfSpeech::Verb, "قال"),
            (1, PartOfSpeech::ProperNoun, "الله"),
        ]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahFiliyyah);
    }

    // ──────────────────────────────────────────
    //  Ambiguity Test (multiple morph analyses)
    // ──────────────────────────────────────────

    #[test]
    fn test_morphological_ambiguity() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Token with multiple morphological analyses: verb + noun alternative
        let token_analyses = vec![TokenAnalysis {
            token_id: 0,
            stem_analyses: vec![
                make_stem(PartOfSpeech::Verb, "كتب", Some("كتب")),
                make_stem(PartOfSpeech::Noun, "كتب", Some("كتب")),
            ],
        }];
        let input = MorphologicalAnalysis {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            token_analyses,
            metadata: agos_core::ir::MorphologicalAnalysisMetadata {
                total_tokens: 1,
                analyzed_tokens: 1,
                ambiguous_tokens: 1,
                unknown_tokens: 0,
                unknown_stems: vec![],
            },
        };
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        // Should parse as verbal (verb analysis comes first)
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahFiliyyah);
    }

    // ──────────────────────────────────────────
    //  Multiple Tokens Test
    // ──────────────────────────────────────────

    #[test]
    fn test_many_tokens() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // A longer sequence
        let tokens: Vec<(usize, PartOfSpeech, &str)> = vec![
            (0, PartOfSpeech::Verb, "كتب"),
            (1, PartOfSpeech::Noun, "محمد"),
            (2, PartOfSpeech::Particle, "في"),
            (3, PartOfSpeech::Noun, "المدرسة"),
        ];
        let input = make_morphology(&tokens);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
        let tree = &output.trees[0];
        assert_eq!(tree.tree_type, SentenceType::JumlahFiliyyah);
        // Should have verb, subject, and khabar-like structure
        let children_count = tree.root.children.len();
        assert!(
            children_count >= 2,
            "Verb+subject sentence should have at least 2 children, got {children_count}"
        );
    }

    // ──────────────────────────────────────────
    //  Sentence Type Detection Tests
    // ──────────────────────────────────────────

    #[test]
    fn test_unknown_pos_parses_as_fallback() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        // Unknown POS should still parse with low confidence
        let input = make_morphology(&[(0, PartOfSpeech::Unknown, "???")]);
        let result = parser.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.sentence_count, 1);
    }

    // ──────────────────────────────────────────
    //  Determinism Test
    // ──────────────────────────────────────────

    #[test]
    fn test_determinism() {
        let parser = SyntaxParser::default();
        let ctx = test_ctx();
        let input = make_morphology(&[
            (0, PartOfSpeech::Verb, "كتب"),
            (1, PartOfSpeech::Noun, "زيد"),
        ]);
        let output1 = parser.process(input.clone(), &ctx).unwrap();
        let output2 = parser.process(input, &ctx).unwrap();
        assert_eq!(output1.trees.len(), output2.trees.len());
        for (t1, t2) in output1.trees.iter().zip(&output2.trees) {
            assert_eq!(t1.tree_type, t2.tree_type);
            assert!(
                (t1.confidence - t2.confidence).abs() < 0.001,
                "Confidence should be identical"
            );
        }
    }
}
