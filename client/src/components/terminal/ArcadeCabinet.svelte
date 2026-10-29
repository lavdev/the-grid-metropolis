<script lang="ts">
  import { isArcadeMode, toggleArcadeMode } from '../../lib/stores/themeStore';
  import { activeProject } from '../../lib/stores/projectStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { Gamepad2, Monitor, Sparkles } from '@lucide/svelte';

  function handleToggleMode() {
    toggleArcadeMode();
    soundFX.playKeyClick();
  }

  function handleInsertCoin() {
    soundFX.playPipelineComplete();
  }
</script>

{#if $isArcadeMode}
  <!-- Modo Gabinete de Fliperama Arcade Retrô Anos 80/90 -->
  <div class="relative w-full h-full flex flex-col items-center justify-between bg-[#04060f] select-none overflow-hidden p-1 sm:p-2">
    <!-- Efeito de Iluminação de Sala Escura de Arcade com Néons ao Fundo -->
    <div class="absolute inset-0 pointer-events-none opacity-40 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-[#131b30] via-[#080d19] to-[#020408]"></div>

    <!-- 1. Topo: Marquee da Máquina Arcade (Caixa de Luz Retroiluminada) -->
    <header class="relative z-30 w-full max-w-5xl h-14 sm:h-16 rounded-t-2xl bg-gradient-to-r from-[#180d2a] via-[#10142a] to-[#180d2a] border-t-2 border-x-2 border-fuchsia-500/60 shadow-[0_0_30px_rgba(255,0,127,0.35),inset_0_2px_8px_rgba(255,255,255,0.2)] flex items-center justify-between px-4 sm:px-8 overflow-hidden">
      <!-- Néon Superior Decorativo -->
      <div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-cyan-400 shadow-[0_0_12px_#ff007f]"></div>

      <!-- Lado Esquerdo: Nave Pixel Art & Estrelas -->
      <div class="flex items-center gap-2 sm:gap-4 shrink-0">
        <span class="text-xl sm:text-2xl animate-pulse">🚀</span>
        <div class="hidden md:flex flex-col text-[9px] font-pixel-title text-cyan-400 tracking-widest leading-none">
          <span>HIGH SCORE</span>
          <span class="text-white font-pixel-terminal text-base">999,990</span>
        </div>
      </div>

      <!-- Centro: Letreiro Dinâmico do Marquee (Nome do Workspace Atual) -->
      <div class="flex flex-col items-center justify-center text-center">
        <h1
          class="font-pixel-title text-base sm:text-2xl md:text-3xl font-black tracking-widest uppercase text-transparent bg-clip-text bg-gradient-to-r from-pink-400 via-fuchsia-200 to-cyan-300 drop-shadow-[0_0_15px_rgba(255,0,127,0.8)] flex items-center gap-2"
        >
          <span>{$activeProject.name.toUpperCase()}</span>
        </h1>
        <span class="text-[8px] sm:text-[9px] font-pixel-title text-cyan-300 tracking-[0.3em] uppercase opacity-90 drop-shadow-[0_0_6px_#00f0ff]">
          ★ THE GRID METROPOLIS ARCADE EDITION ★
        </span>
      </div>

      <!-- Lado Direito: Nave Inimiga / Aliens & Botão de Alternar Modo -->
      <div class="flex items-center gap-2 sm:gap-4 shrink-0">
        <span class="text-xl sm:text-2xl animate-bounce">👾</span>
        <button
          type="button"
          on:click={handleToggleMode}
          class="px-2.5 py-1 rounded bg-[#0a0f1d] hover:bg-cyan-950 border border-cyan-500/50 hover:border-cyan-400 text-cyan-300 font-pixel-title text-[9px] sm:text-[10px] tracking-wider flex items-center gap-1.5 transition cursor-pointer shadow-[0_0_10px_rgba(0,240,255,0.2)]"
          title="Alternar para Modo Monitor Cyber-Deck Limpo"
        >
          <Monitor size={12} />
          <span class="hidden sm:inline">MODO DECK</span>
        </button>
      </div>
    </header>

    <!-- 2. Corpo Central: Asas do Bezel Espacial com Tubos de Néon Laterais + Monitor CRT -->
    <div class="relative w-full max-w-5xl flex-1 flex items-stretch border-x-4 border-[#161a29] shadow-[0_0_50px_rgba(0,0,0,0.9)] overflow-hidden">
      <!-- Asa Esquerda do Bezel: Tubo Néon Ciano + Planeta Saturno e Caças Espaciais -->
      <aside class="relative w-12 sm:w-20 md:w-28 shrink-0 bg-[#060813] border-r-2 border-[#162035] overflow-hidden hidden sm:flex flex-col justify-between p-2 select-none">
        <!-- Tubo Néon Vertical Ciano da Borda Esquerda -->
        <div class="absolute top-0 bottom-0 left-1 w-1.5 rounded-full bg-cyan-400 shadow-[0_0_15px_#00f0ff,0_0_30px_#00f0ff] animate-pulse"></div>

        <!-- Arte Espacial Esquerda: Saturno com Anéis -->
        <div class="relative z-10 pt-4 flex flex-col items-center">
          <div class="text-3xl md:text-4xl filter drop-shadow-[0_0_10px_rgba(255,170,0,0.6)]">🪐</div>
          <span class="text-[8px] font-pixel-title text-amber-400/80 tracking-widest mt-1">SECTOR-01</span>
        </div>

        <!-- Caças em Formação com Lasers Rosa -->
        <div class="relative z-10 flex flex-col items-center gap-3">
          <div class="text-lg rotate-45 animate-pulse">🛸</div>
          <div class="w-1 h-6 bg-pink-500 rounded-full shadow-[0_0_8px_#ff007f] rotate-12"></div>
          <div class="text-base -rotate-12">🚀</div>
        </div>

        <!-- Planeta Inferior -->
        <div class="relative z-10 pb-4 flex flex-col items-center">
          <div class="text-2xl filter drop-shadow-[0_0_8px_rgba(255,0,127,0.5)]">🌕</div>
        </div>
      </aside>

      <!-- Centro: O Monitor CRT Interativo Vivo -->
      <div class="relative flex-1 h-full min-w-0 flex flex-col overflow-hidden bg-black">
        <slot />
      </div>

      <!-- Asa Direita do Bezel: Tubo Néon Magenta + Planeta Marte e Explosões -->
      <aside class="relative w-12 sm:w-20 md:w-28 shrink-0 bg-[#060813] border-l-2 border-[#162035] overflow-hidden hidden sm:flex flex-col justify-between p-2 select-none">
        <!-- Tubo Néon Vertical Magenta da Borda Direita -->
        <div class="absolute top-0 bottom-0 right-1 w-1.5 rounded-full bg-fuchsia-500 shadow-[0_0_15px_#ff007f,0_0_30px_#ff007f] animate-pulse"></div>

        <!-- Arte Espacial Direita: Planeta Vermelho -->
        <div class="relative z-10 pt-4 flex flex-col items-center">
          <div class="text-3xl md:text-4xl filter drop-shadow-[0_0_12px_rgba(255,59,48,0.7)]">🔴</div>
          <span class="text-[8px] font-pixel-title text-rose-400/80 tracking-widest mt-1">CORE-9</span>
        </div>

        <!-- Naves e Lasers Ciano -->
        <div class="relative z-10 flex flex-col items-center gap-3">
          <div class="text-lg -rotate-45 animate-pulse">🛸</div>
          <div class="w-1 h-6 bg-cyan-400 rounded-full shadow-[0_0_8px_#00f0ff] -rotate-12"></div>
          <div class="text-base rotate-12">💥</div>
        </div>

        <!-- Terra / Globo Inferior -->
        <div class="relative z-10 pb-4 flex flex-col items-center">
          <div class="text-2xl filter drop-shadow-[0_0_8px_rgba(0,240,255,0.6)]">🌍</div>
        </div>
      </aside>
    </div>

    <!-- 3. Base: Deck de Controle do Gabinete (Botões Iluminados e Entrada de Ficha) -->
    <footer class="relative z-30 w-full max-w-5xl h-14 sm:h-16 rounded-b-2xl bg-gradient-to-b from-[#141824] via-[#0d101a] to-[#080a12] border-b-2 border-x-2 border-cyan-500/50 shadow-[0_0_30px_rgba(0,240,255,0.3)] flex items-center justify-between px-4 sm:px-8">
      <!-- Painel de Ventilação e Grade Esquerda -->
      <div class="flex items-center gap-2">
        <div class="flex gap-1">
          <div class="w-1.5 h-6 bg-slate-800 rounded-sm"></div>
          <div class="w-1.5 h-6 bg-slate-800 rounded-sm"></div>
          <div class="w-1.5 h-6 bg-slate-800 rounded-sm"></div>
          <div class="w-1.5 h-6 bg-slate-800 rounded-sm"></div>
        </div>
        <span class="text-[9px] font-pixel-title text-slate-500 hidden md:inline">COOLING VENTS</span>
      </div>

      <!-- Centro: Entrada de Ficha (Insert Coin) Interativa com Efeito Sonoro -->
      <div class="flex items-center gap-4">
        <button
          type="button"
          on:click={handleInsertCoin}
          class="flex items-center gap-2 px-3 py-1 rounded-lg bg-black/80 border-2 border-amber-400/80 hover:border-amber-300 text-amber-300 hover:text-amber-200 font-pixel-title text-xs tracking-wider shadow-[0_0_12px_rgba(251,191,36,0.3)] transition cursor-pointer active:scale-95"
          title="Inserir Ficha Retrô (Toca Fanfarra 8-bit)"
        >
          <span class="w-2 h-2 rounded-full bg-amber-400 animate-ping"></span>
          <span>INSERT COIN (1 CREDIT)</span>
        </button>
      </div>

      <!-- Botões de Ação do Arcade Iluminados (1P / 2P / Botões A, B, C) -->
      <div class="flex items-center gap-2">
        <div class="flex items-center gap-1.5">
          <div class="w-4 h-4 sm:w-5 sm:h-5 rounded-full bg-rose-500 border-2 border-rose-300 shadow-[0_0_8px_#ff007f] cursor-pointer" title="Arcade Button 1"></div>
          <div class="w-4 h-4 sm:w-5 sm:h-5 rounded-full bg-cyan-400 border-2 border-cyan-200 shadow-[0_0_8px_#00f0ff] cursor-pointer" title="Arcade Button 2"></div>
          <div class="w-4 h-4 sm:w-5 sm:h-5 rounded-full bg-emerald-400 border-2 border-emerald-200 shadow-[0_0_8px_#39ff14] cursor-pointer" title="Arcade Button 3"></div>
        </div>
        <span class="text-[9px] font-pixel-title text-slate-400 hidden lg:inline ml-2">1P READY</span>
      </div>
    </footer>
  </div>
{:else}
  <!-- Modo Limpo / Cyber-Deck Direto (Full Bleed) -->
  <div class="relative w-full h-full flex flex-col bg-[#050811] overflow-hidden">
    <!-- Botão Flutuante para Retornar ao Modo Arcade -->
    <div class="absolute top-3 right-5 z-50">
      <button
        type="button"
        on:click={handleToggleMode}
        class="px-2.5 py-1 rounded-lg bg-cyan-950/80 hover:bg-cyan-900 border border-cyan-500/50 text-cyan-300 font-pixel-title text-[10px] tracking-wider flex items-center gap-1.5 transition cursor-pointer shadow-[0_0_12px_rgba(0,240,255,0.3)]"
        title="Alternar para Modo Gabinete Fliperama Arcade"
      >
        <Gamepad2 size={13} />
        <span>MODO ARCADE</span>
      </button>
    </div>

    <slot />
  </div>
{/if}
