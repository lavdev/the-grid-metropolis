<script lang="ts">
  import {
    droidRoster,
    isDroidForgeOpen,
    editingDroidId,
    closeDroidForge,
    saveDroid,
    deleteDroid,
    type RosterDroid,
    type DroidRole,
    type AvatarType,
  } from '../../lib/stores/droidRosterStore';
  import { projects, syncRosterDroidToProjects } from '../../lib/stores/projectStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { confirmAction } from '../../lib/stores/uiFeedbackStore';
  import {
    Bot,
    Plus,
    X,
    Cpu,
    Sparkles,
    Trash2,
    Edit3,
    Terminal,
    Check,
    FolderGit2,
    Shield,
    Code,
    Search,
    Wrench,
  } from '@lucide/svelte';

  let activeTab: 'ROSTER' | 'FORGE' = 'ROSTER';

  // Campos do formulário de criação/edição
  let formId: string | undefined = undefined;
  let formName = '';
  let formRole: DroidRole = 'PROGRAMADOR';
  let formModel = 'Codex Dev-Pro';
  let formBadge = 'Systems Specialist';
  let formAvatar: AvatarType = 'hacker';
  let formAccent = '#39ff14';
  let formPrompt = '';
  let formSkillsStr = 'Rust, TypeScript, Microservices';
  let formMaxTokens = 16000;
  let formTemperature = 0.2;

  const COLOR_PALETTE = [
    { name: 'Ciano Cyber', hex: '#00f0ff' },
    { name: 'Verde Neon', hex: '#39ff14' },
    { name: 'Rosa Choque', hex: '#ff007f' },
    { name: 'Roxo Elétrico', hex: '#9d00ff' },
    { name: 'Dourado Âmbar', hex: '#ffaa00' },
    { name: 'Vermelho Carmim', hex: '#ff3b30' },
  ];

  const LLM_MODELS = [
    'Default (Codex CLI)',
    'GPT-6.1 Sol',
    'GPT-6 Astra',
    'GPT-6.1 Terra',
    'GPT-6 Sol',
    'GPT-6 Luna',
    'GPT-5.6 Sol',
    'GPT-5.6 Terra',
    'GPT-5.6 Luna',
    'Claude 3.7 Sonnet',
    'Gemini 2.0 Flash',
  ];

  const ROLE_OPTIONS: { role: DroidRole; label: string; icon: string; defaultPrompt: string }[] = [
    {
      role: 'ARQUITETO',
      label: 'Arquiteto de Sistemas',
      icon: '🏛️',
      defaultPrompt: 'Você é o Arquiteto líder. Planeje a solução técnica, desenhe contratos estritos e orquestre o fluxo.',
    },
    {
      role: 'PROGRAMADOR',
      label: 'Engenheiro de Software (Coder)',
      icon: '💻',
      defaultPrompt: 'Desenvolvedor focado em implementar código limpo, tipado e de alta performance com testes unitários.',
    },
    {
      role: 'REVISOR_DE_CODIGO',
      label: 'Revisor de Código & QA',
      icon: '🛡️',
      defaultPrompt: 'Auditor de código e Pull Requests. Analise memory safety, AST, lint e aprove sem ressalvas.',
    },
    {
      role: 'PESQUISADOR',
      label: 'Pesquisador & RAG Specialist',
      icon: '🔬',
      defaultPrompt: 'Pesquisador de contexto, busca semântica em bases de conhecimento e síntese de documentação técnica.',
    },
    {
      role: 'SECURITY',
      label: 'Cyber-Security & Firewall',
      icon: '🔒',
      defaultPrompt: 'Sentinela de segurança cibernética. Validação de autenticação JWT, isolamento de sandbox e portas.',
    },
    {
      role: 'DEVOPS',
      label: 'DevOps & CI/CD Automator',
      icon: '🚀',
      defaultPrompt: 'Especialista em automação de build, contêineres Docker, pipelines de CI/CD e monitoramento de saúde.',
    },
  ];

  // Monitora se o modal foi aberto para edição
  $: if ($editingDroidId) {
    const existing = $droidRoster.find((d) => d.id === $editingDroidId);
    if (existing) {
      formId = existing.id;
      formName = existing.name;
      formRole = existing.role;
      formModel = existing.model;
      formBadge = existing.planBadge;
      formAvatar = existing.avatarType;
      formAccent = existing.accentColor;
      formPrompt = existing.systemPrompt;
      formSkillsStr = existing.skills.join(', ');
      formMaxTokens = existing.maxTokens;
      formTemperature = existing.temperature;
      activeTab = 'FORGE';
    }
  }

  function handleOpenCreate() {
    formId = undefined;
    formName = '';
    formRole = 'PROGRAMADOR';
    formModel = 'Codex Dev-Pro';
    formBadge = 'Systems Specialist';
    formAvatar = 'hacker';
    formAccent = '#39ff14';
    formPrompt = ROLE_OPTIONS.find((r) => r.role === 'PROGRAMADOR')?.defaultPrompt || '';
    formSkillsStr = 'Rust 2024, TypeScript, Zero-Copy';
    formMaxTokens = 16000;
    formTemperature = 0.2;
    activeTab = 'FORGE';
    soundFX.playKeyClick();
  }

  function handleRoleChange(newRole: DroidRole) {
    formRole = newRole;
    const found = ROLE_OPTIONS.find((r) => r.role === newRole);
    if (found && (!formPrompt || formPrompt.length < 20)) {
      formPrompt = found.defaultPrompt;
    }
  }

  function handleSave() {
    if (!formName.trim()) return;

    const skills = formSkillsStr
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);

    const droid = {
      id: formId,
      name: formName.trim(),
      role: formRole,
      model: formModel,
      planBadge: formBadge.trim() || 'Custom Droid',
      avatarType: formAvatar,
      accentColor: formAccent,
      systemPrompt: formPrompt.trim() || 'Diretriz operacional padrão.',
      skills: skills.length > 0 ? skills : ['General Operations'],
      maxTokens: formMaxTokens,
      temperature: formTemperature,
    };

    saveDroid(droid);
    const editedDroidId = droid.id;
    if (editedDroidId) syncRosterDroidToProjects({ ...droid, id: editedDroidId });

    soundFX.playPipelineComplete();
    activeTab = 'ROSTER';
  }

  async function handleDelete(id: string) {
    if (await confirmAction({
      title: 'Remover Droid do Roster?',
      message: 'Este Droid será removido do Roster corporativo.',
      confirmLabel: 'Remover Droid',
      tone: 'danger',
    })) {
      deleteDroid(id);
      soundFX.playKeyClick();
    }
  }

  // Descobre em quais projetos um droid está alocado
  function getAssignedProjects(droidId: string): string[] {
    return $projects
      .filter((p) => p.droids.some((d) => d.id === droidId || d.name.toLowerCase() === droidId.toLowerCase()))
      .map((p) => p.name);
  }
