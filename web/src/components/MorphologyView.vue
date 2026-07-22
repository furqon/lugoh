<script setup lang="ts">
import type { MorphologicalAnalysis, StemAnalysis } from '../types'

const props = defineProps<{ morphology: MorphologicalAnalysis }>()

function posBadge(pos: string): string {
  if (pos === 'Verb') return 'badge-verb'
  if (pos === 'Noun') return 'badge-noun'
  if (pos === 'Adjective') return 'badge-adjective'
  if (pos === 'Particle') return 'badge-particle'
  if (pos === 'Pronoun') return 'badge-pronoun'
  return 'badge bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300'
}

function formatConfidence(c: number): string {
  return (c * 100).toFixed(0) + '%'
}

const featureIcons: Record<string, string> = {
  tense: '⏱',
  person: '👤',
  gender: '⚤',
  number: '#️⃣',
  verb_form: '📋',
  mood: '🎭',
  voice: '🔊',
  case: '📎',
  state: '🔘',
  pos: '🏷️',
}
</script>

<template>
  <div class="space-y-6">
    <div
      v-for="ta in morphology.token_analyses"
      :key="ta.token_id"
      class="space-y-2"
    >
      <!-- Token Header -->
      <div class="flex items-center gap-3">
        <span class="text-xs font-mono text-slate-400 dark:text-slate-600 w-6">#{{ ta.token_id }}</span>
        <div
          v-for="sa in ta.stem_analyses.slice(0, 3)"
          :key="sa.analysis_id"
          class="flex-1"
        >
          <div class="flex items-center gap-2 flex-wrap">
            <!-- Stem -->
            <span class="font-arabic text-xl font-semibold">{{ sa.stem }}</span>
            <!-- POS Badge -->
            <span :class="posBadge(sa.pos)">{{ sa.pos }}</span>
            <!-- Root -->
            <span v-if="sa.root" class="text-xs font-mono text-slate-500 dark:text-slate-400">
              Root: <span class="font-arabic font-medium">{{ sa.root.text }}</span>
              <span class="opacity-60"> ({{ formatConfidence(sa.root.confidence) }})</span>
            </span>
            <!-- Wazan -->
            <span v-if="sa.wazan" class="text-xs font-mono text-slate-500 dark:text-slate-400">
              Wazan: <span class="font-arabic font-medium">{{ sa.wazan.text }}</span>
              <template v-if="sa.wazan.form"> Form {{ sa.wazan.form }}</template>
              <span class="opacity-60"> ({{ formatConfidence(sa.wazan.confidence) }})</span>
            </span>
          </div>

          <!-- Features Table -->
          <div v-if="sa.features.length > 0" class="mt-2 flex flex-wrap gap-1.5">
            <div
              v-for="f in sa.features"
              :key="f.name"
              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs
                     bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700"
              :title="`${f.name}: ${f.value} (${f.source}, ${formatConfidence(f.confidence)})`"
            >
              <span class="text-[10px]">{{ featureIcons[f.name] || '•' }}</span>
              <span class="text-slate-500 dark:text-slate-400">{{ f.name }}:</span>
              <span class="font-medium text-slate-700 dark:text-slate-300">{{ f.value }}</span>
            </div>
          </div>

          <!-- Alternatives -->
          <div v-if="ta.stem_analyses.length > 1" class="mt-1">
            <span class="text-xs text-amber-600 dark:text-amber-400">
              +{{ ta.stem_analyses.length - 1 }} alternative analysis{{ ta.stem_analyses.length > 2 ? 'es' : '' }}
            </span>
          </div>

          <!-- Separator between analyses -->
          <hr v-if="ta.stem_analyses.length > 1" class="my-2 border-slate-100 dark:border-slate-800" />
        </div>
      </div>
    </div>

    <!-- Empty / Unknown -->
    <div v-if="morphology.token_analyses.length === 0" class="text-center py-8 text-sm text-slate-400 dark:text-slate-600">
      No word tokens to analyze.
    </div>

    <div v-if="morphology.metadata.unknown_stems.length > 0" class="mt-4 p-3 rounded-xl bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800">
      <p class="text-xs font-medium text-amber-800 dark:text-amber-300">
        ⚠️ {{ morphology.metadata.unknown_stems.length }} unknown stem{{ morphology.metadata.unknown_stems.length > 1 ? 's' : '' }}
      </p>
      <div class="flex flex-wrap gap-1.5 mt-1">
        <span v-for="stem in morphology.metadata.unknown_stems" :key="stem" class="font-arabic text-sm text-amber-700 dark:text-amber-400">
          {{ stem }}
        </span>
      </div>
    </div>
  </div>
</template>
