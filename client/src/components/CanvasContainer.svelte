<script lang="ts">
  import { agents, pods, selectedAgent, eventLogs } from '../lib/stores/gridStore';
  import { Terminal, Users, HardDrive, Compass } from '@lucide/svelte';

  function selectAgent(agent: any) {
    selectedAgent.set(agent);
  }
</script>

<div class="relative w-full h-full bg-[#07090e] overflow-hidden flex flex-col justify-between pt-14 select-none">
  <!-- Fundo Isométrico Cyberpunk com Grade e Mar de Dados -->
  <div class="absolute inset-0 opacity-20 pointer-events-none" style="background-image: radial-gradient(#00f0ff 1px, transparent 1px), radial-gradient(#9d00ff 1px, #07090e 1px); background-size: 40px 40px; background-position: 0 0, 20px 20px;"></div>

  <!-- Vista Central do Command Center -->
  <div class="relative z-10 flex-1 flex flex-col items-center justify-center p-6 text-center">
    <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-950/60 border border-cyan-500/40 text-cyan-400 text-xs font-mono mb-4 neon-glow-cyan">
      <Compass class="w-3.5 h-3.5 animate-spin" /> VIRTUAL COMMAND DECK — READY FOR PHASER 3 ENGINE
    </div>

    <h1 class="text-3xl font-black tracking-wider text-slate-100 uppercase mb-2">
      The Grid Metropolis <span class="text-cyan-400">Hub</span>
    </h1>
    <p class="text-sm font-mono text-slate-400 max-w-lg mb-8">
      Megacidade isométrica de agentes de IA suspensa sobre o Data Ocean. Monitore pods, rotas e execute o Modo Diretor.
    </p>

    <!-- Cards Interativos dos Agentes Ativos no Mundo -->
    <div class="w-full max-w-4xl grid grid-cols-1 md:grid-cols-3 gap-4">
      {#each $agents as agent (agent.id)}
        <button
          on:click={() => selectAgent(agent)}
          class="p-4 rounded-xl bg-slate-900/80 hover:bg-slate-800/90 border border-slate-800 hover:border-cyan-400/60 transition text-left group cursor-pointer"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="text-sm font-bold text-slate-200 group-hover:text-cyan-400 transition flex items-center gap-2">
              <span class="w-2 h-2 rounded-full {agent.status === 'WORKING' ? 'bg-amber-400 animate-ping' : agent.status === 'WALKING' ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400'}"></span>
              {agent.name}
            </div>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-950 text-slate-400 border border-slate-850">
              {agent.role}
            </span>
          </div>

          <div class="text-xs font-mono text-slate-400 space-y-1">
            <div class="flex justify-between">
              <span>Status:</span>
              <span class="text-cyan-300 font-semibold">{agent.status}</span>
            </div>
            <div class="flex justify-between">
              <span>Pod Base:</span>
              <span class="text-slate-300 truncate max-w-[120px]">{agent.homePodId}</span>
            </div>
          </div>
        </button>
      {/each}
    </div>
  </div>

  <!-- Feed de Eventos da Metrópole em Tempo Real (Rodapé) -->
  <div class="relative z-10 h-28 bg-slate-950/90 backdrop-blur border-t border-slate-800 px-5 py-2.5 font-mono text-xs flex flex-col justify-between">
    <div class="flex items-center justify-between text-slate-400 text-[11px] pb-1 border-b border-slate-900">
      <span class="flex items-center gap-1.5 font-semibold text-cyan-400"><Terminal class="w-3.5 h-3.5" /> METROPOLIS EVENT LOGS (STREAM EM TEMPO REAL)</span>
      <span>Total: {$eventLogs.length} eventos</span>
    </div>

    <div class="flex-1 overflow-y-auto space-y-1 pt-1.5 pr-2">
      {#if $eventLogs.length === 0}
        <div class="text-slate-600 italic">Nenhum evento registrado ainda. Aguardando despachos da ponte...</div>
      {:else}
        {#each $eventLogs.slice(0, 5) as log (log.id)}
          <div class="flex items-center gap-3 text-[11px]">
            <span class="text-slate-500 font-mono">[{log.time}]</span>
            <span class="px-1.5 py-0.2 rounded text-[10px] font-bold
              {log.topic === 'TASK_ASSIGNED' ? 'bg-cyan-950 text-cyan-400 border border-cyan-500/30' :
               log.topic === 'TASK_COMPLETED' ? 'bg-emerald-950 text-emerald-400 border border-emerald-500/30' :
               log.topic === 'TASK_ERROR' ? 'bg-rose-950 text-rose-400 border border-rose-500/30' :
               'bg-slate-900 text-slate-400'}">
              {log.topic}
            </span>
            <span class="text-slate-300 truncate">{log.text}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
