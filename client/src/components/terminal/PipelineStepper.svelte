<script lang="ts">
  import type { PipelineState, DroidInfo } from '../../lib/stores/projectStore';
  import { CheckCircle2, ChevronRight, Bot, Sparkles, Terminal } from '@lucide/svelte';

  export let pipeline: PipelineState;
  export let droids: DroidInfo[];

  $: isSoloMode = droids.length === 1;
  $: soloDroid = droids[0];

  $: architectDroid = droids.find((d) => d.role === 'ARQUITETO' || d.role === 'CYBER_COMPANION') || droids[0];
  $: coderDroid = droids.find((d) => (d.role === 'PROGRAMADOR' || d.role === 'DEVELOPER') && d.id !== architectDroid?.id) || droids[1] || droids[0];
  $: reviewerDroid = droids.find((d) => (d.role === 'REVISOR_DE_CODIGO' || d.role === 'SECURITY' || d.role === 'PESQUISADOR') && d.id !== architectDroid?.id && d.id !== coderDroid?.id) || droids[2] || droids[0];

  function getStepState(stepStage: 'ARCHITECT' | 'CODER' | 'REVIEWER') {
    if (!pipeline.isActive) {
      return pipeline.stage === 'COMPLETED' ? 'DONE' : 'WAITING';
    }

    const order = ['ARCHITECT', 'CODER', 'REVIEWER'];
    const currentIndex = order.indexOf(pipeline.stage);
    const stepIndex = order.indexOf(stepStage);

    if (currentIndex > stepIndex) return 'DONE';
    if (currentIndex === stepIndex) return 'ACTIVE';
    return 'WAITING';
  }

  $: archState = getStepState('ARCHITECT');
  $: coderState = getStepState('CODER');
  $: revState = getStepState('REVIEWER');
</script>

