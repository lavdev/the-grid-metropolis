<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    activeProject,
    sendCommandToActiveProject,
    selectDroidInActiveProject,
    deleteProjectMessage,
    clearProjectMessages,
  } from '../../lib/stores/projectStore';
  import { isBacklogOpen } from '../../lib/stores/chatStore';
  import { soundFX } from '../../lib/audio/soundFX';
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
    Terminal,
    Cpu,
    FolderGit2,
    CheckCircle2,
    Layers,
    Volume2,
    VolumeX,
    FileCode2,
    Key,
    Trash2,
    ChevronDown,
  } from '@lucide/svelte';
  import { openAiSettings, aiConfig } from '../../lib/stores/aiConfigStore';
  import ModelSelectorDropdown from './ModelSelectorDropdown.svelte';
  import { renderMarkdown } from '../../lib/markdown';

  export let connectionStatus: import('../../lib/socket/client').ConnectionState = 'CONNECTING';
  let isModelMenuOpen = false;

  async function handleClearMessages() {
    if (!$activeProject?.id) return;
    if (confirm(`Deseja limpar todo o histórico de mensagens e tarefas de "${$activeProject.name}"?`)) {
      await clearProjectMessages($activeProject.id);
      soundFX.playKeyClick();
    }
  }

  async function handleDeleteMessage(msgId: string) {
    if (!$activeProject?.id) return;
    await deleteProjectMessage($activeProject.id, msgId);
    soundFX.playKeyClick();
  }

  let commandInput = '';
  let chatScrollContainer: HTMLElement;
  let clockTime = '00:00:00';
  let isMicActive = false;
  let isMuted = false;
  let prefersReducedMotion = false;

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
    const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
    const syncMotionPreference = () => (prefersReducedMotion = motionPreference.matches);
    syncMotionPreference();
    motionPreference.addEventListener('change', syncMotionPreference);
    // Relógio em tempo real
    const timer = setInterval(() => {
      const now = new Date();
      clockTime = now.toTimeString().split(' ')[0];
    }, 1000);

    return () => {
      clearInterval(timer);
      motionPreference.removeEventListener('change', syncMotionPreference);
    };
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

