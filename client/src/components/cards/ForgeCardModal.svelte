<script lang="ts">
  import { isForgeModalOpen, forgeCustomCard } from '../../lib/stores/cardStore';
  import type { TargetAIModel, CardCategory, CardRarity } from '../../types/card';
  import { X, Hammer, Zap, Sparkles, Terminal, Cpu } from '@lucide/svelte';

  let title = '';
  let subtitle = '';
  let prompt = '';
  let category: CardCategory = 'CODE';
  let rarity: CardRarity = 'UNCOMMON';
  let targetModel: TargetAIModel = 'Claude 3.7 Sonnet';
  let energyCost = 2;
  let description = '';

  function close() {
    isForgeModalOpen.set(false);
  }

  function handleForge() {
    if (!title.trim() || !prompt.trim()) {
      alert('Preencha ao menos o Título e a Diretriz (Prompt) da Carta!');
      return;
    }

    forgeCustomCard({
      title: title.trim(),
      subtitle: subtitle.trim() || 'Diretriz Personalizada do Arquiteto',
      category,
      rarity,
      energyCost,
      targetModel,
      targetRole: category === 'RESEARCH' ? 'RESEARCHER' : category === 'SECURITY' ? 'SECURITY' : 'DEVELOPER',
      prompt: prompt.trim(),
      description: description.trim() || prompt.trim().slice(0, 100),
      flavorText: 'Forjada em tempo real pelo Arquiteto nos fornos de silício da metrópole.',
      tags: ['Custom', category, targetModel.split(' ')[0]],
    });

    // Limpa campos
    title = '';
    subtitle = '';
    prompt = '';
    description = '';
  }
</script>

