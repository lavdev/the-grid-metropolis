<script lang="ts">
  import { selectedAgent } from '../lib/stores/gridStore';
  import { chatMessages, sendInstruction, ROSTER_AGENTS } from '../lib/stores/chatStore';
  import { X, Send, Sparkles, Cpu, HardDrive, Zap, Activity, FileCode, CheckCircle2, ChevronDown, ChevronRight, User } from '@lucide/svelte';

  let currentTab: 'CHAT' | 'FILES' | 'METRICS' = 'CHAT';
  let directInput = '';
  let expandedThoughts: Record<string, boolean> = {};

  function close() {
    selectedAgent.set(null);
  }

  $: agentDetails = $selectedAgent
    ? ROSTER_AGENTS.find((a) => a.id === $selectedAgent!.id) || {
        id: $selectedAgent.id,
        name: $selectedAgent.name,
        role: $selectedAgent.role,
        model: 'Codex / GPT-4.5',
        accent: '#00f0ff',
        specialty: 'Engenharia de Sistemas e Automação',
        description: 'Agente especializado na infraestrutura da metrópole.',
      }
    : null;

  // Filtra mensagens relevantes para este agente
  $: agentMessages = $chatMessages.filter(
    (m) =>
      m.agentId === $selectedAgent?.id ||
      (m.sender === 'user' && m.content.toLowerCase().includes(`@${$selectedAgent?.name?.toLowerCase()}`))
  );

  function handleSendDirect() {
    if (!directInput.trim() || !$selectedAgent) return;
    sendInstruction(directInput, $selectedAgent.id);
    directInput = '';
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendDirect();
    }
  }

  function toggleThought(id: string) {
    expandedThoughts[id] = !expandedThoughts[id];
  }
</script>

