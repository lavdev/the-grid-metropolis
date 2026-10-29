import { writable, derived, get } from 'svelte/store';
import type { ChatMessage } from './chatStore';
import { soundFX } from '../audio/soundFX';
import { droidRoster, type RosterDroid } from './droidRosterStore';
import { streamAiCompletion } from '../ai/aiService';

export interface DroidInfo {
  id: string;
  name: string;
  role: string;
  model: string;
  planBadge: string;
  avatarType: 'spark' | 'hacker' | 'scientist' | 'sentinel';
  status: 'ONLINE' | 'WORKING' | 'THINKING' | 'IDLE' | 'COMPLETED';
  statusDetail: string;
  tokens: number;
  accentColor: string;
}

export type PipelineStage = 'IDLE' | 'ARCHITECT' | 'CODER' | 'REVIEWER' | 'COMPLETED';

export interface PipelineState {
  isActive: boolean;
  stage: PipelineStage;
  currentDroidId: string;
  prompt: string;
  progressPercent: number;
  statusText: string;
}

export interface ProjectTerminal {
  id: string;
  name: string;
  path: string;
  model: string;
  status: 'ACTIVE' | 'PROCESSING' | 'IDLE';
  accentColor: string;
  metrics: {
    tokensIn: number;
    tokensOut: number;
    latencyMs: number;
    cpu: string;
    mem: string;
  };
  droids: DroidInfo[];
  activeDroidId: string;
  messages: ChatMessage[];
  pipeline: PipelineState;
  threadId?: string;
}

const EMPTY_PROJECT_ID = '__no_workspace__';
const EMPTY_PROJECT: ProjectTerminal = {
  id: EMPTY_PROJECT_ID,
  name: 'Nenhum workspace selecionado',
  path: 'Crie um workspace para iniciar uma sessão persistente',
  model: '—',
  status: 'ACTIVE',
  accentColor: '#64748b',
  metrics: { tokensIn: 0, tokensOut: 0, latencyMs: 0, cpu: '—', mem: '—' },
  droids: [],
  activeDroidId: '',
  messages: [],
  pipeline: {
    isActive: false,
    stage: 'IDLE',
    currentDroidId: '',
    prompt: '',
    progressPercent: 0,
    statusText: 'Nenhum workspace persistido',
  },
};

const INITIAL_PROJECTS: ProjectTerminal[] = [
  {
    id: 'proj-notex',
    name: 'The Notex',
    path: 'C:\\Workspace\\notex',
    model: 'CODEX CLI (ASSINATURA)',
    status: 'ACTIVE',
    accentColor: '#00f0ff',
    metrics: {
      tokensIn: 0,
      tokensOut: 0,
      latencyMs: 0,
      cpu: '||      ',
      mem: '|       ',
    },
    activeDroidId: 'spark-01',
    pipeline: {
      isActive: false,
      stage: 'IDLE',
      currentDroidId: 'spark-01',
      prompt: '',
      progressPercent: 0,
      statusText: 'Aguardando instrução do operador para The Notex',
    },
    droids: [
      {
        id: 'spark-01',
        name: 'SPARK-01',
        role: 'ARQUITETO',
        model: 'Codex Lead',
        planBadge: 'Lead Architect',
        avatarType: 'spark',
        status: 'ONLINE',
        statusDetail: 'Pronto para planejamento de arquitetura e requisitos',
        tokens: 0,
        accentColor: '#00f0ff',
      },
      {
        id: 'agent-codex-01',
        name: 'Rusty',
        role: 'PROGRAMADOR',
        model: 'Codex Systems',
        planBadge: 'Systems Coder',
        avatarType: 'hacker',
        status: 'ONLINE',
        statusDetail: 'Pronto para codificação técnica em C:\\Workspace\\notex',
        tokens: 0,
        accentColor: '#39ff14',
      },
      {
        id: 'agent-sec-05',
        name: 'Vanguard',
        role: 'REVISOR_DE_CODIGO',
        model: 'Codex QA',
        planBadge: 'Code Reviewer & QA',
        avatarType: 'sentinel',
        status: 'ONLINE',
        statusDetail: 'Pronto para revisão de código, AST e testes',
        tokens: 0,
        accentColor: '#ff007f',
      },
    ],
    messages: [
      {
        id: 'msg-notex-sys',
        sender: 'system',
        content: 'Workspace "The Notex" conectado em C:\\Workspace\\notex. Tríade oficial de droids (SPARK-01, Rusty, Vanguard) pronta.',
        timestamp: '16:58',
      },
      {
        id: 'msg-notex-spark',
        sender: 'agent',
        agentName: 'SPARK-01',
        agentRole: 'ARQUITETO',
        content: 'Tríade a postos para The Notex, Arquiteto! Aguardando sua próxima instrução para iniciarmos a esteira técnica.',
        timestamp: '16:58',
        status: 'DONE',
      },
    ],
  },
];