{#if $isForgeModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 font-mono select-none">
    <div class="w-full max-w-xl bg-slate-950 border-2 border-purple-500/60 rounded-2xl shadow-[0_0_50px_rgba(168,85,247,0.3)] flex flex-col max-h-[90vh] overflow-hidden">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-900/80">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-purple-950 border border-purple-400 flex items-center justify-center text-purple-300 shadow-[0_0_12px_rgba(168,85,247,0.5)]">
            <Hammer class="w-5 h-5" />
          </div>
          <div>
            <div class="text-sm font-black text-purple-200 uppercase tracking-widest flex items-center gap-2">
              FORJA DE CARTAS DO ARQUITETO
            </div>
            <div class="text-[11px] text-slate-400">CRIAÇÃO DE NOVAS DIRETRIZES & CARTAS DE AÇÃO</div>
          </div>
        </div>

        <button on:click={close} class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white transition">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Formulário de Forja -->
      <div class="p-6 overflow-y-auto space-y-4 text-xs text-slate-300">
        <!-- Título e Subtítulo -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label for="forge-title" class="block text-[11px] font-bold text-slate-400 uppercase mb-1">Título da Carta</label>
            <input
              id="forge-title"
              type="text"
              bind:value={title}
              placeholder="Ex: Refatorar WebSocket Client"
              class="w-full p-2.5 rounded-lg bg-slate-900 border border-slate-700 text-slate-100 font-sans text-xs focus:border-purple-400 focus:outline-none"
            />
          </div>
          <div>
            <label for="forge-subtitle" class="block text-[11px] font-bold text-slate-400 uppercase mb-1">Subtítulo / Especialidade</label>
            <input
              id="forge-subtitle"
              type="text"
              bind:value={subtitle}
              placeholder="Ex: Otimização de Buffer e Replay"
              class="w-full p-2.5 rounded-lg bg-slate-900 border border-slate-700 text-slate-100 font-sans text-xs focus:border-purple-400 focus:outline-none"
            />
          </div>
        </div>

        <!-- Modelo de IA & Custo de Energia -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label for="forge-model" class="block text-[11px] font-bold text-slate-400 uppercase mb-1 flex items-center gap-1">
              <Cpu class="w-3.5 h-3.5 text-cyan-400" /> Modelo Alvo (Engine IA)
            </label>
            <select
              id="forge-model"
              bind:value={targetModel}
              class="w-full p-2.5 rounded-lg bg-slate-900 border border-slate-700 text-slate-100 text-xs focus:border-purple-400 focus:outline-none"
            >
              <option value="Claude 3.7 Sonnet">Claude 3.7 Sonnet (Thinking & Arch)</option>
              <option value="Codex / GPT-4.5">Codex / GPT-4.5 (Code Fast)</option>
              <option value="Gemini 2.0 Flash">Gemini 2.0 Flash (Multimodal & Fast)</option>
              <option value="DeepSeek R1">DeepSeek R1 (Math & Deep Logic)</option>
            </select>
          </div>

          <div>
            <label class="block text-[11px] font-bold text-slate-400 uppercase mb-1 flex items-center gap-1">
              <Zap class="w-3.5 h-3.5 text-amber-400" /> Custo de Energia (1 a 5)
            </label>
            <div class="flex items-center gap-2">
              <input
                type="range"
                min="1"
                max="5"
                bind:value={energyCost}
                class="flex-1 accent-purple-500 cursor-pointer"
              />
              <span class="w-8 text-center font-black text-cyan-300 font-mono text-sm bg-slate-900 py-1 rounded border border-slate-700">
                {energyCost}
              </span>
            </div>
          </div>
        </div>

        <!-- Categoria e Raridade -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label for="forge-category" class="block text-[11px] font-bold text-slate-400 uppercase mb-1">Categoria da Carta</label>
            <select
              id="forge-category"
              bind:value={category}
              class="w-full p-2.5 rounded-lg bg-slate-900 border border-slate-700 text-slate-100 text-xs focus:border-purple-400 focus:outline-none"
            >
              <option value="CODE">Código & Implementação</option>
              <option value="ARCH">Arquitetura & Design</option>
              <option value="TEST">Testes & Qualidade</option>
              <option value="RESEARCH">Pesquisa & Docs</option>
              <option value="SECURITY">Segurança & Auditoria</option>
            </select>
          </div>

          <div>
            <label for="forge-rarity" class="block text-[11px] font-bold text-slate-400 uppercase mb-1">Raridade</label>
            <select
              id="forge-rarity"
              bind:value={rarity}
              class="w-full p-2.5 rounded-lg bg-slate-900 border border-slate-700 text-slate-100 text-xs focus:border-purple-400 focus:outline-none"
            >
              <option value="COMMON">Comum</option>
              <option value="UNCOMMON">Incomum</option>
              <option value="RARE">Rara</option>
              <option value="LEGENDARY">Lendária</option>
            </select>
          </div>
        </div>

        <!-- Prompt Executável -->
        <div>
          <label for="forge-prompt" class="block text-[11px] font-bold text-slate-400 uppercase mb-1 flex items-center gap-1">
            <Terminal class="w-3.5 h-3.5 text-purple-400" /> Diretriz de Execução (Prompt do Arquiteto)
          </label>
          <textarea
            id="forge-prompt"
            bind:value={prompt}
            rows="4"
            placeholder="Digite a instrução detalhada que o agente irá executar..."
            class="w-full p-3 rounded-xl bg-slate-900 border border-slate-700 focus:border-purple-400 text-slate-200 font-sans text-xs focus:outline-none leading-relaxed resize-none"
          ></textarea>
        </div>
      </div>

      <!-- Rodapé -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-900/90 flex items-center justify-between">
        <button on:click={close} class="px-4 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold transition text-xs">
          Cancelar
        </button>

        <button
          on:click={handleForge}
          class="px-6 py-2.5 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-400 hover:to-pink-400 text-white font-bold uppercase tracking-wider text-xs flex items-center gap-2 shadow-[0_0_20px_rgba(168,85,247,0.5)] transition active:scale-95"
        >
          <Sparkles class="w-4 h-4" />
          Forjar Carta e Adicionar à Mão
        </button>
      </div>
    </div>
  </div>
{/if}
