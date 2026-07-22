<script setup lang="ts">
import { ref } from 'vue'
import type { AnalyzeResponse } from './types'
import AnalyzeForm from './components/AnalyzeForm.vue'
import ResultsDisplay from './components/ResultsDisplay.vue'

const result = ref<AnalyzeResponse | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const darkMode = ref(
  window.matchMedia('(prefers-color-scheme: dark)').matches
)

function toggleDark() {
  darkMode.value = !darkMode.value
  document.documentElement.classList.toggle('dark')
}

async function onAnalyze(text: string, school: string, stripTashkeel: boolean, stripTatweel: boolean) {
  loading.value = true
  error.value = null
  result.value = null

  try {
    const { analyzeText } = await import('./api')
    const res = await analyzeText({
      text,
      school: school as any,
      strip_tashkeel: stripTashkeel,
      strip_tatweel: stripTatweel,
    })
    if (res.success) {
      result.value = res
    } else {
      error.value = res.error || 'Analysis failed'
    }
  } catch (e: any) {
    error.value = e.message || 'Network error — is the AGOS server running?'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex flex-col" :dir="'auto'">
    <!-- Header -->
    <header class="sticky top-0 z-50 backdrop-blur-xl bg-white/80 dark:bg-slate-950/80 border-b border-slate-200 dark:border-slate-800">
      <div class="max-w-5xl mx-auto px-4 h-16 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-agos-600 flex items-center justify-center text-white font-arabic text-xl font-bold shadow-lg shadow-agos-600/25">
            ا
          </div>
          <div>
            <h1 class="text-lg font-bold tracking-tight">AGOS</h1>
            <p class="text-xs text-slate-500 dark:text-slate-400 -mt-0.5">Arabic Grammar Operating System</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <a href="https://github.com/agos-org/agos" target="_blank" class="btn-secondary text-xs px-3 py-1.5" rel="noopener">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
            GitHub
          </a>
          <button @click="toggleDark" class="btn-secondary text-xs px-3 py-1.5" :title="darkMode ? 'Light mode' : 'Dark mode'">
            <svg v-if="!darkMode" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/></svg>
            <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"/></svg>
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 max-w-5xl mx-auto w-full px-4 py-8 space-y-8">
      <!-- Hero -->
      <div class="text-center space-y-3 pb-2">
        <h2 class="text-3xl sm:text-4xl font-bold tracking-tight">
          <span class="text-agos-600">Arabic</span> Morphology & Syntax
        </h2>
        <p class="text-slate-500 dark:text-slate-400 max-w-xl mx-auto text-sm sm:text-base">
          Enter Arabic text below for full morphological analysis — root extraction, 
          wazan identification, feature extraction, and syntactic parsing.
        </p>
      </div>

      <!-- Analyze Form -->
      <AnalyzeForm
        :loading="loading"
        @analyze="onAnalyze"
      />

      <!-- Error -->
      <div v-if="error" class="card border-red-200 dark:border-red-900/50 bg-red-50 dark:bg-red-950/30">
        <div class="card-body flex items-start gap-3">
          <svg class="w-5 h-5 text-red-500 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <div>
            <p class="font-medium text-red-800 dark:text-red-300">Analysis Error</p>
            <p class="text-sm text-red-600 dark:text-red-400 mt-1">{{ error }}</p>
          </div>
        </div>
      </div>

      <!-- Results -->
      <ResultsDisplay
        v-if="result"
        :result="result"
      />
    </main>

    <!-- Footer -->
    <footer class="border-t border-slate-200 dark:border-slate-800 py-6 text-center text-xs text-slate-400 dark:text-slate-600">
      <p>AGOS — Arabic Grammar Operating System &middot; Open Source &middot; MIT License</p>
    </footer>
  </div>
</template>
