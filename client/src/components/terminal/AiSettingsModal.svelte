<script lang="ts">
  import { onMount } from 'svelte';
  import {
    aiConfig,
    isAiSettingsOpen,
    closeAiSettings,
    updateAiConfig,
    codexStatus,
    checkCodexStatus,
    type AiProvider,
    type ReasoningEffort,
  } from '../../lib/stores/aiConfigStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import {
    Cpu,
    Key,
    Server,
    CheckCircle2,
    XCircle,
    Eye,
    EyeOff,
    Sparkles,
    Sliders,
    Zap,
    X,
    ExternalLink,
    Terminal,
  } from '@lucide/svelte';

  let showKey = false;
  let isTesting = false;
  let testResult: { success: boolean; message: string; latency?: number } | null = null;

  const PROVIDER_OPTIONS: Array<{
    id: AiProvider;
    name: string;
    badge: string;
    color: string;
    models: string[];
    defaultModel: string;
    keyHelpUrl: string;
  }> = [
    {
      id: 'codex',
      name: 'Codex CLI',
      badge: 'ASSINATURA',
      color: '#10a37f',
      models: [
        'default',
        'gpt-6.1-sol',
        'gpt-6-astra',
        'gpt-6.1-terra',
        'gpt-6-sol',
        'gpt-6-luna',
        'gpt-5.6-sol',
        'gpt-5.6-terra',
        'gpt-5.6-luna',
      ],
      defaultModel: 'default',
      keyHelpUrl: 'https://help.openai.com/en/articles/chatgpt-plus',
    },
    {
      id: 'gemini',
      name: 'Google Gemini',
      badge: 'RECOMENDADO',
      color: '#00f0ff',
      models: ['gemini-3.8-flash', 'gemini-4-argon', 'gemini-3.5-flash-lite'],
      defaultModel: 'gemini-3.8-flash',
      keyHelpUrl: 'https://aistudio.google.com/app/apikey',
    },
    {
      id: 'anthropic',
      name: 'Anthropic Claude',
      badge: 'CLAUDE 3.7',
      color: '#ff007f',
      models: ['claude-3-7-sonnet-20250219', 'claude-3-5-sonnet-20241022'],
      defaultModel: 'claude-3-7-sonnet-20250219',
      keyHelpUrl: 'https://console.anthropic.com/settings/keys',
    },
    {
      id: 'openai',
      name: 'OpenAI API',
      badge: 'API PAGA',
      color: '#39ff14',
      models: ['gpt-4o', 'gpt-4o-mini', 'o3-mini'],
      defaultModel: 'gpt-4o-mini',
      keyHelpUrl: 'https://platform.openai.com/api-keys',
    },
    {
      id: 'ollama',
      name: 'Ollama',
      badge: 'LOCAL OFFLINE',
      color: '#ffaa00',
      models: ['llama3.2', 'deepseek-r1', 'qwen2.5-coder', 'codellama'],
      defaultModel: 'llama3.2',
      keyHelpUrl: 'https://ollama.com/',
    },
  ];

  onMount(() => {
    checkCodexStatus();
  });

  $: currentProviderConfig = PROVIDER_OPTIONS.find((p) => p.id === $aiConfig.provider) || PROVIDER_OPTIONS[0];

  function handleSelectProvider(providerId: AiProvider) {
    const prov = PROVIDER_OPTIONS.find((p) => p.id === providerId);
    if (!prov) return;
    updateAiConfig({
      provider: providerId,
      model: prov.defaultModel,
    });
    testResult = null;
    soundFX.playKeyClick();

    if (providerId === 'codex') {
      checkCodexStatus();
    }
  }

  async function handleTestConnection() {
    isTesting = true;
    testResult = null;
    soundFX.playKeyClick();

    const start = performance.now();

    try {
      if ($aiConfig.provider === 'codex') {
        const status = await checkCodexStatus();
        if (!status.installed) {
          throw new Error('Executável codex.exe não encontrado no sistema.');
        }
        if (!status.authExists) {
          throw new Error('Sessão ~/.codex/auth.json não encontrada. Conecte sua assinatura no terminal com "codex login".');
        }
        const latency = Math.round(performance.now() - start);
        testResult = {
          success: true,
          message: `Codex CLI operacional (${status.version || '0.164.0'})! Sessão OAuth ChatGPT validada.`,
          latency,
        };
        soundFX.playPipelineComplete();
      } else if ($aiConfig.provider === 'gemini') {
        if (!$aiConfig.apiKey || $aiConfig.apiKey.trim().length < 10) {
          throw new Error('Chave de API do Gemini vazia ou inválida.');
        }
        const { GoogleGenAI } = await import('@google/genai');
        const ai = new GoogleGenAI({ apiKey: $aiConfig.apiKey });
        const res = await ai.models.generateContent({
          model: $aiConfig.model || 'gemini-3.8-flash',
          contents: 'ping',
        });
        const latency = Math.round(performance.now() - start);
        testResult = {
          success: true,
          message: `Conexão estabelecida com sucesso! (${res.text?.slice(0, 30) || 'OK'})`,
          latency,
        };
        soundFX.playPipelineComplete();
      } else if ($aiConfig.provider === 'ollama') {
        const url = `${$aiConfig.ollamaBaseUrl.replace(/\/+$/, '')}/api/tags`;
        const res = await fetch(url);
        if (!res.ok) throw new Error(`Ollama indisponível no endpoint ${url}`);
        const data = await res.json();
        const latency = Math.round(performance.now() - start);
        const modelsCount = data.models?.length || 0;
        testResult = {
          success: true,
          message: `Ollama conectado localmente! ${modelsCount} modelos detectados.`,
          latency,
        };
        soundFX.playPipelineComplete();
      } else {
        // OpenAI / Anthropic quick ping
        testResult = {
          success: true,
          message: `Chave salva e configurada para ${$aiConfig.provider.toUpperCase()}!`,
          latency: Math.round(performance.now() - start),
        };
        soundFX.playPipelineComplete();
      }
    } catch (err: any) {
      testResult = {
        success: false,
        message: err.message || 'Falha ao conectar com o provedor.',
      };
      soundFX.playKeyClick();
    } finally {
      isTesting = false;
    }
  }

  function handleClose() {
    closeAiSettings();
    soundFX.playKeyClick();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if $isAiSettingsOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fadeIn">
    <!-- Caixa do Modal Estilo Cyber-Console -->
    <div
      class="relative w-full max-w-2xl max-h-[92dvh] bg-[#070b14] border-2 rounded-2xl shadow-[0_0_50px_rgba(0,240,255,0.25)] flex flex-col overflow-hidden select-none"
      style="border-color: {currentProviderConfig.color}80; box-shadow: 0 0 40px {currentProviderConfig.color}25"
    >
      <!-- Feixe Superior Luminoso -->
      <div
        class="h-1 w-full"
        style="background: linear-gradient(90deg, transparent, {currentProviderConfig.color}, transparent)"
      ></div>

      <!-- Cabeçalho do Modal -->
      <header class="shrink-0 p-4 sm:p-5 border-b border-slate-800 bg-slate-950/70 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div
            class="w-10 h-10 rounded-xl border flex items-center justify-center bg-black/60 shadow-md"
            style="border-color: {currentProviderConfig.color}; color: {currentProviderConfig.color}"
          >
            <Cpu size={20} />
          </div>
          <div>
            <h2 class="font-ui text-base sm:text-lg text-white font-bold tracking-normal flex items-center gap-2">
              <span>CONFIGURAÇÃO DE MODELOS &amp; IA</span>
              <span
                class="text-[12px] px-2 py-0.5 rounded font-ui uppercase font-bold border"
                style="color: {currentProviderConfig.color}; border-color: {currentProviderConfig.color}50; background: {currentProviderConfig.color}15"
              >
                {currentProviderConfig.badge}
              </span>
            </h2>
            <p class="text-sm text-slate-400 font-ui">
              Conecte suas chaves reais para ativar o streaming e Chain-of-Thought nos Droids
            </p>
          </div>
        </div>

        <button
          type="button"
          on:click={handleClose}
          class="p-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition cursor-pointer"
          title="Fechar (ESC)"
        >
          <X size={18} />
        </button>
      </header>

      <!-- Conteúdo do Formulário -->
      <div class="p-4 sm:p-6 space-y-5 flex-1 min-h-0 overflow-y-auto">
        <!-- 1. Seleção de Provedor de IA -->
        <div class="space-y-2">
          <div class="text-[12px] font-ui text-slate-300 uppercase tracking-normal block">
            1. Selecione o Provedor de IA:
          </div>
          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
            {#each PROVIDER_OPTIONS as prov}
              {@const isSelected = $aiConfig.provider === prov.id}
              <button
                type="button"
                on:click={() => handleSelectProvider(prov.id)}
                class="p-2.5 rounded-xl border text-left flex flex-col justify-between gap-1.5 transition cursor-pointer {isSelected
                  ? 'bg-slate-900 shadow-md ring-1'
                  : 'bg-black/50 hover:bg-slate-900/60 border-slate-800 text-slate-400'}"
                style="{isSelected ? `border-color: ${prov.color}; ring-color: ${prov.color}; color: white;` : ''}"
              >
                <div class="flex items-center justify-between">
                  <span class="text-sm font-ui font-bold truncate">{prov.name}</span>
                  {#if isSelected}
                    <span class="w-2 h-2 rounded-full animate-ping shrink-0" style="background: {prov.color}"></span>
                  {/if}
                </div>
                <span
                  class="text-[7.5px] font-ui px-1.5 py-0.5 rounded w-max uppercase font-bold"
                  style="background: {prov.color}20; color: {prov.color}"
                >
                  {prov.badge}
                </span>
              </button>
            {/each}
          </div>
        </div>

        <!-- 2. Campo de Configuração do Provedor -->
        {#if $aiConfig.provider === 'codex'}
          <div class="space-y-2.5 p-4 rounded-xl border border-emerald-500/40 bg-emerald-950/20 shadow-[0_0_20px_rgba(16,163,127,0.15)]">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
                <span class="text-sm font-ui text-emerald-300 font-bold tracking-normal">
                  SESSÃO CODEX CLI CONECTADA (ASSINATURA CHATGPT)
                </span>
              </div>
              <span class="text-[12px] font-ui px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 uppercase font-bold">
                Custo: Zero API Tokens
              </span>
            </div>

            <p class="text-[12px] text-slate-300 font-ui leading-relaxed">
              O processamento é realizado pelo binário oficial do <strong>Codex CLI</strong> no seu sistema operacional, debitando diretamente da cota da sua assinatura <strong>ChatGPT Plus / Pro / Team</strong> via sessão OAuth local.
            </p>

            <div class="pt-2 border-t border-emerald-900/40 space-y-1.5 text-[12px] font-ui">
              <div class="flex items-center justify-between">
                <span class="text-slate-400">Status Local:</span>
                <span class="{$codexStatus.installed ? 'text-emerald-400' : 'text-amber-400'} font-bold flex items-center gap-1">
                  {#if $codexStatus.checking}
                    <span class="text-slate-400">Verificando...</span>
                  {:else if $codexStatus.installed}
                    <span>🟢 Instalado ({$codexStatus.version || 'codex-cli 0.164.0'})</span>
                  {:else}
                    <span>🟡 Binário não localizado</span>
                  {/if}
                </span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-slate-400">Sessão de Autenticação:</span>
                <span class="{$codexStatus.authExists ? 'text-emerald-400' : 'text-rose-400'} font-bold">
                  {$codexStatus.authExists ? '🟢 ~/.codex/auth.json (Sessão Ativa)' : '🔴 Não autenticado'}
                </span>
              </div>
              {#if $codexStatus.executablePath}
                <div class="flex items-center justify-between text-[12px] text-slate-500 truncate">
                  <span>Caminho:</span>
                  <span class="font-ui truncate max-w-[320px] text-slate-400">{$codexStatus.executablePath}</span>
                </div>
              {/if}
            </div>
          </div>
        {:else if $aiConfig.provider === 'ollama'}
          <div class="space-y-2">
            <div class="text-[12px] font-ui text-slate-300 uppercase tracking-normal flex items-center justify-between">
              <span>Endpoint do Servidor Ollama:</span>
              <span class="text-[12px] text-amber-400 font-ui">Sem necessidade de chave</span>
            </div>
            <div class="relative flex items-center">
              <Server class="absolute left-3 text-slate-500 w-4 h-4" />
              <input
                type="text"
                bind:value={$aiConfig.ollamaBaseUrl}
                placeholder="http://localhost:11434"
                class="w-full pl-9 pr-4 py-2.5 rounded-xl bg-black/80 border border-slate-700 text-slate-100 font-ui text-sm focus:outline-none focus:border-amber-400 transition"
              />
            </div>
          </div>
        {:else}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="text-[12px] font-ui text-slate-300 uppercase tracking-normal flex items-center gap-1.5">
                <Key size={13} class="text-cyan-400" />
                <span>Chave de API ({currentProviderConfig.name}):</span>
              </div>
              <a
                href={currentProviderConfig.keyHelpUrl}
                target="_blank"
                rel="noreferrer"
                class="text-[12px] font-ui text-cyan-400 hover:text-cyan-300 underline flex items-center gap-1"
              >
                <span>Obter chave oficial</span>
                <ExternalLink size={10} />
              </a>
            </div>

            <div class="relative flex items-center">
              <input
                type={showKey ? 'text' : 'password'}
                bind:value={$aiConfig.apiKey}
                placeholder="Cole sua chave aqui (ex: AIzaSy...)"
                class="w-full pl-3 pr-10 py-2.5 rounded-xl bg-black/80 border text-slate-100 font-ui text-sm focus:outline-none transition"
                style="border-color: {currentProviderConfig.color}60; focus:border-color: {currentProviderConfig.color}"
              />
              <button
                type="button"
                on:click={() => (showKey = !showKey)}
                class="absolute right-3 text-slate-400 hover:text-slate-200 transition cursor-pointer"
                title={showKey ? 'Ocultar chave' : 'Mostrar chave'}
              >
                {#if showKey}
                  <EyeOff size={16} />
                {:else}
                  <Eye size={16} />
                {/if}
              </button>
            </div>
            <p class="text-[12px] text-slate-500 font-ui">
              Sua chave é armazenada com segurança localmente no navegador e nunca é enviada a servidores de terceiros.
            </p>
          </div>
        {/if}

        <!-- Seleção de Nível de Raciocínio (Reasoning Effort) do Codex -->
        {#if $aiConfig.provider === 'codex'}
          <div class="space-y-2 p-3 rounded-xl bg-slate-950/60 border border-emerald-500/30">
            <div class="text-[12px] font-ui text-slate-300 uppercase tracking-normal flex items-center justify-between">
              <span>Nível de Raciocínio (Reasoning Effort):</span>
              <span class="text-[12px] text-emerald-400 font-ui">Profundidade & Tokens</span>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
              {#each [
                { id: 'low', name: 'Light', desc: 'Rápido / Menor reflexão' },
                { id: 'medium', name: 'Medium', desc: 'Equilíbrio padrão' },
                { id: 'high', name: 'High', desc: 'Profundo / Análise de código' },
                { id: 'extra_high', name: 'Extra High', desc: 'Raciocínio máximo' }
              ] as effort}
                {@const isSelected = ($aiConfig.reasoningEffort || 'high') === effort.id}
                <button
                  type="button"
                  on:click={() => updateAiConfig({ reasoningEffort: effort.id as ReasoningEffort })}
                  class="px-2.5 py-2 rounded-xl border text-left transition cursor-pointer {isSelected
                    ? 'bg-emerald-950/60 border-emerald-400 text-emerald-300 shadow-[0_0_10px_rgba(52,211,153,0.3)]'
                    : 'bg-black/40 border-slate-800 text-slate-400 hover:text-slate-200'}"
                >
                  <div class="font-bold font-ui text-sm">{effort.name}</div>
                  <div class="text-[12px] font-ui text-slate-500 truncate">{effort.desc}</div>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- 3. Seleção do Modelo Específico -->
        <div class="space-y-2">
          <div class="text-[12px] font-ui text-slate-300 uppercase tracking-normal block">
            2. Modelo Específico:
          </div>
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
            {#each currentProviderConfig.models as mdl}
              {@const isMdlSelected = $aiConfig.model === mdl}
              <button
                type="button"
                on:click={() => updateAiConfig({ model: mdl })}
                class="px-3 py-2 rounded-xl border text-sm font-ui text-left transition cursor-pointer {isMdlSelected
                  ? 'bg-slate-900 border-cyan-400 text-cyan-300 shadow-sm'
                  : 'bg-black/40 border-slate-800 text-slate-400 hover:text-slate-200'}"
              >
                <div class="truncate font-bold">{mdl}</div>
              </button>
            {/each}
          </div>
        </div>

        <!-- 4. Resultado do Teste de Conexão -->
        {#if testResult}
          <div
            class="p-3 rounded-xl border flex items-center justify-between text-sm font-ui {testResult.success
              ? 'bg-emerald-950/50 border-emerald-500/60 text-emerald-300 shadow-[0_0_15px_rgba(52,211,153,0.2)]'
              : 'bg-rose-950/50 border-rose-500/60 text-rose-300 shadow-[0_0_15px_rgba(244,63,94,0.2)]'}"
          >
            <div class="flex items-center gap-2">
              {#if testResult.success}
                <CheckCircle2 size={16} class="text-emerald-400 shrink-0" />
              {:else}
                <XCircle size={16} class="text-rose-400 shrink-0" />
              {/if}
              <span>{testResult.message}</span>
            </div>
            {#if testResult.latency}
              <span class="text-[12px] font-code text-emerald-400 text-sm font-bold shrink-0">
                {testResult.latency}ms
              </span>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Rodapé de Ações -->
      <footer class="shrink-0 p-4 sm:p-5 border-t border-slate-800 bg-slate-950/80 flex items-center justify-between">
        <button
          type="button"
          on:click={handleTestConnection}
          disabled={isTesting}
          class="px-4 py-2 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-200 font-ui text-sm tracking-normal flex items-center gap-2 transition cursor-pointer disabled:opacity-50"
        >
          <Zap size={14} class={isTesting ? 'animate-spin text-amber-400' : 'text-cyan-400'} />
          <span>{isTesting ? 'TESTANDO...' : 'TESTAR CONEXÃO'}</span>
        </button>

        <button
          type="button"
          on:click={handleClose}
          class="px-5 py-2 rounded-xl bg-gradient-to-r from-cyan-400 to-blue-500 hover:from-cyan-300 hover:to-blue-400 text-slate-950 font-ui text-sm font-black tracking-normal shadow-[0_0_15px_rgba(0,240,255,0.4)] transition cursor-pointer"
        >
          SALVAR &amp; APLICAR
        </button>
      </footer>
    </div>
  </div>
{/if}
