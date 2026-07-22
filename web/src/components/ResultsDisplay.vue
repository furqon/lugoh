<script setup lang="ts">
import { ref, computed } from 'vue'
import type { AnalyzeResponse } from '../types'
import MorphologyView from './MorphologyView.vue'
import SyntaxTreeView from './SyntaxTreeView.vue'

const props = defineProps<{ result: AnalyzeResponse }>()

const activeTab = ref<'morphology' | 'syntax'>('morphology')

const totalTime = computed(() => {
  const ms = Object.values(props.result.timing_ms)
  return ms.reduce((a, b) => a + b, 0).toFixed(2)
})

const stages = computed(() => [
  {
    id: 'normalized',
    label: 'MOD-01 Unicode Validation',
    time: props.result.timing_ms['MOD-01'],
    icon: '🔤',
    active: !!props.result.stages.normalized,
  },
  {
    id: 'tokens',
    label: 'MOD-02 Lexer',
    time: props.result.timing_ms['MOD-02'],
    icon: '📝',
    active: !!props.result.stages.tokens,
  },
  {
    id: 'segmented',
    label: 'MOD-03 Tokenizer',
    time: props.result.timing_ms['MOD-03'],
    icon: '🔗',
    active: !!props.result.stages.segmented,
  },
  {
    id: 'morphology',
    label: 'MOD-04 Morphology',
    time: props.result.timing_ms['MOD-04'],
    icon: '📊',
    active: !!props.result.stages.morphology,
  },
  {
    id: 'syntax',
    label: 'MOD-05 Syntax',
    time: props.result.timing_ms['MOD-05'],
    icon: '🌳',
    active: !!props.result.stages.syntax,
  },
])

const n = props.result.stages.normalized
const t = props.result.stages.tokens
const s = props.result.stages.segmented
const m = props.result.stages.morphology
</script>

<template>
  <div class="space-y-4">
    <!-- Summary Bar -->
    <div class="card">
      <div class="card-body flex flex-wrap items-center gap-4 text-sm">
        <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
          <span>{{ totalTime }}ms</span>
        </div>
        <div v-if="m" class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
          <span>{{ m.metadata.analyzed_tokens }} tokens analyzed</span>
        </div>
        <div v-if="m && m.metadata.ambiguous_tokens > 0" class="flex items-center gap-2 text-amber-600 dark:text-amber-400">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"/></svg>
          <span>{{ m.metadata.ambiguous_tokens }} ambiguous</span>
        </div>
        <div v-if="m && m.metadata.unknown_tokens > 0" class="flex items-center gap-2 text-red-500">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01"/></svg>
          <span>{{ m.metadata.unknown_tokens }} unknown</span>
        </div>
      </div>
    </div>

    <!-- Stage Timing Accordion -->
    <div class="card overflow-hidden">
      <div class="divide-y divide-slate-100 dark:divide-slate-800">
        <div v-for="stage in stages" :key="stage.id" class="px-5 py-3 flex items-center justify-between text-sm">
          <div class="flex items-center gap-3">
            <span class="text-base">{{ stage.icon }}</span>
            <span :class="stage.active ? 'text-slate-900 dark:text-slate-100' : 'text-slate-400 dark:text-slate-600'">
              {{ stage.label }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span v-if="stage.time !== undefined" class="font-mono text-xs text-slate-500 dark:text-slate-400">
              {{ stage.time.toFixed(2) }}ms
            </span>
            <span v-if="stage.active" class="w-2 h-2 rounded-full bg-emerald-500"></span>
            <span v-else class="w-2 h-2 rounded-full bg-slate-300 dark:bg-slate-700"></span>
          </div>
        </div>
      </div>
    </div>

    <!-- Normalized Text Preview -->
    <div v-if="n" class="card">
      <div class="card-header">
        <h3 class="font-medium text-sm text-slate-700 dark:text-slate-300">Normalized Text</h3>
      </div>
      <div class="card-body">
        <div class="font-arabic text-lg leading-relaxed bg-slate-50 dark:bg-slate-800/50 rounded-xl p-4">
          {{ n.normalized_text }}
        </div>
        <div class="flex flex-wrap gap-3 mt-3 text-xs text-slate-400 dark:text-slate-500">
          <span>{{ n.char_count }} chars</span>
          <span>{{ n.word_count_estimate }} word estimate</span>
          <span v-if="n.has_tashkeel">Has tashkeel</span>
          <span v-if="n.has_tatweel">Has tatweel</span>
          <span v-if="n.has_non_arabic">Has non-Arabic</span>
        </div>
      </div>
    </div>

    <!-- Token Details (collapsible) -->
    <div v-if="t" class="card">
      <div class="card-header">
        <h3 class="font-medium text-sm text-slate-700 dark:text-slate-300">Tokens ({{ t.token_count }})</h3>
      </div>
      <div class="card-body">
        <div class="flex flex-wrap gap-2">
          <div
            v-for="token in t.tokens"
            :key="token.id"
            class="px-3 py-1.5 rounded-lg text-sm border"
            :class="token.token_type === 'Word'
              ? 'bg-agos-50 dark:bg-agos-950/30 border-agos-200 dark:border-agos-800 font-arabic'
              : token.token_type === 'Whitespace'
              ? 'bg-slate-100 dark:bg-slate-800 border-slate-200 dark:border-slate-700 text-slate-400 dark:text-slate-600'
              : 'bg-amber-50 dark:bg-amber-950/30 border-amber-200 dark:border-amber-800'"
          >
            <span class="font-medium">{{ token.text === ' ' ? '␣' : token.text }}</span>
            <span class="text-xs ml-1.5 opacity-60">{{ token.token_type }}</span>
          </div>
        </div>
        <div v-if="s" class="mt-3 flex gap-3 text-xs text-slate-400 dark:text-slate-500">
          <span>{{ s.segmentable_tokens }} segmentable</span>
          <span>{{ s.ambiguous_tokens }} ambiguous segmentations</span>
        </div>
      </div>
    </div>

    <!-- Morphology / Syntax Tabs -->
    <div v-if="m" class="card overflow-hidden">
      <div class="flex border-b border-slate-200 dark:border-slate-800">
        <button
          @click="activeTab = 'morphology'"
          class="flex-1 px-5 py-3 text-sm font-medium transition-colors"
          :class="activeTab === 'morphology'
            ? 'text-agos-600 border-b-2 border-agos-600 bg-agos-50/50 dark:bg-agos-950/20'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'"
        >
          📊 Morphological Analysis
        </button>
        <button
          @click="activeTab = 'syntax'"
          class="flex-1 px-5 py-3 text-sm font-medium transition-colors"
          :class="activeTab === 'syntax'
            ? 'text-agos-600 border-b-2 border-agos-600 bg-agos-50/50 dark:bg-agos-950/20'
            : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'"
        >
          🌳 Syntax Trees
        </button>
      </div>

      <div class="card-body">
        <MorphologyView v-if="activeTab === 'morphology'" :morphology="m" />
        <SyntaxTreeView v-else :syntax="result.stages.syntax" />
      </div>
    </div>
  </div>
</template>
