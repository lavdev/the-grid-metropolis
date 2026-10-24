<script lang="ts">
  import type { ConnectionState } from '../lib/socket/client';
  import { agents, tasks } from '../lib/stores/gridStore';
  import { Activity, Cpu, Radio, Shield, Terminal } from '@lucide/svelte';

  export let connectionStatus: ConnectionState = 'DISCONNECTED';

  $: totalAgents = $agents.length;
  $: activeAgents = $agents.filter((a) => a.status === 'WORKING' || a.status === 'WALKING').length;
  $: totalTasks = $tasks.length;
  $: totalTokens = $agents.reduce((acc, a) => acc + (a.metrics?.tokensConsumed || 0), 0);
</script>

<header class="fixed top-0 left-0 right-0 h-14 bg-slate-950/90 backdrop-blur-md border-b border-cyan-500/20 z-40 flex items-center justify-between px-5 select-none">
  <!-- Logotipo e Nome do Mundo -->
  <div class="flex items-center gap-3">
    <div class="w-8 h-8 rounded bg-cyan-950 border border-cyan-400 flex items-center justify-center neon-glow-cyan">
      <Cpu class="w-5 h-5 text-cyan-400 animate-pulse" />
    </div>
    <div>
      <div class="text-sm font-bold tracking-widest text-cyan-400 flex items-center gap-2">
        THE GRID METROPOLIS
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-cyan-950/80 border border-cyan-500/30 text-cyan-300">
          CORE v0.1
        </span>
      </div>
      <div class="text-[10px] font-mono text-slate-400">ISOMETRIC MULTI-AGENT COMMAND CENTER</div>
    </div>
  </div>

  <!-- Métricas Rápidas do Sistema -->
  <div class="flex items-center gap-6 font-mono text-xs">
    <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900/80 border border-slate-800">
      <Activity class="w-4 h-4 text-emerald-400" />
      <span class="text-slate-400">Agentes:</span>
      <span class="text-emerald-400 font-bold">{activeAgents}/{totalAgents}</span>
    </div>

    <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900/80 border border-slate-800">
      <Terminal class="w-4 h-4 text-cyan-400" />
      <span class="text-slate-400">Tarefas:</span>
      <span class="text-cyan-400 font-bold">{totalTasks}</span>
    </div>

    <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900/80 border border-slate-800">
      <Shield class="w-4 h-4 text-purple-400" />
      <span class="text-slate-400">Tokens:</span>
      <span class="text-purple-400 font-bold">{totalTokens.toLocaleString()}</span>
    </div>

    <!-- Indicador de Status WebSocket com Servidor Rust -->
    <div class="flex items-center gap-2 px-3 py-1.5 rounded bg-slate-900/80 border border-slate-800">
      <Radio class="w-4 h-4 {connectionStatus === 'CONNECTED' ? 'text-emerald-400 animate-pulse' : 'text-rose-500'}" />
      <span class="text-slate-400">Bridge:</span>
      {#if connectionStatus === 'CONNECTED'}
        <span class="text-emerald-400 font-semibold text-[11px]">ONLINE (RUST)</span>
      {:else if connectionStatus === 'CONNECTING'}
        <span class="text-amber-400 font-semibold text-[11px]">CONECTANDO...</span>
      {:else}
        <span class="text-rose-400 font-semibold text-[11px]">OFFLINE</span>
      {/if}
    </div>
  </div>
</header>
