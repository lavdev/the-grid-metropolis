<script lang="ts">
  import { isBacklogOpen, missionBacklog, ROSTER_AGENTS, sendInstruction, deleteMission } from '../../lib/stores/chatStore';
  import { X, Plus, CheckCircle2, Clock, Cpu, FileCode, Tag, ArrowUpRight, Filter, AlertCircle, Trash2 } from '@lucide/svelte';

  let currentTab: 'ALL' | 'IN_PROGRESS' | 'COMPLETED' = 'ALL';
  let isCreating = false;

  // Form de criação
  let newTitle = '';
  let newPrompt = '';
  let newAgentId = 'agent-codex-01';
  let newPriority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL' = 'HIGH';

  function close() {
    isBacklogOpen.set(false);
    isCreating = false;
  }

  function handleCreateMission() {
    if (!newPrompt.trim()) {
      alert('Digite ao menos a instrução/prompt da missão.');
      return;
    }

    sendInstruction(newPrompt, newAgentId);

    // Reseta form
    newTitle = '';
    newPrompt = '';
    isCreating = false;
  }

  $: filteredMissions = $missionBacklog.filter((m) => {
    if (currentTab === 'ALL') return true;
    return m.status === currentTab;
  });

  const priorityColors = {
    CRITICAL: 'bg-rose-950/80 text-rose-300 border-rose-500/50',
    HIGH: 'bg-amber-950/80 text-amber-300 border-amber-500/50',
    MEDIUM: 'bg-cyan-950/80 text-cyan-300 border-cyan-500/50',
    LOW: 'bg-slate-800 text-slate-300 border-slate-700',
  };
</script>

