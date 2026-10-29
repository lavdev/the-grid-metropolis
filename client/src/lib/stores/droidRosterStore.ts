import { writable, get } from 'svelte/store';

export type DroidRole =
  | 'ARQUITETO'
  | 'PROGRAMADOR'
  | 'REVISOR_DE_CODIGO'
  | 'PESQUISADOR'
  | 'DEVOPS'
  | 'SECURITY';

export type AvatarType = 'spark' | 'hacker' | 'scientist' | 'sentinel';

export interface RosterDroid {
  id: string;
  name: string;
  role: DroidRole;
  model: string;
  codexModel?: string;
  reasoningEffort?: 'low' | 'medium' | 'high' | 'extra_high';
  planBadge: string;
  avatarType: AvatarType;
  accentColor: string;
  systemPrompt: string;
  skills: string[];
  maxTokens: number;
  temperature: number;
  isCustom?: boolean;
}

export const INITIAL_ROSTER: RosterDroid[] = [
  {
    id: 'spark-01',
    name: 'SPARK-01',
    role: 'ARQUITETO',
    model: 'Omni-Agent v4.2',
    codexModel: 'gpt-6.1-sol',
    reasoningEffort: 'high',
    planBadge: 'Lead Architect',
    avatarType: 'spark',
    accentColor: '#00f0ff',
    systemPrompt:
      'Você é o Arquiteto líder da The Grid Metropolis. Planeje sistemas modulares escaláveis, defina contratos estritos e coordene o handoff entre os especialistas.',
    skills: ['System Architecture', 'Tokio Concurrency', 'Protocol Design', 'Handoff Orchestration'],
    maxTokens: 32000,
    temperature: 0.3,
  },
  {
    id: 'agent-codex-01',
    name: 'Rusty',
    role: 'PROGRAMADOR',
    model: 'Codex Dev-Pro',
    codexModel: 'gpt-6-astra',
    reasoningEffort: 'medium',
    planBadge: 'Rust / Systems',
    avatarType: 'hacker',
    accentColor: '#39ff14',
    systemPrompt:
      'Desenvolvedor especialista em Rust e baixo nível. Escreva código enxuto, estritamente tipado, com testes unitários e foco em alta performance e zero-copy IO.',
    skills: ['Rust 2024', 'TypeScript', 'OPFS & SQLite', 'Zero-Copy IO', 'Unit Testing'],
    maxTokens: 16000,
    temperature: 0.2,
  },
  {
    id: 'agent-sec-05',
    name: 'Vanguard',
    role: 'REVISOR_DE_CODIGO',
    model: 'Claude 3.7 Sonnet',
    codexModel: 'gpt-6.1-sol',
    reasoningEffort: 'high',
    planBadge: 'Code Review & QA',
    avatarType: 'sentinel',
    accentColor: '#ff007f',
    systemPrompt:
      'Revisor de código e segurança implacável. Analise Pull Requests, árvore sintática (AST), memory safety, condições de corrida e aprove commits apenas sem ressalvas.',
    skills: ['AST Security Audit', 'Memory Safety', 'Threat Modeling', 'PR Review', 'Clippy Lint'],
    maxTokens: 24000,
    temperature: 0.1,
  },
  {
    id: 'agent-claude-02',
    name: 'Dr. Cipher',
    role: 'PESQUISADOR',
    model: 'Claude 3.7 Sonnet',
    codexModel: 'gpt-6-sol',
    reasoningEffort: 'high',
    planBadge: 'R&D / RAG Specialist',
    avatarType: 'scientist',
    accentColor: '#9d00ff',
    systemPrompt:
      'Cientista de dados focado em algoritmos distribuídos, sincronização de dados via CRDTs, indexação vetorial e recuperação contextual avançada.',
    skills: ['Vector Search', 'CRDTs', 'Embedding Pipelines', 'Paper Analysis', 'Context Optimization'],
    maxTokens: 32000,
    temperature: 0.4,
  },
  {
    id: 'agent-kaelen-04',
    name: 'Kaelen',
    role: 'PROGRAMADOR',
    model: 'Gemini 2.0 Flash',
    codexModel: 'gpt-6-luna',
    reasoningEffort: 'low',
    planBadge: 'Fast QA & Bench',
    avatarType: 'scientist',
    accentColor: '#ffaa00',
    systemPrompt:
      'Testador e benchmarkista de alta velocidade. Execute microbenchmarks de latência, verificação contínua de regressão e documentação técnica expressa.',
    skills: ['E2E Testing', 'Fuzzing', 'Microbenchmarks', 'Documentation', 'API Health'],
    maxTokens: 16000,
    temperature: 0.2,
  },
  {
    id: 'agent-sec-sentinel',
    name: 'Aegis-9',
    role: 'SECURITY',
    model: 'Gemini 2.0 Pro',
    codexModel: 'gpt-6.1-terra',
    reasoningEffort: 'medium',
    planBadge: 'Cyber-Sec Sentinel',
    avatarType: 'sentinel',
    accentColor: '#ff3b30',
    systemPrompt:
      'Sentinela de defesa cibernética e infraestrutura. Monitoramento de portas de rede, rotação de chaves JWT, sandbox e mitigação de ataques em tempo real.',
    skills: ['Docker Sandbox', 'Firewall Rules', 'CI/CD Pipelines', 'JWT & Auth', 'Kernel Isolation'],
    maxTokens: 24000,
    temperature: 0.1,
  },
];

const ROSTER_STORAGE_KEY = 'the_grid_metropolis_roster';

function loadInitialRoster(): RosterDroid[] {
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      const saved = window.localStorage.getItem(ROSTER_STORAGE_KEY);
      if (saved) return JSON.parse(saved);
    } catch (e) {
      console.warn('Erro ao carregar roster do localStorage:', e);
    }
  }
  return INITIAL_ROSTER;
}

export const droidRoster = writable<RosterDroid[]>(loadInitialRoster());

if (typeof window !== 'undefined') {
  droidRoster.subscribe((r) => {
    try {
      window.localStorage.setItem(ROSTER_STORAGE_KEY, JSON.stringify(r));
    } catch (e) {
      console.error('Erro ao salvar roster no localStorage:', e);
    }
  });
}
export const isDroidForgeOpen = writable<boolean>(false);
export const editingDroidId = writable<string | null>(null);

export function openDroidForge(droidIdToEdit?: string) {
  editingDroidId.set(droidIdToEdit || null);
  isDroidForgeOpen.set(true);
}

export function closeDroidForge() {
  editingDroidId.set(null);
  isDroidForgeOpen.set(false);
}

export function saveDroid(droidData: Omit<RosterDroid, 'id'> & { id?: string }) {
  droidRoster.update((roster) => {
    if (droidData.id) {
      // Atualizar existente
      return roster.map((d) => (d.id === droidData.id ? ({ ...d, ...droidData } as RosterDroid) : d));
    } else {
      // Criar novo
      const newId = `custom-droid-${Date.now()}`;
      const newDroid: RosterDroid = {
        ...droidData,
        id: newId,
        isCustom: true,
      };
      return [...roster, newDroid];
    }
  });
  closeDroidForge();
}

export function deleteDroid(id: string) {
  droidRoster.update((roster) => roster.filter((d) => d.id !== id));
}
