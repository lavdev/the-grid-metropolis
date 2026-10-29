<script lang="ts">
  import { onMount } from 'svelte';

  onMount(() => {
    const compactViewport = window.matchMedia('(max-width: 900px)');
    const syncSidebar = () => isSidebarCollapsed.set(compactViewport.matches);
    syncSidebar();
    compactViewport.addEventListener('change', syncSidebar);
    return () => compactViewport.removeEventListener('change', syncSidebar);
  });
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
  import { confirmAction } from '../../lib/stores/uiFeedbackStore';
  import {
    Search,
    FolderGit2,
    Bot,
    Plus,
    ChevronLeft,
    ChevronRight,
    ChevronDown,
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
  const PROJECT_EXPANSION_STORAGE_KEY = 'the_grid_metropolis_project_expansion';

  function loadProjectExpansionState(): Record<string, boolean> {
    if (typeof window === 'undefined') return {};
    try {
      const stored = window.localStorage.getItem(PROJECT_EXPANSION_STORAGE_KEY);
      if (!stored) return {};
      const parsed: unknown = JSON.parse(stored);
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {};
      return Object.fromEntries(
        Object.entries(parsed).filter((entry): entry is [string, boolean] => typeof entry[1] === 'boolean')
      );
    } catch (error) {
      console.warn('Erro ao ler estado de expansão dos projetos:', error);
      return {};
    }
  }

  let expandedProjects = loadProjectExpansionState();

  function toggleProjectExpansion(projectId: string): void {
    expandedProjects = {
      ...expandedProjects,
      [projectId]: !(expandedProjects[projectId] ?? true),
    };
    try {
      window.localStorage.setItem(PROJECT_EXPANSION_STORAGE_KEY, JSON.stringify(expandedProjects));
    } catch (error) {
      console.warn('Erro ao persistir estado de expansão dos projetos:', error);
    }
  }

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
    if (await confirmAction({
      title: `Excluir workspace "${project.name}"?`,
      message: 'Esta ação removerá o projeto e todo o histórico persistido do SQLite.',
      confirmLabel: 'Excluir workspace',
      tone: 'danger',
    })) {
      await deleteProject(project.id);
      soundFX.playKeyClick();
    }
  }
</script>

