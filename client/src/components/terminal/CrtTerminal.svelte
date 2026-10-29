<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    activeProject,
    sendCommandToActiveProject,
    selectDroidInActiveProject,
    deleteProjectMessage,
    clearProjectMessages,
    type ProjectTerminal,
  } from '../../lib/stores/projectStore';
  import { isBacklogOpen } from '../../lib/stores/chatStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { openDroidForge } from '../../lib/stores/droidRosterStore';
  import {
    workspaceFiles,
    terminalViewTab,
    droidViewFilter,
    openFileInVim,
    setDroidFilter,
  } from '../../lib/stores/workspaceFilesStore';
  import SparkMascot from './SparkMascot.svelte';
  import PipelineStepper from './PipelineStepper.svelte';
  import VimInspector from './VimInspector.svelte';
  import {
    Send,
    Mic,
    ListTodo,
    Sparkles,
    Terminal,
    Cpu,
    HardDrive,
    Wifi,
    Clock,
    FolderGit2,
    Bot,
    CheckCircle2,
    Layers,
    Volume2,
    VolumeX,
    FileCode2,
    Filter,
    Key,
    Trash2,
    ChevronDown,
  } from '@lucide/svelte';
  import { openAiSettings, aiConfig, hasConfiguredApiKey } from '../../lib/stores/aiConfigStore';
  import ModelSelectorDropdown from './ModelSelectorDropdown.svelte';

  let isModelMenuOpen = false;

  async function handleClearMessages() {
    if (confirm(`Deseja limpar todo o histórico de mensagens e tarefas de "${$activeProject.name}"?`)) {
      await clearProjectMessages($activeProject.id);
      soundFX.playKeyClick();
    }
  }

  async function handleDeleteMessage(msgId: string) {
    await deleteProjectMessage($activeProject.id, msgId);
    soundFX.playKeyClick();
  }

  let commandInput = '';
  let chatScrollContainer: HTMLElement;
  let clockTime = '00:00:00';
  let isMicActive = false;
  let isMuted = false;

  // Arquivos do projeto e filtro de droid individual
  $: currentProjectId = $activeProject?.id || 'proj-alfa';
  $: projectFiles = $workspaceFiles[currentProjectId] || [];
  $: activeFilteredDroid = $droidViewFilter
    ? $activeProject?.droids.find((d) => d.id === $droidViewFilter)
    : null;
  $: droidFiles = activeFilteredDroid
    ? projectFiles.filter((f) => f.authorDroid.toLowerCase() === activeFilteredDroid?.name.toLowerCase())
    : [];

  $: displayedMessages = ($activeProject?.messages || []).filter((msg) => {
    if (!$droidViewFilter) return true;
    if (msg.sender === 'user' || msg.sender === 'system') return true;
    if (!activeFilteredDroid) return true;
    return (
      msg.agentName?.toLowerCase() === activeFilteredDroid.name.toLowerCase() ||
      msg.agentRole?.toLowerCase() === activeFilteredDroid.role.toLowerCase()
    );
  });

  // Estado do mascote SPARK-01
  let mascotState: 'idle' | 'speaking' | 'thinking' | 'celebrate' = 'idle';
  let lastProcessedMsgKey = '';

  // Monitora as mensagens do projeto ativo para reagir com o mascote
  $: {
    const msgs = $activeProject?.messages || [];
    const lastMsg = msgs[msgs.length - 1];
    if (lastMsg) {
      const currentKey = `${lastMsg.id}:${lastMsg.status || ''}`;
      if (currentKey !== lastProcessedMsgKey) {
        lastProcessedMsgKey = currentKey;

        // Apenas reage com celebração se for uma finalização recente após o mount
        if (lastMsg.sender === 'user') {
          mascotState = 'thinking';
        } else if (lastMsg.status === 'STREAMING') {
          mascotState = 'speaking';
        } else if (lastMsg.status === 'DONE' && lastMsg.filesModified) {
          mascotState = 'celebrate';
          setTimeout(() => {
            if (mascotState === 'celebrate') mascotState = 'idle';
          }, 3500);
        } else {
          mascotState = 'idle';
        }
      }
    }
  }

  // Auto-scroll sempre que novas mensagens chegarem
  $: if ($activeProject?.messages) {
    scrollToBottom();
  }

  async function scrollToBottom() {
    await tick();
    if (chatScrollContainer) {
      chatScrollContainer.scrollTop = chatScrollContainer.scrollHeight;
    }
  }

  onMount(() => {
    // Relógio em tempo real
    const timer = setInterval(() => {
      const now = new Date();
      clockTime = now.toTimeString().split(' ')[0];
    }, 1000);

    return () => clearInterval(timer);
  });

  function handleSubmit() {
    if (!commandInput.trim()) return;
    const text = commandInput;
    commandInput = '';
    sendCommandToActiveProject(text);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
      soundFX.playKeyClick();
    }
  }

  function toggleAudio() {
    isMuted = soundFX.toggleMute();
  }

  function toggleMic() {
    isMicActive = !isMicActive;
    if (isMicActive) {
      commandInput = 'Refatorar modulos de persistencia e verificar testes';
      setTimeout(() => {
        isMicActive = false;
      }, 1500);
    }
  }

  function openBacklog() {
    isBacklogOpen.set(true);
  }
</script>

