<script lang="ts">
  import { inspectingCard, playCard, architectEnergy } from '../../lib/stores/cardStore';
  import { X, Zap, Cpu, Sparkles, Terminal, FileCode, Tag, CheckCircle2 } from '@lucide/svelte';

  let customPromptOverride = '';

  $: if ($inspectingCard) {
    customPromptOverride = $inspectingCard.prompt;
  }

  $: hasEnergy = $inspectingCard ? $architectEnergy >= $inspectingCard.energyCost : false;

  function close() {
    inspectingCard.set(null);
  }

  function handleCast() {
    if (!$inspectingCard) return;
    if ($inspectingCard && customPromptOverride !== $inspectingCard.prompt) {
      $inspectingCard.prompt = customPromptOverride;
    }
    playCard($inspectingCard.id);
  }
</script>

{#if $inspectingCard}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 font-mono select-none">
    <!-- Card Hologram Container -->
    <div class="w-full max-w-2xl bg-gradient-to-b from-slate-900 via-slate-950 to-black rounded-2xl border-2 border-cyan-500/60 shadow-[0_0_50px_rgba(6,182,212,0.3)] overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Header do Modal -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/80">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-cyan-950 border border-cyan-400 flex items-center justify-center text-cyan-300 shadow-[0_0_12px_rgba(6,182,212,0.5)]">
            <Sparkles class="w-4 h-4 animate-spin" />
          </div>
          <div>
            <div class="text-sm font-black text-slate-100 uppercase tracking-widest flex items-center gap-2">
              INSPEÇÃO DO CONTRATO
              <span class="text-[10px] px-2 py-0.5 rounded font-bold uppercase bg-cyan-950 text-cyan-300 border border-cyan-500/40">
                {$inspectingCard.rarity}
              </span>
            </div>
            <div class="text-[11px] text-slate-400">{$inspectingCard.subtitle}</div>
          </div>
        </div>

        <button on:click={close} class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Corpo da Carta Holográfica -->
      <div class="p-6 overflow-y-auto space-y-5 text-xs text-slate-300">
        <!-- Título e Atributos Principais -->
        <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 p-4 rounded-xl bg-slate-900/90 border border-slate-800">
          <div>
            <div class="text-base font-black text-cyan-300 tracking-wide mb-1">{$inspectingCard.title}</div>
            <div class="text-slate-400 text-xs leading-relaxed font-sans">{$inspectingCard.description}</div>
          </div>

          <div class="flex items-center gap-3 shrink-0">
            <!-- Custo de Energia -->
            <div class="flex flex-col items-center px-3 py-1.5 rounded-lg bg-cyan-950/80 border border-cyan-500/50">
              <span class="text-[10px] text-cyan-400 font-bold uppercase">Custo</span>
              <div class="flex items-center gap-1 text-cyan-200 font-black text-base">
                <Zap class="w-4 h-4 fill-cyan-400 text-cyan-400" />
                {$inspectingCard.energyCost}
              </div>
            </div>

            <!-- Modelo Designado -->
            <div class="flex flex-col items-center px-3 py-1.5 rounded-lg bg-purple-950/80 border border-purple-500/50">
              <span class="text-[10px] text-purple-400 font-bold uppercase">Engine IA</span>
              <div class="flex items-center gap-1 text-purple-200 font-bold text-xs">
                <Cpu class="w-3.5 h-3.5 text-purple-400" />
                {$inspectingCard.targetModel}
              </div>
            </div>
          </div>
        </div>

        <!-- Flavor Text Imersivo -->
        <div class="p-3 rounded-lg bg-black/60 border-l-2 border-cyan-400 italic text-[11px] text-cyan-200/80">
          "{$inspectingCard.flavorText}"
        </div>

        <!-- Prompt Executável & Customizável -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <span class="text-[11px] font-bold uppercase text-slate-400 flex items-center gap-1.5">
              <Terminal class="w-3.5 h-3.5 text-cyan-400" /> Diretriz de Execução (Prompt)
            </span>
            <span class="text-[10px] text-slate-500">Editável antes do despacho</span>
          </div>
          <textarea
            bind:value={customPromptOverride}
            rows="4"
            class="w-full p-3 rounded-xl bg-slate-950 border border-slate-700 focus:border-cyan-400 text-slate-200 font-sans text-xs focus:outline-none transition leading-relaxed resize-none"
          ></textarea>
        </div>

        <!-- Arquivos de Contexto e Tags -->
        {#if $inspectingCard.contextFiles && $inspectingCard.contextFiles.length > 0}
          <div>
            <div class="text-[11px] font-bold uppercase text-slate-400 flex items-center gap-1.5 mb-2">
              <FileCode class="w-3.5 h-3.5 text-emerald-400" /> Arquivos no Escopo da Execução
            </div>
            <div class="flex flex-wrap gap-2">
              {#each $inspectingCard.contextFiles as file}
                <span class="px-2.5 py-1 rounded-md bg-slate-900 border border-slate-800 text-[11px] text-emerald-300 font-mono flex items-center gap-1">
                  📄 {file}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Tags -->
        <div class="flex items-center gap-2 pt-2 border-t border-slate-800/80">
          <Tag class="w-3.5 h-3.5 text-slate-500" />
          <div class="flex flex-wrap gap-1.5">
            {#each $inspectingCard.tags as tag}
              <span class="text-[10px] px-2 py-0.5 rounded bg-slate-900 text-slate-400 border border-slate-800 font-mono">
                #{tag}
              </span>
            {/each}
          </div>
        </div>
      </div>

      <!-- Rodapé do Modal com Botão de Despacho -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/90 flex items-center justify-between">
        <button
          on:click={close}
          class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold transition text-xs"
        >
          Retornar à Mão
        </button>

        <button
          on:click={handleCast}
          disabled={!hasEnergy}
          class="px-6 py-2.5 rounded-xl font-bold uppercase tracking-wider text-xs flex items-center gap-2 transition
          {hasEnergy
            ? 'bg-gradient-to-r from-cyan-400 to-blue-500 hover:from-cyan-300 hover:to-blue-400 text-slate-950 shadow-[0_0_20px_rgba(6,182,212,0.6)] cursor-pointer active:scale-95'
            : 'bg-slate-800 text-slate-500 cursor-not-allowed border border-slate-700'}"
        >
          <Zap class="w-4 h-4 fill-current" />
          {#if hasEnergy}CONJURAR CARTA (DISPATCH TO AGENT){:else}ENERGIA INSUFICIENTE{/if}
        </button>
      </div>
    </div>
  </div>
{/if}