<aside
  class="workspace-sidebar h-full flex flex-col transition-all duration-300 relative select-none z-40 {$isSidebarCollapsed ? 'sidebar-collapsed' : 'sidebar-expanded'}"
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
    <div class="sidebar-header">
      <div class="city-brand">
        <div class="brand-icon"><Terminal size={21} /></div>
        <div><div class="font-brand brand-name">THE GRID</div><div class="brand-subtitle">METROPOLIS / COMMAND CENTER</div></div>
      </div>
      <div class="sidebar-heading-row">
        <h2>Workspaces <span class="workspace-count">{$projects.length}</span></h2>
        <div class="flex items-center gap-1.5">
          <button type="button" on:click={() => openDroidForge()} class="sidebar-action forge-action" title="Abrir Droid Forge & Roster"><Bot size={14} /> Forge</button>
          <button type="button" on:click={() => (showNewProjectModal = true)} class="sidebar-action" title="Novo Projeto / Terminal"><Plus size={14} /> Novo</button>
        </div>
      </div>

      <!-- Barra de Pesquisa de Projetos / Droids -->
      <div class="relative">
        <Search size={15} class="absolute left-3 top-3 text-slate-400" />
        <input
          type="text"
          aria-label="Buscar projetos e droids"
          placeholder="Buscar projeto ou droid..."
          bind:value={$projectSearchQuery}
          class="sidebar-search"
        />
        {#if $projectSearchQuery}
          <button
            type="button"
            on:click={() => projectSearchQuery.set('')}
            class="absolute right-2.5 top-2 text-slate-400 hover:text-white text-sm cursor-pointer"
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
        {@const isProjectExpanded = expandedProjects[project.id] ?? true}
        <div
          class="project-card rounded-lg border transition-all duration-200 overflow-hidden {isSelected
            ? 'bg-[#101c2b] border-[#3a6376]'
            : 'bg-[#0a0f18]/70 border-[#182335] hover:border-slate-700/80 hover:bg-[#0c1320]'}"
        >
          <!-- Cabeçalho do Projeto -->
          <div class="project-card-header w-full text-left flex items-center justify-between border-b border-[#26364c] group">
            <button
              type="button"
              on:click={() => {
                selectProject(project.id);
                setDroidFilter(null);
              }}
              class="flex items-center gap-2 min-w-0 flex-1 text-left cursor-pointer focus:outline-none"
            >
              <div
                class="w-6 h-6 rounded flex items-center justify-center shrink-0 text-sm font-ui"
                style="background: {project.accentColor}18; color: {project.accentColor}; border: 1px solid {project.accentColor}40"
              >
                <FolderGit2 size={13} />
              </div>
              <div class="truncate">
                <div class="flex items-center gap-1.5">
                  <span class="font-ui text-sm font-semibold text-slate-200 group-hover:text-cyan-300 transition-colors">
                    {project.name}
                  </span>
                  {#if project.status === 'PROCESSING'}
                    <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
                  {:else}
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  {/if}
                </div>
                <div class="text-[12px] text-slate-500 font-mono truncate" title={project.path}>
                  {project.path}
                </div>
              </div>
            </button>

            <button
              type="button"
              aria-label={`${isProjectExpanded ? 'Recolher' : 'Expandir'} ${project.name}`}
              aria-expanded={isProjectExpanded}
              aria-controls={`project-droids-${project.id}`}
              on:click={() => toggleProjectExpansion(project.id)}
              class="p-1 rounded bg-[#101726] hover:bg-cyan-950 text-slate-400 hover:text-cyan-300 border border-slate-700/60 hover:border-cyan-500/60 transition cursor-pointer focus-visible:outline focus-visible:outline-2 focus-visible:outline-cyan-400"
              title={`${isProjectExpanded ? 'Recolher' : 'Expandir'} lista de droids`}
            >
              <ChevronDown
                size={14}
                class={`transition-transform ${!isProjectExpanded ? 'rotate-180' : ''}`}
              />
            </button>

            <!-- Badges e Botão de Configuração de Droids (Harness) -->
            <div class="flex items-center gap-1.5 shrink-0 ml-2">
              <span class="text-[12px] px-1.5 py-0.5 rounded bg-[#131c2c] text-cyan-400 font-mono border border-cyan-500/20">
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
          <div
            id={`project-droids-${project.id}`}
            class="p-2 flex flex-col gap-1.5 bg-[#080c14]/60"
            hidden={!isProjectExpanded}
          >
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
                class="droid-card w-full text-left rounded-lg flex items-start gap-2.5 transition-all duration-300 cursor-pointer relative overflow-hidden {isWorking
                  ? 'bg-[#0f172a] shadow-xl ring-2 ring-offset-1 ring-offset-[#070b14]'
                  : isDroidActive
                  ? 'bg-[#17283b] border border-[#426880]'
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
                      <div class="text-sm font-ui font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
                        &lt;/&gt;
                      </div>
                    {:else if droid.avatarType === 'scientist'}
                      <div class="text-sm font-ui font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
                        [EXP]
                      </div>
                    {:else}
                      <div class="text-sm font-ui font-bold {isWorking ? 'animate-pulse' : ''}" style="color: {droid.accentColor}">
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
                  <div class="droid-identity">
                    <div class="flex items-center gap-1.5 truncate">
                      <span class="droid-name font-ui font-semibold truncate" style="color: {isWorking ? droid.accentColor : '#e2e8f0'}">
                        {droid.name}
                      </span>
                      {#if isWorking}
                        <span class="w-1.5 h-1.5 rounded-full animate-ping shrink-0" style="background: {droid.accentColor}"></span>
                      {/if}
                    </div>

                    <span
                      class="droid-role"

                    >
                      {isWorking ? '⚡ ATIVO' : droid.planBadge}
                    </span>
                  </div>

                  <div class="flex items-center gap-1.5 mt-0.5">
                    <span class="text-[12px] font-mono font-bold" style="color: {isWorking ? droid.accentColor : '#94a3b8'}">
                      {isWorking ? (droid.status === 'THINKING' ? '💭 Elaborando...' : '⚡ Codificando...') : droid.status === 'ONLINE' ? '● Pronto' : '● Idle'}
                    </span>
                    <span class="text-[12px] text-slate-600 font-mono">•</span>
                    <span class="text-[12px] font-mono text-cyan-400/90">
                      {(droid.tokens / 1000).toFixed(1)}k tok
                    </span>
                  </div>

                  <!-- Detalhe da Tarefa Atual / Último Status -->
                  <div class="droid-task mt-2 text-[12px] text-slate-400 font-ui flex items-start gap-1.5">
                    {#if isWorking}
                      <span class="w-2 h-2 rounded-full animate-pulse shrink-0" style="background: {droid.accentColor}"></span>
                    {:else if droid.status === 'ONLINE'}
                      <CheckCircle2 size={10} class="text-emerald-400 shrink-0" />
                    {:else}
                      <Clock size={10} class="text-amber-400 shrink-0" />
                    {/if}
                    <span class="task-description" title={droid.statusDetail} style="{isWorking ? 'color: #f1f5f9; font-weight: 600;' : ''}">{droid.statusDetail}</span>
                  </div>
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/each}

      {#if filteredProjects.length === 0}
        <div class="text-center py-8 text-slate-500 font-mono text-sm">
          Nenhum projeto ou droid encontrado.
        </div>
      {/if}
    </div>

    <!-- Rodapé da Sidebar: Estatísticas Globais -->
    <div class="sidebar-footer">
      <div class="flex items-center gap-1.5">
        <CircleDot size={10} class="text-emerald-400 animate-pulse" />
        <span>Core v0.3</span>
      </div>
      <div class="text-slate-400">
        {$droidRoster.length} droids no roster
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
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-3 sm:p-4 font-ui select-none">
    <div class="w-full max-w-2xl max-h-[92vh] bg-[#090e1a] border-2 border-[#00f0ff]/50 rounded-2xl shadow-[0_0_40px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden">
      <!-- Cabeçalho -->
      <div class="px-5 py-4 border-b border-[#18263e] bg-[#0c1322] flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-cyan-950/80 border border-cyan-400/50 flex items-center justify-center text-cyan-400">
            <Terminal size={17} />
          </div>
          <div>
            <h2 class="font-ui text-sm sm:text-base text-cyan-300 font-bold tracking-normal">
              NOVO WORKSPACE & HARNESS DE DROIDS
            </h2>
            <p class="text-[12px] text-slate-400 font-mono">
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
            <label for="input-proj-name" class="text-sm text-slate-300 block mb-1 font-bold">
              Nome do Projeto:
            </label>
            <input
              id="input-proj-name"
              type="text"
              bind:value={newProjName}
              placeholder="Ex: Notex RAG Engine, Auth Microservice..."
              class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-sm px-3 py-2 rounded-lg outline-none"
            />
          </div>

          <div>
            <label for="input-proj-model" class="text-sm text-slate-300 block mb-1 font-bold">
              Modelo Principal do Terminal:
            </label>
            <select
              id="input-proj-model"
              bind:value={selectedModel}
              class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-sm px-3 py-2 rounded-lg outline-none cursor-pointer"
            >
              {#each MODEL_OPTIONS as opt}
                <option value={opt}>{opt}</option>
              {/each}
            </select>
          </div>
        </div>

        <div>
          <label for="input-proj-path" class="text-sm text-slate-300 block mb-1 font-bold">
            Caminho do Diretório (Workspace Path):
          </label>
          <input
            id="input-proj-path"
            type="text"
            bind:value={newProjPath}
            placeholder="Ex: C:\Workspace\meu-projeto"
            class="w-full bg-[#050810] border border-[#1e2a3f] focus:border-[#00f0ff] text-slate-100 text-sm px-3 py-2 rounded-lg outline-none"
          />
        </div>

        <!-- Seção de Alocação de Droids (Paperclip Harness) -->
        <div class="pt-2 border-t border-[#18263e]">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-slate-200 font-bold flex items-center gap-1.5">
              <Bot size={13} class="text-cyan-400" />
              <span>ALOCAÇÃO DE DROIDS ({selectedDroidIds.length} selecionado{selectedDroidIds.length === 1 ? '' : 's'})</span>
            </span>

            <!-- Presets Rápidos -->
            <div class="flex items-center gap-1 text-[12px]">
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
                  : 'bg-[#060a14] border-[#182436] hover:border-slate-600 hover:bg-[#101a29]'}"
              >
                <div class="flex items-center gap-2.5 min-w-0">
                  <div
                    class="w-9 h-9 rounded-lg border flex items-center justify-center bg-black/60 overflow-hidden shrink-0"
                    style="border-color: {droid.accentColor}70;"
                  >
                    {#if droid.avatarType === 'spark'}
                      <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-7 h-7 object-contain pixelated" />
                    {:else if droid.avatarType === 'hacker'}
                      <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">&lt;/&gt;</span>
                    {:else if droid.avatarType === 'scientist'}
                      <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">[EXP]</span>
                    {:else}
                      <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">🛡️</span>
                    {/if}
                  </div>

                  <div class="truncate">
                    <div class="font-ui text-sm font-bold text-slate-100 truncate">
                      {droid.name}
                    </div>
                    <div class="text-[12px] text-slate-400 font-ui truncate">
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
          class="px-3.5 py-1.5 rounded-lg bg-slate-800 text-slate-300 text-sm hover:bg-slate-700 cursor-pointer"
        >
          Cancelar
        </button>
        <button
          type="button"
          on:click={handleCreateProject}
          disabled={!newProjName.trim() || selectedDroidIds.length === 0}
          class="px-4 py-1.5 rounded-lg font-ui text-sm tracking-normal transition-all cursor-pointer {newProjName.trim() && selectedDroidIds.length > 0
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
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-3 sm:p-4 font-ui select-none">
    <div class="w-full max-w-xl max-h-[90vh] bg-[#090e1a] border-2 border-cyan-500/50 rounded-2xl shadow-[0_0_40px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden">
      <!-- Cabeçalho -->
      <div class="px-5 py-4 border-b border-[#18263e] bg-[#0c1322] flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-cyan-950/80 border border-cyan-400/50 flex items-center justify-center text-cyan-400">
            <Wrench size={17} />
          </div>
          <div>
            <h2 class="font-ui text-sm sm:text-base text-cyan-300 font-bold tracking-normal">
              HARNESS CONFIG: {reconfigTargetProject.name.toUpperCase()}
            </h2>
            <p class="text-[12px] text-slate-400 font-mono">
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
          <span class="text-sm text-slate-300 font-bold">
            Selecione os droids atribuídos ({reconfigSelectedDroidIds.length} selecionado{reconfigSelectedDroidIds.length === 1 ? '' : 's'}):
          </span>

          <div class="flex items-center gap-1 text-[12px]">
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
                : 'bg-[#060a14] border-[#182436] hover:border-slate-600 hover:bg-[#101a29]'}"
            >
              <div class="flex items-center gap-2.5 min-w-0">
                <div
                  class="w-9 h-9 rounded-lg border flex items-center justify-center bg-black/60 overflow-hidden shrink-0"
                  style="border-color: {droid.accentColor}70;"
                >
                  {#if droid.avatarType === 'spark'}
                    <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-7 h-7 object-contain pixelated" />
                  {:else if droid.avatarType === 'hacker'}
                    <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">&lt;/&gt;</span>
                  {:else if droid.avatarType === 'scientist'}
                    <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">[EXP]</span>
                  {:else}
                    <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">🛡️</span>
                  {/if}
                </div>

                <div class="truncate">
                  <div class="font-ui text-sm font-bold text-slate-100 truncate">
                    {droid.name}
                  </div>
                  <div class="text-[12px] text-slate-400 font-ui truncate">
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
          class="px-3.5 py-1.5 rounded-lg bg-slate-800 text-slate-300 text-sm hover:bg-slate-700 cursor-pointer"
        >
          Cancelar
        </button>
        <button
          type="button"
          on:click={handleSaveReconfig}
          disabled={reconfigSelectedDroidIds.length === 0}
          class="px-4 py-1.5 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-black font-bold font-ui text-sm tracking-normal transition-all cursor-pointer shadow-[0_0_15px_rgba(0,240,255,0.4)]"
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
  .workspace-sidebar { flex-shrink: 0; background: #0b111cf5; border-right: 1px solid #2a394d; }
  .sidebar-expanded { width: 320px; }
  .sidebar-collapsed { width: 64px; }
  .sidebar-header { display: flex; flex-direction: column; gap: 16px; padding: 24px 18px 18px; border-bottom: 1px solid #243247; }
  .city-brand { display: flex; align-items: center; gap: 12px; margin-bottom: 10px; }
  .brand-icon { display: flex; align-items: center; justify-content: center; width: 38px; height: 38px; color: #67e8f9; background: #132936; border: 1px solid #326174; border-radius: 8px; box-shadow: 0 0 18px #00f0ff0c; }
  .brand-name { color: #a5f3fc; font-size: 18px; line-height: 1.4; }
  .brand-subtitle { color: #a4b2c8; font-size: 9px; letter-spacing: .1em; margin-top: 3px; }
  .sidebar-heading-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .sidebar-heading-row h2 { font-size: 14px; font-weight: 600; }
  .workspace-count { margin-left: 4px; padding: 1px 5px; border-radius: 4px; background: #1a293e; color: #a4b2c8; font-size: 12px; }
  .sidebar-action { display: flex; align-items: center; gap: 5px; min-height: 30px; padding: 4px 8px; border: 1px solid #335065; border-radius: 5px; color: #b5ecf5; background: #152739; font-size: 12px; cursor: pointer; }
  .sidebar-action:hover { border-color: #67e8f9; background: #1c3447; }
  .forge-action { color: #e0b9f6; background: #241b32; border-color: #493657; }
  .sidebar-search { width: 100%; padding: 9px 30px 9px 35px; background: #101a28; border: 1px solid #2b3a50; border-radius: 6px; color: #edf3fc; font-size: 13px; }
  .project-card { flex-shrink: 0; }
  .project-card-header { padding: 14px 10px; flex-wrap: wrap; gap: 10px; }
  .droid-card { padding: 14px 10px; }
  .droid-identity { display: flex; flex-direction: column; align-items: flex-start; gap: 3px; }
  .droid-name { font-size: 14px; }
  .droid-role { color: #a4b2c8; font-family: var(--font-sans); font-size: 12px; line-height: 1.4; }
  .droid-task :global(svg) { margin-top: 3px; }
  .task-description { display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; line-clamp: 2; overflow: hidden; line-height: 1.55; overflow-wrap: anywhere; }
  .sidebar-footer { display: flex; align-items: center; flex-wrap: wrap; justify-content: space-between; gap: 8px; padding: 16px; border-top: 1px solid #243247; color: #a4b2c8; font-family: var(--font-sans); font-size: 12px; }
  @media (max-width: 600px) {
    .sidebar-expanded { position: absolute; inset: 0 auto 0 0; max-width: calc(100vw - 48px); box-shadow: 20px 0 60px #000b; }
  }
  .pixelated {
    image-rendering: pixelated;
  }
</style>