{#if isSoloMode}
  <!-- Barra de Modo Solo (1 Operador Atribuído - Execução Direta) -->
  <div class="px-4 py-2 border-b border-cyan-500/20 bg-[#060a14]/90 backdrop-blur-md select-none font-mono">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
      <div class="flex items-center gap-2 text-xs">
        <span class="px-2 py-0.5 rounded bg-cyan-950/70 border border-cyan-500/50 text-cyan-300 font-pixel-title text-[10px] tracking-wider flex items-center gap-1.5 shadow-[0_0_8px_rgba(0,240,255,0.2)]">
          <Bot size={12} />
          <span>HARNESS MODO SOLO</span>
        </span>

        <span class="text-slate-400 font-mono text-[11px] flex items-center gap-1">
          <span>Operador Único:</span>
          <strong style="color: {soloDroid.accentColor}">{soloDroid.name}</strong>
          <span class="text-slate-600">[{soloDroid.planBadge}]</span>
        </span>
      </div>

      <div class="flex items-center gap-3 text-xs">
        {#if pipeline.isActive}
          <div class="flex items-center gap-1.5 text-cyan-300 font-mono">
            <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
            <span class="text-[11px] truncate max-w-[280px]">{pipeline.statusText}</span>
          </div>
        {:else if pipeline.stage === 'COMPLETED'}
          <div class="flex items-center gap-1.5 text-emerald-400 font-mono text-[11px]">
            <CheckCircle2 size={12} />
            <span>Última execução concluída com sucesso</span>
          </div>
        {:else}
          <div class="text-[11px] text-slate-500 font-mono">
            ● Pronto para comando direto
          </div>
        {/if}

        <div class="flex items-center gap-2">
          <div class="w-20 h-1.5 rounded-full bg-slate-900 border border-slate-800 overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-cyan-400 to-emerald-400 transition-all duration-300"
              style="width: {pipeline.progressPercent}%;"
            ></div>
          </div>
          <span class="font-pixel-terminal text-xs text-cyan-400 font-bold">
            {pipeline.progressPercent}%
          </span>
        </div>
      </div>
    </div>
  </div>
{:else}
  <!-- Barra Retrô Cyberpunk de Handoff Sequencial (Stepper Multi-Droid) -->
  <div class="px-4 py-2 border-b border-cyan-500/20 bg-[#060a14]/90 backdrop-blur-md select-none font-mono">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
      <!-- Stepper Sequencial das Etapas -->
      <div class="flex items-center gap-1.5 sm:gap-2 overflow-x-auto text-xs py-0.5">
        <!-- ETAPA 1: ARQUITETO -->
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg border transition-all {archState === 'ACTIVE'
            ? 'bg-cyan-950/80 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(6,182,212,0.4)] ring-1 ring-cyan-400/50'
            : archState === 'DONE'
            ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-400'
            : 'bg-slate-900/40 border-slate-800 text-slate-500'}"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold font-pixel-title {archState === 'ACTIVE'
            ? 'bg-cyan-400 text-black animate-pulse'
            : archState === 'DONE'
            ? 'bg-emerald-500 text-black'
            : 'bg-slate-800 text-slate-400'}">
            {#if archState === 'DONE'}✓{:else}1{/if}
          </span>
          <span class="font-pixel-title text-[10px] tracking-wider uppercase font-bold">
            {architectDroid?.name || 'SPARK'}: ARQUITETO
          </span>
          {#if archState === 'ACTIVE'}
            <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping ml-0.5"></span>
          {/if}
        </div>

        <ChevronRight size={13} class="text-slate-600 shrink-0" />

        <!-- ETAPA 2: PROGRAMADOR (CODER) -->
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg border transition-all {coderState === 'ACTIVE'
            ? 'bg-emerald-950/80 border-emerald-400 text-emerald-300 shadow-[0_0_12px_rgba(52,211,153,0.4)] ring-1 ring-emerald-400/50'
            : coderState === 'DONE'
            ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-400'
            : 'bg-slate-900/40 border-slate-800 text-slate-500'}"
        >
          <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold font-pixel-title {coderState === 'ACTIVE'
            ? 'bg-emerald-400 text-black animate-pulse'
            : coderState === 'DONE'
            ? 'bg-emerald-500 text-black'
            : 'bg-slate-800 text-slate-400'}">
            {#if coderState === 'DONE'}✓{:else}2{/if}
          </span>
          <span class="font-pixel-title text-[10px] tracking-wider uppercase font-bold">
            {coderDroid?.name || 'RUSTY'}: CODER
          </span>
          {#if coderState === 'ACTIVE'}
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping ml-0.5"></span>
          {/if}
        </div>

        {#if droids.length >= 3}
          <ChevronRight size={13} class="text-slate-600 shrink-0" />

          <!-- ETAPA 3: REVISOR (CODE REVIEW) -->
          <div
            class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg border transition-all {revState === 'ACTIVE'
              ? 'bg-fuchsia-950/80 border-fuchsia-400 text-fuchsia-300 shadow-[0_0_12px_rgba(217,70,239,0.4)] ring-1 ring-fuchsia-400/50'
              : revState === 'DONE'
              ? 'bg-emerald-950/40 border-emerald-500/40 text-emerald-400'
              : 'bg-slate-900/40 border-slate-800 text-slate-500'}"
          >
            <span class="w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold font-pixel-title {revState === 'ACTIVE'
              ? 'bg-fuchsia-400 text-black animate-pulse'
              : revState === 'DONE'
              ? 'bg-emerald-500 text-black'
              : 'bg-slate-800 text-slate-400'}">
              {#if revState === 'DONE'}✓{:else}3{/if}
            </span>
            <span class="font-pixel-title text-[10px] tracking-wider uppercase font-bold">
              {reviewerDroid?.name || 'VANGUARD'}: REVIEW
            </span>
            {#if revState === 'ACTIVE'}
              <span class="w-1.5 h-1.5 rounded-full bg-fuchsia-400 animate-ping ml-0.5"></span>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Status Atual da Esteira / Progresso em Porcentagem -->
      <div class="flex items-center gap-2 shrink-0">
        <div class="w-24 sm:w-28 h-2 rounded-full bg-slate-900 border border-slate-800 overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-cyan-400 via-emerald-400 to-fuchsia-400 transition-all duration-500"
            style="width: {pipeline.progressPercent}%;"
          ></div>
        </div>
        <span class="font-pixel-terminal text-sm text-cyan-400 font-bold w-10 text-right">
          {pipeline.progressPercent}%
        </span>
      </div>
    </div>

    <!-- Texto de Status Detalhado da Ação da Esteira -->
    {#if pipeline.isActive || pipeline.stage === 'COMPLETED'}
      <div class="mt-1.5 flex items-center gap-1.5 text-[11px] text-slate-300 font-mono tracking-wide">
        {#if pipeline.stage === 'COMPLETED'}
          <CheckCircle2 size={12} class="text-emerald-400 shrink-0" />
        {:else}
          <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping shrink-0"></span>
        {/if}
        <span class="truncate">{pipeline.statusText}</span>
      </div>
    {/if}
  </div>
{/if}
