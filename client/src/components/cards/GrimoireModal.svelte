<script lang="ts">
  import { isGrimoireOpen, graveyard } from '../../lib/stores/cardStore';
  import { X, BookOpen, Clock, Cpu, CheckCircle2, AlertTriangle, FileText, ArrowUpRight } from '@lucide/svelte';

  function close() {
    isGrimoireOpen.set(false);
  }
</script>

{#if $isGrimoireOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 font-mono select-none">
    <div class="w-full max-w-3xl bg-slate-950 border-2 border-amber-500/50 rounded-2xl shadow-[0_0_50px_rgba(245,158,11,0.25)] flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header do Grimório -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/80">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-amber-950 border border-amber-400 flex items-center justify-center text-amber-300 shadow-[0_0_12px_rgba(245,158,11,0.4)]">
            <BookOpen class="w-5 h-5" />
          </div>
          <div>
            <div class="text-sm font-black text-amber-200 uppercase tracking-widest flex items-center gap-2">
              GRIMÓRIO DO ARQUITETO
              <span class="text-[10px] px-2 py-0.5 rounded font-bold bg-amber-950/80 border border-amber-400/40 text-amber-300">
                {$graveyard.length} CARTAS REGISTRADAS
              </span>
            </div>
            <div class="text-[11px] text-slate-400">HISTÓRICO DE DIRETRIZES CONJURADAS & RESOLUÇÕES</div>
          </div>
        </div>

        <button on:click={close} class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Lista de Cartas Jogadas -->
      <div class="p-6 overflow-y-auto space-y-4 text-xs">
        {#if $graveyard.length === 0}
          <div class="flex flex-col items-center justify-center py-16 text-slate-500 space-y-2">
            <BookOpen class="w-10 h-10 opacity-30 text-amber-400" />
            <div class="text-sm font-bold">Nenhuma carta conjurada neste ciclo.</div>
            <div class="text-xs text-slate-600">Jogue cartas da sua mão para iniciar tarefas com os agentes.</div>
          </div>
        {:else}
          {#each $graveyard as record (record.recordId)}
            <div class="p-4 rounded-xl bg-slate-900/80 border border-slate-800 hover:border-slate-700 transition space-y-3">
              <!-- Top line: Status, Título e Agente -->
              <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800/80 pb-2.5">
                <div class="flex items-center gap-2">
                  <span class="px-2 py-0.5 rounded text-[10px] font-black uppercase flex items-center gap-1
                    {record.status === 'RESOLVED' ? 'bg-emerald-950 border border-emerald-500/50 text-emerald-300' :
                     record.status === 'EXECUTING' ? 'bg-amber-950 border border-amber-500/50 text-amber-300 animate-pulse' :
                     record.status === 'FAILED' ? 'bg-rose-950 border border-rose-500/50 text-rose-300' :
                     'bg-cyan-950 border border-cyan-500/50 text-cyan-300'}">
                    {#if record.status === 'RESOLVED'}
                      <CheckCircle2 class="w-3 h-3" /> CONCLUÍDA
                    {:else if record.status === 'EXECUTING'}
                      <Clock class="w-3 h-3" /> EM EXECUÇÃO
                    {:else}
                      <AlertTriangle class="w-3 h-3" /> ERRO
                    {/if}
                  </span>

                  <span class="text-xs font-black text-slate-200">{record.card.title}</span>
                  <span class="text-[10px] text-slate-500 font-mono">({record.taskId})</span>
                </div>

                <div class="flex items-center gap-3 text-[11px] text-slate-400">
                  <span class="flex items-center gap-1 text-cyan-400 font-bold">
                    <Cpu class="w-3.5 h-3.5" /> {record.assignedAgentName}
                  </span>
                  <span>{record.playedAt}</span>
                </div>
              </div>

              <!-- Snippet do Raciocínio (Chain-of-Thought) -->
              {#if record.reasoningSnippet}
                <div class="p-2.5 rounded-lg bg-black/50 border border-slate-800/80 text-[11px] text-slate-300 font-sans leading-relaxed">
                  <div class="text-[10px] font-bold uppercase text-amber-400 font-mono mb-1">
                    Trace de Raciocínio ({record.card.targetModel}):
                  </div>
                  {record.reasoningSnippet}
                </div>
              {/if}

              <!-- Artefatos e Tokens -->
              <div class="flex flex-wrap items-center justify-between gap-2 pt-1 text-[11px]">
                {#if record.artifactsCreated && record.artifactsCreated.length > 0}
                  <div class="flex items-center gap-1 text-slate-400">
                    <FileText class="w-3.5 h-3.5 text-emerald-400" />
                    <span>Artefatos:</span>
                    <span class="text-emerald-300 font-mono">{record.artifactsCreated.join(', ')}</span>
                  </div>
                {/if}

                {#if record.tokensUsed}
                  <div class="text-purple-400 font-mono text-[10px] font-bold">
                    {record.tokensUsed.toLocaleString()} tokens consumidos
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Rodapé -->
      <div class="px-6 py-3 border-t border-slate-800 bg-slate-900/90 flex justify-end">
        <button on:click={close} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold transition text-xs">
          Fechar Grimório
        </button>
      </div>
    </div>
  </div>
{/if}
