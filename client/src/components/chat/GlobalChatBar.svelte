<script lang="ts">
  import { sendInstruction, isBacklogOpen, ROSTER_AGENTS, missionBacklog } from '../../lib/stores/chatStore';
  import { selectedAgent } from '../../lib/stores/gridStore';
  import { Send, ListTodo, AtSign, Sparkles, Terminal, ChevronUp } from '@lucide/svelte';

  let inputText = '';
  let showMentionDropdown = false;
  let activeTargetId: string = 'agent-codex-01';

  $: activeAgent = ROSTER_AGENTS.find((a) => a.id === activeTargetId) || ROSTER_AGENTS[0];
  $: inProgressCount = $missionBacklog.filter((m) => m.status === 'IN_PROGRESS').length;

  function handleSubmit() {
    if (!inputText.trim()) return;
    sendInstruction(inputText, activeTargetId);
    inputText = '';
    showMentionDropdown = false;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === '@') {
      showMentionDropdown = true;
    }
  }

  function selectMention(agentId: string) {
    activeTargetId = agentId;
    showMentionDropdown = false;
    const ag = ROSTER_AGENTS.find((a) => a.id === agentId);
    if (ag) {
      if (!inputText.includes(`@${ag.name}`)) {
        inputText = `@${ag.name} ${inputText}`;
      }
    }
  }

  function openBacklog() {
    isBacklogOpen.set(true);
  }
</script>

<!-- Global Chat / Terminal Bar (Estilo Habbo / Habblaud) -->
<div class="fixed bottom-4 left-1/2 -translate-x-1/2 z-40 w-full max-w-4xl px-4 select-none">
  <!-- Dropdown de Menções Rápidas (@Agentes) -->
  {#if showMentionDropdown}
    <div class="mb-2 p-2 rounded-xl bg-slate-950/95 border border-cyan-500/40 backdrop-blur-md shadow-[0_0_25px_rgba(6,182,212,0.3)] grid grid-cols-2 sm:grid-cols-3 gap-2 font-mono text-xs">
      <div class="col-span-full text-[10px] text-slate-400 px-2 py-1 uppercase tracking-wider font-bold border-b border-slate-800 flex items-center justify-between">
        <span>Selecione um Agente para a Instrução</span>
        <button on:click={() => (showMentionDropdown = false)} class="text-slate-500 hover:text-white">✕</button>
      </div>

      {#each ROSTER_AGENTS as agent}
        <button
          on:click={() => selectMention(agent.id)}
          class="flex items-center gap-2.5 p-2 rounded-lg bg-slate-900/80 hover:bg-slate-800/90 border border-slate-800 hover:border-cyan-500/50 transition text-left group"
        >
          <div
            class="w-7 h-7 rounded-md flex items-center justify-center font-black text-xs shrink-0"
            style="background-color: {agent.accent}20; color: {agent.accent}; border: 1px solid {agent.accent}60"
          >
            {agent.name[0]}
          </div>
          <div class="min-w-0">
            <div class="text-xs font-bold text-slate-200 group-hover:text-cyan-300 truncate">@{agent.name}</div>
            <div class="text-[10px] text-slate-400 font-sans truncate">{agent.model}</div>
          </div>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Barra de Entrada Principal -->
  <div class="flex items-center gap-2 p-2 rounded-2xl bg-slate-950/90 border-2 border-cyan-500/40 backdrop-blur-xl shadow-[0_0_30px_rgba(0,0,0,0.8)]">
    <!-- Seletor do Agente Alvo -->
    <button
      on:click={() => (showMentionDropdown = !showMentionDropdown)}
      class="flex items-center gap-2 px-3 py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-xs font-mono font-bold transition shrink-0 group"
      title="Escolher Agente para Despacho"
    >
      <div
        class="w-5 h-5 rounded flex items-center justify-center text-[10px] font-black"
        style="background-color: {activeAgent.accent}20; color: {activeAgent.accent}; border: 1px solid {activeAgent.accent}60"
      >
        {activeAgent.name[0]}
      </div>
      <span class="text-slate-200 group-hover:text-cyan-300">@{activeAgent.name}</span>
      <span class="text-[10px] text-slate-500 hidden sm:inline">({activeAgent.model.split(' ')[0]})</span>
      <ChevronUp class="w-3.5 h-3.5 text-slate-500 group-hover:text-cyan-400 transition" />
    </button>

    <!-- Campo de Digitação -->
    <div class="flex-1 relative flex items-center">
      <input
        type="text"
        bind:value={inputText}
        on:keydown={handleKeyDown}
        placeholder="Instrua a equipe... (ex: @Rusty refatore o driver SQLite para WASM OPFS)"
        class="w-full px-4 py-2 rounded-xl bg-slate-900/80 border border-slate-800 focus:border-cyan-400 text-slate-100 font-sans text-xs sm:text-sm focus:outline-none transition leading-normal placeholder:text-slate-500"
      />
    </div>

    <!-- Botão de Envio (Send) -->
    <button
      on:click={handleSubmit}
      disabled={!inputText.trim()}
      class="px-4 py-2.5 rounded-xl font-mono text-xs font-bold uppercase tracking-wider flex items-center gap-1.5 transition shrink-0
      {inputText.trim()
        ? 'bg-gradient-to-r from-cyan-400 to-blue-500 hover:from-cyan-300 hover:to-blue-400 text-slate-950 shadow-[0_0_15px_rgba(6,182,212,0.5)] cursor-pointer active:scale-95 font-black'
        : 'bg-slate-900 text-slate-600 cursor-not-allowed border border-slate-800'}"
    >
      <Send class="w-3.5 h-3.5 fill-current" />
      <span class="hidden sm:inline">Despachar</span>
    </button>

    <!-- Divisor -->
    <div class="w-px h-6 bg-slate-800 hidden sm:block"></div>

    <!-- Botão Backlog / Missões (Estilo Linear) -->
    <button
      on:click={openBacklog}
      class="p-2 sm:px-3 sm:py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white font-mono text-xs flex items-center gap-1.5 transition shrink-0 relative"
      title="Abrir Painel de Missões & Backlog"
    >
      <ListTodo class="w-4 h-4 text-cyan-400" />
      <span class="hidden sm:inline font-bold">Missões</span>
      {#if inProgressCount > 0}
        <span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399] animate-pulse"></span>
      {/if}
    </button>
  </div>
</div>
