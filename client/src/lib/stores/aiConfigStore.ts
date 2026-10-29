import { writable, get } from 'svelte/store';
import { apiUrl } from '../api';

export type AiProvider = 'codex' | 'gemini' | 'anthropic' | 'openai' | 'ollama';

export interface CodexStatus {
  installed: boolean;
  version?: string;
  executablePath?: string;
  authExists: boolean;
  authPath?: string;
  checking: boolean;
  error?: string;
}

export const codexStatus = writable<CodexStatus>({
  installed: false,
  authExists: false,
  checking: false,
});

export async function checkCodexStatus(): Promise<CodexStatus> {
  codexStatus.update((s) => ({ ...s, checking: true, error: undefined }));
  try {
    const res = await fetch(apiUrl('/api/v1/codex/status'));
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    const data = await res.json();
    const result: CodexStatus = {
      installed: Boolean(data.installed),
      version: data.version,
      executablePath: data.executable_path,
      authExists: Boolean(data.auth_exists),
      authPath: data.auth_path,
      checking: false,
    };
    codexStatus.set(result);
    return result;
  } catch (err) {
    const fallback: CodexStatus = {
      installed: false,
      authExists: false,
      checking: false,
      error: (err as Error).message,
    };
    codexStatus.set(fallback);
    return fallback;
  }
}

export interface CodexModelOption {
  id: string;
  name: string;
  description: string;
  tier: 'flagship' | 'balanced' | 'fast' | 'legacy';
}

export const CODEX_MODELS: CodexModelOption[] = [
  { id: 'default', name: 'Default', description: 'Conjunto recomendado pelo OpenAI Codex', tier: 'flagship' },
  { id: 'gpt-6.1-sol', name: 'GPT-6.1 Sol', description: 'Next-gen flagship com alto raciocínio', tier: 'flagship' },
  { id: 'gpt-6-astra', name: 'GPT-6 Astra', description: 'Especialista em codificação sistêmica e agentes', tier: 'flagship' },
  { id: 'gpt-6.1-terra', name: 'GPT-6.1 Terra', description: 'Desempenho e velocidade balanceados', tier: 'balanced' },
  { id: 'gpt-6-sol', name: 'GPT-6 Sol', description: 'Raciocínio profundo para arquitetura e QA', tier: 'flagship' },
  { id: 'gpt-6-luna', name: 'GPT-6 Luna', description: 'Raciocínio ultrarrápido e econômico', tier: 'fast' },
  { id: 'gpt-5.6-sol', name: 'GPT-5.6 Sol', description: 'Flagship estável v5.6 de alta precisão', tier: 'balanced' },
  { id: 'gpt-5.6-terra', name: 'GPT-5.6 Terra', description: 'Versão v5.6 balanceada para codificação', tier: 'balanced' },
  { id: 'gpt-5.6-luna', name: 'GPT-5.6 Luna', description: 'Versão v5.6 rápida para tarefas frequentes', tier: 'fast' },
];

export type ReasoningEffort = 'low' | 'medium' | 'high' | 'extra_high';

export const REASONING_EFFORTS: { id: ReasoningEffort; name: string; desc: string }[] = [
  { id: 'low', name: 'Light', desc: 'Rápido, menor tempo de pensamento' },
  { id: 'medium', name: 'Medium', desc: 'Equilíbrio padrão para engenharia' },
  { id: 'high', name: 'High', desc: 'Análise profunda e revisão minuciosa' },
  { id: 'extra_high', name: 'Extra High', desc: 'Capacidade máxima de raciocínio' },
];

export interface AiConfig {
  provider: AiProvider;
  apiKey: string;
  model: string;
  codexModel: string;
  reasoningEffort: ReasoningEffort;
  ollamaBaseUrl: string;
  temperature: number;
  enableThinking: boolean;
  thinkingBudget: number;
}

const STORAGE_KEY = 'the_grid_metropolis_ai_config';

function loadInitialConfig(): AiConfig {
  const envKey = (import.meta as any).env?.VITE_GEMINI_API_KEY || '';
  const defaultConfig: AiConfig = {
    provider: 'codex',
    apiKey: envKey,
    model: 'default',
    codexModel: 'default',
    reasoningEffort: 'high',
    ollamaBaseUrl: 'http://localhost:11434',
    temperature: 0.7,
    enableThinking: true,
    thinkingBudget: 1024,
  };

  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      const saved = window.localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        return {
          ...defaultConfig,
          ...parsed,
          // Preserva chave do env se localStorage estiver vazia
          apiKey: parsed.apiKey || envKey,
        };
      }
    } catch (e) {
      console.warn('Erro ao carregar configuracao de IA do localStorage:', e);
    }
  }

  return defaultConfig;
}

export const aiConfig = writable<AiConfig>(loadInitialConfig());

// Salva automaticamente no localStorage a cada alteracao
if (typeof window !== 'undefined') {
  aiConfig.subscribe((val) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
    } catch (e) {
      console.error('Erro ao salvar configuracao de IA no localStorage:', e);
    }
  });
}

export const isAiSettingsOpen = writable<boolean>(false);

export function openAiSettings() {
  isAiSettingsOpen.set(true);
}

export function closeAiSettings() {
  isAiSettingsOpen.set(false);
}

export function updateAiConfig(updates: Partial<AiConfig>) {
  aiConfig.update((curr) => ({ ...curr, ...updates }));
}

export function hasConfiguredApiKey(): boolean {
  const cfg = get(aiConfig);
  if (cfg.provider === 'codex' || cfg.provider === 'ollama') return true; // Codex CLI e Ollama rodam localmente sem API key
  return Boolean(cfg.apiKey && cfg.apiKey.trim().length > 5);
}
