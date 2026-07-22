<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  analyze: [text: string, school: string, stripTashkeel: boolean, stripTatweel: boolean]
}>()

defineProps<{ loading: boolean }>()

const text = ref('')
const school = ref('Basra')
const stripTashkeel = ref(false)
const stripTatweel = ref(true)

const exampleTexts = [
  'السَّلَامُ عَلَيْكُمْ وَرَحْمَةُ اللَّهِ وَبَرَكَاتُهُ',
  'كتب زيد رسالة',
  'الرجل كبير جداً',
  'محمد كريم',
]

function setExample(ex: string) {
  text.value = ex
}

function submit() {
  if (!text.value.trim()) return
  emit('analyze', text.value, school.value, stripTashkeel.value, stripTatweel.value)
}
</script>

<template>
  <div class="card">
    <div class="card-body space-y-4">
      <!-- Text Input -->
      <div>
        <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1.5">
          Arabic Text
        </label>
        <textarea
          v-model="text"
          rows="4"
          class="input-field font-arabic text-lg leading-relaxed resize-y min-h-[120px]"
          placeholder="اكتب النص العربي هنا..."
          dir="rtl"
          @keydown.ctrl.enter="submit"
        ></textarea>
      </div>

      <!-- Examples -->
      <div class="flex flex-wrap gap-1.5">
        <span class="text-xs text-slate-400 dark:text-slate-500 self-center ml-1">Examples:</span>
        <button
          v-for="ex in exampleTexts"
          :key="ex"
          @click="setExample(ex)"
          class="text-xs px-2.5 py-1 rounded-lg border border-slate-200 dark:border-slate-700
                 hover:border-agos-300 dark:hover:border-agos-700 hover:bg-agos-50 dark:hover:bg-agos-950/30
                 transition-colors font-arabic text-slate-600 dark:text-slate-400"
        >
          {{ ex.slice(0, 20) }}{{ ex.length > 20 ? '...' : '' }}
        </button>
      </div>

      <!-- Options Row -->
      <div class="flex flex-wrap items-center gap-4">
        <!-- School Select -->
        <div class="flex items-center gap-2">
          <label class="text-xs font-medium text-slate-500 dark:text-slate-400">School:</label>
          <select v-model="school" class="input-field text-xs py-1.5 px-2.5 w-auto">
            <option value="Basra">Basra</option>
            <option value="Kufa">Kufa</option>
            <option value="Baghdad">Baghdad</option>
            <option value="Andalus">Andalus</option>
            <option value="Modern">Modern</option>
          </select>
        </div>

        <!-- Toggles -->
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="stripTashkeel" class="rounded border-slate-300 dark:border-slate-700 text-agos-600 focus:ring-agos-500" />
          <span class="text-xs text-slate-500 dark:text-slate-400">Strip Tashkeel</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" v-model="stripTatweel" class="rounded border-slate-300 dark:border-slate-700 text-agos-600 focus:ring-agos-500" />
          <span class="text-xs text-slate-500 dark:text-slate-400">Strip Tatweel</span>
        </label>
      </div>

      <!-- Submit -->
      <div class="flex items-center gap-3 pt-1">
        <button
          @click="submit"
          :disabled="loading || !text.trim()"
          class="btn-primary"
        >
          <svg v-if="loading" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
          </svg>
          <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
          </svg>
          {{ loading ? 'Analyzing...' : 'Analyze' }}
        </button>
        <span v-if="loading" class="text-xs text-slate-400 dark:text-slate-500">
          Processing through 5 pipeline stages...
        </span>
      </div>
    </div>
  </div>
</template>