<!-- City atmosphere surrounds a clean, readable operator surface. -->
<div class="cyber-deck">
  <div class="city-backdrop" aria-hidden="true">
    <img src="/assets/grid_metropolis_wallpaper_city.jpg" alt="" />
    <div class="city-shade"></div>
    <div class="city-grid"></div>
    <span class="district-marker font-code">NEXUS / THE GRID METROPOLIS</span>
  </div>
  <div class="deck-frame">
    <div class="deck-screen">
      <header class="deck-header">
        <div class="workspace-identity">
          <div class="eyebrow"><span class="identity-line"></span> OPERATOR CONSOLE <span class="text-slate-500">/ 01</span></div>
          <h1>{$activeProject.name}</h1>
          <div class="workspace-path font-code" title={$activeProject.path}>
            <FolderGit2 size={13} /> <span class="truncate">{$activeProject.path}</span>
          </div>
        </div>
        <div class="header-actions">
          <img class="mobile-companion" src="/assets/spark/idle_0.png" alt="SPARK-01" />
          <span class="connection-badge" class:connected={connectionStatus === 'CONNECTED'}>
            <span class="status-dot"></span>
            {connectionStatus === 'CONNECTED' ? 'Bridge online' : connectionStatus === 'CONNECTING' ? 'Conectando' : 'Bridge offline'}
          </span>
          <button type="button" on:click={toggleAudio} class="icon-button" title={isMuted ? 'Ativar áudio 8-bit' : 'Silenciar áudio 8-bit'} aria-pressed={!isMuted}>
            {#if isMuted}<VolumeX size={17} />{:else}<Volume2 size={17} />{/if}
          </button>
          <button type="button" on:click={openAiSettings} class="icon-button" title="Abrir configurações de IA"><Key size={17} /></button>
        </div>
      </header>

      <div class="deck-toolbar">
        <div class="model-controls">
          <div class="relative min-w-0">
            <button type="button" on:click={() => (isModelMenuOpen = !isModelMenuOpen)}
              class="model-dropdown-trigger model-trigger" aria-expanded={isModelMenuOpen} title="Selecionar modelo e nível de raciocínio">
              <Cpu size={16} class="shrink-0 text-cyan-300" />
              <span class="truncate">{$aiConfig.model === 'default' ? 'Modelo padrão' : $aiConfig.model}</span>
              {#if $aiConfig.provider === 'codex'}<span class="model-effort">{$aiConfig.reasoningEffort}</span>{/if}
              <ChevronDown size={14} class="shrink-0" />
            </button>
            <ModelSelectorDropdown isOpen={isModelMenuOpen} on:close={() => (isModelMenuOpen = false)} />
          </div>
          {#if $activeProject.pipeline.isActive}
            <span class="execution-state text-cyan-300"><span class="status-dot animate-pulse"></span> Em execução · {$activeProject.pipeline.progressPercent}%</span>
          {:else if $activeProject.pipeline.stage === 'COMPLETED'}
            <span class="execution-state text-emerald-300"><CheckCircle2 size={14} /> Concluído</span>
          {:else}
            <span class="execution-state text-slate-400">Aguardando instruções</span>
          {/if}
        </div>
        <details class="telemetry-panel">
          <summary><Layers size={14} /> Telemetria <ChevronDown size={13} /></summary>
          <div class="telemetry-popover font-code">
            <div class="telemetry-heading">WORKSPACE / MÉTRICAS</div>
            <dl>
              <div><dt>Tokens de entrada</dt><dd>{$activeProject.metrics.tokensIn.toLocaleString()}</dd></div>
              <div><dt>Tokens de saída</dt><dd>{$activeProject.metrics.tokensOut.toLocaleString()}</dd></div>
              <div><dt>Latência</dt><dd>{$activeProject.metrics.latencyMs} ms</dd></div>
              <div><dt>CPU</dt><dd>{$activeProject.metrics.cpu}</dd></div>
              <div><dt>Memória</dt><dd>{$activeProject.metrics.mem}</dd></div>
              <div><dt>Horário local</dt><dd>{clockTime}</dd></div>
            </dl>
            <button type="button" class="clear-history" on:click={handleClearMessages}><Trash2 size={14} /> Limpar histórico</button>
          </div>
        </details>
      </div>

      <!-- 2.1 Barra do Pipeline Sequencial Multi-Droid (Handoff Automático) -->
      <PipelineStepper pipeline={$activeProject.pipeline} droids={$activeProject.droids} />

      <!-- 2.2 Barra de Controle de Visualização: CONSOLE vs INSPETOR VIM e Filtro de Droids -->
      <div class="view-toolbar">
        <div class="flex items-center gap-1.5 shrink-0">
          <!-- Botão Modo Console / Chat -->
          <button
            type="button"
            on:click={() => terminalViewTab.set('CONSOLE')}
            aria-pressed={$terminalViewTab === 'CONSOLE'}
            class="px-2.5 py-1 rounded-lg font-ui text-[12px] tracking-normal flex items-center gap-1.5 transition cursor-pointer {$terminalViewTab === 'CONSOLE'
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
            aria-pressed={$terminalViewTab === 'VIM_INSPECTOR'}
            class="px-2.5 py-1 rounded-lg font-ui text-[12px] tracking-normal flex items-center gap-1.5 transition cursor-pointer {$terminalViewTab === 'VIM_INSPECTOR'
              ? 'bg-emerald-500 text-black font-bold shadow-[0_0_10px_rgba(52,211,153,0.4)]'
              : 'bg-slate-900 text-slate-400 hover:text-slate-200 border border-slate-800'}"
          >
            <FileCode2 size={12} />
            <span>INSPETOR VIM ({projectFiles.length})</span>
          </button>
        </div>

        <!-- Filtro de Droid Específico (Visão Individual vs Todos) -->
        <div class="droid-filters">
          <span class="text-slate-500 text-[12px] hidden sm:inline">Visão:</span>
          <button
            type="button"
            on:click={() => setDroidFilter(null)}
            aria-pressed={!$droidViewFilter}
            class="px-2 py-0.5 rounded text-[12px] font-ui tracking-normal transition cursor-pointer {!$droidViewFilter
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
              aria-pressed={isFilterActive}
              class="px-2 py-0.5 rounded text-[12px] font-ui tracking-normal flex items-center gap-1 transition cursor-pointer {isFilterActive
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
          class="conversation relative z-20 flex-1 min-h-0 overflow-y-auto space-y-6 custom-scrollbar"
        >
          <div class="session-banner">
            <span class="session-rule"></span><span><Terminal size={13} /> Sessão de trabalho</span><span class="session-rule"></span>
          </div>

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
                      <span class="text-sm font-ui font-bold" style="color: {activeFilteredDroid.accentColor}">&lt;/&gt;</span>
                    {:else if activeFilteredDroid.avatarType === 'scientist'}
                      <span class="text-sm font-ui font-bold" style="color: {activeFilteredDroid.accentColor}">[EXP]</span>
                    {:else}
                      <span class="text-sm font-ui font-bold" style="color: {activeFilteredDroid.accentColor}">🛡️</span>
                    {/if}
                  </div>

                  <div>
                    <div class="flex items-center gap-2">
                      <h3 class="font-ui text-sm sm:text-base text-white font-bold tracking-normal">
                        CONTRIBUIÇÃO INDIVIDUAL: {activeFilteredDroid.name}
                      </h3>
                      <span
                        class="text-[12px] font-mono px-2 py-0.5 rounded border uppercase tracking-normal font-bold"
                        style="color: {activeFilteredDroid.accentColor}; background: {activeFilteredDroid.accentColor}18; border-color: {activeFilteredDroid.accentColor}40"
                      >
                        {activeFilteredDroid.role} // {activeFilteredDroid.planBadge}
                      </span>
                    </div>
                    <div class="text-sm text-slate-300 font-mono mt-0.5 flex items-center gap-2">
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
            aria-pressed={!$droidViewFilter}
                  class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm font-ui transition cursor-pointer self-start sm:self-auto shrink-0"
                >
                  ✕ Ver Todos os Droids
                </button>
              </div>

              <!-- Detalhe da atividade e arquivos criados por este droid -->
              <div class="mt-3 pt-2.5 border-t border-slate-800 flex flex-col gap-2">
                <div class="text-sm font-mono text-slate-300">
                  <span class="text-slate-500 uppercase font-bold text-[12px]">Atividade / Tarefa:</span>
                  <span class="ml-1 text-slate-100">{activeFilteredDroid.statusDetail}</span>
                </div>

                {#if droidFiles.length > 0}
                  <div class="flex items-center flex-wrap gap-2 pt-1 text-sm font-mono">
                    <span class="text-slate-500 text-[12px] uppercase font-bold">Artefatos Produzidos:</span>
                    {#each droidFiles as f}
                      <button
                        type="button"
                        on:click={() => openFileInVim(f.path, currentProjectId)}
                        class="px-2 py-0.5 rounded bg-black/60 hover:bg-cyan-950 border text-sm font-mono flex items-center gap-1.5 transition cursor-pointer"
                        style="border-color: {activeFilteredDroid.accentColor}50; color: {activeFilteredDroid.accentColor};"
                        title="Abrir no Inspetor Vim"
                      >
                        <FileCode2 size={12} />
                        <span>{f.path}</span>
                        <span class="text-[12px] opacity-70">({f.linesCount} linhas)</span>
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
              <!-- Mensagem do usuário alinhada à direita. -->
              <div class="flex justify-end">
                <div class="message-card user-message">
                  <div class="message-heading text-emerald-300">
                    <span>Você · Arquiteto</span>
                    <div class="flex items-center gap-1.5">
                      <span class="text-emerald-500/70 text-[12px] font-code text-sm">{msg.timestamp}</span>
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
                  <div class="message-body markdown-body readable-text">
                    {@html renderMarkdown(msg.content)}
                  </div>
                </div>
              </div>
            {:else if msg.sender === 'agent'}
              <!-- Balão do Droid/Agente com cores e estilização por papel -->
              {@const isArchitect = msg.agentRole === 'ARQUITETO' || msg.agentName === 'SPARK-01'}
              {@const isCoder = msg.agentRole === 'PROGRAMADOR' || msg.agentName === 'Rusty'}
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
                    <span class="text-sm font-ui text-emerald-400 font-bold">&lt;/&gt;</span>
                  {:else}
                    <span class="text-sm font-ui text-fuchsia-400 font-bold">🛡️</span>
                  {/if}
                </div>

                <div class="message-card agent-message" style="--message-accent: {isArchitect ? '#67e8f9' : isCoder ? '#6ee7b7' : '#f0abfc'}">
                  <div class="message-heading {accentText}">
                    <span class="flex items-center gap-1.5">
                      <span>{msg.agentName || 'Droid'} <span class="message-role">/ {msg.agentRole || 'Agente'}</span></span>
                      {#if msg.status === 'STREAMING'}
                        <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
                      {/if}
                    </span>
                    <div class="flex items-center gap-1.5">
                      <span class="opacity-70 text-[12px] font-code text-sm">[{msg.timestamp}]</span>
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

                  <!-- Resposta legível, selecionável e com quebra de linhas preservada. -->
                  <div class="message-body markdown-body readable-text">
                    {@html renderMarkdown(msg.content)}
                    {#if msg.status === 'STREAMING'}
                      <span class="inline-block w-1.5 h-3.5 ml-1 bg-cyan-400 animate-pulse align-middle"></span>
                    {/if}
                  </div>

                  <!-- Processo de Pensamento (Chain-of-Thought) se existir -->
                  {#if msg.thinking}
                    <div class="mt-2 p-2 rounded-lg bg-black/60 border border-slate-700/60 text-slate-300 font-code text-sm sm:text-sm">
                      <div class="text-[12px] font-ui text-amber-300 font-bold uppercase mb-1">🧠 Chain-of-Thought ({msg.agentName}):</div>
                      <div class="markdown-body whitespace-pre-wrap leading-tight text-slate-200">{@html renderMarkdown(msg.thinking)}</div>
                    </div>
                  {/if}

                  <!-- Arquivos Modificados / Artefatos com botão para abrir no Vim -->
                  {#if msg.filesModified && msg.filesModified.length > 0}
                    <div class="mt-2 pt-2 border-t border-slate-700/60 flex items-center flex-wrap gap-2 text-sm font-mono text-cyan-300">
                      <span class="flex items-center gap-1 text-emerald-400 shrink-0">
                        <CheckCircle2 size={13} />
                        <span>Artefatos:</span>
                      </span>
                      {#each msg.filesModified as file}
                        <button
                          type="button"
                          on:click={() => openFileInVim(file, currentProjectId)}
                          class="px-2 py-0.5 rounded bg-cyan-950/80 hover:bg-cyan-900 border border-cyan-500/40 hover:border-cyan-400 text-cyan-300 text-[12px] font-mono flex items-center gap-1 transition cursor-pointer"
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
              <div class="system-message readable-text">
                <div class="markdown-body px-2.5 py-1 rounded bg-slate-900/60 border border-slate-800">
                  {@html renderMarkdown(msg.content)}
                </div>
              </div>
            {/if}
          {/each}

          {#if displayedMessages.length === 0}
            <div class="text-center py-10 text-slate-500 font-mono text-sm">
              Nenhuma contribuição registrada para este filtro de droid no momento.
            </div>
          {/if}
        </main>
      {/if}

      <!-- Faixa dedicada ao companheiro, fora da área de leitura. -->
      <div class="mascot-airspace" class:mascot-hidden={$terminalViewTab !== 'CONSOLE'}>
        <SparkMascot
          docked={true}
          state={mascotState}
          label={$activeProject.droids.find((d) => d.id === $activeProject.activeDroidId)?.name || 'SPARK-01'}
          wander={!prefersReducedMotion}
          on:click={() => {
            mascotState = 'celebrate';
            setTimeout(() => (mascotState = 'idle'), 3000);
          }}
        />
      </div>


      <!-- 4. Rodapé: Barra de Comando CLI -->
      <footer class="command-footer">
        <!-- Barra de Comando Estilo Terminal CLI (Néon Ciano com Prompt >>> |_) -->
        <div class="command-composer">
          <!-- Prefixo CLI >>> -->
          <div class="flex items-center gap-1 font-code text-cyan-400 font-bold text-base sm:text-lg pl-2 select-none">
            <span class="font-code">&gt;_</span>
          </div>

          <!-- Campo de comando com tipografia de interface. -->
          <div class="flex-1 min-w-0 relative flex items-center">
            <input
              type="text"
              bind:value={commandInput}
              on:keydown={handleKeyDown}
              aria-label="Comando para {$activeProject.name}"
              placeholder="O que vamos construir em {$activeProject.name}?"
              class="command-input"
            />
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
            class="p-2 sm:px-4 sm:py-2.5 rounded-xl font-ui text-[12px] font-black uppercase tracking-normal flex items-center gap-1.5 transition shrink-0
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
            class="p-2 sm:px-3 sm:py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white font-ui text-[12px] flex items-center gap-1.5 transition shrink-0 relative cursor-pointer"
            title="Abrir Backlog de Missões"
          >
            <ListTodo class="w-4 h-4 text-cyan-400" />
            <span class="hidden md:inline font-bold">Missões</span>
          </button>
        </div>
        <div class="composer-hint"><span>ENTER <span class="text-slate-500">para executar</span></span><span>THE GRID <span class="text-cyan-300">/</span> HUMAN IN COMMAND</span></div>
      </footer>
    </div>
  </div>
</div>


<style>
  .cyber-deck { position: relative; width: 100%; height: 100%; min-width: 0; padding: 28px; overflow: hidden; background: #050a12; }
  .city-backdrop, .city-shade, .city-grid { position: absolute; inset: 0; pointer-events: none; }
  .city-backdrop img { width: 100%; height: 100%; object-fit: cover; opacity: .4; }
  .city-shade { background: linear-gradient(135deg, #06101b99, #050912db 65%, #160c24b3); }
  .city-grid { background-image: linear-gradient(#67e8f908 1px, transparent 1px), linear-gradient(90deg, #67e8f908 1px, transparent 1px); background-size: 48px 48px; }
  .district-marker { position: absolute; bottom: 7px; right: 32px; font-size: 10px; letter-spacing: .12em; color: #a4b2c8; }
  .deck-frame { position: relative; height: 100%; max-width: 1480px; margin: 0 auto; padding: 5px; border: 1px solid #34465b; border-radius: 18px; background: linear-gradient(135deg, #253448, #121c2c 40%, #35263e); box-shadow: 0 24px 80px #0009, 0 0 32px #00f0ff0a; }
  .deck-screen { position: relative; height: 100%; display: flex; flex-direction: column; overflow: hidden; border: 1px solid #1b2b40; border-radius: 13px; background: #090f19f7; }
  .mascot-airspace { position: relative; flex-shrink: 0; height: 86px; pointer-events: none; z-index: 25; background: linear-gradient(0deg, #10203045, transparent); }
  .mascot-hidden { display: none; }
  .deck-header { position: relative; z-index: 30; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 24px 28px 20px; border-bottom: 1px solid #243247; background: linear-gradient(110deg, #111e2c, #0b131f); }
  .workspace-identity { min-width: 0; }
  .eyebrow { display: flex; align-items: center; gap: 9px; font-size: 11px; font-weight: 600; letter-spacing: .14em; color: #a4b2c8; }
  .identity-line { width: 18px; height: 2px; background: #67e8f9; box-shadow: 0 0 10px #67e8f970; }
  h1 { margin: 6px 0 5px; font-size: clamp(22px, 2.2vw, 30px); line-height: 1.2; font-weight: 650; letter-spacing: -.035em; color: #f0f6ff; overflow-wrap: anywhere; }
  .workspace-path { display: flex; align-items: center; gap: 7px; font-size: 12px; color: #a4b2c8; }
  .mobile-companion { display: none; width: 30px; height: 30px; image-rendering: pixelated; }
  .header-actions { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
  .connection-badge { display: flex; align-items: center; gap: 7px; padding: 7px 10px; font-size: 12px; white-space: nowrap; color: #fcd997; border: 1px solid #5c482f; border-radius: 6px; background: #241e14; }
  .connection-badge.connected { color: #8ee8bf; background: #0d251f; border-color: #224d40; }
  .status-dot { display: inline-block; flex-shrink: 0; width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
  .icon-button { display: flex; align-items: center; justify-content: center; width: 34px; height: 34px; border: 1px solid #2c3a50; border-radius: 6px; color: #afbed2; background: #142031; cursor: pointer; }
  .icon-button:hover { color: #67e8f9; border-color: #4c8598; }
  .deck-toolbar { position: relative; z-index: 35; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 12px 28px; background: #0d1622; border-bottom: 1px solid #243247; }
  .model-controls { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; min-width: 0; }
  .model-trigger { display: flex; align-items: center; gap: 8px; max-width: 100%; padding: 7px 10px; border: 1px solid #34465b; border-radius: 6px; background: #152233; color: #e2ebf8; font-size: 13px; cursor: pointer; }
  .model-trigger:hover { border-color: #67e8f9; }
  .model-effort { padding: 1px 5px; border-radius: 3px; color: #a4b2c8; background: #09131f; font-size: 12px; }
  .execution-state { display: flex; align-items: center; gap: 6px; font-size: 12px; }
  .telemetry-panel { position: relative; flex-shrink: 0; }
  summary { display: flex; align-items: center; gap: 6px; min-height: 34px; font-size: 12px; color: #b1bfd1; cursor: pointer; list-style: none; }
  summary::-webkit-details-marker { display: none; }
  .telemetry-popover { position: absolute; right: 0; top: calc(100% + 10px); width: 280px; padding: 18px; border: 1px solid #3c526c; border-radius: 10px; background: #131e2e; box-shadow: 0 16px 40px #0008; font-size: 12px; }
  .telemetry-heading { color: #67e8f9; font-size: 11px; letter-spacing: .05em; margin-bottom: 14px; }
  dl > div { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: 10px; }
  dt { color: #a4b2c8; } dd { margin: 0; color: #edf3fc; }
  .clear-history { display: flex; align-items: center; gap: 8px; width: 100%; margin-top: 18px; padding-top: 14px; border-top: 1px solid #34465b; color: #fda4af; cursor: pointer; }
  .view-toolbar { position: relative; z-index: 30; display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px; padding: 12px 28px; border-bottom: 1px solid #243247; background: #0c1420; }
  .view-toolbar :global(button) { font-size: 12px; min-height: 32px; box-shadow: none; border-radius: 5px; white-space: nowrap; }
  .droid-filters { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; font-size: 12px; }
  .conversation { padding: 22px 28px 24px; background: radial-gradient(ellipse at 80% 80%, #10213250, transparent 65%); }
  .session-banner { display: flex; align-items: center; gap: 12px; margin-bottom: 28px; color: #9baec6; font-size: 12px; }
  .session-banner > span:not(.session-rule) { display: flex; align-items: center; gap: 7px; white-space: nowrap; }
  .session-rule { height: 1px; flex: 1; background: #253449; }
  .message-card { min-width: 0; max-width: min(88%, 840px); padding: 17px 20px; border: 1px solid #2b3e55; border-radius: 8px; background: #142031; }
  .agent-message { --message-accent: #67e8f9; border-left: 2px solid var(--message-accent); }
  .user-message { background: #142b2b; border-color: #2d5351; }
  .message-heading { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 8px 16px; margin-bottom: 10px; font-size: 13px; font-weight: 600; }
  .message-role { color: #a4b2c8; font-weight: 400; font-size: 12px; }
  .message-heading :global(button) { display: flex; align-items: center; justify-content: center; width: 24px; height: 24px; }
  .message-body { font-family: var(--font-sans); font-size: 16px; line-height: 1.75; letter-spacing: normal; color: #e7eef8; overflow-wrap: anywhere; }
  .markdown-body :global(p) { margin: 0 0 12px; }
  .markdown-body :global(p:last-child) { margin-bottom: 0; }
  .markdown-body :global(h1), .markdown-body :global(h2), .markdown-body :global(h3), .markdown-body :global(h4), .markdown-body :global(h5), .markdown-body :global(h6) { margin: 18px 0 8px; color: #b9f7ff; font-family: var(--font-ui); line-height: 1.25; letter-spacing: .01em; }
  .markdown-body :global(h1) { font-size: 1.35em; text-shadow: 0 0 14px #00f0ff44; }
  .markdown-body :global(h2) { font-size: 1.2em; color: #8eeeff; }
  .markdown-body :global(h3), .markdown-body :global(h4), .markdown-body :global(h5), .markdown-body :global(h6) { font-size: 1.05em; }
  .markdown-body :global(ul), .markdown-body :global(ol) { margin: 8px 0 14px; padding-left: 24px; }
  .markdown-body :global(li) { margin: 5px 0; padding-left: 3px; }
  .markdown-body :global(ul) { list-style: none; }
  .markdown-body :global(ul li)::before { content: '▸'; display: inline-block; width: 18px; margin-left: -18px; color: #67e8f9; text-shadow: 0 0 8px #00f0ff; }
  .markdown-body :global(ol) { list-style: decimal; }
  .markdown-body :global(strong) { color: #f5fbff; font-weight: 700; }
  .markdown-body :global(em) { color: #c8b5ff; }
  .markdown-body :global(del) { color: #fda4af; }
  .markdown-body :global(a) { color: #67e8f9; text-decoration: underline; text-decoration-color: #67e8f966; text-underline-offset: 3px; }
  .markdown-body :global(a:hover) { color: #d7fbff; text-shadow: 0 0 10px #00f0ff; }
  .markdown-body :global(code) { padding: 2px 6px; border: 1px solid #31506a; border-radius: 4px; background: #08131f; color: #a7f3d0; font-family: var(--font-mono); font-size: .88em; }
  .markdown-body :global(pre) { margin: 14px 0; padding: 14px 16px; overflow-x: auto; border: 1px solid #31506a; border-left: 2px solid #67e8f9; border-radius: 7px; background: #07111d; box-shadow: inset 0 0 24px #00f0ff0a; }
  .markdown-body :global(pre code) { display: block; padding: 0; border: 0; background: transparent; color: #d1fae5; font-size: .86em; line-height: 1.65; white-space: pre; }
  .markdown-body :global(.code-token-keyword) { color: #d8b4fe; font-weight: 700; }
  .markdown-body :global(.code-token-string) { color: #86efac; }
  .markdown-body :global(.code-token-comment) { color: #718096; font-style: italic; }
  .markdown-body :global(.code-token-number) { color: #fbbf24; }
  .markdown-body :global(.code-token-tag) { color: #67e8f9; }
  .markdown-body :global(blockquote) { margin: 12px 0; padding: 8px 14px; border-left: 3px solid #c084fc; background: #160f2b; color: #ddd6fe; }
  .markdown-body :global(hr) { margin: 18px 0; border: 0; border-top: 1px solid #31506a; }
  .markdown-body :global(.md-table-wrap) { margin: 14px 0; overflow-x: auto; border: 1px solid #31506a; border-radius: 7px; }
  .markdown-body :global(table) { width: 100%; border-collapse: collapse; font-size: .9em; }
  .markdown-body :global(th), .markdown-body :global(td) { padding: 8px 10px; border-bottom: 1px solid #263b50; text-align: left; vertical-align: top; }
  .markdown-body :global(th) { color: #b9f7ff; background: #0b1b2b; font-family: var(--font-ui); font-size: .86em; text-transform: uppercase; letter-spacing: .04em; }
  .markdown-body :global(tr:last-child td) { border-bottom: 0; }
  .system-message { margin: 12px auto; max-width: 72ch; font-size: 12px; line-height: 1.7; color: #a4b2c8; text-align: center; }
  .system-message > div { display: inline-block; background: #101a29; border-color: #26344a; text-align: left; }
  .command-footer { position: relative; z-index: 30; flex-shrink: 0; padding: 16px 24px 10px; border-top: 1px solid #26374b; background: #0d1623; }
  .command-composer { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border: 1px solid #42566f; border-radius: 8px; background: #090f19; transition: border-color .2s, box-shadow .2s; }
  .command-composer:focus-within { border-color: #67e8f9; box-shadow: 0 0 0 3px #67e8f914; }
  .command-input { width: 100%; min-width: 0; padding: 8px 0; background: transparent; border: 0; color: #edf3fc; font-family: var(--font-sans); font-size: 16px; line-height: 1.5; caret-color: #67e8f9; }
  .command-input:focus-visible { outline: none; }
  .command-composer :global(button) { min-height: 38px; border-radius: 6px; font-size: 13px; box-shadow: none; text-transform: none; letter-spacing: normal; }
  .composer-hint { display: flex; justify-content: space-between; gap: 8px; padding: 9px 2px 0; font-family: var(--font-mono); font-size: 10px; color: #a4b2c8; letter-spacing: .04em; }
  @media (max-width: 1200px) {
    .cyber-deck { padding: 16px; }
    .district-marker { display: none; }
    .deck-header { padding: 20px; }
    .deck-toolbar, .view-toolbar { padding-left: 20px; padding-right: 20px; }
    .conversation { padding-left: 20px; padding-right: 20px; }
    .execution-state { display: none; }
  }
  @media (max-width: 760px) {
    .cyber-deck { padding: 8px; }
    .deck-header { padding: 16px; flex-wrap: wrap; }
    .header-actions { gap: 6px; }
    .deck-toolbar, .view-toolbar { padding: 10px 14px; }
    .conversation { padding: 16px 14px 20px; }
    .message-card { max-width: 100%; padding: 14px; }
    .command-footer { padding: 12px; }
    .command-composer { gap: 6px; padding: 8px; flex-wrap: wrap; }
    .composer-hint > span:last-child { display: none; }
    .model-effort { display: none; }
  }
  @media (max-width: 600px) { .mascot-airspace { display: none; } .mobile-companion { display: block; } }
  @media (max-height: 600px) { .mascot-airspace { display: none; } }
  @media (max-width: 480px) {
    .cyber-deck { padding: 4px; }
    .deck-frame { padding: 2px; border-radius: 10px; }
    .deck-screen { border-radius: 7px; }
    .deck-header { gap: 10px; }
    .eyebrow { font-size: 10px; }
    .connection-badge { padding: 5px 7px; }
    .telemetry-popover { position: fixed; top: 180px; right: 16px; max-width: calc(100vw - 100px); }
    .model-trigger { max-width: 190px; }
    .command-composer :global(button > span) { display: none; }
  }
</style>
