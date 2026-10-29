<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import {
    aiConfig,
    updateAiConfig,
    CODEX_MODELS,
    REASONING_EFFORTS,
    type ReasoningEffort,
  } from '../../lib/stores/aiConfigStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { Check, ChevronDown, Sparkles, Brain } from '@lucide/svelte';

  export let isOpen = false;
  export let align: 'left' | 'right' = 'left';

  const dispatch = createEventDispatcher<{
    close: void;
    select: { model: string; reasoning: ReasoningEffort };
  }>();

  let dropdownRef: HTMLElement;

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (isOpen && dropdownRef && !dropdownRef.contains(target) && !target.closest('.model-dropdown-trigger')) {
      isOpen = false;
      dispatch('close');
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside, true);
    return () => {
      document.removeEventListener('click', handleClickOutside, true);
    };
  });

  function selectModel(modelId: string) {
    updateAiConfig({ model: modelId, codexModel: modelId });
    soundFX.playKeyClick();
    dispatch('select', { model: modelId, reasoning: $aiConfig.reasoningEffort });
    isOpen = false;
  }

  function selectReasoning(effort: ReasoningEffort) {
    updateAiConfig({ reasoningEffort: effort });
    soundFX.playKeyClick();
  }

  const modelOptions = [
    { id: 'default', label: 'Default', subtitle: 'Recommended set of models', isDefault: true },
    { id: 'gpt-6.1-sol', label: 'GPT-6.1 Sol' },
    { id: 'gpt-6-astra', label: 'GPT-6 Astra' },
    { id: 'gpt-6.1-terra', label: 'GPT-6.1 Terra' },
    { id: 'gpt-6-sol', label: 'GPT-6 Sol' },
    { id: 'gpt-6-luna', label: 'GPT-6 Luna' },
    { id: 'gpt-5.6-sol', label: 'GPT-5.6 Sol' },
    { id: 'gpt-5.6-terra', label: 'GPT-5.6 Terra' },
    { id: 'gpt-5.6-luna', label: 'GPT-5.6 Luna' },
  ];
</script>

{#if isOpen}
  <div
    bind:this={dropdownRef}
    class="absolute top-full mt-2 {align === 'right' ? 'right-0' : 'left-0'} z-50 w-64 bg-[#1f222a] border border-[#323644] rounded-2xl shadow-[0_12px_40px_rgba(0,0,0,0.85)] p-1.5 text-slate-200 select-none animate-fadeIn font-sans backdrop-blur-md"
    style="filter: drop-shadow(0 0 16px rgba(0,240,255,0.15));"
  >
    <!-- Título do Menu -->
    <div class="px-3 pt-2 pb-1.5 text-[11px] font-medium text-slate-400 tracking-normal">
      Select model
    </div>

    <!-- Opção Padrão (Default) -->
    <button
      type="button"
      on:click={() => selectModel('default')}
      class="w-full text-left px-3 py-2 rounded-xl transition cursor-pointer flex items-center justify-between group {($aiConfig.model === 'default' || $aiConfig.model === 'CODEX CLI (ASSINATURA)') ? 'bg-[#2b303c]' : 'hover:bg-[#282d38]'}"
    >
      <div>
        <div class="text-[13px] font-semibold text-white">Default</div>
        <div class="text-[10px] text-slate-400 group-hover:text-slate-300">Recommended set of models</div>
      </div>
      {#if $aiConfig.model === 'default' || $aiConfig.model === 'CODEX CLI (ASSINATURA)'}
        <Check size={15} class="text-white shrink-0 ml-2" />
      {/if}
    </button>

    <div class="my-1 border-t border-[#2e3340]"></div>

    <!-- Lista de Variantes de Modelos -->
    <div class="space-y-0.5 max-h-60 overflow-y-auto custom-scrollbar pr-0.5">
      {#each modelOptions.filter(m => !m.isDefault) as opt}
        {@const isSelected = $aiConfig.model.toLowerCase() === opt.id || $aiConfig.codexModel.toLowerCase() === opt.id}
        <button
          type="button"
          on:click={() => selectModel(opt.id)}
          class="w-full text-left px-3 py-1.5 rounded-xl transition cursor-pointer flex items-center justify-between text-[13px] {isSelected ? 'bg-[#2b303c] text-white font-medium' : 'text-slate-300 hover:bg-[#282d38] hover:text-white'}"
        >
          <span>{opt.label}</span>
          {#if isSelected}
            <Check size={15} class="text-white shrink-0 ml-2" />
          {/if}
        </button>
      {/each}
    </div>

    <div class="my-1.5 border-t border-[#2e3340]"></div>

    <!-- Controle Rápido de Reasoning Effort integrado -->
    <div class="px-3 py-1.5 bg-[#171920]/80 rounded-xl border border-[#2b303c] space-y-1.5">
      <div class="flex items-center justify-between text-[10px] text-slate-400">
        <span class="font-medium flex items-center gap-1 text-cyan-300">
          <Brain size={11} /> Reasoning Effort:
        </span>
        <span class="uppercase font-bold text-[9px] text-emerald-400">
          {$aiConfig.reasoningEffort}
        </span>
      </div>

      <div class="grid grid-cols-4 gap-1">
        {#each [
          { id: 'low', label: 'Light' },
          { id: 'medium', label: 'Medium' },
          { id: 'high', label: 'High' },
          { id: 'extra_high', label: 'Extra' }
        ] as effort}
          {@const isEffortActive = ($aiConfig.reasoningEffort || 'high') === effort.id}
          <button
            type="button"
            on:click={() => selectReasoning(effort.id as ReasoningEffort)}
            class="py-1 rounded text-[10px] font-bold text-center transition cursor-pointer {isEffortActive ? 'bg-cyan-500 text-black shadow-sm' : 'bg-[#222631] text-slate-400 hover:text-slate-200 hover:bg-[#2a2f3d]'}"
          >
            {effort.label}
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}
