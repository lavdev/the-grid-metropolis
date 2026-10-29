import { writable, get } from 'svelte/store';
import { gameBridge } from '../game/gameBridge';
import { agents, tasks, selectedAgent } from './gridStore';
import type { Agent } from '../../types';

export interface ChatMessage {
  id: string;
  sender: 'user' | 'agent' | 'system';
  agentId?: string;
  agentName?: string;
  agentRole?: string;
  agentModel?: string;
  content: string;
  thinking?: string;       // Cadeia de pensamento / Chain-of-Thought
  timestamp: string;
  status?: 'STREAMING' | 'DONE' | 'ERROR';
  taskId?: string;
  filesModified?: string[];
}

export interface MissionTask {
  id: string;
  title: string;
  prompt: string;
  agentId: string;
  agentName: string;
  priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  status: 'PENDING' | 'IN_PROGRESS' | 'COMPLETED' | 'FAILED';
  createdAt: string;
  completedAt?: string;
  tokensConsumed: number;
  files?: string[];
}

export const ROSTER_AGENTS = [
  {
    id: 'agent-codex-01',
    name: 'Rusty',
    role: 'DEVELOPER',
    model: 'Codex / GPT-4.5',
    accent: '#00f0ff',
    specialty: 'Sistemas Rust, APIs Axum, WASM & Otimizações de Memória',
    description: 'Engenheiro focado em código limpo, pipelines de baixa latência e concorrência Tokio.',
  },
  {
    id: 'agent-claude-02',
    name: 'Dr. Cipher',
    role: 'RESEARCHER',
    model: 'Claude 3.7 Sonnet',
    accent: '#9d00ff',
    specialty: 'Extended Thinking, Arquitetura Hexagonal & Análise de Trade-offs',
    description: 'Arquiteto analítico com reflexão deliberada de múltiplos passos.',
  },
  {
    id: 'agent-sec-05',
    name: 'Vanguard',
    role: 'SECURITY',
    model: 'Claude 3.7 Sonnet',
    accent: '#ff3b30',
    specialty: 'Red-Team, Auditoria de Vulnerabilidades & Zero-Day',
    description: 'Sentinela de cibersegurança que protege endpoints e buffers contra invasões.',
  },
  {
    id: 'agent-kaelen-04',
    name: 'Kaelen',
    role: 'RESEARCHER',
    model: 'Gemini 2.0 Flash',
    accent: '#ff007f',
    specialty: 'Multimodal, Varredura Web & Documentações Upstream',
    description: 'Especialista em síntese veloz de documentações técnicas e referências.',
  },
  {
    id: 'agent-nova-03',
    name: 'Nova',
    role: 'DEVELOPER',
    model: 'DeepSeek R1',
    accent: '#ff9900',
    specialty: 'Algoritmos, Provas Formais & Resolução Matemática',
    description: 'Desenvolvedora voltada a pathfinding A*, matrizes e grafos de dependência.',
  },
  {
    id: 'agent-bot-01',
    name: 'Unit-7',
    role: 'BOT_RUNNER',
    model: 'CI/CD Automaton',
    accent: '#39ff14',
    specialty: 'Transporte de Dados, Builds de Produção & Automação de Rotinas',
    description: 'Androide incansável que patrulha as passarelas e entrega pacotes de dados.',
  },
];

// Stores
export const chatMessages = writable<ChatMessage[]>([
  {
    id: 'msg-init-1',
    sender: 'system',
    content: '🌐 The Grid Metropolis Command Center conectado. Digite uma instrução usando @Nome (ex: @Rusty refatore o driver SQLite) ou selecione um agente.',
    timestamp: new Date().toLocaleTimeString(),
  },
  {
    id: 'msg-init-2',
    sender: 'agent',
    agentId: 'agent-codex-01',
    agentName: 'Rusty',
    agentRole: 'DEVELOPER',
    agentModel: 'Codex / GPT-4.5',
    content: 'Sistemas a postos, Arquiteto! Pronto para refatorar ou codificar novos módulos.',
    timestamp: new Date().toLocaleTimeString(),
  },
]);

