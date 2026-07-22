<script setup lang="ts">
import type { SyntaxTree } from '../types'

defineProps<{ syntax: SyntaxTree | null }>()

const roleColors: Record<string, string> = {
  FiL: 'text-emerald-600 dark:text-emerald-400',
  Fail: 'text-blue-600 dark:text-blue-400',
  Mubtada: 'text-violet-600 dark:text-violet-400',
  Khabar: 'text-rose-600 dark:text-rose-400',
  MafUl: 'text-amber-600 dark:text-amber-400',
  Majrur: 'text-cyan-600 dark:text-cyan-400',
  Mudaf: 'text-indigo-600 dark:text-indigo-400',
  MudafIlayh: 'text-orange-600 dark:text-orange-400',
  NaAt: 'text-pink-600 dark:text-pink-400',
  HarfJarr: 'text-slate-600 dark:text-slate-400',
}


const typeColors: Record<string, string> = {
  JumlahFiliyyah: 'bg-emerald-100 dark:bg-emerald-900/30 border-emerald-300 dark:border-emerald-700',
  JumlahIsmiyyah: 'bg-violet-100 dark:bg-violet-900/30 border-violet-300 dark:border-violet-700',
  JumlahShartiyyah: 'bg-amber-100 dark:bg-amber-900/30 border-amber-300 dark:border-amber-700',
}

</script>

<template>
  <div class="space-y-4">
    <div v-if="!syntax || syntax.trees.length === 0" class="text-center py-8 text-sm text-slate-400 dark:text-slate-600">
      No syntax trees produced.
    </div>

    <div v-for="tree in syntax?.trees || []" :key="tree.id" class="space-y-2">
      <!-- Tree Header -->
      <div class="flex items-center gap-3 flex-wrap">
        <span
          class="px-3 py-1 rounded-lg text-xs font-medium border"
          :class="typeColors[tree.tree_type] || 'bg-slate-100 dark:bg-slate-800 border-slate-200 dark:border-slate-700'"
        >
          {{ tree.tree_type }}
        </span>
        <span class="text-xs text-slate-400 dark:text-slate-500 font-mono">
          {{ (tree.confidence * 100).toFixed(0) }}% confidence
        </span>
        <span class="text-xs text-slate-400 dark:text-slate-500">
          {{ tree.source }}
        </span>
      </div>

      <!-- Tree Rendering -->
      <div class="bg-slate-50 dark:bg-slate-800/30 rounded-xl p-4 font-mono text-xs leading-relaxed overflow-x-auto">
        <div class="tree-container">
          <!-- Root Node -->
          <div class="flex flex-col items-start gap-1">
            <div class="flex items-center gap-2 px-2 py-1 rounded-lg bg-slate-100 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700">
              <span class="font-medium text-slate-700 dark:text-slate-300">{{ tree.root.role }}</span>
              <span class="text-[10px] text-slate-400">{{ tree.root.node_type }}</span>
              <span v-if="tree.root.token_ids.length > 0" class="text-[10px] text-agos-600 font-mono">
                [{{ tree.root.token_ids.join(', ') }}]
              </span>
            </div>

            <!-- Recursive Children -->
            <div class="ml-4 border-r-2 border-slate-300 dark:border-slate-700 pl-4 space-y-1">
              <div v-for="(child, ci) in tree.root.children" :key="ci" class="flex flex-col gap-1">
                <div class="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800/30 transition-colors">
                  <span class="w-2 h-2 rounded-full bg-slate-300 dark:bg-slate-600 shrink-0"></span>
                  <span :class="roleColors[child.role] || 'text-slate-600 dark:text-slate-400'" class="font-medium text-xs">
                    {{ child.role }}
                  </span>
                  <span class="text-[10px] text-slate-400">{{ child.node_type }}</span>
                  <span v-if="child.token_ids.length > 0" class="text-[10px] font-mono text-agos-600">
                    [{{ child.token_ids.join(', ') }}]
                  </span>
                  <span v-if="Object.keys(child.features).length > 0" class="text-[10px] text-slate-400 dark:text-slate-500">
                    {{ JSON.stringify(child.features) }}
                  </span>
                </div>

                <!-- Grandchildren (if any) -->
                <div v-if="child.children.length > 0" class="ml-6 border-r-2 border-slate-200 dark:border-slate-700/50 pl-4 space-y-1">
                  <div v-for="(gc, gci) in child.children" :key="gci" class="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800/30 transition-colors">
                    <span class="w-1.5 h-1.5 rounded-full bg-slate-300 dark:bg-slate-600 shrink-0"></span>
                    <span :class="roleColors[gc.role] || 'text-slate-600 dark:text-slate-400'" class="font-medium text-xs">
                      {{ gc.role }}
                    </span>
                    <span class="text-[10px] text-slate-400">{{ gc.node_type }}</span>
                    <span v-if="gc.token_ids.length > 0" class="text-[10px] font-mono text-agos-600">
                      [{{ gc.token_ids.join(', ') }}]
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Syntax Metadata -->
    <div v-if="syntax" class="flex flex-wrap gap-3 text-xs text-slate-400 dark:text-slate-500 pt-1">
      <span>{{ syntax.metadata.sentence_count }} sentence{{ syntax.metadata.sentence_count > 1 ? 's' : '' }}</span>
      <span>{{ syntax.metadata.tokens_parsed }} tokens parsed</span>
      <span v-if="syntax.metadata.ambiguity_count > 0">{{ syntax.metadata.ambiguity_count }} ambiguous parses</span>
      <span>{{ syntax.metadata.parse_time_ms.toFixed(2) }}ms parse time</span>
    </div>
  </div>
</template>
