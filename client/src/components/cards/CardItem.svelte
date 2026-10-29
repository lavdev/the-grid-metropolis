<script lang="ts">
  import type { ActionCard } from '../../types/card';
  import { inspectingCard, playCard, architectEnergy } from '../../lib/stores/cardStore';
  import { Zap, Eye, Sparkles, Cpu, Code, Shield, Search, Terminal } from '@lucide/svelte';

  export let card: ActionCard;
  export let index: number = 0;
  export let totalInHand: number = 4;

  $: hasEnergy = $architectEnergy >= card.energyCost;

  // Cálculo de rotação em leque (fan)
  $: rotationDeg = totalInHand > 1 
    ? (index - (totalInHand - 1) / 2) * 3.5 
    : 0;

  $: translateYPx = totalInHand > 1
    ? Math.abs(index - (totalInHand - 1) / 2) * 5
    : 0;

  const rarityBorders: Record<ActionCard['rarity'], { border: string; glow: string; badge: string; text: string }> = {
    COMMON: {
      border: 'border-cyan-500/40 hover:border-cyan-400',
      glow: 'hover:shadow-[0_0_20px_rgba(6,182,212,0.4)]',
      badge: 'bg-cyan-950/80 text-cyan-300 border-cyan-500/40',
      text: 'text-cyan-400',
    },
    UNCOMMON: {
      border: 'border-emerald-500/50 hover:border-emerald-400',
      glow: 'hover:shadow-[0_0_22px_rgba(16,185,129,0.45)]',
      badge: 'bg-emerald-950/80 text-emerald-300 border-emerald-500/40',
      text: 'text-emerald-400',
    },
    RARE: {
      border: 'border-purple-500/60 hover:border-purple-400',
      glow: 'hover:shadow-[0_0_25px_rgba(168,85,247,0.5)]',
      badge: 'bg-purple-950/80 text-purple-300 border-purple-500/40',
      text: 'text-purple-400',
    },
    LEGENDARY: {
      border: 'border-amber-400/80 hover:border-amber-300',
      glow: 'hover:shadow-[0_0_30px_rgba(251,191,36,0.6)] animate-pulse',
      badge: 'bg-amber-950/80 text-amber-200 border-amber-400/50',
      text: 'text-amber-300',
    },
  };

  function handlePlay(e: MouseEvent) {
    e.stopPropagation();
    playCard(card.id);
  }

  function handleInspect(e: MouseEvent) {
    e.stopPropagation();
    inspectingCard.set(card);
  }
</script>

<div
  role="button"
  tabindex="0"
  on:click={handleInspect}
  on:keydown={(e) => { if (e.key === 'Enter') inspectingCard.set(card); }}
  style="transform: rotate({rotationDeg}deg) translateY({translateYPx}px);"
  class="group relative select-none cursor-pointer transition-all duration-300 ease-out hover:!-translate-y-10 hover:!rotate-0 hover:z-50 hover:scale-105"
>
  <!-- Card Container -->
  <div
    class="w-48 sm:w-52 h-72 sm:h-76 rounded-xl bg-gradient-to-b from-slate-900 via-slate-950 to-black p-3.5 flex flex-col justify-between border-2 backdrop-blur-md relative overflow-hidden transition-all duration-300
    {rarityBorders[card.rarity].border} {rarityBorders[card.rarity].glow} {!hasEnergy ? 'opacity-65 grayscale-[30%]' : ''}"
  >
    <!-- Background Holographic Pattern & Scanlines -->
    <div class="absolute inset-0 bg-gradient-to-br from-cyan-500/5 via-transparent to-purple-500/5 pointer-events-none"></div>
    <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-800/20 via-transparent to-transparent pointer-events-none"></div>

    <!-- Header: Custo de Energia e Categoria -->
    <div class="flex items-center justify-between z-10">
      <!-- Custo de Energia (Hexágono Cyberpunk) -->
      <div class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-cyan-950/90 border border-cyan-400/50 shadow-[0_0_10px_rgba(6,182,212,0.4)]">
        <Zap class="w-3.5 h-3.5 text-cyan-400 fill-cyan-400" />
        <span class="text-xs font-black font-mono text-cyan-200">{card.energyCost}</span>
      </div>

      <!-- Badge de Raridade / Categoria -->
      <span class="text-[9px] font-bold tracking-wider font-mono px-2 py-0.5 rounded uppercase border {rarityBorders[card.rarity].badge}">
        {card.rarity}
      </span>
    </div>

    <!-- Arte / Ícone Central da Carta -->
    <div class="my-auto flex flex-col items-center justify-center text-center z-10 py-1">
      <div class="w-14 h-14 rounded-lg bg-slate-900/90 border border-slate-700/80 flex items-center justify-center mb-2 shadow-inner group-hover:scale-110 transition-transform duration-300 relative">
        {#if card.category === 'CODE'}
          <Code class="w-7 h-7 {rarityBorders[card.rarity].text}" />
        {:else if card.category === 'ARCH'}
          <Cpu class="w-7 h-7 {rarityBorders[card.rarity].text}" />
        {:else if card.category === 'SECURITY'}
          <Shield class="w-7 h-7 {rarityBorders[card.rarity].text}" />
        {:else if card.category === 'RESEARCH'}
          <Search class="w-7 h-7 {rarityBorders[card.rarity].text}" />
        {:else}
          <Terminal class="w-7 h-7 {rarityBorders[card.rarity].text}" />
        {/if}

        <Sparkles class="w-3 h-3 absolute -top-1 -right-1 {rarityBorders[card.rarity].text} opacity-80" />
      </div>

      <div class="text-xs font-black text-slate-100 tracking-wide line-clamp-1">{card.title}</div>
      <div class="text-[10px] text-slate-400 font-mono line-clamp-1">{card.subtitle}</div>
    </div>

    <!-- Bloco de Modelo IA & Estimativa de Tokens -->
    <div class="z-10 bg-slate-900/80 rounded-md p-2 border border-slate-800 text-[10px] font-mono space-y-1">
      <div class="flex items-center justify-between text-slate-300">
        <span class="text-slate-500">Modelo:</span>
        <span class="text-cyan-300 font-bold truncate max-w-[95px]">{card.targetModel}</span>
      </div>
      <div class="flex items-center justify-between text-slate-400">
        <span class="text-slate-500">Estimativa:</span>
        <span class="text-purple-400 font-semibold">{card.tokenEstimate}</span>
      </div>
    </div>

    <!-- Rodapé: Botões de Ação -->
    <div class="mt-2 pt-2 border-t border-slate-800/80 flex items-center justify-between gap-1 z-10">
      <button
        on:click={handleInspect}
        title="Inspecionar e Customizar Carta"
        class="p-1.5 rounded bg-slate-800/90 hover:bg-slate-700 text-slate-300 hover:text-white transition text-xs flex items-center gap-1"
      >
        <Eye class="w-3.5 h-3.5" />
      </button>

      <button
        on:click={handlePlay}
        disabled={!hasEnergy}
        class="flex-1 py-1 px-2 rounded font-mono text-[10px] font-black uppercase tracking-wider flex items-center justify-center gap-1 transition shadow-sm
        {hasEnergy 
          ? 'bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-slate-950 font-bold cursor-pointer active:scale-95 shadow-[0_0_12px_rgba(6,182,212,0.5)]'
          : 'bg-slate-800 text-slate-500 cursor-not-allowed border border-slate-700'}"
      >
        <Zap class="w-3 h-3 fill-current" />
        {#if hasEnergy}JOGAR{:else}SEM ENERGIA{/if}
      </button>
    </div>
  </div>
</div>