export const projects = writable<ProjectTerminal[]>(INITIAL_PROJECTS);
const ACTIVE_PROJECT_STORAGE_KEY = 'the_grid_metropolis_active_project_id';

function getInitialActiveProjectId(): string {
  if (typeof window !== 'undefined' && window.localStorage) {
    try {
      const saved = window.localStorage.getItem(ACTIVE_PROJECT_STORAGE_KEY);
      if (saved) return saved;
    } catch (e) {
      console.warn('Erro ao ler activeProjectId do localStorage:', e);
    }
  }
  return 'proj-notex';
}

export const activeProjectId = writable<string>(getInitialActiveProjectId());

if (typeof window !== 'undefined') {
  activeProjectId.subscribe((id) => {
    try {
      if (id) {
        window.localStorage.setItem(ACTIVE_PROJECT_STORAGE_KEY, id);
      } else {
        window.localStorage.removeItem(ACTIVE_PROJECT_STORAGE_KEY);
      }
    } catch (e) {
      console.warn('Erro ao persistir activeProjectId:', e);
    }
  });
}
export const isSidebarCollapsed = writable<boolean>(false);
export const projectSearchQuery = writable<string>('');

export const activeProject = derived(
  [projects, activeProjectId],
  ([$projects, $activeProjectId]) => {
    return $projects.find((p) => p.id === $activeProjectId) || $projects[0] || EMPTY_PROJECT;
  }
);

// --- PERSISTÊNCIA EM BANCO DE DADOS RELACIONAL SQLITE ---
const messagePersistTimers = new Map<string, ReturnType<typeof setTimeout>>();