export const missionBacklog = writable<MissionTask[]>([
  {
    id: 'TASK-101',
    title: 'Implementar driver SQLite OPFS no client',
    prompt: 'Escreva a camada de persistência local usando SQLite WASM e OPFS com migrações atômicas.',
    agentId: 'agent-codex-01',
    agentName: 'Rusty',
    priority: 'HIGH',
    status: 'IN_PROGRESS',
    createdAt: '11:30:00',
    tokensConsumed: 1840,
    files: ['src/db/schema.rs', 'src/storage/driver.rs'],
  },
  {
    id: 'TASK-102',
    title: 'Auditoria de segurança dos canais WebSocket',
    prompt: 'Verificar handshake com Bearer Token e isolamento de broadcast Tokio.',
    agentId: 'agent-sec-05',
    agentName: 'Vanguard',
    priority: 'CRITICAL',
    status: 'COMPLETED',
    createdAt: '10:15:00',
    completedAt: '10:18:32',
    tokensConsumed: 3200,
    files: ['server/src/hub.rs'],
  },
]);

export const isBacklogOpen = writable<boolean>(false);

export function deleteMission(taskId: string) {
  missionBacklog.update((tasks) => tasks.filter((t) => t.id !== taskId));
}
export const activeMention = writable<string | null>(null);

/**
 * Despacha uma instrução do Arquiteto para o mundo dos agentes
 */