{#if $isBacklogOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 font-mono select-none">
    <div class="w-full max-w-4xl bg-slate-950 border border-cyan-500/40 rounded-2xl shadow-[0_0_50px_rgba(0,0,0,0.9)] flex flex-col max-h-[88vh] overflow-hidden">
      <!-- Header do Modal -->
      <div class="px-6 py-4 border-b border-slate-800 bg-slate-900/90 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-cyan-950 border border-cyan-400 flex items-center justify-center text-cyan-300">
            <span class="text-sm font-black">📋</span>
          </div>
          <div>
            <div class="text-sm font-black text-slate-100 uppercase tracking-widest flex items-center gap-2">
              PAINEL DE MISSÕES & BACKLOG
              <span class="text-[10px] px-2 py-0.5 rounded font-mono bg-cyan-950 text-cyan-300 border border-cyan-500/30">
                {$missionBacklog.length} TAREFAS
              </span>
            </div>
            <div class="text-[11px] text-slate-400">GESTÃO DE DEMANDAS SINCRONIZADA COM OS AGENTES</div>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            on:click={() => (isCreating = !isCreating)}
            class="px-3 py-1.5 rounded-lg bg-cyan-950 hover:bg-cyan-900 border border-cyan-500/40 text-cyan-200 font-bold text-xs flex items-center gap-1.5 transition"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>{isCreating ? 'Cancelar' : 'Nova Missão'}</span>
          </button>

          <button on:click={close} class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition">
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Formulário de Criação Rápida de Missão (Inline) -->
      {#if isCreating}
        <div class="p-5 bg-slate-900/95 border-b border-slate-800 space-y-3 text-xs">
          <div class="text-xs font-bold text-cyan-300 uppercase tracking-wider flex items-center gap-1.5">
            <Plus class="w-3.5 h-3.5" /> Despachar Nova Demanda para o Cluster
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div class="sm:col-span-2">
              <label for="mission-prompt" class="block text-[10px] text-slate-400 uppercase font-bold mb-1">Instrução / Prompt</label>
              <input
                id="mission-prompt"
                type="text"
                bind:value={newPrompt}
                placeholder="Ex: Otimizar o algoritmo de pathfinding A* no módulo client"
                class="w-full p-2.5 rounded-lg bg-slate-950 border border-slate-700 text-slate-100 text-xs focus:border-cyan-400 focus:outline-none"
              />
            </div>

            <div>
              <label for="mission-agent" class="block text-[10px] text-slate-400 uppercase font-bold mb-1">Agente Responsável</label>
              <select
                id="mission-agent"
                bind:value={newAgentId}
                class="w-full p-2.5 rounded-lg bg-slate-950 border border-slate-700 text-slate-100 text-xs focus:border-cyan-400 focus:outline-none"
              >
                {#each ROSTER_AGENTS as agent}
                  <option value={agent.id}>@{agent.name} ({agent.model.split(' ')[0]})</option>
                {/each}
              </select>
            </div>
          </div>

          <div class="flex items-center justify-between pt-1">
            <div class="flex items-center gap-2">
              <span class="text-[10px] text-slate-400 uppercase font-bold">Prioridade:</span>
              {#each ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'] as p}
                <button
                  type="button"
                  on:click={() => (newPriority = p as any)}
                  class="px-2 py-0.5 rounded text-[10px] font-bold border transition
                  {newPriority === p ? priorityColors[p as keyof typeof priorityColors] : 'bg-slate-950 border-slate-800 text-slate-500'}"
                >
                  {p}
                </button>
              {/each}
            </div>

            <button
              on:click={handleCreateMission}
              class="px-4 py-2 rounded-lg bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-black text-xs uppercase tracking-wider transition shadow-[0_0_12px_rgba(6,182,212,0.5)] cursor-pointer"
            >
              Despachar Missão
            </button>
          </div>
        </div>
      {/if}

      <!-- Barra de Filtros / Tabs -->
      <div class="px-6 py-2.5 bg-slate-950 border-b border-slate-800/80 flex items-center justify-between text-xs">
        <div class="flex items-center gap-2">
          <button
            on:click={() => (currentTab = 'ALL')}
            class="px-3 py-1 rounded-md transition font-bold {currentTab === 'ALL'
              ? 'bg-slate-800 text-cyan-300'
              : 'text-slate-400 hover:text-white'}"
          >
            Todas ({$missionBacklog.length})
          </button>
          <button
            on:click={() => (currentTab = 'IN_PROGRESS')}
            class="px-3 py-1 rounded-md transition font-bold {currentTab === 'IN_PROGRESS'
              ? 'bg-slate-800 text-amber-300'
              : 'text-slate-400 hover:text-white'}"
          >
            Em Andamento ({$missionBacklog.filter((m) => m.status === 'IN_PROGRESS').length})
          </button>
          <button
            on:click={() => (currentTab = 'COMPLETED')}
            class="px-3 py-1 rounded-md transition font-bold {currentTab === 'COMPLETED'
              ? 'bg-slate-800 text-emerald-300'
              : 'text-slate-400 hover:text-white'}"
          >
            Concluídas ({$missionBacklog.filter((m) => m.status === 'COMPLETED').length})
          </button>
        </div>
      </div>

      <!-- Lista de Missões (Cards no estilo Linear / Notion) -->
      <div class="p-6 overflow-y-auto space-y-3 text-xs flex-1">
        {#if filteredMissions.length === 0}
          <div class="flex flex-col items-center justify-center py-16 text-slate-500 space-y-2">
            <span class="text-2xl">⚡</span>
            <div class="text-sm font-bold">Nenhuma missão encontrada nesta visualização.</div>
          </div>
        {:else}
          {#each filteredMissions as mission (mission.id)}
            <div class="p-4 rounded-xl bg-slate-900/70 border border-slate-800 hover:border-slate-700 transition space-y-3 group">
              <!-- Linha Superior: ID, Status, Prioridade e Agente -->
              <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-slate-800/60 pb-2.5">
                <div class="flex items-center gap-2">
                  <span class="text-[11px] font-mono font-bold text-slate-400 group-hover:text-cyan-300">
                    {mission.id}
                  </span>

                  <!-- Badge de Status -->
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase flex items-center gap-1
                    {mission.status === 'COMPLETED' ? 'bg-emerald-950 text-emerald-300 border border-emerald-500/40' :
                     mission.status === 'IN_PROGRESS' ? 'bg-amber-950 text-amber-300 border border-amber-500/40 animate-pulse' :
                     'bg-slate-800 text-slate-300'}">
                    {#if mission.status === 'COMPLETED'}
                      <CheckCircle2 class="w-3 h-3" /> Concluída
                    {:else if mission.status === 'IN_PROGRESS'}
                      <Clock class="w-3 h-3" /> Em Andamento
                    {:else}
                      Pendente
                    {/if}
                  </span>

                  <!-- Prioridade -->
                  <span class="px-2 py-0.5 rounded text-[9px] font-bold uppercase border {priorityColors[mission.priority]}">
                    {mission.priority}
                  </span>
                </div>

                <!-- Agente e Timestamp -->
                <div class="flex items-center gap-3 text-[11px] text-slate-400 font-mono">
                  <span class="flex items-center gap-1 text-cyan-400 font-bold">
                    <Cpu class="w-3.5 h-3.5" /> @{mission.agentName}
                  </span>
                  <span>{mission.createdAt}</span>

                  <button
                    type="button"
                    on:click={() => deleteMission(mission.id)}
                    class="p-1 rounded hover:bg-rose-950 text-slate-500 hover:text-rose-400 transition cursor-pointer"
                    title="Excluir Missão"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              <!-- Descrição / Prompt da Missão -->
              <div class="text-slate-200 font-sans text-xs leading-relaxed font-normal">
                {mission.prompt}
              </div>

              <!-- Rodapé da Missão: Arquivos e Tokens -->
              <div class="flex flex-wrap items-center justify-between gap-2 pt-1 text-[11px] text-slate-400">
                {#if mission.files && mission.files.length > 0}
                  <div class="flex items-center gap-1.5 font-mono">
                    <FileCode class="w-3.5 h-3.5 text-emerald-400" />
                    <span>Arquivos:</span>
                    <span class="text-emerald-300">{mission.files.join(', ')}</span>
                  </div>
                {/if}

                {#if mission.tokensConsumed > 0}
                  <div class="text-purple-400 font-mono font-bold text-[10px]">
                    {mission.tokensConsumed.toLocaleString()} tokens consumidos
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Rodapé do Modal -->
      <div class="px-6 py-3 border-t border-slate-800 bg-slate-900/90 flex justify-end">
        <button on:click={close} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold transition text-xs">
          Fechar Painel
        </button>
      </div>
    </div>
  </div>
{/if}