async function persistProjectMessage(projectId: string, message: ChatMessage): Promise<void> {
  try {
    const res = await fetch(`/api/v1/projects/${projectId}/messages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ...message, projectId }),
    });

    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch (err) {
    console.error(`[SQLite] Erro ao persistir mensagem do projeto ${projectId}:`, err);
  }
}

function queueProjectMessagePersistence(projectId: string, messageId: string, delayMs = 350): void {
  const key = `${projectId}:${messageId}`;
  const pendingTimer = messagePersistTimers.get(key);
  if (pendingTimer) clearTimeout(pendingTimer);

  const timer = setTimeout(() => {
    messagePersistTimers.delete(key);
    const project = get(projects).find((item) => item.id === projectId);
    const message = project?.messages.find((item) => item.id === messageId);
    if (message) void persistProjectMessage(projectId, message);
  }, delayMs);

  messagePersistTimers.set(key, timer);
}

function persistProjectSnapshot(projectId: string): void {
  const project = get(projects).find((item) => item.id === projectId);
  if (project) void syncProjectToDatabase(project);
}

function getProjectThreadContext(projectId: string) {
  const project = get(projects).find((item) => item.id === projectId);
  return {
    threadId: project?.threadId,
    onThreadStarted: (threadId: string) => setProjectThreadId(projectId, threadId),
  };
}

export async function loadProjectsFromDatabase(): Promise<ProjectTerminal[]> {
  try {
    const res = await fetch('/api/v1/projects');
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data: ProjectTerminal[] = await res.json();
    if (Array.isArray(data)) {
      projects.set(data);
      const savedActiveId = typeof window !== 'undefined' ? window.localStorage.getItem(ACTIVE_PROJECT_STORAGE_KEY) : null;
      if (data.length === 0) {
        activeProjectId.set('');
      } else if (savedActiveId && data.some((p) => p.id === savedActiveId)) {
        activeProjectId.set(savedActiveId);
      } else {
        const activeId = get(activeProjectId);
        if (!data.some((p) => p.id === activeId)) {
          activeProjectId.set(data[0].id);
        }
      }
      return data;
    }
  } catch (err) {
    console.warn('[SQLite] Erro ao carregar projetos do backend:', err);
  }
  return get(projects);
}

export async function syncProjectToDatabase(project: ProjectTerminal): Promise<void> {
  try {
    const res = await fetch('/api/v1/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(project),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch (err) {
    console.error('[SQLite] Erro ao sincronizar projeto com o backend:', err);
  }
}

export async function syncProjectThreadToDatabase(projectId: string, threadId?: string): Promise<void> {
  try {
    await fetch(`/api/v1/projects/${projectId}/thread`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ thread_id: threadId || null }),
    });
  } catch (err) {
    console.error('[SQLite] Erro ao sincronizar thread_id do projeto:', err);
  }
}

export function selectProject(id: string) {
  if (id) activeProjectId.set(id);
}

export function toggleSidebar() {
  isSidebarCollapsed.update((v) => !v);
}

export function selectDroidInActiveProject(droidId: string) {
  projects.update((all) => {
    const activeId = get(activeProjectId);
    return all.map((proj) => {
      if (proj.id === activeId) {
        return { ...proj, activeDroidId: droidId };
      }
      return proj;
    });
  });
}

function getTimeStr() {
  const now = new Date();
  return now.toTimeString().split(' ')[0].substring(0, 5);
}

function appendMessageToken(projectId: string, msgId: string, token: string, isThinking = false) {
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          messages: p.messages.map((m) => {
            if (m.id === msgId) {
              if (isThinking) {
                return { ...m, thinking: (m.thinking || '') + token };
              } else {
                return { ...m, content: (m.content || '') + token };
              }
            }
            return m;
          }),
        };
      }
      return p;
    });
  });
  queueProjectMessagePersistence(projectId, msgId);
}

function finalizeMessage(
  projectId: string,
  msgId: string,
  updates: {
    status: 'DONE' | 'ERROR';
    content?: string;
    thinking?: string;
    filesModified?: string[];
  }
) {
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          messages: p.messages.map((m) => {
            if (m.id === msgId) {
              return {
                ...m,
                status: updates.status,
                ...(updates.content !== undefined ? { content: updates.content } : {}),
                ...(updates.thinking !== undefined ? { thinking: updates.thinking } : {}),
                ...(updates.filesModified ? { filesModified: updates.filesModified } : {}),
              };
            }
            return m;
          }),
        };
      }
      return p;
    });
  });
  queueProjectMessagePersistence(projectId, msgId, 0);
}

/**
 * Dispara o Pipeline Sequencial Multi-Droid ou Solo com IA e Streaming em Tempo Real
 */
export async function sendCommandToActiveProject(commandText: string) {
  if (!commandText.trim()) return;

  const currentProj = get(activeProject);
  if (!currentProj || currentProj.id === EMPTY_PROJECT_ID) return;

  const timeStr = getTimeStr();

  const userMsg: ChatMessage = {
    id: `msg-${Date.now()}`,
    sender: 'user',
    content: commandText,
    timestamp: timeStr,
  };

  const droids = currentProj.droids;

  // --- MODO 1: EXECUÇÃO SOLO DIRETA (1 Droid no Workspace) ---
  if (droids.length === 1) {
    const soloDroid = droids[0];
    const soloMsgId = `solo-${Date.now()}`;
    const initialSoloMsg: ChatMessage = {
      id: soloMsgId,
      sender: 'agent',
      agentName: soloDroid.name,
      agentRole: soloDroid.role,
      agentModel: soloDroid.model,
      content: '',
      thinking: '',
      timestamp: timeStr,
      status: 'STREAMING',
    };

    projects.update((all) => {
      return all.map((p) => {
        if (p.id === currentProj.id) {
          return {
            ...p,
            status: 'PROCESSING',
            activeDroidId: soloDroid.id,
            pipeline: {
              isActive: true,
              stage: 'ARCHITECT',
              currentDroidId: soloDroid.id,
              prompt: commandText,
              progressPercent: 40,
              statusText: `[SOLO] ${soloDroid.name} executando instrução com streaming...`,
            },
            metrics: {
              ...p.metrics,
              tokensIn: p.metrics.tokensIn + Math.floor(commandText.length * 1.5),
            },
            messages: [...p.messages, userMsg, initialSoloMsg],
          };
        }
        return p;
      });
    });

    queueProjectMessagePersistence(currentProj.id, userMsg.id, 500);
    queueProjectMessagePersistence(currentProj.id, soloMsgId, 500);

    updateDroidState(currentProj.id, soloDroid.id, 'WORKING', `Processando: "${commandText.slice(0, 28)}..."`);

    try {
      const res = await streamAiCompletion({
        prompt: commandText,
        workingDir: currentProj.path,
        droidName: soloDroid.name,
        droidRole: soloDroid.role,
        ...getProjectThreadContext(currentProj.id),
        onToken: (tok) => appendMessageToken(currentProj.id, soloMsgId, tok, false),
        onThinking: (thought) => appendMessageToken(currentProj.id, soloMsgId, thought, true),
      });

      finalizeMessage(currentProj.id, soloMsgId, {
        status: 'DONE',
        content: res.text,
        thinking: res.thinking,
        filesModified: res.filesModified,
      });

      projects.update((all) => {
        return all.map((p) => {
          if (p.id === currentProj.id) {
            return {
              ...p,
              status: 'ACTIVE',
              pipeline: {
                ...p.pipeline,
                stage: 'COMPLETED',
                progressPercent: 100,
                statusText: `Execução concluída com sucesso por ${soloDroid.name}`,
              },
              activeDroidId: soloDroid.id,
              metrics: {
                ...p.metrics,
                tokensOut: p.metrics.tokensOut + res.tokensOut,
                latencyMs: res.latencyMs,
              },
            };
          }
          return p;
        });
      });

      soundFX.playPipelineComplete();
      updateDroidState(currentProj.id, soloDroid.id, 'ONLINE', 'Comando executado com sucesso');
      persistProjectSnapshot(currentProj.id);
    } catch (err: any) {
      finalizeMessage(currentProj.id, soloMsgId, {
        status: 'ERROR',
        content: `Falha na execução: ${err.message}`,
      });
      updateDroidState(currentProj.id, soloDroid.id, 'ONLINE', 'Erro na execução');
      persistProjectSnapshot(currentProj.id);
    }

    return;
  }

  // --- MODO 2: ESTEIRA SEQUENCIAL MULTI-DROID COM STREAMING REAL ---
  const architectDroid = droids.find((d) => d.role === 'ARQUITETO' || d.role === 'CYBER_COMPANION') || droids[0];
  const coderDroid = droids.find((d) => (d.role === 'PROGRAMADOR' || d.role === 'DEVELOPER') && d.id !== architectDroid.id) || droids[1] || droids[0];
  const reviewerDroid = droids.find((d) => (d.role === 'REVISOR_DE_CODIGO' || d.role === 'SECURITY' || d.role === 'PESQUISADOR') && d.id !== architectDroid.id && d.id !== coderDroid.id) || droids[2] || droids[0];

  // Adiciona a mensagem do usuário
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          status: 'PROCESSING',
          activeDroidId: architectDroid.id,
          pipeline: {
            isActive: true,
            stage: 'ARCHITECT',
            currentDroidId: architectDroid.id,
            prompt: commandText,
            progressPercent: 15,
            statusText: `[1/3] ${architectDroid.name} elaborando especificação técnica...`,
          },
          metrics: {
            ...p.metrics,
            tokensIn: p.metrics.tokensIn + Math.floor(commandText.length * 1.5),
          },
          messages: [...p.messages, userMsg],
        };
      }
      return p;
    });
  });

  queueProjectMessagePersistence(currentProj.id, userMsg.id, 500);

  updateDroidState(currentProj.id, architectDroid.id, 'THINKING', `Estruturando arquitetura para: "${commandText.slice(0, 28)}..."`);

  // --- ETAPA 1: O Arquiteto gera a especificação técnica em streaming ---
  const archMsgId = `arch-${Date.now()}`;
  const initialArchMsg: ChatMessage = {
    id: archMsgId,
    sender: 'agent',
    agentName: architectDroid.name,
    agentRole: 'ARQUITETO',
    agentModel: architectDroid.model,
    content: '',
    thinking: '',
    timestamp: getTimeStr(),
    status: 'STREAMING',
  };

  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          messages: [...p.messages, initialArchMsg],
        };
      }
      return p;
    });
  });

  queueProjectMessagePersistence(currentProj.id, archMsgId, 500);

  let archResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    archResult = await streamAiCompletion({
      workingDir: currentProj.path,
      prompt: `Instrução do usuário: "${commandText}". Atuando como Arquiteto de Software, trace a especificação técnica, requisitos fundamentais e contratos em tópicos diretos para passar a @${coderDroid.name}.`,
      droidName: architectDroid.name,
      droidRole: 'ARQUITETO',
      ...getProjectThreadContext(currentProj.id),
      onToken: (tok) => appendMessageToken(currentProj.id, archMsgId, tok, false),
      onThinking: (thought) => appendMessageToken(currentProj.id, archMsgId, thought, true),
    });

    finalizeMessage(currentProj.id, archMsgId, {
      status: 'DONE',
      content: archResult.text,
      thinking: archResult.thinking,
      filesModified: archResult.filesModified,
    });
    updateDroidState(currentProj.id, architectDroid.id, 'ONLINE', 'Arquitetura definida');
  } catch (err: any) {
    finalizeMessage(currentProj.id, archMsgId, {
      status: 'ERROR',
      content: `Erro na arquitetura: ${err.message}`,
    });
  }

  // --- HANDOFF 1: Arquiteto -> Coder ---
  soundFX.playHandoff();
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          pipeline: {
            ...p.pipeline,
            stage: 'CODER',
            currentDroidId: coderDroid.id,
            progressPercent: 55,
            statusText: `[2/3] Handoff para ${coderDroid.name} codificar módulo...`,
          },
          activeDroidId: coderDroid.id,
          metrics: { ...p.metrics, tokensOut: p.metrics.tokensOut + archResult.tokensOut },
        };
      }
      return p;
    });
  });

  updateDroidState(currentProj.id, coderDroid.id, 'WORKING', 'Codificando implementação...');

  // --- ETAPA 2: O Coder implementa a solução com streaming ---
  const coderMsgId = `coder-${Date.now()}`;
  const initialCoderMsg: ChatMessage = {
    id: coderMsgId,
    sender: 'agent',
    agentName: coderDroid.name,
    agentRole: 'PROGRAMADOR',
    agentModel: coderDroid.model,
    content: '',
    thinking: '',
    timestamp: getTimeStr(),
    status: 'STREAMING',
  };

  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          messages: [...p.messages, initialCoderMsg],
        };
      }
      return p;
    });
  });

  queueProjectMessagePersistence(currentProj.id, coderMsgId, 500);

  let coderResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    coderResult = await streamAiCompletion({
      workingDir: currentProj.path,
      prompt: `Com base na especificação do Arquiteto:
${archResult.text}

Implemente os módulos técnicos em código limpo com blocos de código markdown tipados.`,
      droidName: coderDroid.name,
      droidRole: 'PROGRAMADOR',
      ...getProjectThreadContext(currentProj.id),
      onToken: (tok) => appendMessageToken(currentProj.id, coderMsgId, tok, false),
      onThinking: (thought) => appendMessageToken(currentProj.id, coderMsgId, thought, true),
    });

    finalizeMessage(currentProj.id, coderMsgId, {
      status: 'DONE',
      content: coderResult.text,
      thinking: coderResult.thinking,
      filesModified: coderResult.filesModified,
    });
    updateDroidState(currentProj.id, coderDroid.id, 'ONLINE', 'Código gerado');
  } catch (err: any) {
    finalizeMessage(currentProj.id, coderMsgId, {
      status: 'ERROR',
      content: `Erro na codificação: ${err.message}`,
    });
  }

  // --- HANDOFF 2: Coder -> Revisor ---
  soundFX.playHandoff();
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          pipeline: {
            ...p.pipeline,
            stage: 'REVIEWER',
            currentDroidId: reviewerDroid.id,
            progressPercent: 85,
            statusText: `[3/3] Handoff para ${reviewerDroid.name} auditar qualidade...`,
          },
          activeDroidId: reviewerDroid.id,
          metrics: { ...p.metrics, tokensOut: p.metrics.tokensOut + coderResult.tokensOut },
        };
      }
      return p;
    });
  });

  updateDroidState(currentProj.id, reviewerDroid.id, 'WORKING', 'Auditando conformidade...');

  // --- ETAPA 3: O Revisor audita e valida a entrega com streaming ---
  const revMsgId = `rev-${Date.now()}`;
  const initialRevMsg: ChatMessage = {
    id: revMsgId,
    sender: 'agent',
    agentName: reviewerDroid.name,
    agentRole: 'REVISOR_DE_CODIGO',
    agentModel: reviewerDroid.model,
    content: '',
    thinking: '',
    timestamp: getTimeStr(),
    status: 'STREAMING',
  };

  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          messages: [...p.messages, initialRevMsg],
        };
      }
      return p;
    });
  });

  queueProjectMessagePersistence(currentProj.id, revMsgId, 500);

  let revResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    revResult = await streamAiCompletion({
      workingDir: currentProj.path,
      prompt: `Audite o código implementado:
${coderResult.text}

Faça um relatório conciso de qualidade, testes, possíveis falhas de segurança e dê o veredito final de aprovação do Pull Request.`,
      droidName: reviewerDroid.name,
      droidRole: 'REVISOR_DE_CODIGO',
      ...getProjectThreadContext(currentProj.id),
      onToken: (tok) => appendMessageToken(currentProj.id, revMsgId, tok, false),
      onThinking: (thought) => appendMessageToken(currentProj.id, revMsgId, thought, true),
    });

    finalizeMessage(currentProj.id, revMsgId, {
      status: 'DONE',
      content: revResult.text,
      thinking: revResult.thinking,
      filesModified: revResult.filesModified,
    });
  } catch (err: any) {
    finalizeMessage(currentProj.id, revMsgId, {
      status: 'ERROR',
      content: `Erro na revisão: ${err.message}`,
    });
  }

  // --- FINALIZAÇÃO COMPLETA DA ESTEIRA ---
  soundFX.playPipelineComplete();
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === currentProj.id) {
        return {
          ...p,
          status: 'ACTIVE',
          pipeline: {
            ...p.pipeline,
            stage: 'COMPLETED',
            currentDroidId: architectDroid.id,
            progressPercent: 100,
            statusText: 'Pipeline concluído! Todos os droids aprovaram a entrega.',
          },
          activeDroidId: architectDroid.id,
          metrics: {
            ...p.metrics,
            tokensOut: p.metrics.tokensOut + revResult.tokensOut,
            latencyMs: revResult.latencyMs,
          },
        };
      }
      return p;
    });
  });

  updateDroidState(currentProj.id, reviewerDroid.id, 'ONLINE', 'Revisão aprovada');
  updateDroidState(currentProj.id, architectDroid.id, 'COMPLETED', 'Esteira finalizada com 100% de sucesso');
  persistProjectSnapshot(currentProj.id);
}

export function updateDroidState(
  projectId: string,
  droidId: string,
  status: DroidInfo['status'],
  statusDetail?: string
) {
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          droids: p.droids.map((d) => {
            if (d.id === droidId) {
              return {
                ...d,
                status,
                statusDetail: statusDetail ?? d.statusDetail,
                tokens: d.tokens + 150,
              };
            }
            return d;
          }),
        };
      }
      return p;
    });
  });
}

export function createNewProject(
  name: string,
  path: string,
  selectedDroidIds: string[] = ['spark-01', 'agent-codex-01', 'agent-sec-05'],
  model = 'OMNI-AGENT v4.2'
) {
  const newId = `proj-${Date.now()}`;
  const roster = get(droidRoster);

  // Mapeia os droids selecionados a partir do Droid Roster
  const assignedDroids: DroidInfo[] = selectedDroidIds.map((droidId) => {
    const fromRoster = roster.find((r) => r.id === droidId);
    if (fromRoster) {
      return {
        id: fromRoster.id,
        name: fromRoster.name,
        role: fromRoster.role,
        model: fromRoster.model,
        planBadge: fromRoster.planBadge,
        avatarType: fromRoster.avatarType,
        status: 'ONLINE' as const,
        statusDetail: 'Alocado no workspace e pronto para tarefas',
        tokens: 0,
        accentColor: fromRoster.accentColor,
      };
    }
    return {
      id: droidId,
      name: `DROID-${droidId.slice(0, 4).toUpperCase()}`,
      role: 'PROGRAMADOR',
      model,
      planBadge: 'Specialist',
      avatarType: 'hacker' as const,
      status: 'ONLINE' as const,
      statusDetail: 'Pronto para operação',
      tokens: 0,
      accentColor: '#39ff14',
    };
  });

  // Se nenhum droid tiver sido selecionado por engano, garante ao menos o SPARK-01
  if (assignedDroids.length === 0) {
    const spark = roster.find((r) => r.id === 'spark-01');
    if (spark) {
      assignedDroids.push({
        id: spark.id,
        name: spark.name,
        role: spark.role,
        model: spark.model,
        planBadge: spark.planBadge,
        avatarType: spark.avatarType,
        status: 'ONLINE',
        statusDetail: 'Operador solo inicializado',
        tokens: 0,
        accentColor: spark.accentColor,
      });
    }
  }

  const primaryDroid = assignedDroids[0];
  const newProj: ProjectTerminal = {
    id: newId,
    name,
    path,
    model,
    status: 'ACTIVE',
    accentColor: primaryDroid.accentColor || '#00f0ff',
    metrics: {
      tokensIn: 0,
      tokensOut: 0,
      latencyMs: 105,
      cpu: '||||    ',
      mem: '||      ',
    },
    activeDroidId: primaryDroid.id,
    pipeline: {
      isActive: false,
      stage: 'IDLE',
      currentDroidId: primaryDroid.id,
      prompt: '',
      progressPercent: 0,
      statusText: assignedDroids.length === 1 ? 'Pronto para execução solo' : 'Esteira multi-agente pronta',
    },
    droids: assignedDroids,
    messages: [
      {
        id: `msg-${Date.now()}`,
        sender: 'system',
        content: `Novo workspace '${name}' criado em '${path}'. Harness configurado com ${assignedDroids.length} droid(s): ${assignedDroids.map((d) => d.name).join(', ')}.`,
        timestamp: getTimeStr(),
      },
    ],
  };

  projects.update((all) => [newProj, ...all]);
  activeProjectId.set(newId);
  syncProjectToDatabase(newProj);
}

export function reconfigureProjectDroids(projectId: string, selectedDroidIds: string[]) {
  const roster = get(droidRoster);
  const matchedDroids: DroidInfo[] = selectedDroidIds.map((id) => {
    const fromRoster = roster.find((r) => r.id === id);
    if (fromRoster) {
      return {
        id: fromRoster.id,
        name: fromRoster.name,
        role: fromRoster.role,
        model: fromRoster.model,
        planBadge: fromRoster.planBadge,
        avatarType: fromRoster.avatarType,
        status: 'ONLINE' as const,
        statusDetail: 'Pronto para operação',
        tokens: 0,
        accentColor: fromRoster.accentColor,
      };
    }
    return {
      id,
      name: id,
      role: 'PROGRAMADOR',
      model: 'Codex Dev-Pro',
      planBadge: 'Specialist',
      avatarType: 'hacker' as const,
      status: 'ONLINE' as const,
      statusDetail: 'Pronto',
      tokens: 0,
      accentColor: '#39ff14',
    };
  });

  if (matchedDroids.length === 0) return;

  projects.update((all) => {
    const updatedAll = all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          droids: matchedDroids,
          activeDroidId: matchedDroids[0]?.id || p.activeDroidId,
          pipeline: {
            ...p.pipeline,
            isActive: false,
            stage: 'IDLE' as const,
            progressPercent: 0,
            statusText: matchedDroids.length === 1 ? 'Modo Solo pronto' : 'Esteira multi-agente pronta',
          },
        };
      }
      return p;
    });
    const updated = updatedAll.find((p) => p.id === projectId);
    if (updated) syncProjectToDatabase(updated);
    return updatedAll;
  });
}

export function setProjectThreadId(projectId: string, threadId: string): void {
  syncProjectThreadToDatabase(projectId, threadId);
  projects.update((all) => {
    const updatedAll = all.map((p) => {
      if (p.id === projectId) {
        return { ...p, threadId };
      }
      return p;
    });
    const updated = updatedAll.find((p) => p.id === projectId);
    if (updated) syncProjectToDatabase(updated);
    return updatedAll;
  });
}

export function resetProjectThread(projectId: string): void {
  void syncProjectThreadToDatabase(projectId, undefined);
  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        const copy = { ...p };
        return { ...p, threadId: undefined };
      }
      return p;
    });
  });
}

export async function deleteProject(projectId: string): Promise<boolean> {
  try {
    const res = await fetch(`/api/v1/projects/${projectId}`, {
      method: 'DELETE',
    });
    if (!res.ok && res.status !== 404) {
      throw new Error(`HTTP ${res.status}`);
    }
  } catch (err) {
    console.error('[SQLite] Erro ao deletar projeto no backend:', err);
    return false;
  }

  projects.update((all) => {
    const remaining = all.filter((p) => p.id !== projectId);
    if (get(activeProjectId) === projectId) {
      activeProjectId.set(remaining[0]?.id || '');
    }
    return remaining;
  });

  return true;
}

export async function deleteProjectMessage(projectId: string, messageId: string): Promise<boolean> {
  try {
    const res = await fetch(`/api/v1/projects/${projectId}/messages/${messageId}`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch (err) {
    console.error('[SQLite] Erro ao deletar mensagem no backend:', err);
    return false;
  }

  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          messages: p.messages.filter((m) => m.id !== messageId),
        };
      }
      return p;
    });
  });

  return true;
}

export async function clearProjectMessages(projectId: string): Promise<boolean> {
  try {
    const res = await fetch(`/api/v1/projects/${projectId}/messages`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch (err) {
    console.error('[SQLite] Erro ao limpar mensagens no backend:', err);
    return false;
  }

  projects.update((all) => {
    return all.map((p) => {
      if (p.id === projectId) {
        return {
          ...p,
          messages: [],
        };
      }
      return p;
    });
  });

  return true;
}

// Inicializa carregamento do SQLite no navegador
if (typeof window !== 'undefined') {
  loadProjectsFromDatabase();
}
