<script lang="ts">
  import {
    projects,
    activeProjectId,
    selectProject,
    selectDroidInActiveProject,
    isSidebarCollapsed,
    toggleSidebar,
    projectSearchQuery,
    createNewProject,
    reconfigureProjectDroids,
    deleteProject,
    type ProjectTerminal,
    type DroidInfo,
  } from '../../lib/stores/projectStore';
  import {
    droidRoster,
    openDroidForge,
    type RosterDroid,
  } from '../../lib/stores/droidRosterStore';
  import { setDroidFilter } from '../../lib/stores/workspaceFilesStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import {
    Search,
    FolderGit2,
    Bot,
    Plus,
    ChevronLeft,
    ChevronRight,
    Terminal,
    CheckCircle2,
    Clock,
    Flame,
    Cpu,
    Sparkles,
    CircleDot,
    Settings,
    Check,
    Layers,
    Wrench,
    Zap,
    Trash2,
  } from '@lucide/svelte';

  let showNewProjectModal = false;
  let newProjName = '';
  let newProjPath = '';
  let selectedModel = 'GPT-6 Luna';
  let selectedDroidIds: string[] = ['spark-01', 'agent-codex-01', 'agent-sec-05'];

  // Modal de Reconfiguração de Droids do Projeto Existente
  let showReconfigModal = false;
  let reconfigTargetProject: ProjectTerminal | null = null;
  let reconfigSelectedDroidIds: string[] = [];

  const MODEL_OPTIONS = [
    'Default (Codex CLI)',
    'GPT-6.1 Sol',
    'GPT-6 Astra',
    'GPT-6.1 Terra',
    'GPT-6 Sol',
    'GPT-6 Luna',
    'GPT-5.6 Sol',
    'GPT-5.6 Terra',
    'GPT-5.6 Luna',
  ];

  // Filtro de busca reativo
  $: filteredProjects = $projects.filter((p) => {
    const query = $projectSearchQuery.toLowerCase();
    if (!query) return true;
    const matchProj =
      p.name.toLowerCase().includes(query) ||
      p.path.toLowerCase().includes(query) ||
      p.model.toLowerCase().includes(query);
    const matchDroid = p.droids.some(
      (d) =>
        d.name.toLowerCase().includes(query) ||
        d.role.toLowerCase().includes(query) ||
        d.statusDetail.toLowerCase().includes(query)
    );
    return matchProj || matchDroid;
  });

  function applyPreset(preset: 'SOLO' | 'TRIAD' | 'RESEARCH' | 'QA', isReconfig = false) {
    let ids: string[] = [];
    if (preset === 'SOLO') {
      ids = ['spark-01'];
    } else if (preset === 'TRIAD') {
      ids = ['spark-01', 'agent-codex-01', 'agent-sec-05'];
    } else if (preset === 'RESEARCH') {
      ids = ['spark-01', 'agent-claude-02'];
    } else if (preset === 'QA') {
      ids = ['spark-01', 'agent-codex-01', 'agent-kaelen-04'];
    }

    if (isReconfig) {
      reconfigSelectedDroidIds = ids;
    } else {
      selectedDroidIds = ids;
    }
    soundFX.playKeyClick();
  }

  function toggleDroidSelection(id: string, isReconfig = false) {
    if (isReconfig) {
      if (reconfigSelectedDroidIds.includes(id)) {
        if (reconfigSelectedDroidIds.length > 1) {
          reconfigSelectedDroidIds = reconfigSelectedDroidIds.filter((x) => x !== id);
        }
      } else {
        reconfigSelectedDroidIds = [...reconfigSelectedDroidIds, id];
      }
    } else {
      if (selectedDroidIds.includes(id)) {
        if (selectedDroidIds.length > 1) {
          selectedDroidIds = selectedDroidIds.filter((x) => x !== id);
        }
      } else {
        selectedDroidIds = [...selectedDroidIds, id];
      }
    }
    soundFX.playKeyClick();
  }

  function handleCreateProject() {
    if (!newProjName.trim()) return;
    const path = newProjPath.trim() || `C:\\Workspace\\${newProjName.toLowerCase().replace(/\s+/g, '-')}`;
    createNewProject(newProjName.trim(), path, selectedDroidIds, selectedModel);
    newProjName = '';
    newProjPath = '';
    selectedDroidIds = ['spark-01', 'agent-codex-01', 'agent-sec-05'];
    showNewProjectModal = false;
    soundFX.playPipelineComplete();
  }

  function openReconfigModal(project: ProjectTerminal) {
    reconfigTargetProject = project;
    reconfigSelectedDroidIds = project.droids.map((d) => d.id);
    showReconfigModal = true;
    soundFX.playKeyClick();
  }

  function handleSaveReconfig() {
    if (!reconfigTargetProject || reconfigSelectedDroidIds.length === 0) return;
    reconfigureProjectDroids(reconfigTargetProject.id, reconfigSelectedDroidIds);
    showReconfigModal = false;
    reconfigTargetProject = null;
    soundFX.playPipelineComplete();
  }

  async function handleDeleteProject(project: ProjectTerminal) {
    if (confirm(`Tem certeza que deseja excluir o workspace "${project.name}"?\nEsta ação removerá o projeto e todo o histórico persistido do SQLite.`)) {
      await deleteProject(project.id);
      soundFX.playKeyClick();
    }
  }
