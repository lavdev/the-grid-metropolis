<script lang="ts">
  import { selectedAgent } from '../lib/stores/gridStore';
  import { X, User, HardDrive, Cpu, Zap, Activity } from '@lucide/svelte';

  function close() {
    selectedAgent.set(null);
  }
</script>

{#if $selectedAgent}
  <aside class="fixed top-14 right-0 bottom-0 w-80 bg-slate-950/95 backdrop-blur-md border-l border-cyan-500/30 z-30 p-5 flex flex-col justify-between font-mono select-none">
    <div>
      <!-- Cabeçalho do Agente -->
      <div class="flex items-start justify-between border-b border-slate-800 pb-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-cyan-950 border border-cyan-500 flex items-center justify-center neon-glow-cyan text-cyan-400">
            <User class="w-6 h-6" />
          </div>
          <div>
            <div class="text-sm font-bold text-slate-100">{$selectedAgent.name}</div>
            <div class="text-[10px] text-cyan-400 tracking-wider font-semibold">{$selectedAgent.role}</div>
          </div>
        </div>
        <button on:click={close} class="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-white transition">
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Status Atual & Zona -->
      <div class="mt-4 space-y-3 text-xs">
        <div class="p-2.5 rounded bg-slate-900 border border-slate-800 flex justify-between items-center">
          <span class="text-slate-400 flex items-center gap-1.5"><Activity class="w-3.5 h-3.5" /> Estado:</span>
          <span class="px-2 py-0.5 rounded font-bold text-[10px]
            {$selectedAgent.status === 'WORKING' ? 'bg-amber-950 text-amber-400 border border-amber-500/50' :
             $selectedAgent.status === 'WALKING' ? 'bg-cyan-950 text-cyan-400 border border-cyan-500/50' :
             $selectedAgent.status === 'ERROR' ? 'bg-rose-950 text-rose-400 border border-rose-500/50' :
             'bg-slate-800 text-slate-300'}">
            {$selectedAgent.status}
          </span>
        </div>

        <div class="p-2.5 rounded bg-slate-900 border border-slate-800 flex justify-between items-center">
          <span class="text-slate-400 flex items-center gap-1.5"><HardDrive class="w-3.5 h-3.5" /> Base / Pod:</span>
          <span class="text-slate-200 font-bold truncate max-w-[140px]">{$selectedAgent.homePodId}</span>
        </div>

        <div class="p-2.5 rounded bg-slate-900 border border-slate-800 flex justify-between items-center">
          <span class="text-slate-400 flex items-center gap-1.5"><Cpu class="w-3.5 h-3.5" /> Zona Atual:</span>
          <span class="text-cyan-400 font-semibold">{$selectedAgent.currentLocation.currentZone}</span>
        </div>
      </div>

      <!-- Scratchpad / Chain-of-Thought -->
      <div class="mt-5">
        <div class="text-[11px] text-slate-400 font-bold tracking-wider uppercase mb-2 flex items-center gap-1.5">
          <Zap class="w-3.5 h-3.5 text-amber-400" /> Scratchpad (LLM Reasoning)
        </div>
        <div class="p-3 rounded bg-slate-900/90 border border-slate-800 text-[11px] text-slate-300 leading-relaxed font-sans max-h-48 overflow-y-auto">
          {#if $selectedAgent.activeTaskId}
            <div class="text-cyan-400 font-mono mb-1 font-semibold">Tarefa: {$selectedAgent.activeTaskId}</div>
            Processando contrato de tarefas... Deslocando pelas passarelas de néon até a estação do projeto para inicializar a compilação.
          {:else}
            <span class="text-slate-500 italic">Agente em repouso no The Battery Lounge. Aguardando novo despacho de contrato do Nexus.</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Métricas do Agente -->
    <div class="border-t border-slate-800 pt-4 text-[11px] space-y-2">
      <div class="flex justify-between text-slate-400">
        <span>Tarefas Concluídas:</span>
        <span class="text-emerald-400 font-bold">{$selectedAgent.metrics.tasksCompleted}</span>
      </div>
      <div class="flex justify-between text-slate-400">
        <span>Tokens Consumidos:</span>
        <span class="text-purple-400 font-bold">{$selectedAgent.metrics.tokensConsumed.toLocaleString()}</span>
      </div>
      <div class="flex justify-between text-slate-400">
        <span>ID Único:</span>
        <span class="text-slate-500 font-mono text-[9px]">{$selectedAgent.id}</span>
      </div>
    </div>
  </aside>
{/if}