export function sendInstruction(rawText: string, defaultAgentId?: string) {
  const text = rawText.trim();
  if (!text) return;

  const timeStr = new Date().toLocaleTimeString();

  // 1. Identifica se há menção (@Rusty, @DrCipher, @Vanguard, etc.)
  let targetAgent = ROSTER_AGENTS.find((a) => defaultAgentId && a.id === defaultAgentId);

  for (const agent of ROSTER_AGENTS) {
    const mentionPattern = new RegExp(`@${agent.name.replace(/[^a-zA-Z0-9]/g, '')}|@${agent.name}`, 'i');
    if (mentionPattern.test(text)) {
      targetAgent = agent;
      break;
    }
  }

  // Se nenhum específico foi mencionado, padrão é Rusty
  if (!targetAgent) {
    targetAgent = ROSTER_AGENTS[0];
  }

  // Verifica se há delegação ou encaminhamento (ex: "@Rusty encaminhe para Nova..." ou "delegue para Dr. Cipher")
  const handoverMatch = text.match(/(?:encaminh(?:e|ar)|delegu(?:e|ar)|pass(?:e|ar))\s+(?:para|a)\s+@?([a-zA-Z0-9_\-\.]+)/i);
  let handoverTargetAgent: typeof ROSTER_AGENTS[0] | undefined;

  if (handoverMatch) {
    const rawTarget = handoverMatch[1].toLowerCase().replace(/[^a-z0-9]/g, '');
    handoverTargetAgent = ROSTER_AGENTS.find(
      (a) => a.name.toLowerCase().replace(/[^a-z0-9]/g, '') === rawTarget || a.id.toLowerCase().includes(rawTarget)
    );
  }

  const finalExecAgent = handoverTargetAgent || targetAgent;

  // 2. Adiciona mensagem do usuário ao chat
  const userMsgId = `usr-${Date.now()}`;
  chatMessages.update((msgs) => [
    ...msgs,
    {
      id: userMsgId,
      sender: 'user',
      content: text,
      timestamp: timeStr,
    },
  ]);

  const taskId = `TASK-${Math.floor(100 + Math.random() * 900)}`;

  // 3. Reação física imediata no Phaser (Habbo / Habblaud)
  if (handoverTargetAgent && handoverTargetAgent.id !== targetAgent.id) {
    // Pipeline Handover (T-5.3): agente primário caminha até o colega e entrega o cubo de dados
    gameBridge.orderHandoverInstruction(targetAgent.id, handoverTargetAgent.id, text);
  } else {
    // Despacho direto: caminha até o Dev Lab e senta na cadeira gamer para executar
    const speechReply = `Entendido, Arquiteto! Indo ao Dev Lab para executar: "${text.slice(0, 26)}..."`;
    gameBridge.orderAgentInstruction(targetAgent.id, speechReply);
  }

  // 4. Cria tarefa no Backlog
  const newTask: MissionTask = {
    id: taskId,
    title: text.length > 50 ? `${text.slice(0, 47)}...` : text,
    prompt: text,
    agentId: finalExecAgent.id,
    agentName: finalExecAgent.name,
    priority: 'HIGH',
    status: 'IN_PROGRESS',
    createdAt: timeStr,
    tokensConsumed: 0,
    files: ['src/main.rs', 'client/src/App.svelte'],
  };

  missionBacklog.update((tasks) => [newTask, ...tasks]);

  // Atualiza agente no gridStore para WALKING -> WORKING
  agents.update((list) =>
    list.map((a) =>
      a.id === finalExecAgent.id
        ? { ...a, status: 'WALKING', activeTaskId: taskId }
        : a
    )
  );

  // 5. Simulação do processo de pensamento (Chain-of-Thought) e resposta da IA
  const agentMsgId = `agent-${Date.now()}`;
  
  setTimeout(() => {
    // Agente senta na mesa e entra em WORKING
    agents.update((list) =>
      list.map((a) =>
        a.id === finalExecAgent.id
          ? { ...a, status: 'WORKING' }
          : a
      )
    );

    // Mensagem inicial de streaming com pensamento CoT
    chatMessages.update((msgs) => [
      ...msgs,
      {
        id: agentMsgId,
        sender: 'agent',
        agentId: finalExecAgent.id,
        agentName: finalExecAgent.name,
        agentRole: finalExecAgent.role,
        agentModel: finalExecAgent.model,
        content: `Executando diretriz com motor **${finalExecAgent.model}**...`,
        thinking: `[${finalExecAgent.model} Thinking Process]\n1. Analisando intenção do usuário: "${text}"\n2. Inspecionando escopo de arquivos e dependências...\n3. Sintetizando solução com validação de tipos...`,
        timestamp: new Date().toLocaleTimeString(),
        status: 'STREAMING',
        taskId,
      },
    ]);

    // Resolução da tarefa após alguns segundos
    setTimeout(() => {
      const generatedCodeSnippet = `\`\`\`rust\n// Implementado por ${finalExecAgent.name} via ${finalExecAgent.model}\npub async fn execute_task_${taskId.toLowerCase()}() -> Result<(), MetropolisError> {\n    tracing::info!("Diretriz concluída com sucesso no cluster.");\n    Ok(())\n}\n\`\`\``;

      chatMessages.update((msgs) =>
        msgs.map((m) =>
          m.id === agentMsgId
            ? {
                ...m,
                status: 'DONE',
                content: `Diretriz finalizada com sucesso!\n\n${generatedCodeSnippet}\n\nTodos os critérios de aceite foram validados. Deseja que eu faça o deploy ou execute os testes?`,
                thinking: `[${finalExecAgent.model} Thinking Process]\n1. Analisando intenção do usuário: "${text}"\n2. Inspecionando escopo de arquivos e dependências...\n3. Sintetizando solução com validação de tipos...\n4. Compilação concluída com 0 erros. Diff gerado.`,
                filesModified: ['src/core/mod.rs', 'src/storage/driver.rs'],
              }
            : m
        )
      );

      // Marca tarefa como concluída no Backlog
      missionBacklog.update((tasks) =>
        tasks.map((t) =>
          t.id === taskId
            ? {
                ...t,
                status: 'COMPLETED',
                completedAt: new Date().toLocaleTimeString(),
                tokensConsumed: 2150,
              }
            : t
        )
      );

      // Agente volta para IDLE e diz conclusão
      agents.update((list) =>
        list.map((a) =>
          a.id === finalExecAgent.id
            ? {
                ...a,
                status: 'IDLE',
                activeTaskId: undefined,
                metrics: {
                  ...a.metrics,
                  tasksCompleted: a.metrics.tasksCompleted + 1,
                  tokensConsumed: a.metrics.tokensConsumed + 2150,
                },
              }
            : a
        )
      );

      gameBridge.orderAgentInstruction(finalExecAgent.id, `Tarefa ${taskId} concluída com sucesso! 🚀`);
    }, 4500);
  }, 2000);
}