</script>

<aside
  class="h-full bg-[#07090e]/95 border-r border-[#1a2333] flex flex-col transition-all duration-300 relative select-none z-40 backdrop-blur-md {$isSidebarCollapsed ? 'w-16' : 'w-80 md:w-96'}"
>
  <!-- Botão de Minimizar/Expandir Lateral -->
  <button
    type="button"
    on:click={toggleSidebar}
    class="absolute -right-3.5 top-6 z-50 w-7 h-7 bg-[#0d131f] border border-[#00f0ff]/40 text-[#00f0ff] rounded-full flex items-center justify-center hover:bg-[#00f0ff] hover:text-black hover:shadow-[0_0_12px_#00f0ff] transition-all cursor-pointer"
    title={$isSidebarCollapsed ? 'Expandir Workspaces' : 'Minimizar Sidebar'}
  >
    {#if $isSidebarCollapsed}
      <ChevronRight size={14} />
    {:else}
      <ChevronLeft size={14} />
    {/if}
  </button>

  {#if !$isSidebarCollapsed}
    <!-- Cabeçalho Estilo Habblaud / Retro Office -->
    <div class="p-3.5 border-b border-[#162032] flex flex-col gap-2.5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-7 h-7 rounded bg-[#00f0ff]/10 border border-[#00f0ff]/40 flex items-center justify-center text-[#00f0ff]">
            <Terminal size={16} />
          </div>
          <div>
            <span class="font-pixel-title text-sm tracking-wider text-cyan-300 font-bold">WORKSPACES</span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-emerald-950/80 text-emerald-400 border border-emerald-500/30 ml-1.5 font-mono">
              ONLINE
            </span>
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <!-- Botão Droid Forge (Estilo Paperclip) -->
          <button
            type="button"
            on:click={() => openDroidForge()}
            class="flex items-center gap-1 px-2.5 py-1 rounded bg-fuchsia-950/70 hover:bg-fuchsia-900/90 text-fuchsia-300 border border-fuchsia-500/40 hover:border-fuchsia-400 font-pixel-title text-[10px] tracking-wide transition-all cursor-pointer shadow-[0_0_8px_rgba(255,0,127,0.2)]"
            title="Abrir Droid Forge & Roster"
          >
            <Bot size={12} />
            <span>FORGE</span>
          </button>

          <!-- Botão Novo Workspace -->
          <button
            type="button"
            on:click={() => (showNewProjectModal = true)}
            class="flex items-center gap-1 px-2.5 py-1 rounded bg-[#00f0ff]/10 hover:bg-[#00f0ff]/20 text-[#00f0ff] border border-[#00f0ff]/40 hover:border-[#00f0ff] font-pixel-title text-[10px] tracking-wide transition-all cursor-pointer"
            title="Novo Projeto / Terminal"
          >
            <Plus size={12} />
            <span>NOVO</span>
          </button>
        </div>
      </div>

      <!-- Barra de Pesquisa de Projetos / Droids -->
      <div class="relative">
        <Search size={13} class="absolute left-2.5 top-2.5 text-slate-500" />
        <input
          type="text"
          placeholder="Buscar projeto, droid ou instrução..."
          bind:value={$projectSearchQuery}
          class="w-full bg-[#0a0f18] border border-[#1e2a3f] focus:border-[#00f0ff]/70 text-slate-200 text-xs pl-8 pr-3 py-1.5 rounded outline-none font-mono transition-colors placeholder:text-slate-600"
        />
        {#if $projectSearchQuery}
          <button
            type="button"
            on:click={() => projectSearchQuery.set('')}
            class="absolute right-2.5 top-2 text-slate-400 hover:text-white text-xs cursor-pointer"
          >
            ×
          </button>
        {/if}
      </div>
    </div>

    <!-- Lista de Projetos e Seus Droids (Estilo Habblaud Rooms) -->
    <div class="flex-1 overflow-y-auto px-3 py-3 flex flex-col gap-3 custom-scrollbar">
      {#each filteredProjects as project (project.id)}
        {@const isSelected = project.id === $activeProjectId}
        <div
          class="rounded-lg border transition-all duration-200 overflow-hidden {isSelected
            ? 'bg-[#0f1726]/90 border-[#00f0ff]/60 shadow-[0_0_15px_rgba(0,240,255,0.15)] ring-1 ring-[#00f0ff]/30'
            : 'bg-[#0a0f18]/70 border-[#182335] hover:border-slate-700/80 hover:bg-[#0c1320]'}"
        >
          <!-- Cabeçalho do Projeto -->
          <div class="w-full text-left p-2.5 flex items-center justify-between border-b border-[#162235]/60 group">
            <button
              type="button"
              on:click={() => {
                selectProject(project.id);
                setDroidFilter(null);
              }}
              class="flex items-center gap-2 min-w-0 flex-1 text-left cursor-pointer focus:outline-none"
            >
              <div
                class="w-6 h-6 rounded flex items-center justify-center shrink-0 text-xs font-pixel-title"
                style="background: {project.accentColor}18; color: {project.accentColor}; border: 1px solid {project.accentColor}40"
              >
                <FolderGit2 size={13} />
              </div>
              <div class="truncate">
                <div class="flex items-center gap-1.5">
                  <span class="font-pixel-title text-xs font-semibold text-slate-200 group-hover:text-cyan-300 transition-colors">
                    {project.name}
                  </span>
                  {#if project.status === 'PROCESSING'}
                    <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
                  {:else}
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  {/if}
                </div>
                <div class="text-[10px] text-slate-500 font-mono truncate" title={project.path}>
                  {project.path}
                </div>
              </div>
            </button>

            <!-- Badges e Botão de Configuração de Droids (Harness) -->
            <div class="flex items-center gap-1.5 shrink-0 ml-2">
              <span class="text-[9px] px-1.5 py-0.5 rounded bg-[#131c2c] text-cyan-400 font-mono border border-cyan-500/20">
                {project.droids.length} {project.droids.length === 1 ? 'Droid' : 'Droids'}
              </span>

              <button
                type="button"
                on:click={() => openReconfigModal(project)}
                class="p-1 rounded bg-[#101726] hover:bg-cyan-950 text-slate-400 hover:text-cyan-300 border border-slate-700/60 hover:border-cyan-500/60 transition cursor-pointer"
                title="Configurar Droids Alocados neste Projeto"
              >
                <Settings size={11} />
              </button>

              <button
                type="button"
                on:click={() => handleDeleteProject(project)}
                class="p-1 rounded bg-[#101726] hover:bg-rose-950 text-slate-400 hover:text-rose-400 border border-slate-700/60 hover:border-rose-500/60 transition cursor-pointer"
                title="Excluir Workspace Permanentemente"
              >
                <Trash2 size={11} />
              </button>
            </div>
          </div>

          <!-- Lista de Droids Alocados neste Projeto -->
          <div class="p-2 flex flex-col gap-1.5 bg-[#080c14]/60">
            {#each project.droids as droid (droid.id)}
              {@const isDroidActive = isSelected && project.activeDroidId === droid.id}
              {@const isWorking = isSelected && (droid.status === 'WORKING' || droid.status === 'THINKING' || (project.pipeline.isActive && project.pipeline.currentDroidId === droid.id))}
              <button
                type="button"
                on:click={() => {
                  selectProject(project.id);
                  selectDroidInActiveProject(droid.id);
                  setDroidFilter(droid.id);
                }}
                class="w-full text-left p-2.5 rounded-xl flex items-start gap-2.5 transition-all duration-300 cursor-pointer focus:outline-none relative overflow-hidden {isWorking
                  ? 'bg-[#0f172a] shadow-xl ring-2 ring-offset-1 ring-offset-[#070b14]'
                  : isDroidActive
                  ? 'bg-[#142033] border border-[#00f0ff]/50 shadow-[0_0_10px_rgba(0,240,255,0.15)]'
                  : 'bg-[#0a0f19] border border-[#162032] hover:bg-[#0e1624] hover:border-slate-700'}"
                style="{isWorking ? `border: 1.5px solid ${droid.accentColor}; box-shadow: 0 0 20px ${droid.accentColor}55, inset 0 0 12px ${droid.accentColor}25; --tw-ring-color: ${droid.accentColor};` : ''}"
              >
                <!-- Feixe luminoso / Scanner animado no topo do card quando o droid está trabalhando -->
                {#if isWorking}
                  <div
                    class="absolute top-0 left-0 right-0 h-[2px] animate-pulse"
                    style="background: {droid.accentColor}; box-shadow: 0 0 10px {droid.accentColor};"
                  ></div>
                {/if}

                <!-- Avatar Pixel Art do Droid -->
                <div class="relative shrink-0 mt-0.5">
                  <div
                    class="w-8 h-8 rounded-lg border-2 flex items-center justify-center overflow-hidden bg-black/70 transition-all duration-300"
                    style="border-color: {isWorking ? droid.accentColor : droid.accentColor + '60'}; box-shadow: {isWorking ? `0 0 14px ${droid.accentColor}` : `0 0 8px ${droid.accentColor}25`}"
                  >
                    {#if droid.avatarType === 'spark'}
                      <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-7 h-7 object-contain pixelated {isWorking ? 'animate-bounce' : ''}" />
                    {:else if droid.avatarType === 'hacker'}
                      <div class="text-xs font-pixel-title font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
                        &lt;/&gt;
                      </div>
                    {:else if droid.avatarType === 'scientist'}
                      <div class="text-xs font-pixel-title font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
                        [EXP]
                      </div>
                    {:else}
                      <div class="text-xs font-pixel-title font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
                        🛡️
                      </div>
                    {/if}
                  </div>
                  <!-- Indicador de Status -->
                  <div
                    class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border border-black {isWorking
                      ? 'animate-ping'
                      : droid.status === 'ONLINE'
                      ? 'bg-emerald-400'
                      : 'bg-slate-500'}"
                    style="{isWorking ? `background: ${droid.accentColor};` : ''}"
                  ></div>
                </div>

                <!-- Info do Droid (Estilo Habblaud Card) -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between gap-1">
                    <div class="flex items-center gap-1.5 truncate">
                      <span class="font-pixel-title text-[11px] font-bold truncate" style="color: {isWorking ? droid.accentColor : '#e2e8f0'}">
                        {droid.name}
                      </span>
                      {#if isWorking}
                        <span class="w-1.5 h-1.5 rounded-full animate-ping shrink-0" style="background: {droid.accentColor}"></span>
                      {/if}
                    </div>

                    <span
                      class="text-[8px] font-mono px-1 py-0.2 rounded border uppercase tracking-wider font-bold shrink-0"
                      style="color: {droid.accentColor}; background: {droid.accentColor}18; border-color: {droid.accentColor}{isWorking ? '90' : '30'}"
                    >
                      {isWorking ? '⚡ ATIVO' : droid.planBadge}
                    </span>
                  </div>

                  <div class="flex items-center gap-1.5 mt-0.5">
                    <span class="text-[9px] font-mono font-bold" style="color: {isWorking ? droid.accentColor : '#94a3b8'}">
                      {isWorking ? (droid.status === 'THINKING' ? '💭 Elaborando...' : '⚡ Codificando...') : droid.status === 'ONLINE' ? '● Pronto' : '● Idle'}
                    </span>
                    <span class="text-[9px] text-slate-600 font-mono">•</span>
                    <span class="text-[9px] font-mono text-cyan-400/90">
                      {(droid.tokens / 1000).toFixed(1)}k tok
                    </span>
                  </div>

                  <!-- Detalhe da Tarefa Atual / Último Status -->
                  <div class="mt-1 text-[10px] text-slate-400 font-mono truncate flex items-center gap-1">
                    {#if isWorking}
                      <span class="w-2 h-2 rounded-full animate-pulse shrink-0" style="background: {droid.accentColor}"></span>
                    {:else if droid.status === 'ONLINE'}
                      <CheckCircle2 size={10} class="text-emerald-400 shrink-0" />
                    {:else}
                      <Clock size={10} class="text-amber-400 shrink-0" />
                    {/if}
                    <span class="truncate" style="{isWorking ? 'color: #f1f5f9; font-weight: 600;' : ''}">{droid.statusDetail}</span>
                  </div>
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/each}

      {#if filteredProjects.length === 0}
        <div class="text-center py-8 text-slate-500 font-mono text-xs">
          Nenhum projeto ou droid encontrado.
        </div>
      {/if}
    </div>

    <!-- Rodapé da Sidebar: Estatísticas Globais -->
    <div class="p-3 border-t border-[#162032] bg-[#05080e] flex items-center justify-between text-[10px] font-mono text-slate-400">
      <div class="flex items-center gap-1.5">
        <CircleDot size={10} class="text-emerald-400 animate-pulse" />
        <span>Core Terminal v0.3</span>
      </div>
      <div class="text-cyan-400/80">
        {$projects.length} Workspaces // {$droidRoster.length} Droids
      </div>
    </div>
  {:else}
    <!-- Sidebar Minimizado: Apenas Ícones Rápidos -->
    <div class="flex-1 flex flex-col items-center py-4 gap-3">
      <div class="w-8 h-8 rounded bg-[#00f0ff]/10 border border-[#00f0ff]/40 flex items-center justify-center text-[#00f0ff] mb-2">
        <Terminal size={16} />
      </div>

      {#each $projects as project (project.id)}
        {@const isSelected = project.id === $activeProjectId}
        <button
          type="button"
          on:click={() => selectProject(project.id)}
          class="w-10 h-10 rounded-lg flex items-center justify-center transition-all relative group cursor-pointer {isSelected
            ? 'bg-[#00f0ff]/20 text-[#00f0ff] border border-[#00f0ff]'
            : 'bg-[#0f1726] text-slate-400 border border-[#1e2a3f] hover:text-white hover:border-slate-600'}"
          title={`${project.name} (${project.path})`}
        >
          <FolderGit2 size={16} />
          {#if project.status === 'PROCESSING'}
            <span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-amber-400 animate-ping"></span>
          {/if}
        </button>
      {/each}

      <button
        type="button"
        on:click={() => {
          isSidebarCollapsed.set(false);
          openDroidForge();
        }}
        class="w-9 h-9 rounded-lg bg-fuchsia-950/60 border border-fuchsia-500/40 hover:border-fuchsia-400 text-fuchsia-300 flex items-center justify-center mt-auto mb-1 transition-colors cursor-pointer"
        title="Abrir Droid Forge"
      >
        <Bot size={15} />
      </button>

      <button
        type="button"
        on:click={() => {
          isSidebarCollapsed.set(false);
          showNewProjectModal = true;
        }}
        class="w-9 h-9 rounded-lg border border-dashed border-slate-700 hover:border-cyan-400 text-slate-500 hover:text-cyan-400 flex items-center justify-center mb-2 transition-colors cursor-pointer"
        title="Criar Novo Workspace"
      >
        <Plus size={16} />
      </button>
    </div>
  {/if}
</aside>

<!-- Modal: Criar Novo Projeto / Workspace com Harness de Droids -->
{#if showNewProjectModal}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-3 sm:p-4 font-mono select-none">
    <div class="w-full max-w-2xl max-h-[92vh] bg-[#090e1a] border-2 border-[#00f0ff]/50 rounded-2xl shadow-[0_0_40px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden">
      <!-- Cabeçalho -->
      <div class="px-5 py-4 border-b border-[#18263e] bg-[#0c1322] flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-cyan-950/80 border border-cyan-400/50 flex items-center justify-center text-cyan-400">
            <Terminal size={17} />
          </div>
          <div>
            <h2 class="font-pixel-title text-sm sm:text-base text-cyan-300 font-bold tracking-wider">
              NOVO WORKSPACE & HARNESS DE DROIDS
            </h2>
            <p class="text-[11px] text-slate-400 font-mono">
              Defina o diretório de trabalho e selecione os droids especialistas para a missão.
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => (showNewProjectModal = false)}
          class="w-7 h-7 rounded-lg bg-slate-900 border border-slate-700 text-slate-400 hover:text-white text-sm flex items-center justify-center transition cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Corpo com Scroll -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4 custom-scrollbar">
        <!-- Nome e Path -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label for="input-proj-name" class="text-xs text-slate-300 block mb-1 font-bold">
              Nome do Projeto:
            </label>
            <input
              id="input-proj-name"
              type="text"
              bind:value={newProjName}
              placeholder="Ex: Notex RAG Engine, Auth Microservice..."
              class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-xs px-3 py-2 rounded-lg outline-none"
            />
          </div>

          <div>
            <label for="input-proj-model" class="text-xs text-slate-300 block mb-1 font-bold">
              Modelo Principal do Terminal:
            </label>
            <select
              id="input-proj-model"
              bind:value={selectedModel}
              class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-xs px-3 py-2 rounded-lg outline-none cursor-pointer"
            >
              {#each MODEL_OPTIONS as opt}
                <option value={opt}>{opt}</option>
              {/each}
            </select>
          </div>
        </div>

        <div>
          <label for="input-proj-path" class="text-xs text-slate-300 block mb-1 font-bold">
            Caminho do Diretório (Workspace Path):
          </label>
          <input
            id="input-proj-path"
            type="text"
            bind:value={newProjPath}
            placeholder="Ex: C:\Workspace\meu-projeto"
            class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-xs px-3 py-2 rounded-lg outline-none"
          />
        </div>

        <!-- Seção de Alocação de Droids (Paperclip Harness) -->
        <div class="pt-2 border-t border-[#18263e]">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-slate-200 font-bold flex items-center gap-1.5">
              <Bot size={13} class="text-cyan-400" />
              <span>ALOCAÇÃO DE DROIDS ({selectedDroidIds.length} selecionado{selectedDroidIds.length === 1 ? '' : 's'})</span>
            </span>

            <!-- Presets Rápidos -->
            <div class="flex items-center gap-1 text-[10px]">
              <span class="text-slate-500 mr-1 hidden sm:inline">Presets:</span>
              <button
                type="button"
                on:click={() => applyPreset('SOLO')}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition cursor-pointer"
              >
                Solo (1)
              </button>
              <button
                type="button"
                on:click={() => applyPreset('RESEARCH')}
                class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition cursor-pointer"
              >
                R&D (2)
              </button>
              <button
                type="button"
                on:click={() => applyPreset('TRIAD')}
                class="px-2 py-0.5 rounded bg-cyan-950/80 hover:bg-cyan-900 text-cyan-300 border border-cyan-500/40 transition cursor-pointer font-bold"
              >
                Tríade (3)
              </button>
            </div>
          </div>

          <!-- Cards de Seleção dos Droids do Roster -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 max-h-56 overflow-y-auto p-1 custom-scrollbar">
            {#each $droidRoster as droid (droid.id)}
              {@const isChecked = selectedDroidIds.includes(droid.id)}
              <button
                type="button"
                on:click={() => toggleDroidSelection(droid.id)}
                class="text-left p-2.5 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-2.5 {isChecked
                  ? 'bg-[#101b2e] border-cyan-400/80 shadow-[0_0_12px_rgba(0,240,255,0.15)] ring-1 ring-cyan-400/40'
                  : 'bg-[#060a14] border-[#182436] hover:border-slate-600 opacity-60 hover:opacity-100'}"
              >
                <div class="flex items-center gap-2.5 min-w-0">
                  <div
                    class="w-9 h-9 rounded-lg border flex items-center justify-center bg-black/60 overflow-hidden shrink-0"
                    style="border-color: {droid.accentColor}70;"
                  >
                    {#if droid.avatarType === 'spark'}
                      <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-7 h-7 object-contain pixelated" />
                    {:else if droid.avatarType === 'hacker'}
                      <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">&lt;/&gt;</span>
                    {:else if droid.avatarType === 'scientist'}
                      <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">[EXP]</span>
                    {:else}
                      <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">🛡️</span>
                    {/if}
                  </div>

                  <div class="truncate">
                    <div class="font-pixel-title text-xs font-bold text-slate-100 truncate">
                      {droid.name}
                    </div>
                    <div class="text-[9px] text-slate-400 font-mono truncate">
                      {droid.role} • {droid.planBadge}
                    </div>
                  </div>
                </div>

                <div
                  class="w-5 h-5 rounded-md border flex items-center justify-center shrink-0 transition-all {isChecked
                    ? 'bg-cyan-500 border-cyan-400 text-black font-bold'
                    : 'border-slate-700 bg-slate-900'}"
                >
                  {#if isChecked}
                    <Check size={12} strokeWidth={3} />
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Rodapé do Modal -->
      <div class="px-5 py-3 border-t border-[#18263e] bg-[#0c1322] flex justify-end gap-2">
        <button
          type="button"
          on:click={() => (showNewProjectModal = false)}
          class="px-3.5 py-1.5 rounded-lg bg-slate-800 text-slate-300 text-xs hover:bg-slate-700 cursor-pointer"
        >
          Cancelar
        </button>
        <button
          type="button"
          on:click={handleCreateProject}
          disabled={!newProjName.trim() || selectedDroidIds.length === 0}
          class="px-4 py-1.5 rounded-lg font-pixel-title text-xs tracking-wider transition-all cursor-pointer {newProjName.trim() && selectedDroidIds.length > 0
            ? 'bg-[#00f0ff] text-black font-bold hover:bg-cyan-300 shadow-[0_0_15px_rgba(0,240,255,0.4)]'
            : 'bg-slate-800 text-slate-500 cursor-not-allowed'}"
        >
          INICIALIZAR WORKSPACE ({selectedDroidIds.length} DROID{selectedDroidIds.length === 1 ? '' : 'S'})
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Reconfigurar Droids de Workspace Existente -->
{#if showReconfigModal && reconfigTargetProject}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-3 sm:p-4 font-mono select-none">
    <div class="w-full max-w-xl max-h-[90vh] bg-[#090e1a] border-2 border-cyan-500/50 rounded-2xl shadow-[0_0_40px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden">
      <!-- Cabeçalho -->
      <div class="px-5 py-4 border-b border-[#18263e] bg-[#0c1322] flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-cyan-950/80 border border-cyan-400/50 flex items-center justify-center text-cyan-400">
            <Wrench size={17} />
          </div>
          <div>
            <h2 class="font-pixel-title text-sm sm:text-base text-cyan-300 font-bold tracking-wider">
              HARNESS CONFIG: {reconfigTargetProject.name.toUpperCase()}
            </h2>
            <p class="text-[11px] text-slate-400 font-mono">
              Altere a equipe de droids atuando neste workspace em tempo real.
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => (showReconfigModal = false)}
          class="w-7 h-7 rounded-lg bg-slate-900 border border-slate-700 text-slate-400 hover:text-white text-sm flex items-center justify-center transition cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Presets & Lista de Seleção -->
      <div class="p-5 flex-1 overflow-y-auto space-y-3 custom-scrollbar">
        <div class="flex items-center justify-between">
          <span class="text-xs text-slate-300 font-bold">
            Selecione os droids atribuídos ({reconfigSelectedDroidIds.length} selecionado{reconfigSelectedDroidIds.length === 1 ? '' : 's'}):
          </span>

          <div class="flex items-center gap-1 text-[10px]">
            <button
              type="button"
              on:click={() => applyPreset('SOLO', true)}
              class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition cursor-pointer"
            >
              Solo
            </button>
            <button
              type="button"
              on:click={() => applyPreset('RESEARCH', true)}
              class="px-2 py-0.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition cursor-pointer"
            >
              R&D
            </button>
            <button
              type="button"
              on:click={() => applyPreset('TRIAD', true)}
              class="px-2 py-0.5 rounded bg-cyan-950/80 hover:bg-cyan-900 text-cyan-300 border border-cyan-500/40 transition cursor-pointer font-bold"
            >
              Tríade
            </button>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
          {#each $droidRoster as droid (droid.id)}
            {@const isChecked = reconfigSelectedDroidIds.includes(droid.id)}
            <button
              type="button"
              on:click={() => toggleDroidSelection(droid.id, true)}
              class="text-left p-2.5 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-2.5 {isChecked
                ? 'bg-[#101b2e] border-cyan-400/80 shadow-[0_0_12px_rgba(0,240,255,0.15)] ring-1 ring-cyan-400/40'
                : 'bg-[#060a14] border-[#182436] hover:border-slate-600 opacity-60 hover:opacity-100'}"
            >
              <div class="flex items-center gap-2.5 min-w-0">
                <div
                  class="w-9 h-9 rounded-lg border flex items-center justify-center bg-black/60 overflow-hidden shrink-0"
                  style="border-color: {droid.accentColor}70;"
                >
                  {#if droid.avatarType === 'spark'}
                    <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-7 h-7 object-contain pixelated" />
                  {:else if droid.avatarType === 'hacker'}
                    <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">&lt;/&gt;</span>
                  {:else if droid.avatarType === 'scientist'}
                    <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">[EXP]</span>
                  {:else}
                    <span class="text-xs font-pixel-title font-bold" style="color: {droid.accentColor}">🛡️</span>
                  {/if}
                </div>

                <div class="truncate">
                  <div class="font-pixel-title text-xs font-bold text-slate-100 truncate">
                    {droid.name}
                  </div>
                  <div class="text-[9px] text-slate-400 font-mono truncate">
                    {droid.role} • {droid.planBadge}
                  </div>
                </div>
              </div>

              <div
                class="w-5 h-5 rounded-md border flex items-center justify-center shrink-0 transition-all {isChecked
                  ? 'bg-cyan-500 border-cyan-400 text-black font-bold'
                  : 'border-slate-700 bg-slate-900'}"
              >
                {#if isChecked}
                  <Check size={12} strokeWidth={3} />
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- Rodapé -->
      <div class="px-5 py-3 border-t border-[#18263e] bg-[#0c1322] flex justify-end gap-2">
        <button
          type="button"
          on:click={() => (showReconfigModal = false)}
          class="px-3.5 py-1.5 rounded-lg bg-slate-800 text-slate-300 text-xs hover:bg-slate-700 cursor-pointer"
        >
          Cancelar
        </button>
        <button
          type="button"
          on:click={handleSaveReconfig}
          disabled={reconfigSelectedDroidIds.length === 0}
          class="px-4 py-1.5 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-black font-bold font-pixel-title text-xs tracking-wider transition-all cursor-pointer shadow-[0_0_15px_rgba(0,240,255,0.4)]"
        >
          SALVAR HARNESS ({reconfigSelectedDroidIds.length} DROID{reconfigSelectedDroidIds.length === 1 ? '' : 'S'})
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #1a2538;
    border-radius: 2px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #00f0ff40;
  }
  .pixelated {
    image-rendering: pixelated;
  }
</style>