{#if $selectedAgent && agentDetails}
  <aside class="fixed top-14 right-0 bottom-0 w-96 sm:w-[440px] bg-slate-950/95 backdrop-blur-xl border-l border-cyan-500/30 z-40 flex flex-col justify-between font-mono select-none shadow-[0_0_40px_rgba(0,0,0,0.9)] animate-in slide-in-from-right duration-200">
    <!-- Cabeçalho do Agente (Perfil & Engine IA) -->
    <div class="p-4 border-b border-slate-800 bg-slate-900/80 shrink-0">
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-3">
          <div
            class="w-11 h-11 rounded-xl flex items-center justify-center font-black text-sm relative shrink-0 shadow-lg"
            style="background-color: {agentDetails.accent}20; border: 1.5px solid {agentDetails.accent}; color: {agentDetails.accent}"
          >
            {agentDetails.name[0]}
            {#if $selectedAgent.status === 'WORKING'}
              <span class="absolute -top-1 -right-1 w-3 h-3 bg-amber-400 rounded-full border border-black animate-ping"></span>
            {:else if $selectedAgent.status === 'WALKING'}
              <span class="absolute -top-1 -right-1 w-3 h-3 bg-cyan-400 rounded-full border border-black"></span>
            {/if}
          </div>

          <div>
            <div class="text-sm font-black text-slate-100 flex items-center gap-2">
              {agentDetails.name}
              <span class="text-[9px] px-1.5 py-0.5 rounded font-bold uppercase bg-slate-800 text-slate-300 border border-slate-700">
                {agentDetails.role}
              </span>
            </div>
            <div class="text-[11px] text-cyan-400 font-bold flex items-center gap-1 mt-0.5">
              <Cpu class="w-3 h-3" />
              <span>{agentDetails.model}</span>
            </div>
          </div>
        </div>

        <button on:click={close} class="p-1 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Tabs de Navegação da Gaveta -->
      <div class="flex items-center gap-2 mt-4 text-xs border-t border-slate-800/80 pt-2.5">
        <button
          on:click={() => (currentTab = 'CHAT')}
          class="flex-1 py-1 rounded-lg text-center font-bold transition {currentTab === 'CHAT'
            ? 'bg-cyan-950 border border-cyan-500/50 text-cyan-300'
            : 'text-slate-400 hover:text-white bg-slate-900/50'}"
        >
          💬 Chat Contínuo
        </button>
        <button
          on:click={() => (currentTab = 'FILES')}
          class="flex-1 py-1 rounded-lg text-center font-bold transition {currentTab === 'FILES'
            ? 'bg-cyan-950 border border-cyan-500/50 text-cyan-300'
            : 'text-slate-400 hover:text-white bg-slate-900/50'}"
        >
          📂 Arquivos
        </button>
        <button
          on:click={() => (currentTab = 'METRICS')}
          class="flex-1 py-1 rounded-lg text-center font-bold transition {currentTab === 'METRICS'
            ? 'bg-cyan-950 border border-cyan-500/50 text-cyan-300'
            : 'text-slate-400 hover:text-white bg-slate-900/50'}"
        >
          📊 Métricas
        </button>
      </div>
    </div>

    <!-- Conteúdo Central Baseado na Tab Ativa -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3 text-xs font-sans">
      {#if currentTab === 'CHAT'}
        {#if agentMessages.length === 0}
          <div class="flex flex-col items-center justify-center py-16 text-slate-500 font-mono space-y-2 text-center">
            <span class="text-2xl">✨</span>
            <div class="text-xs font-bold text-slate-400">Nenhuma instrução enviada para {agentDetails.name} ainda.</div>
            <div class="text-[11px] text-slate-600 max-w-[240px]">
              Digite uma instrução abaixo para ver o agente caminhando e trabalhando no mundo isométrico!
            </div>
          </div>
        {:else}
          {#each agentMessages as msg (msg.id)}
            {#if msg.sender === 'user'}
              <!-- Mensagem do Arquiteto (Usuário) -->
              <div class="flex flex-col items-end space-y-1">
                <span class="text-[10px] text-slate-500 font-mono">Você ({msg.timestamp})</span>
                <div class="max-w-[85%] p-3 rounded-2xl rounded-tr-none bg-gradient-to-r from-cyan-600 to-blue-600 text-white text-xs leading-relaxed shadow-md">
                  {msg.content}
                </div>
              </div>
            {:else}
              <!-- Resposta do Agente com Bloco de Pensamento (CoT) -->
              <div class="flex flex-col items-start space-y-1">
                <div class="flex items-center gap-1.5 text-[10px] font-mono text-cyan-400">
                  <span class="font-bold">@{agentDetails.name}</span>
                  <span class="text-slate-500">({msg.timestamp})</span>
                </div>

                <!-- Bloco de Pensamento (Chain-of-Thought) expansível -->
                {#if msg.thinking}
                  <div class="w-full rounded-xl bg-slate-900/90 border border-purple-500/30 overflow-hidden mb-1 font-mono text-[11px]">
                    <button
                      type="button"
                      on:click={() => toggleThought(msg.id)}
                      class="w-full px-3 py-1.5 bg-purple-950/40 hover:bg-purple-950/60 flex items-center justify-between text-purple-300 font-bold transition"
                    >
                      <span class="flex items-center gap-1.5">
                        <Zap class="w-3.5 h-3.5 text-purple-400 fill-purple-400" />
                        <span>Raciocínio Interno (Thinking Process)</span>
                      </span>
                      {#if expandedThoughts[msg.id]}
                        <ChevronDown class="w-3.5 h-3.5" />
                      {:else}
                        <ChevronRight class="w-3.5 h-3.5" />
                      {/if}
                    </button>

                    {#if expandedThoughts[msg.id]}
                      <div class="p-3 text-[11px] text-slate-300 whitespace-pre-wrap leading-relaxed border-t border-purple-500/20 font-sans bg-black/40">
                        {msg.thinking}
                      </div>
                    {/if}
                  </div>
                {/if}

                <!-- Conteúdo da Mensagem / Código -->
                <div class="max-w-[95%] p-3.5 rounded-2xl rounded-tl-none bg-slate-900 border border-slate-800 text-slate-200 text-xs leading-relaxed whitespace-pre-wrap font-sans shadow-md">
                  {msg.content}

                  {#if msg.filesModified && msg.filesModified.length > 0}
                    <div class="mt-2.5 pt-2 border-t border-slate-800 flex flex-wrap gap-1 font-mono text-[10px]">
                      {#each msg.filesModified as file}
                        <span class="px-2 py-0.5 rounded bg-slate-950 text-emerald-300 border border-emerald-500/30 flex items-center gap-1">
                          📄 {file}
                        </span>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          {/each}
        {/if}

      {:else if currentTab === 'FILES'}
        <div class="space-y-3 font-mono">
          <div class="text-[11px] font-bold text-slate-400 uppercase">Arquivos no Escopo deste Agente</div>
          <div class="space-y-2">
            <div class="p-3 rounded-xl bg-slate-900 border border-slate-800 text-xs">
              <div class="text-cyan-300 font-bold flex items-center gap-1.5 mb-1">
                <FileCode class="w-3.5 h-3.5" /> src/db/schema.rs
              </div>
              <div class="text-[10px] text-slate-400 font-sans">Definição dos schemas SQLite e migrações locais.</div>
            </div>

            <div class="p-3 rounded-xl bg-slate-900 border border-slate-800 text-xs">
              <div class="text-emerald-300 font-bold flex items-center gap-1.5 mb-1">
                <FileCode class="w-3.5 h-3.5" /> src/storage/driver.rs
              </div>
              <div class="text-[10px] text-slate-400 font-sans">Driver OPFS com transações seguras.</div>
            </div>
          </div>
        </div>

      {:else if currentTab === 'METRICS'}
        <div class="space-y-3 font-mono text-xs">
          <div class="p-3 rounded-xl bg-slate-900 border border-slate-800 space-y-2">
            <div class="flex justify-between text-slate-400">
              <span>Status Atual:</span>
              <span class="font-bold text-cyan-300">{$selectedAgent.status}</span>
            </div>
            <div class="flex justify-between text-slate-400">
              <span>Base / Pod:</span>
              <span class="font-bold text-slate-200">{$selectedAgent.homePodId}</span>
            </div>
            <div class="flex justify-between text-slate-400">
              <span>Tarefas Concluídas:</span>
              <span class="font-bold text-emerald-400">{$selectedAgent.metrics.tasksCompleted}</span>
            </div>
            <div class="flex justify-between text-slate-400">
              <span>Tokens Consumidos:</span>
              <span class="font-bold text-purple-400">{$selectedAgent.metrics.tokensConsumed.toLocaleString()}</span>
            </div>
          </div>

          <div class="p-3 rounded-xl bg-slate-900/60 border border-slate-800/80 text-[11px] text-slate-400 font-sans leading-relaxed">
            <strong class="text-slate-200 font-mono uppercase block mb-1">Especialidade:</strong>
            {agentDetails.specialty}
          </div>
        </div>
      {/if}
    </div>

    <!-- Campo de Envio Direto ao Agente (no rodapé da gaveta) -->
    <div class="p-3 border-t border-slate-800 bg-slate-900/90 shrink-0">
      <div class="flex items-center gap-2">
        <input
          type="text"
          bind:value={directInput}
          on:keydown={handleKeyDown}
          placeholder="Instruir @{agentDetails.name} diretamente..."
          class="flex-1 px-3 py-2 rounded-xl bg-slate-950 border border-slate-700 focus:border-cyan-400 text-slate-100 font-sans text-xs focus:outline-none"
        />
        <button
          on:click={handleSendDirect}
          disabled={!directInput.trim()}
          class="p-2 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold transition disabled:opacity-40 disabled:cursor-not-allowed shadow-[0_0_10px_rgba(6,182,212,0.4)]"
          title="Enviar instrução"
        >
          <Send class="w-4 h-4" />
        </button>
      </div>
    </div>
  </aside>
{/if}