<!-- CRT Monitor Master Container com Papel de Parede Temático The Grid Metropolis -->
<div class="relative w-full h-full flex items-center justify-center p-2 sm:p-4 select-none overflow-hidden bg-[#020409]">
  <!-- 0. Fundo Temático / Papel de Parede The Grid Metropolis -->
  <div class="absolute inset-0 z-0 pointer-events-none overflow-hidden select-none">
    <!-- Imagem de Fundo Oficial em Alta Resolução da Metrópole e Data Ocean -->
    <img
      src="/assets/grid_metropolis_wallpaper_city.jpg"
      alt="The Grid Metropolis City Artwork"
      class="w-full h-full object-cover object-center opacity-85 filter brightness-95 contrast-115 scale-[1.01]"
    />

    <!-- Vinheta Suave e Backglow Escuro apenas no Centro (Atrás do CRT) para Legibilidade -->
    <div class="absolute inset-0 bg-gradient-to-t from-[#020409]/80 via-transparent to-[#020409]/60"></div>
    <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_#020409ee_20%,_#020409aa_50%,_transparent_80%)]"></div>

    <!-- Grade Isométrica Futurista Suave no Fundo -->
    <div
      class="absolute inset-0 opacity-15"
      style="background-image: linear-gradient(rgba(0, 240, 255, 0.2) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 240, 255, 0.2) 1px, transparent 1px); background-size: 40px 40px;"
    ></div>

    <!-- Faixas Laterais Decorativas HUD (Visíveis nos espaços vazios laterais marcados pelo Arquiteto) -->
    <!-- LADO ESQUERDO: Telemetria do Nexus & Data Ocean -->
    <div class="absolute top-1/4 left-3 xl:left-6 flex-col gap-3 hidden md:flex text-cyan-400/80 font-pixel-title text-[9px] tracking-widest max-w-[130px] xl:max-w-[160px]">
      <div class="flex items-center gap-1.5 text-cyan-300 drop-shadow-[0_0_8px_#00f0ff]">
        <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
        <span class="font-bold text-[10px]">THE GRID METROPOLIS</span>
      </div>
      <div class="p-2 rounded bg-black/60 border border-cyan-500/30 backdrop-blur-sm space-y-1 shadow-[0_0_15px_rgba(0,240,255,0.15)]">
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>DISTRICT:</span>
          <span class="text-cyan-300 font-bold">NEXUS-01</span>
        </div>
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>DATA OCEAN:</span>
          <span class="text-emerald-400 font-bold">-420m FLUID</span>
        </div>
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>MESH LINK:</span>
          <span class="text-cyan-300 font-bold">ACTIVE 99.9%</span>
        </div>
        <div class="w-full bg-cyan-950/60 h-1 rounded-full overflow-hidden mt-1 border border-cyan-500/30">
          <div class="bg-cyan-400 h-full w-[78%] animate-pulse"></div>
        </div>
      </div>
      <div class="text-[8px] text-slate-500 font-mono tracking-normal leading-tight">
        LAT: -23.55 // LON: -46.63<br />
        SUB-OCEAN HARVEST: ONLINE
      </div>
    </div>

    <!-- Tubo Néon Vertical Ciano na Borda Esquerda -->
    <div class="absolute top-12 bottom-12 left-1 w-1 rounded-full bg-gradient-to-b from-transparent via-cyan-400 to-transparent shadow-[0_0_15px_#00f0ff] opacity-75 hidden lg:block animate-pulse"></div>

    <!-- LADO DIREITO: Telemetria do The Core & Cyber-Deck Workstation -->
    <div class="absolute top-1/4 right-3 xl:right-6 flex-col gap-3 hidden md:flex items-end text-fuchsia-400/80 font-pixel-title text-[9px] tracking-widest max-w-[130px] xl:max-w-[160px] text-right">
      <div class="flex items-center gap-1.5 text-fuchsia-300 drop-shadow-[0_0_8px_#ff007f]">
        <span class="font-bold text-[10px]">THE CORE // OPERATOR</span>
        <span class="w-2 h-2 rounded-full bg-fuchsia-400 animate-ping"></span>
      </div>
      <div class="p-2 rounded bg-black/60 border border-fuchsia-500/30 backdrop-blur-sm space-y-1 shadow-[0_0_15px_rgba(255,0,127,0.15)] w-full">
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>CONSOLE:</span>
          <span class="text-fuchsia-300 font-bold">CYBER-DECK</span>
        </div>
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>AI RUNNER:</span>
          <span class="text-cyan-300 font-bold">CONNECTED</span>
        </div>
        <div class="flex justify-between text-[8px] text-slate-400">
          <span>FPS:</span>
          <span class="text-emerald-400 font-bold">60 WEBGL</span>
        </div>
        <div class="w-full bg-fuchsia-950/60 h-1 rounded-full overflow-hidden mt-1 border border-fuchsia-500/30">
          <div class="bg-fuchsia-400 h-full w-[92%] animate-pulse"></div>
        </div>
      </div>
      <div class="text-[8px] text-slate-500 font-mono tracking-normal leading-tight">
        BUS: DUAL-CHANNEL AXUM<br />
        TELEMETRY: SYNCHRONIZED
      </div>
    </div>

    <!-- Tubo Néon Vertical Magenta na Borda Direita -->
    <div class="absolute top-12 bottom-12 right-1 w-1 rounded-full bg-gradient-to-b from-transparent via-fuchsia-500 to-transparent shadow-[0_0_15px_#ff007f] opacity-75 hidden lg:block animate-pulse"></div>
  </div>

  <!-- Cabos coloridos decorativos de fundo saindo do monitor e entrando no conduíte da parede -->
  <div class="absolute top-1 right-8 xl:right-16 w-72 h-28 pointer-events-none opacity-85 hidden lg:block z-10">
    <svg viewBox="0 0 220 90" class="w-full h-full filter drop-shadow-[0_0_8px_rgba(0,240,255,0.4)]">
      <path d="M 10 80 C 60 15, 130 75, 210 20" fill="none" stroke="#ff007f" stroke-width="3" />
      <path d="M 20 88 C 80 25, 140 85, 215 30" fill="none" stroke="#00f0ff" stroke-width="3" />
      <path d="M 5 70 C 50 10, 120 55, 205 12" fill="none" stroke="#ffaa00" stroke-width="2.5" />
      <path d="M 30 88 C 70 35, 150 80, 218 40" fill="none" stroke="#39ff14" stroke-width="2.5" />
      <!-- Conector / Soquete na parede da estação -->
      <rect x="205" y="8" width="12" height="38" rx="3" fill="#141c2c" stroke="#00f0ff" stroke-width="1.5" />
      <circle cx="211" cy="15" r="2" fill="#ff007f" />
      <circle cx="211" cy="23" r="2" fill="#00f0ff" />
      <circle cx="211" cy="31" r="2" fill="#ffaa00" />
      <circle cx="211" cy="39" r="2" fill="#39ff14" />
    </svg>
  </div>

  <!-- Carcaça do Monitor CRT (Centralizada com Efeito de Glow Neon sobre o Fundo Temático) -->
  <div class="relative z-10 w-full max-w-6xl h-full max-h-[920px] rounded-[36px] p-3 sm:p-5 bg-gradient-to-b from-[#1b1e28] via-[#141720] to-[#0e1017] border-4 border-[#2b3142] shadow-[0_0_80px_rgba(0,0,0,0.95),0_0_35px_rgba(0,240,255,0.18),inset_0_2px_4px_rgba(255,255,255,0.15)] flex flex-col justify-between">
    <!-- Tela de Vidro CRT Curvada com Scanlines -->
    <div class="relative w-full h-full rounded-[26px] bg-[#03070d] border-2 border-[#162338] shadow-[inset_0_0_45px_rgba(0,0,0,0.9)] overflow-hidden flex flex-col justify-between crt-glass">
      <!-- Efeito de Scanlines e Fósforo Verde/Azul -->
      <div class="crt-scanlines pointer-events-none absolute inset-0 z-20 opacity-40"></div>
      <div class="crt-vignette pointer-events-none absolute inset-0 z-20"></div>

      <!-- Fluxo de Código em Fundo Escuro (Matrix Cyberpunk) -->
      <div class="absolute inset-0 z-0 pointer-events-none opacity-10 font-mono text-[10px] text-emerald-400 p-6 leading-tight select-none overflow-hidden">
        <div>c.sondoeov(); ~ rudrroret();</div>
        <div>else &#123; carnto("workspace.logs: "prec); &#125;</div>
        <div>consterarganent = new DroidHub();</div>
        <div class="mt-8">const mose ereste = contmad();</div>
        <div>vormon = questehozdoe(tak.toge);</div>
        <div>extagsersorehese(khadswd);</div>
        <div>return (snaitetxleye-termineunal);</div>
        <div class="mt-8">conso nevtration centaxwoste;</div>
        <div>conso sercse scanenndce0(); mover;</div>
        <div>conto (petarn w-enener.aw.solectian)) &#123;</div>
        <div>&nbsp;&nbsp;METROPOLIS_CORE "_ L</div>
        <div>&#125;</div>
      </div>

      <!-- Zona de Voo 2D Livre do Droid SPARK-01 por Todo o Monitor CRT -->
      <div class="absolute inset-0 pointer-events-none z-40 overflow-hidden">
        <SparkMascot
          state={mascotState}
          label={$activeProject.droids.find((d) => d.id === $activeProject.activeDroidId)?.name || 'SPARK-01'}
          wander={true}
          on:click={() => {
            mascotState = 'celebrate';
            setTimeout(() => (mascotState = 'idle'), 3000);
          }}
        />
      </div>

      <!-- 1. Cabeçalho Superior de Telemetria Dinâmico por Projeto -->
      <header class="relative z-30 px-4 sm:px-6 py-3 border-b border-cyan-500/20 bg-slate-950/60 backdrop-blur-md flex items-center justify-between gap-4">
        <!-- Título do Projeto e Path em Pixel Font -->
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-2">
            <h1
              class="text-base sm:text-lg font-bold font-pixel-title tracking-wider drop-shadow-[0_0_10px_rgba(0,240,255,0.7)] flex items-center gap-2"
              style="color: {$activeProject.accentColor}"
            >
              <span>{$activeProject.name.toUpperCase()}</span>
            </h1>
            <span
              class="text-[9px] px-2 py-0.5 rounded font-pixel-title font-bold uppercase border"
              style="background: {$activeProject.accentColor}18; color: {$activeProject.accentColor}; border-color: {$activeProject.accentColor}40"
            >
              CORE v0.3
            </span>
          </div>

          <div class="hidden xl:flex items-center gap-1.5 text-[10px] text-slate-500 font-mono px-2 py-0.5 rounded bg-black/40 border border-slate-800">
            <FolderGit2 size={12} class="text-cyan-400" />
            <span class="truncate max-w-[200px]">{$activeProject.path}</span>
          </div>
        </div>

        <!-- Indicadores de Telemetria em Fonte Pixelada -->
        <div class="flex items-center gap-3 sm:gap-6 font-pixel-title text-[11px]">
          <div class="flex items-center gap-1.5 text-emerald-400 drop-shadow-[0_0_6px_rgba(52,211,153,0.5)]">
            <Cpu class="w-3.5 h-3.5" />
            <span>CPU [{$activeProject.metrics.cpu}]</span>
          </div>

          <div class="flex items-center gap-1.5 text-cyan-400 drop-shadow-[0_0_6px_rgba(6,182,212,0.5)] hidden sm:flex">
            <HardDrive class="w-3.5 h-3.5" />
            <span>MEM [{$activeProject.metrics.mem}]</span>
          </div>

          <div class="flex items-center gap-1.5 text-emerald-400">
            <Wifi class="w-3.5 h-3.5 animate-pulse" />
            <span>NET: ON</span>
          </div>

          <div class="px-2.5 py-1 rounded-lg bg-emerald-950/60 border border-emerald-500/50 text-emerald-300 font-bold tracking-widest flex items-center gap-1.5 shadow-[0_0_12px_rgba(52,211,153,0.4)]">
            <Clock class="w-3.5 h-3.5" />
            <span class="font-pixel-terminal text-sm">{clockTime}</span>
          </div>

          <!-- Botão Ativar/Desativar Efeitos Sonoros 8-bit -->
          <button
            type="button"
            on:click={toggleAudio}
            class="p-1 sm:px-2 sm:py-1 rounded-lg border font-pixel-title text-[10px] transition flex items-center gap-1 cursor-pointer {isMuted
              ? 'bg-slate-900/60 border-slate-800 text-slate-500 hover:text-slate-300'
              : 'bg-cyan-950/60 border-cyan-500/50 text-cyan-300 shadow-[0_0_8px_rgba(6,182,212,0.3)] hover:border-cyan-400'}"
            title={isMuted ? 'Áudio 8-bit Mutado' : 'Áudio 8-bit Ativo'}
          >
            {#if isMuted}
              <VolumeX class="w-3.5 h-3.5" />
              <span class="hidden md:inline font-mono text-[9px]">MUDO</span>
            {:else}
              <Volume2 class="w-3.5 h-3.5 animate-pulse text-cyan-400" />
              <span class="hidden md:inline font-mono text-[9px]">SFX 8-BIT</span>
            {/if}
          </button>
        </div>
      </header>

      <!-- 2. Sub-Barra de Métricas e Progresso da Tarefa Ativa -->
      <div class="relative z-30 px-4 sm:px-6 py-2 border-b border-cyan-500/20 bg-slate-950/40 flex items-center justify-between text-xs font-pixel-title overflow-visible gap-4">
        <div class="flex items-center gap-2.5">
          <!-- Botão Dropdown Rápido de Seleção de Modelo (Codex / Reasoning) -->
          <div class="relative">
            <button
              type="button"
              on:click={() => (isModelMenuOpen = !isModelMenuOpen)}
              class="model-dropdown-trigger flex items-center gap-1.5 px-2.5 py-1 rounded border text-cyan-300 font-bold text-[11px] shadow-sm hover:border-cyan-400 transition cursor-pointer shrink-0 {hasConfiguredApiKey()
                ? 'bg-cyan-950/50 border-cyan-500/50 hover:bg-cyan-900/60 shadow-[0_0_10px_rgba(0,240,255,0.25)]'
                : 'bg-amber-950/40 border-amber-500/50 text-amber-300 hover:bg-amber-900/40'}"
              title="Selecionar Modelo & Reasoning Effort"
            >
              <Cpu class="w-3.5 h-3.5 text-cyan-400" />
              <span>
                {$aiConfig.provider === 'codex'
                  ? ($aiConfig.model === 'default' ? 'DEFAULT' : $aiConfig.model.toUpperCase())
                  : $aiConfig.model.toUpperCase()}
              </span>
              {#if $aiConfig.provider === 'codex'}
                <span class="text-[8.5px] px-1 py-0.2 rounded bg-black/50 text-emerald-400 font-mono border border-emerald-500/30">
                  {$aiConfig.reasoningEffort === 'low' ? 'LIGHT' : $aiConfig.reasoningEffort === 'medium' ? 'MED' : $aiConfig.reasoningEffort === 'high' ? 'HIGH' : 'EXTRA'}
                </span>
              {/if}
              <ChevronDown size={11} class="text-cyan-400" />
            </button>

            <!-- Dropdown Popover Idêntico ao Codex -->
            <ModelSelectorDropdown isOpen={isModelMenuOpen} on:close={() => (isModelMenuOpen = false)} />
          </div>

          <!-- Botão de Configuração de Chaves & Provedor -->
          <button
            type="button"
            on:click={openAiSettings}
            class="p-1 rounded border border-slate-700 bg-slate-900/80 text-slate-400 hover:text-cyan-300 hover:border-cyan-500 transition cursor-pointer"
            title="Abrir Configurações Globais de IA e Chaves de API"
          >
            <Key class="w-3.5 h-3.5" />
          </button>

          <!-- Indicador Dinâmico de Progresso (%) enquanto os droids trabalham -->
          {#if $activeProject.pipeline.isActive}
            {@const currentDroid = $activeProject.droids.find((d) => d.id === $activeProject.pipeline.currentDroidId)}
            <div
              class="flex items-center gap-2 px-3 py-1 rounded bg-slate-950/80 border text-[11px] font-mono tracking-wide shadow-md transition-all animate-pulse"
              style="border-color: {currentDroid?.accentColor || '#00f0ff'}; box-shadow: 0 0 12px {currentDroid?.accentColor || '#00f0ff'}40;"
            >
              <span class="w-2 h-2 rounded-full animate-ping" style="background: {currentDroid?.accentColor || '#00f0ff'}"></span>
              <span class="text-slate-200 uppercase font-bold">
                {currentDroid?.name || 'DROID'}:
              </span>
              <span class="font-pixel-terminal text-sm font-bold" style="color: {currentDroid?.accentColor || '#00f0ff'}">
                {$activeProject.pipeline.progressPercent}%
              </span>
            </div>
          {:else if $activeProject.pipeline.stage === 'COMPLETED'}
            <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-emerald-950/50 border border-emerald-500/40 text-emerald-400 text-[10px] font-pixel-title">
              <CheckCircle2 size={12} />
              <span>ESTEIRA CONCLUÍDA [100%]</span>
            </div>
          {/if}
        </div>

        <div class="flex items-center gap-3 text-slate-300 text-[11px] shrink-0">
          <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-900/70 border border-slate-800">
            <span class="text-slate-400">TOKENS IN:</span>
            <span class="font-bold text-cyan-400 font-pixel-terminal text-sm">{$activeProject.metrics.tokensIn.toLocaleString()}</span>
          </div>

          <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-900/70 border border-slate-800">
            <span class="text-slate-400">TOKENS OUT:</span>
            <span class="font-bold text-fuchsia-400 font-pixel-terminal text-sm">{$activeProject.metrics.tokensOut.toLocaleString()}</span>
          </div>

          <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-900/70 border border-slate-800">
            <span class="text-slate-400">LATÊNCIA:</span>
            <span class="font-bold text-emerald-400 font-pixel-terminal text-sm">{$activeProject.metrics.latencyMs}ms</span>
          </div>

          <!-- Botão Limpar Chat / Tarefas do Workspace -->
          <button
            type="button"
            on:click={handleClearMessages}
            class="flex items-center gap-1.5 px-2.5 py-1 rounded border border-rose-500/30 bg-rose-950/30 hover:bg-rose-950/60 text-rose-300 text-[10px] font-pixel-title transition cursor-pointer"
            title="Limpar histórico de mensagens deste workspace"
          >
            <Trash2 size={12} />
            <span class="hidden sm:inline">LIMPAR CHAT</span>
          </button>
        </div>
      </div>

      <!-- 2.1 Barra do Pipeline Sequencial Multi-Droid (Handoff Automático) -->
      <PipelineStepper pipeline={$activeProject.pipeline} droids={$activeProject.droids} />

      <!-- 2.2 Barra de Controle de Visualização: CONSOLE vs INSPETOR VIM e Filtro de Droids -->
      <div class="relative z-30 px-4 py-1.5 bg-[#060b16] border-b border-cyan-500/20 flex items-center justify-between text-xs font-mono select-none overflow-x-auto gap-2">
        <div class="flex items-center gap-1.5 shrink-0">
          <!-- Botão Modo Console / Chat -->
          <button
            type="button"
            on:click={() => terminalViewTab.set('CONSOLE')}
            class="px-2.5 py-1 rounded-lg font-pixel-title text-[10px] tracking-wider flex items-center gap-1.5 transition cursor-pointer {$terminalViewTab === 'CONSOLE'
              ? 'bg-cyan-500 text-black font-bold shadow-[0_0_10px_rgba(0,240,255,0.4)]'
              : 'bg-slate-900 text-slate-400 hover:text-slate-200 border border-slate-800'}"
          >
            <Terminal size={12} />
            <span>CONSOLE</span>
          </button>

          <!-- Botão Modo Vim Inspector -->
          <button
            type="button"
            on:click={() => terminalViewTab.set('VIM_INSPECTOR')}
            class="px-2.5 py-1 rounded-lg font-pixel-title text-[10px] tracking-wider flex items-center gap-1.5 transition cursor-pointer {$terminalViewTab === 'VIM_INSPECTOR'
              ? 'bg-emerald-500 text-black font-bold shadow-[0_0_10px_rgba(52,211,153,0.4)]'
              : 'bg-slate-900 text-slate-400 hover:text-slate-200 border border-slate-800'}"
          >
            <FileCode2 size={12} />
            <span>INSPETOR VIM ({projectFiles.length})</span>
          </button>
        </div>

        <!-- Filtro de Droid Específico (Visão Individual vs Todos) -->
        <div class="flex items-center gap-1.5 shrink-0 text-[11px]">
          <span class="text-slate-500 text-[10px] hidden sm:inline">Visão:</span>
          <button
            type="button"
            on:click={() => setDroidFilter(null)}
            class="px-2 py-0.5 rounded text-[10px] font-pixel-title tracking-wider transition cursor-pointer {!$droidViewFilter
              ? 'bg-cyan-950 border border-cyan-400 text-cyan-300'
              : 'bg-slate-900 text-slate-400 hover:text-slate-200 border border-slate-800'}"
          >
            TODOS ({$activeProject.droids.length})
          </button>

          {#each $activeProject.droids as droid}
            {@const isFilterActive = $droidViewFilter === droid.id}
            <button
              type="button"
              on:click={() => setDroidFilter(droid.id)}
              class="px-2 py-0.5 rounded text-[10px] font-pixel-title tracking-wider flex items-center gap-1 transition cursor-pointer {isFilterActive
                ? 'shadow-md ring-1'
                : 'bg-slate-900/80 text-slate-400 hover:text-slate-200 border border-slate-800'}"
              style="{isFilterActive ? `background: ${droid.accentColor}25; border: 1px solid ${droid.accentColor}; color: ${droid.accentColor}; ring-color: ${droid.accentColor};` : ''}"
            >
              <span>{droid.name}</span>
            </button>
          {/each}
        </div>
      </div>

      {#if $terminalViewTab === 'VIM_INSPECTOR'}
        <!-- 3. Modo Inspetor de Código Retrô (Vim/Nano) -->
        <div class="relative z-20 flex-1 overflow-hidden">
          <VimInspector />
        </div>
      {:else}
        <!-- 3. Área de Diálogos / Stream de Mensagens do Terminal do Projeto -->
        <main
          bind:this={chatScrollContainer}
          class="relative z-20 flex-1 p-4 sm:p-6 overflow-y-auto space-y-4 leading-relaxed custom-scrollbar"
        >
          <!-- Mensagem de Boas-Vindas do Sistema do Projeto -->
          <div class="text-center py-1">
            <span
              class="px-3 py-1 rounded-full border text-[10px] font-pixel-title tracking-wider shadow-[0_0_10px_rgba(0,240,255,0.2)]"
              style="background: #070d18; border-color: {$activeProject.accentColor}50; color: {$activeProject.accentColor}"
            >
              ⚡ TERMINAL CRT OPERACIONAL // WORKSPACE: {$activeProject.name.toUpperCase()} // SESSÃO ATIVA
            </span>
          </div>

          <!-- Banner de Foco Individual do Droid Selecionado -->
          {#if activeFilteredDroid}
            <div
              class="p-3.5 sm:p-4 rounded-xl border-2 bg-[#090d19]/95 shadow-xl relative overflow-hidden select-none animate-fadeIn"
              style="border-color: {activeFilteredDroid.accentColor}; box-shadow: 0 0 25px {activeFilteredDroid.accentColor}25"
            >
              <!-- Feixe luminoso superior -->
              <div
                class="absolute top-0 left-0 right-0 h-[2px]"
                style="background: {activeFilteredDroid.accentColor}; box-shadow: 0 0 10px {activeFilteredDroid.accentColor}"
              ></div>

              <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                <div class="flex items-center gap-3">
                  <div
                    class="w-11 h-11 rounded-lg border-2 flex items-center justify-center bg-black/70 overflow-hidden shrink-0 shadow-md"
                    style="border-color: {activeFilteredDroid.accentColor}"
                  >
                    {#if activeFilteredDroid.avatarType === 'spark'}
                      <img src="/assets/spark/idle_0.png" alt={activeFilteredDroid.name} class="w-9 h-9 object-contain pixelated" />
                    {:else if activeFilteredDroid.avatarType === 'hacker'}
                      <span class="text-sm font-pixel-title font-bold" style="color: {activeFilteredDroid.accentColor}">&lt;/&gt;</span>
                    {:else if activeFilteredDroid.avatarType === 'scientist'}
                      <span class="text-sm font-pixel-title font-bold" style="color: {activeFilteredDroid.accentColor}">[EXP]</span>
                    {:else}
                      <span class="text-sm font-pixel-title font-bold" style="color: {activeFilteredDroid.accentColor}">🛡️</span>
                    {/if}
                  </div>

                  <div>
                    <div class="flex items-center gap-2">
                      <h3 class="font-pixel-title text-sm sm:text-base text-white font-bold tracking-wider">
                        CONTRIBUIÇÃO INDIVIDUAL: {activeFilteredDroid.name}
                      </h3>
                      <span
                        class="text-[9px] font-mono px-2 py-0.5 rounded border uppercase tracking-wider font-bold"
                        style="color: {activeFilteredDroid.accentColor}; background: {activeFilteredDroid.accentColor}18; border-color: {activeFilteredDroid.accentColor}40"
                      >
                        {activeFilteredDroid.role} // {activeFilteredDroid.planBadge}
                      </span>
                    </div>
                    <div class="text-xs text-slate-300 font-mono mt-0.5 flex items-center gap-2">
                      <span>Modelo: <strong class="text-cyan-300">{activeFilteredDroid.model}</strong></span>
                      <span>•</span>
                      <span>Tokens: <strong class="text-fuchsia-300">{(activeFilteredDroid.tokens / 1000).toFixed(1)}k</strong></span>
                      <span>•</span>
                      <span class="text-emerald-400">● {activeFilteredDroid.status}</span>
                    </div>
                  </div>
                </div>

                <!-- Ação para voltar a ver todos -->
                <button
                  type="button"
                  on:click={() => setDroidFilter(null)}
                  class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-pixel-title transition cursor-pointer self-start sm:self-auto shrink-0"
                >
                  ✕ Ver Todos os Droids
                </button>
              </div>

              <!-- Detalhe da atividade e arquivos criados por este droid -->
              <div class="mt-3 pt-2.5 border-t border-slate-800 flex flex-col gap-2">
                <div class="text-xs font-mono text-slate-300">
                  <span class="text-slate-500 uppercase font-bold text-[10px]">Atividade / Tarefa:</span>
                  <span class="ml-1 text-slate-100">{activeFilteredDroid.statusDetail}</span>
                </div>

                {#if droidFiles.length > 0}
                  <div class="flex items-center flex-wrap gap-2 pt-1 text-xs font-mono">
                    <span class="text-slate-500 text-[10px] uppercase font-bold">Artefatos Produzidos:</span>
                    {#each droidFiles as f}
                      <button
                        type="button"
                        on:click={() => openFileInVim(f.path)}
                        class="px-2 py-0.5 rounded bg-black/60 hover:bg-cyan-950 border text-xs font-mono flex items-center gap-1.5 transition cursor-pointer"
                        style="border-color: {activeFilteredDroid.accentColor}50; color: {activeFilteredDroid.accentColor};"
                        title="Abrir no Inspetor Vim"
                      >
                        <FileCode2 size={12} />
                        <span>{f.path}</span>
                        <span class="text-[9px] opacity-70">({f.linesCount} linhas)</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Renderização dos Diálogos deste Terminal (Filtrado ou Completo) -->
          {#each displayedMessages as msg (msg.id)}
            {#if msg.sender === 'user'}
              <!-- Balão do Usuário (Néon Verde Fluorescente alinhado à direita com fonte pixelada VT323) -->
              <div class="flex justify-end">
                <div class="max-w-[85%] sm:max-w-[70%] p-2.5 sm:p-3 rounded-xl bg-emerald-950/70 border-2 border-emerald-400/90 shadow-[0_0_15px_rgba(52,211,153,0.3)] text-emerald-200">
                  <div class="text-[10px] font-pixel-title text-emerald-400 font-bold uppercase tracking-wider mb-1 flex items-center justify-between gap-4">
                    <span>[ARQUITETO@{$activeProject.name.toUpperCase()}]: ~</span>
                    <div class="flex items-center gap-1.5">
                      <span class="text-emerald-500/70 text-[8.5px] font-pixel-terminal text-xs">{msg.timestamp}</span>
                      <button
                        type="button"
                        on:click={() => handleDeleteMessage(msg.id)}
                        class="opacity-50 hover:opacity-100 hover:text-rose-400 text-slate-400 transition p-0.5 cursor-pointer"
                        title="Excluir mensagem"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  </div>
                  <div class="font-pixel-terminal text-sm sm:text-base text-emerald-100 font-normal leading-snug tracking-wide">
                    {msg.content}
                  </div>
                </div>
              </div>
            {:else if msg.sender === 'agent'}
              <!-- Balão do Droid/Agente com cores e estilização por papel -->
              {@const isArchitect = msg.agentRole === 'ARQUITETO' || msg.agentName === 'SPARK-01'}
              {@const isCoder = msg.agentRole === 'PROGRAMADOR' || msg.agentName === 'Rusty'}
              {@const isReviewer = msg.agentRole === 'REVISOR_DE_CODIGO' || msg.agentName === 'Vanguard'}
              {@const accentBorder = isArchitect ? 'border-cyan-400/90' : isCoder ? 'border-emerald-400/90' : 'border-fuchsia-400/90'}
              {@const accentBg = isArchitect ? 'bg-cyan-950/60' : isCoder ? 'bg-emerald-950/60' : 'bg-fuchsia-950/60'}
              {@const accentShadow = isArchitect ? 'shadow-[0_0_20px_rgba(6,182,212,0.3)]' : isCoder ? 'shadow-[0_0_20px_rgba(52,211,153,0.3)]' : 'shadow-[0_0_20px_rgba(217,70,239,0.3)]'}
              {@const accentText = isArchitect ? 'text-cyan-300' : isCoder ? 'text-emerald-300' : 'text-fuchsia-300'}

              <div class="flex items-start gap-3">
                <!-- Miniatura da cabeça do droid com moldura temática -->
                <div
                  class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 border shadow-md"
                  style="background: #080c14; border-color: {isArchitect ? '#00f0ff' : isCoder ? '#39ff14' : '#ff007f'}; box-shadow: 0 0 10px {isArchitect ? '#00f0ff40' : isCoder ? '#39ff1440' : '#ff007f40'}"
                >
                  {#if isArchitect}
                    <span class="text-sm">🤖</span>
                  {:else if isCoder}
                    <span class="text-xs font-pixel-title text-emerald-400 font-bold">&lt;/&gt;</span>
                  {:else}
                    <span class="text-xs font-pixel-title text-fuchsia-400 font-bold">🛡️</span>
                  {/if}
                </div>

                <div class="max-w-[90%] sm:max-w-[78%] p-2.5 sm:p-3 rounded-xl {accentBg} border-2 {accentBorder} {accentShadow} text-slate-100">
                  <div class="text-[10px] font-pixel-title {accentText} font-bold uppercase tracking-wider mb-1 flex items-center justify-between gap-4">
                    <span class="flex items-center gap-1.5">
                      <span>[{msg.agentName || 'DROID'}] // {msg.agentRole || 'AGENTE'}: ~</span>
                      {#if msg.status === 'STREAMING'}
                        <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
                      {/if}
                    </span>
                    <div class="flex items-center gap-1.5">
                      <span class="opacity-70 text-[8.5px] font-pixel-terminal text-xs">[{msg.timestamp}]</span>
                      <button
                        type="button"
                        on:click={() => handleDeleteMessage(msg.id)}
                        class="opacity-50 hover:opacity-100 hover:text-rose-400 text-slate-400 transition p-0.5 cursor-pointer"
                        title="Excluir mensagem"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  </div>

                  <!-- Conteúdo da Resposta em Fonte Pixelada VT323 -->
                  <div class="font-pixel-terminal text-sm sm:text-base text-slate-100 font-normal leading-snug tracking-wide whitespace-pre-wrap">
                    {msg.content}
                    {#if msg.status === 'STREAMING'}
                      <span class="inline-block w-1.5 h-3.5 ml-1 bg-cyan-400 animate-pulse align-middle"></span>
                    {/if}
                  </div>

                  <!-- Processo de Pensamento (Chain-of-Thought) se existir -->
                  {#if msg.thinking}
                    <div class="mt-2 p-2 rounded-lg bg-black/60 border border-slate-700/60 text-slate-300 font-pixel-terminal text-xs sm:text-sm">
                      <div class="text-[10px] font-pixel-title text-amber-300 font-bold uppercase mb-1">🧠 Chain-of-Thought ({msg.agentName}):</div>
                      <div class="whitespace-pre-wrap leading-tight text-slate-200">{msg.thinking}</div>
                    </div>
                  {/if}

                  <!-- Arquivos Modificados / Artefatos com botão para abrir no Vim -->
                  {#if msg.filesModified && msg.filesModified.length > 0}
                    <div class="mt-2 pt-2 border-t border-slate-700/60 flex items-center flex-wrap gap-2 text-xs font-mono text-cyan-300">
                      <span class="flex items-center gap-1 text-emerald-400 shrink-0">
                        <CheckCircle2 size={13} />
                        <span>Artefatos:</span>
                      </span>
                      {#each msg.filesModified as file}
                        <button
                          type="button"
                          on:click={() => openFileInVim(file)}
                          class="px-2 py-0.5 rounded bg-cyan-950/80 hover:bg-cyan-900 border border-cyan-500/40 hover:border-cyan-400 text-cyan-300 text-[10px] font-mono flex items-center gap-1 transition cursor-pointer"
                          title="Abrir no Inspetor Vim"
                        >
                          <FileCode2 size={11} />
                          <span>{file}</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            {:else}
              <!-- Mensagem de Sistema / Evento -->
              <div class="text-center my-1.5 text-slate-400 font-pixel-terminal text-xs">
                <span class="px-2.5 py-1 rounded bg-slate-900/60 border border-slate-800">
                  {msg.content}
                </span>
              </div>
            {/if}
          {/each}

          {#if displayedMessages.length === 0}
            <div class="text-center py-10 text-slate-500 font-mono text-xs">
              Nenhuma contribuição registrada para este filtro de droid no momento.
            </div>
          {/if}
        </main>
      {/if}

      <!-- 4. Rodapé: Barra de Comando CLI -->
      <footer class="relative z-30 p-3 sm:p-5 border-t border-cyan-500/20 bg-slate-950/80 backdrop-blur-md">
        <!-- Barra de Comando Estilo Terminal CLI (Néon Ciano com Prompt >>> |_) -->
        <div class="relative flex items-center gap-2 sm:gap-2.5 p-2 sm:p-2.5 rounded-xl bg-slate-950/95 border-2 border-cyan-400/80 shadow-[0_0_20px_rgba(6,182,212,0.4)]">
          <!-- Prefixo CLI >>> -->
          <div class="flex items-center gap-1 font-pixel-terminal text-cyan-400 font-bold text-base sm:text-lg pl-2 select-none">
            <span class="animate-pulse">&gt;&gt;&gt;</span>
          </div>

          <!-- Campo de Digitação com Fonte Pixelada VT323 -->
          <div class="flex-1 relative flex items-center">
            <input
              type="text"
              bind:value={commandInput}
              on:keydown={handleKeyDown}
              placeholder="Digite seu comando para {$activeProject.name}... (ex: refatorar modulos ou compilar)"
              class="w-full bg-transparent text-cyan-100 placeholder:text-slate-500 font-pixel-terminal text-sm sm:text-base focus:outline-none transition leading-normal pl-1 tracking-wide"
            />
            <!-- Cursor piscante retrô -->
            {#if !commandInput}
              <span class="w-1.5 h-3.5 bg-cyan-400 animate-pulse ml-0.5 pointer-events-none"></span>
            {/if}
          </div>

          <!-- Botão Microfone / Voz -->
          <button
            on:click={toggleMic}
            class="p-2 sm:p-2.5 rounded-xl font-bold transition shrink-0 {isMicActive
              ? 'bg-fuchsia-500 text-slate-950 shadow-[0_0_15px_#d946ef] animate-bounce'
              : 'bg-slate-900/90 hover:bg-fuchsia-950/80 text-fuchsia-400 border border-fuchsia-500/40 hover:border-fuchsia-400 shadow-[0_0_10px_rgba(217,70,239,0.3)] cursor-pointer'}"
            title="Gravar comando por voz"
          >
            <Mic class="w-4 h-4" />
          </button>

          <!-- Botão Enviar / Despachar -->
          <button
            on:click={handleSubmit}
            disabled={!commandInput.trim()}
            class="p-2 sm:px-4 sm:py-2.5 rounded-xl font-pixel-title text-[11px] font-black uppercase tracking-wider flex items-center gap-1.5 transition shrink-0
            {commandInput.trim()
              ? 'bg-gradient-to-r from-cyan-400 to-blue-500 hover:from-cyan-300 hover:to-blue-400 text-slate-950 shadow-[0_0_20px_rgba(6,182,212,0.6)] cursor-pointer active:scale-95'
              : 'bg-slate-900 text-slate-600 cursor-not-allowed border border-slate-800'}"
            title="Executar comando"
          >
            <Send class="w-4 h-4 fill-current" />
            <span class="hidden sm:inline">Executar</span>
          </button>

          <!-- Divisor -->
          <div class="w-px h-6 bg-slate-800 hidden sm:block"></div>

          <!-- Botão Backlog / Missões -->
          <button
            on:click={openBacklog}
            class="p-2 sm:px-3 sm:py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white font-pixel-title text-[10px] flex items-center gap-1.5 transition shrink-0 relative cursor-pointer"
            title="Abrir Backlog de Missões"
          >
            <ListTodo class="w-4 h-4 text-cyan-400" />
            <span class="hidden md:inline font-bold">Missões</span>
          </button>
        </div>
      </footer>
    </div>
  </div>
</div>

<style>
  /* Efeito de Vidro Curvo CRT */
  .crt-glass {
    box-shadow: inset 0 0 60px rgba(0, 0, 0, 0.95), inset 0 0 15px rgba(6, 182, 212, 0.15);
  }

  /* Linhas de Varredura (Scanlines) */
  .crt-scanlines {
    background: linear-gradient(
      rgba(18, 16, 16, 0) 50%, 
      rgba(0, 0, 0, 0.45) 50%
    ), linear-gradient(
      90deg,
      rgba(255, 0, 0, 0.03),
      rgba(0, 255, 0, 0.01),
      rgba(0, 255, 0, 0.03)
    );
    background-size: 100% 3px, 6px 100%;
  }

  /* Vinheta periférica e aberração cromática suave */
  .crt-vignette {
    background: radial-gradient(circle at center, transparent 65%, rgba(0, 0, 0, 0.75) 100%);
  }
</style>
