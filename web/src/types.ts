export interface AnalyzeRequest {
  text: string
  school?: School
  strip_tashkeel?: boolean
  strip_tatweel?: boolean
}

export type School = 'Basra' | 'Kufa' | 'Baghdad' | 'Andalus' | 'Modern'

export interface AnalyzeResponse {
  success: boolean
  error: string | null
  timing_ms: Record<string, number>
  stages: StageOutputs
}

export interface StageOutputs {
  normalized: NormalizedOutput | null
  tokens: TokenOutput | null
  segmented: SegmentedOutput | null
  morphology: MorphologicalAnalysis | null
  syntax: SyntaxTree | null
}

export interface NormalizedOutput {
  normalized_text: string
  char_count: number
  word_count_estimate: number
  has_tashkeel: boolean
  has_tatweel: boolean
  has_non_arabic: boolean
}

export interface TokenOutput {
  token_count: number
  word_count: number
  tokens: RawTokenSummary[]
}

export interface RawTokenSummary {
  id: number
  text: string
  token_type: string
}

export interface SegmentedOutput {
  total_tokens: number
  segmentable_tokens: number
  ambiguous_tokens: number
}

export interface MorphologicalAnalysis {
  spec: string
  version: string
  token_analyses: TokenAnalysis[]
  metadata: MorphologicalAnalysisMetadata
}

export interface TokenAnalysis {
  token_id: number
  stem_analyses: StemAnalysis[]
}

export interface StemAnalysis {
  analysis_id: string
  segmentation_id: string
  stem: string
  root: RootRef | null
  wazan: WazanRef | null
  pos: string
  features: NamedFeature[]
  is_ambiguous: boolean
  alternatives: StemAnalysis[]
  evidence: any[]
}

export interface RootRef {
  text: string
  source: string
  confidence: number
}

export interface WazanRef {
  text: string
  source: string
  form: number | null
  confidence: number
}

export interface NamedFeature {
  name: string
  value: string
  category: string
  confidence: number
  source: string
}

export interface MorphologicalAnalysisMetadata {
  total_tokens: number
  analyzed_tokens: number
  ambiguous_tokens: number
  unknown_tokens: number
  unknown_stems: string[]
}

export interface SyntaxTree {
  spec: string
  version: string
  trees: ParseTree[]
  metadata: SyntaxTreeMetadata
}

export interface ParseTree {
  id: string
  tree_type: string
  root: Constituent
  confidence: number
  source: string
}

export interface Constituent {
  node_type: string
  role: string
  token_ids: number[]
  children: Constituent[]
  features: Record<string, string>
  implicit: boolean
}

export interface SyntaxTreeMetadata {
  sentence_count: number
  tokens_parsed: number
  ambiguity_count: number
  parse_time_ms: number
}
