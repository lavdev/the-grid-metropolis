<script lang="ts">
  import {
    hand,
    architectEnergy,
    maxArchitectEnergy,
    drawCard,
    isGrimoireOpen,
    isForgeModalOpen,
    graveyard,
    deck,
  } from '../../lib/stores/cardStore';
  import CardItem from './CardItem.svelte';
  import { Zap, Plus, BookOpen, Layers, ChevronDown, ChevronUp } from '@lucide/svelte';

  let isCollapsed = false;

  $: currentEnergy = $architectEnergy;
  $: maxEnergy = $maxArchitectEnergy;
  $: handCount = $hand.length;
  $: graveyardCount = $graveyard.length;
  $: deckCount = $deck.length;

  function toggleCollapse() {
    isCollapsed = !isCollapsed;
  }
</script>

<div class="fixed bottom-0 left-0 right-0 z-40 flex flex-col items-center pointer-events-none select-none">
  <!-- Botão de Recolher/Expandir Barra da Mão -->
  <button
    on:click={toggleCollapse}
    class="pointer-events-auto px-4 py-1 rounded-t-lg bg-slate-950/90 border-t border-x border-cyan-500/40 text-cyan-400 hover:text-cyan-200 text-xs font-mono font-bold flex items-center gap-1.5 backdrop-blur-md shadow-[0_-5px_15px_rgba(6,182,212,0.2)] transition"
  >
    <Layers class="w-3.5 h-3.5" />
    <span>MÃO DO ARQUITETO ({handCount}/6)</span>
    {#if isCollapsed}
      <ChevronUp class="w-3.5 h-3.5" />
    {:else}
      <ChevronDown class="w-3.5 h-3.5" />
    {/if}
  </button>

  {#if !isCollapsed}
    <!-- Container Principal do HUD da Mão -->
    <div class="w-full bg-gradient-to-t from-black via-slate-950/95 to-slate-950/80 border-t border-cyan-500/30 backdrop-blur-lg px-6 py-4 flex flex-col items-center pointer-events-auto shadow-[0_-10px_35px_rgba(0,0,0,0.8)]">
      <!-- Barra Superior de Controle: Energia e Ações de Deck -->
      <div class="w-full max-w-6xl flex items-center justify-between mb-3 border-b border-slate-800/80 pb-2.5 font-mono">
        <!-- Indicador de Cristais de Mana/Energia -->
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 text-xs font-bold text-cyan-300">
            <Zap class="w-4 h-4 text-cyan-400 fill-cyan-400" />
            <span class="tracking-wider">ENERGIA DO ARQUITETO:</span>
            <span class="text-white bg-cyan-950/80 px-2 py-0.5 rounded border border-cyan-500/40 font-black">
              {currentEnergy} / {maxEnergy}
            </span>
          </div>

          <!-- Cristais Visuais -->
          <div class="flex items-center gap-1">
            {#each Array(maxEnergy) as _, i}
              <div
                class="w-3 h-5 rounded-sm transition-all duration-300 {i < currentEnergy
                  ? 'bg-gradient-to-t from-cyan-600 to-cyan-300 shadow-[0_0_8px_#00f0ff] border border-cyan-200 scale-105'
                  : 'bg-slate-900 border border-slate-800 opacity-40'}"
              ></div>
            {/each}
          </div>
        </div>

        <!-- Botões de Ação do Deck: Comprar, Forjar e Grimório -->
        <div class="flex items-center gap-2 text-xs">
          <!-- Botão Comprar Carta -->
          <button
            on:click={() => drawCard()}
            disabled={handCount >= 6}
            class="px-3 py-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-200 hover:text-white font-bold flex items-center gap-1.5 transition disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Layers class="w-3.5 h-3.5 text-cyan-400" />
            <span>Comprar ({deckCount})</span>
          </button>

          <!-- Botão Forjar Carta (Novo Prompt) -->
          <button
            on:click={() => isForgeModalOpen.set(true)}
            class="px-3 py-1.5 rounded-lg bg-gradient-to-r from-purple-900/60 to-pink-900/60 hover:from-purple-800 hover:to-pink-800 border border-purple-500/40 text-purple-200 font-bold flex items-center gap-1.5 transition shadow-[0_0_10px_rgba(168,85,247,0.3)]"
          >
            <Plus class="w-3.5 h-3.5 text-pink-400" />
            <span>Forjar Carta</span>
          </button>

          <!-- Botão Grimório (Histórico) -->
          <button
            on:click={() => isGrimoireOpen.set(true)}
            class="px-3 py-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-200 hover:text-white font-bold flex items-center gap-1.5 transition relative"
          >
            <BookOpen class="w-3.5 h-3.5 text-amber-400" />
            <span>Grimório</span>
            {#if graveyardCount > 0}
              <span class="px-1.5 py-0.2 rounded-full bg-amber-500/20 border border-amber-400/60 text-amber-300 text-[10px] font-black">
                {graveyardCount}
              </span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Leque das Cartas na Mão -->
      <div class="w-full flex items-end justify-center gap-2 sm:gap-4 overflow-x-auto py-2 px-4 scrollbar-none min-h-[300px]">
        {#if handCount === 0}
          <div class="flex flex-col items-center justify-center text-slate-500 py-10 font-mono text-xs">
            <Layers class="w-8 h-8 mb-2 opacity-40 text-cyan-400 animate-bounce" />
            <div>Sua mão está vazia.</div>
            <button
              on:click={() => drawCard()}
              class="mt-2 px-3 py-1 rounded bg-cyan-950 border border-cyan-500/40 text-cyan-300 font-bold hover:bg-cyan-900 transition"
            >
              Comprar Cartas do Deck
            </button>
          </div>
        {:else}
          {#each $hand as card, index (card.id)}
            <CardItem {card} {index} totalInHand={handCount} />
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