</script>

{#if $isDroidForgeOpen}
  <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-3 sm:p-6 select-none font-ui">
    <!-- Card Principal do Droid Forge -->
    <div class="w-full max-w-4xl max-h-[92vh] bg-[#070b14] border-2 border-cyan-500/50 rounded-2xl shadow-[0_0_50px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden">
      <!-- Cabeçalho do Forge -->
      <div class="px-5 py-4 border-b border-[#182740] bg-[#0a0f1d] flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-cyan-950/80 border border-cyan-400/60 flex items-center justify-center text-cyan-300 shadow-[0_0_12px_rgba(0,240,255,0.3)]">
            <Bot size={20} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="font-ui text-base sm:text-lg text-cyan-300 font-bold tracking-normal">
                DROID FORGE & HARNESS ROSTER
              </h2>
              <span class="text-[12px] px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-emerald-400 font-ui">
                HARNESS v1.0
              </span>
            </div>
            <p class="text-sm text-slate-400 font-ui">
              Pool global de agentes sintéticos da metrópole (estilo Paperclip). Crie, configure e delegue droids por projeto.
            </p>
          </div>
        </div>

        <button
          type="button"
          on:click={closeDroidForge}
          class="w-8 h-8 rounded-lg bg-slate-900 border border-slate-700 text-slate-400 hover:text-white hover:border-cyan-400 flex items-center justify-center transition-all cursor-pointer"
          title="Fechar Forge"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Abas de Navegação -->
      <div class="px-5 pt-3 border-b border-[#162238] bg-[#080d19] flex items-center justify-between">
        <div class="flex gap-2">
          <button
            type="button"
            on:click={() => (activeTab = 'ROSTER')}
            class="px-4 py-2 rounded-t-lg font-ui text-sm tracking-normal border-t border-x transition-all cursor-pointer {activeTab === 'ROSTER'
              ? 'bg-[#0f172a] border-cyan-500/60 text-cyan-300 shadow-[0_-2px_10px_rgba(0,240,255,0.15)]'
              : 'bg-transparent border-transparent text-slate-400 hover:text-slate-200'}"
          >
            📋 ROSTER ATIVO ({$droidRoster.length})
          </button>
          <button
            type="button"
            on:click={handleOpenCreate}
            class="px-4 py-2 rounded-t-lg font-ui text-sm tracking-normal border-t border-x transition-all cursor-pointer {activeTab === 'FORGE'
              ? 'bg-[#0f172a] border-cyan-500/60 text-cyan-300 shadow-[0_-2px_10px_rgba(0,240,255,0.15)]'
              : 'bg-transparent border-transparent text-slate-400 hover:text-slate-200'}"
          >
            {formId ? '✏️ EDITAR DROID' : '⚡ FORJAR NOVO DROID'}
          </button>
        </div>

        {#if activeTab === 'ROSTER'}
          <button
            type="button"
            on:click={handleOpenCreate}
            class="px-3 py-1.5 rounded-lg bg-cyan-950/70 border border-cyan-500/60 text-cyan-300 hover:bg-cyan-500 hover:text-black font-ui text-sm flex items-center gap-1.5 transition-all cursor-pointer mb-2"
          >
            <Plus size={14} />
            <span>NOVO DROID</span>
          </button>
        {/if}
      </div>

      <!-- Conteúdo das Abas -->
      <div class="flex-1 overflow-y-auto p-5 custom-scrollbar">
        {#if activeTab === 'ROSTER'}
          <!-- Grid de Droids do Roster -->
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each $droidRoster as droid (droid.id)}
              {@const assigned = getAssignedProjects(droid.id)}
              <div
                class="rounded-xl border p-4 bg-[#0a0f1d]/90 flex flex-col justify-between transition-all hover:border-cyan-500/60 relative group"
                style="border-color: {droid.accentColor}40; box-shadow: 0 0 15px {droid.accentColor}10;"
              >
                <!-- Topo do Card -->
                <div>
                  <div class="flex items-start justify-between gap-2 mb-3">
                    <div class="flex items-center gap-3">
                      <!-- Avatar Pixel Art -->
                      <div
                        class="w-11 h-11 rounded-lg border-2 flex items-center justify-center bg-black/60 overflow-hidden shrink-0"
                        style="border-color: {droid.accentColor}; box-shadow: 0 0 12px {droid.accentColor}30"
                      >
                        {#if droid.avatarType === 'spark'}
                          <img src="/assets/spark/idle_0.png" alt={droid.name} class="w-9 h-9 object-contain pixelated" />
                        {:else if droid.avatarType === 'hacker'}
                          <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">&lt;/&gt;</span>
                        {:else if droid.avatarType === 'scientist'}
                          <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">[EXP]</span>
                        {:else}
                          <span class="text-sm font-ui font-bold" style="color: {droid.accentColor}">🛡️</span>
                        {/if}
                      </div>

                      <div>
                        <div class="flex items-center gap-1.5">
                          <h3 class="font-ui text-sm text-slate-100 font-bold">
                            {droid.name}
                          </h3>
                          {#if droid.isCustom}
                            <span class="text-[12px] px-1 rounded bg-fuchsia-950/80 text-fuchsia-400 border border-fuchsia-500/40">
                              CUSTOM
                            </span>
                          {/if}
                        </div>
                        <span
                          class="text-[12px] font-ui px-1.5 py-0.2 rounded border uppercase tracking-normal inline-block mt-0.5"
                          style="color: {droid.accentColor}; background: {droid.accentColor}15; border-color: {droid.accentColor}40"
                        >
                          {droid.planBadge}
                        </span>
                      </div>
                    </div>

                    <!-- Ações Rápidas -->
                    <div class="flex items-center gap-1">
                      <button
                        type="button"
                        on:click={() => {
                          formId = droid.id;
                          formName = droid.name;
                          formRole = droid.role;
                          formModel = droid.model;
                          formBadge = droid.planBadge;
                          formAvatar = droid.avatarType;
                          formAccent = droid.accentColor;
                          formPrompt = droid.systemPrompt;
                          formSkillsStr = droid.skills.join(', ');
                          formMaxTokens = droid.maxTokens;
                          formTemperature = droid.temperature;
                          activeTab = 'FORGE';
                        }}
                        class="p-1 rounded bg-slate-900 border border-slate-700 text-slate-400 hover:text-cyan-400 hover:border-cyan-400 transition cursor-pointer"
                        title="Editar Droid"
                      >
                        <Edit3 size={12} />
                      </button>

                      {#if droid.isCustom}
                        <button
                          type="button"
                          on:click={() => handleDelete(droid.id)}
                          class="p-1 rounded bg-slate-900 border border-slate-700 text-slate-400 hover:text-rose-400 hover:border-rose-400 transition cursor-pointer"
                          title="Remover Droid"
                        >
                          <Trash2 size={12} />
                        </button>
                      {/if}
                    </div>
                  </div>

                  <!-- Modelo e Papel -->
                  <div class="flex items-center justify-between text-[12px] text-slate-400 mb-2 pb-2 border-b border-slate-800">
                    <span class="flex items-center gap-1">
                      <Cpu size={11} class="text-slate-500" />
                      <span>{droid.model}</span>
                    </span>
                    <span class="text-cyan-400/90 font-ui font-bold">
                      {droid.role}
                    </span>
                  </div>

                  <!-- System Prompt / Persona -->
                  <p class="text-[12px] text-slate-300 line-clamp-2 italic mb-3 font-ui leading-snug">
                    "{droid.systemPrompt}"
                  </p>

                  <!-- Skills Tags -->
                  <div class="flex flex-wrap gap-1 mb-3">
                    {#each droid.skills.slice(0, 3) as skill}
                      <span class="text-[12px] px-1.5 py-0.5 rounded bg-slate-900 border border-slate-800 text-slate-300 font-ui">
                        #{skill}
                      </span>
                    {/each}
                    {#if droid.skills.length > 3}
                      <span class="text-[12px] px-1.5 py-0.5 rounded bg-slate-900 border border-slate-800 text-slate-500 font-ui">
                        +{droid.skills.length - 3}
                      </span>
                    {/if}
                  </div>
                </div>

                <!-- Rodapé do Card: Workspaces onde opera -->
                <div class="pt-2 border-t border-slate-800/80 flex items-center justify-between text-[12px] text-slate-500 font-ui">
                  <span class="flex items-center gap-1">
                    <FolderGit2 size={10} class="text-slate-400" />
                    <span>{assigned.length > 0 ? `${assigned.length} workspace(s)` : 'Banco de Reserva'}</span>
                  </span>
                  {#if assigned.length > 0}
                    <span class="text-emerald-400 truncate max-w-[120px]" title={assigned.join(', ')}>
                      ● {assigned[0]}
                    </span>
                  {:else}
                    <span class="text-slate-600">Disponível</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <!-- Tab 2: Formulário do Forge & Preview em Tempo Real -->
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <!-- Coluna 1 & 2: Formulário -->
            <div class="lg:col-span-2 flex flex-col gap-4">
              <!-- Nome e Modelo -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label for="form-name" class="text-sm text-slate-300 block mb-1 font-bold">
                    Nome do Droid:
                  </label>
                  <input
                    id="form-name"
                    type="text"
                    bind:value={formName}
                    placeholder="Ex: Byte-Runner, Sentinel-X, Cipher-9..."
                    class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none"
                  />
                </div>

                <div>
                  <label for="form-model" class="text-sm text-slate-300 block mb-1 font-bold">
                    Motor LLM de Execução:
                  </label>
                  <select
                    id="form-model"
                    bind:value={formModel}
                    class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none cursor-pointer"
                  >
                    {#each LLM_MODELS as m}
                      <option value={m}>{m}</option>
                    {/each}
                  </select>
                </div>
              </div>

              <!-- Papel e Especialidade -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label for="form-role" class="text-sm text-slate-300 block mb-1 font-bold">
                    Papel Corporativo (Harness Role):
                  </label>
                  <select
                    id="form-role"
                    bind:value={formRole}
                    on:change={() => handleRoleChange(formRole)}
                    class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none cursor-pointer"
                  >
                    {#each ROLE_OPTIONS as opt}
                      <option value={opt.role}>{opt.icon} {opt.label}</option>
                    {/each}
                  </select>
                </div>

                <div>
                  <label for="form-badge" class="text-sm text-slate-300 block mb-1 font-bold">
                    Especialidade / Título da Badge:
                  </label>
                  <input
                    id="form-badge"
                    type="text"
                    bind:value={formBadge}
                    placeholder="Ex: Rust & Kernel Specialist, UI Architect..."
                    class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none"
                  />
                </div>
              </div>

              <!-- Tipo de Avatar & Cor de Acento -->
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <span class="text-sm text-slate-300 block mb-1 font-bold">
                    Skin / Estilo do Avatar:
                  </span>
                  <div class="grid grid-cols-4 gap-2">
                    <button
                      type="button"
                      on:click={() => (formAvatar = 'spark')}
                      class="p-2 rounded-lg border text-center transition-all cursor-pointer {formAvatar === 'spark'
                        ? 'bg-cyan-950 border-cyan-400 text-cyan-300'
                        : 'bg-[#050810] border-[#1e2a40] text-slate-400'}"
                    >
                      <div class="text-base mb-1">🤖</div>
                      <div class="text-[12px] font-ui">SPARK</div>
                    </button>

                    <button
                      type="button"
                      on:click={() => (formAvatar = 'hacker')}
                      class="p-2 rounded-lg border text-center transition-all cursor-pointer {formAvatar === 'hacker'
                        ? 'bg-emerald-950 border-emerald-400 text-emerald-300'
                        : 'bg-[#050810] border-[#1e2a40] text-slate-400'}"
                    >
                      <div class="text-base mb-1">💻</div>
                      <div class="text-[12px] font-ui">HACKER</div>
                    </button>

                    <button
                      type="button"
                      on:click={() => (formAvatar = 'scientist')}
                      class="p-2 rounded-lg border text-center transition-all cursor-pointer {formAvatar === 'scientist'
                        ? 'bg-purple-950 border-purple-400 text-purple-300'
                        : 'bg-[#050810] border-[#1e2a40] text-slate-400'}"
                    >
                      <div class="text-base mb-1">🔬</div>
                      <div class="text-[12px] font-ui">CIENTISTA</div>
                    </button>

                    <button
                      type="button"
                      on:click={() => (formAvatar = 'sentinel')}
                      class="p-2 rounded-lg border text-center transition-all cursor-pointer {formAvatar === 'sentinel'
                        ? 'bg-rose-950 border-rose-400 text-rose-300'
                        : 'bg-[#050810] border-[#1e2a40] text-slate-400'}"
                    >
                      <div class="text-base mb-1">🛡️</div>
                      <div class="text-[12px] font-ui">SENTINELA</div>
                    </button>
                  </div>
                </div>

                <div>
                  <span class="text-sm text-slate-300 block mb-1 font-bold">
                    Cor Néon de Acento:
                  </span>
                  <div class="flex items-center gap-2 mt-1">
                    {#each COLOR_PALETTE as col}
                      <button
                        type="button"
                        on:click={() => (formAccent = col.hex)}
                        class="w-7 h-7 rounded-full border-2 transition-transform cursor-pointer {formAccent === col.hex ? 'scale-125 ring-2 ring-white' : 'opacity-70 hover:opacity-100'}"
                        style="background: {col.hex}; border-color: #000;"
                        title={col.name}
                      ></button>
                    {/each}
                  </div>
                </div>
              </div>

              <!-- Skills (separadas por vírgula) -->
              <div>
                <label for="form-skills" class="text-sm text-slate-300 block mb-1 font-bold">
                  Competências / Skills Técnicas (separadas por vírgula):
                </label>
                <input
                  id="form-skills"
                  type="text"
                  bind:value={formSkillsStr}
                  placeholder="Ex: Rust, WebSockets, Memory Safety, SQLite..."
                  class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none font-ui"
                />
              </div>

              <!-- System Prompt / Persona -->
              <div>
                <label for="form-prompt" class="text-sm text-slate-300 block mb-1 font-bold">
                  Persona & Diretrizes Operacionais (System Prompt):
                </label>
                <textarea
                  id="form-prompt"
                  bind:value={formPrompt}
                  rows="3"
                  placeholder="Defina o comportamento, tom de voz, regras de validação e foco de atuação deste agente..."
                  class="w-full bg-[#050810] border border-[#1e2a40] focus:border-cyan-400 text-slate-100 text-sm px-3 py-2 rounded-lg outline-none font-ui leading-relaxed resize-none"
                ></textarea>
              </div>
            </div>

            <!-- Coluna 3: Cartão Holográfico de Pré-Visualização em Tempo Real -->
            <div class="flex flex-col">
              <span class="text-sm text-slate-400 font-bold mb-2">
                HOLOGRAPHIC PREVIEW:
              </span>

              <div
                class="rounded-xl border-2 p-5 bg-[#090d19] flex-1 flex flex-col justify-between shadow-2xl relative overflow-hidden"
                style="border-color: {formAccent}; box-shadow: 0 0 25px {formAccent}25"
              >
                <!-- Luz Holográfica Superior -->
                <div
                  class="absolute top-0 left-0 right-0 h-1"
                  style="background: {formAccent}; box-shadow: 0 0 10px {formAccent}"
                ></div>

                <div>
                  <div class="flex items-center gap-3 mb-4">
                    <div
                      class="w-14 h-14 rounded-xl border-2 flex items-center justify-center bg-black/70 overflow-hidden shrink-0"
                      style="border-color: {formAccent}; box-shadow: 0 0 15px {formAccent}40"
                    >
                      {#if formAvatar === 'spark'}
                        <img src="/assets/spark/idle_0.png" alt="Preview" class="w-12 h-12 object-contain pixelated" />
                      {:else if formAvatar === 'hacker'}
                        <span class="text-lg font-ui font-bold" style="color: {formAccent}">&lt;/&gt;</span>
                      {:else if formAvatar === 'scientist'}
                        <span class="text-lg font-ui font-bold" style="color: {formAccent}">[EXP]</span>
                      {:else}
                        <span class="text-lg font-ui font-bold" style="color: {formAccent}">🛡️</span>
                      {/if}
                    </div>

                    <div>
                      <h4 class="font-ui text-base text-white font-bold tracking-normal">
                        {formName || 'NOME DO DROID'}
                      </h4>
                      <span
                        class="text-[12px] font-ui px-2 py-0.5 rounded border uppercase tracking-normal inline-block mt-1 font-bold"
                        style="color: {formAccent}; background: {formAccent}15; border-color: {formAccent}40"
                      >
                        {formBadge || 'ESPECIALIDADE'}
                      </span>
                    </div>
                  </div>

                  <div class="space-y-2 text-sm font-ui">
                    <div class="flex justify-between text-slate-400 border-b border-slate-800 pb-1">
                      <span>Papel:</span>
                      <span class="text-slate-100 font-bold">{formRole}</span>
                    </div>
                    <div class="flex justify-between text-slate-400 border-b border-slate-800 pb-1">
                      <span>Modelo:</span>
                      <span class="text-cyan-400">{formModel}</span>
                    </div>
                    <div class="flex justify-between text-slate-400 border-b border-slate-800 pb-1">
                      <span>Tokens Max:</span>
                      <span class="text-slate-200">{formMaxTokens.toLocaleString()}</span>
                    </div>
                  </div>

                  <div class="mt-4">
                    <span class="text-[12px] text-slate-500 font-ui uppercase block mb-1">
                      Diretrizes Ativas:
                    </span>
                    <p class="text-[12px] text-slate-300 font-ui italic bg-black/40 p-2.5 rounded-lg border border-slate-800/80 line-clamp-3">
                      "{formPrompt || 'Aguardando preenchimento das diretrizes...'}"
                    </p>
                  </div>
                </div>

                <!-- Botões de Ação do Form -->
                <div class="mt-6 flex flex-col gap-2">
                  <button
                    type="button"
                    on:click={handleSave}
                    disabled={!formName.trim()}
                    class="w-full py-2.5 rounded-lg font-ui text-sm tracking-normal transition-all flex items-center justify-center gap-2 cursor-pointer {formName.trim()
                      ? 'bg-cyan-500 text-black font-bold hover:bg-cyan-400 shadow-[0_0_15px_rgba(0,240,255,0.4)]'
                      : 'bg-slate-800 text-slate-500 cursor-not-allowed'}"
                  >
                    <Check size={14} />
                    <span>SALVAR DROID NO ROSTER</span>
                  </button>

                  <button
                    type="button"
                    on:click={() => (activeTab = 'ROSTER')}
                    class="w-full py-1.5 rounded-lg bg-slate-900 border border-slate-800 text-slate-400 hover:text-slate-200 text-sm transition cursor-pointer"
                  >
                    Cancelar
                  </button>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: #080c14;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #1c2738;
    border-radius: 3px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #00f0ff50;
  }
  .pixelated {
    image-rendering: pixelated;
  }
</style>
