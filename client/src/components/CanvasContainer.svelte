<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createGridMetropolisGame, destroyGridMetropolisGame } from '../lib/game/gameManager';
  import { eventLogs, agents, selectedAgent } from '../lib/stores/gridStore';
  import { Terminal, Compass, ZoomIn, ZoomOut, RotateCcw } from '@lucide/svelte';
  import type Phaser from 'phaser';

  let canvasWrapper: HTMLElement;
  let gameInstance: Phaser.Game | null = null;

  onMount(() => {
    if (canvasWrapper) {
      gameInstance = createGridMetropolisGame(canvasWrapper);
    }
  });

  onDestroy(() => {
    destroyGridMetropolisGame(gameInstance);
    gameInstance = null;
  });

  function resetCamera() {
    if (gameInstance) {
      const scene = gameInstance.scene.getScene('MainIsometricScene') as any;
      if (scene && scene.cameras?.main) {
        scene.cameras.main.centerOn(0, 200);
        scene.cameras.main.setZoom(0.85);
      }
    }
  }

  function adjustZoom(factor: number) {
    if (gameInstance) {
      const scene = gameInstance.scene.getScene('MainIsometricScene') as any;
      if (scene && scene.cameras?.main) {
        const currentZoom = scene.cameras.main.zoom;
        scene.cameras.main.setZoom(Math.min(Math.max(currentZoom * factor, 0.35), 2.2));
      }
    }
  }
</script>

<div class="relative w-full h-full bg-[#05070c] overflow-hidden flex flex-col justify-between pt-14 select-none">
  <!-- Container do Canvas Phaser 3 (WebGL) -->
  <div bind:this={canvasWrapper} class="absolute inset-0 z-0"></div>

  <!-- Controles Flutuantes de Câmera (Modo Diretor) -->
  <div class="absolute top-18 left-5 z-20 flex flex-col gap-2 font-mono">
    <div class="p-1 rounded bg-slate-950/85 backdrop-blur border border-slate-800 flex flex-col gap-1 shadow-lg">
      <button
        on:click={() => adjustZoom(1.2)}
        title="Aproximar Zoom"
        class="p-2 rounded hover:bg-slate-800 text-slate-300 hover:text-cyan-400 transition cursor-pointer"
      >
        <ZoomIn class="w-4 h-4" />
      </button>
      <button
        on:click={() => adjustZoom(0.8)}
        title="Afastar Zoom"
        class="p-2 rounded hover:bg-slate-800 text-slate-300 hover:text-cyan-400 transition cursor-pointer"
      >
        <ZoomOut class="w-4 h-4" />
      </button>
      <button
        on:click={resetCamera}
        title="Centralizar no Nexus"
        class="p-2 rounded hover:bg-slate-800 text-slate-300 hover:text-cyan-400 transition cursor-pointer border-t border-slate-850"
      >
        <RotateCcw class="w-4 h-4" />
      </button>
    </div>

    <!-- Indicador de Navegação -->
    <div class="px-2.5 py-1 rounded bg-slate-950/80 border border-slate-800/80 text-[10px] text-slate-400 flex items-center gap-1.5 backdrop-blur">
      <Compass class="w-3 h-3 text-cyan-400" />
      <span>Arraste para mover • Scroll para zoom</span>
    </div>
  </div>

  <!-- Lista Rápida de Agentes Flutuante (Lateral Esquerda Inferior) -->
  <div class="absolute bottom-32 left-5 z-20 flex flex-col gap-1.5 max-w-xs font-mono">
    <div class="text-[10px] font-bold tracking-wider text-slate-400 uppercase">Agentes no Grid:</div>
    <div class="flex flex-wrap gap-2">
      {#each $agents as agent (agent.id)}
        <button
          on:click={() => selectedAgent.set(agent)}
          class="px-2.5 py-1 rounded-lg bg-slate-950/90 hover:bg-slate-900 border border-slate-800 hover:border-cyan-400/60 transition text-xs flex items-center gap-2 cursor-pointer shadow-md backdrop-blur"
        >
          <span class="w-2 h-2 rounded-full {agent.status === 'WORKING' ? 'bg-amber-400 animate-ping' : agent.status === 'WALKING' ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400'}"></span>
          <span class="text-slate-200 font-bold">{agent.name}</span>
          <span class="text-[10px] text-cyan-400/80">({agent.status})</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Feed de Eventos em Tempo Real (Rodapé) -->
  <div class="relative z-20 h-28 bg-slate-950/90 backdrop-blur-md border-t border-slate-800/80 px-5 py-2 font-mono text-xs flex flex-col justify-between mt-auto">
    <div class="flex items-center justify-between text-slate-400 text-[11px] pb-1 border-b border-slate-900">
      <span class="flex items-center gap-1.5 font-semibold text-cyan-400">
        <Terminal class="w-3.5 h-3.5" /> METROPOLIS EVENT LOGS (STREAM EM TEMPO REAL)
      </span>
      <span>Total: {$eventLogs.length} eventos</span>
    </div>

    <div class="flex-1 overflow-y-auto space-y-1 pt-1.5 pr-2">
      {#if $eventLogs.length === 0}
        <div class="text-slate-600 italic">Conectando ao hub do servidor... Aguardando eventos.</div>
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
