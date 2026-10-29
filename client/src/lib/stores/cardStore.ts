import { writable, get } from 'svelte/store';
import type { ActionCard, PlayedCardRecord } from '../../types/card';
import { agents, tasks } from './gridStore';

export const STARTER_DECK: ActionCard[] = [
  {
    id: 'card-sqlite-driver',
    title: 'Refatorar Driver SQLite',
    subtitle: 'Persistência OPFS & WASM',
    category: 'CODE',
    rarity: 'RARE',
    energyCost: 3,
    tokenEstimate: '3.2k tokens',
    targetModel: 'Codex / GPT-4.5',
    targetRole: 'DEVELOPER',
    description: 'Implementa camada de persistência local em alto desempenho usando SQLite WASM com locking atômico.',
    prompt: 'Implementar a camada de persistência local usando SQLite WASM e Origin Private File System (OPFS) com transações atômicas e migrações idempotentes.',
    contextFiles: ['src/db/schema.rs', 'src/storage/driver.rs'],
    flavorText: 'Quando o compilador chora, os bytes queimam com precisão cirúrgica.',
    tags: ['Rust', 'SQLite', 'WASM', 'OPFS'],
  },
  {
    id: 'card-extended-thinking',
    title: 'Extended Thinking Protocol',
    subtitle: 'Raciocínio Arquitetural Profundo',
    category: 'ARCH',
    rarity: 'LEGENDARY',
    energyCost: 4,
    tokenEstimate: '6.8k tokens',
    targetModel: 'Claude 3.7 Sonnet',
    targetRole: 'DEVELOPER',
    description: 'Ativa reflexão deliberada de múltiplos passos para resolver problemas de arquitetura complexos sem alucinações.',
    prompt: 'Analise detalhadamente a topologia de concorrência dos canais Tokio e projete um subsistema desacoplado de broadcast com backpressure e replay buffer.',
    contextFiles: ['src/hub.rs', 'src/dispatch.rs'],
    flavorText: 'Pensar antes de digitar: a virtude mais rara na era dos circuitos quânticos.',
    tags: ['Architecture', 'Claude 3.7', 'Deep Reasoning'],
  },
  {
    id: 'card-deep-research',
    title: 'Deep Web Exploration',
    subtitle: 'Varredura de Docs & Artigos',
    category: 'RESEARCH',
    rarity: 'UNCOMMON',
    energyCost: 2,
    tokenEstimate: '2.1k tokens',
    targetModel: 'Gemini 2.0 Flash',
    targetRole: 'RESEARCHER',
    description: 'Varre especificações técnicas, repositórios upstream e documentações para extrair as melhores práticas.',
    prompt: 'Faça uma pesquisa exaustiva sobre padrões de WebSocket streaming de baixa latência em WebGL 2D com Phaser 3 e z-sorting isométrico.',
    contextFiles: ['client/src/lib/game/Scene.ts'],
    flavorText: 'O oceano de dados nunca dorme; apenas aguarda quem saiba navegar suas correntes.',
    tags: ['Research', 'Phaser', 'Documentation'],
  },
  {
    id: 'card-security-audit',
    title: 'Auditoria de Vulnerabilidades',
    subtitle: 'Red-Team & Análise Estática',
    category: 'SECURITY',
    rarity: 'RARE',
    energyCost: 3,
    tokenEstimate: '4.0k tokens',
    targetModel: 'Claude 3.7 Sonnet',
    targetRole: 'SECURITY',
    description: 'Inspeciona handlers de rota, cabeçalhos de autenticação e inputs de usuário em busca de injeções e vazamentos.',
    prompt: 'Audite os endpoints HTTP e o upgrade WebSocket quanto a validações de Bearer Token, CSRF, buffer overflow e sanitização de dados.',
    contextFiles: ['server/src/dispatch.rs', 'server/src/auth.rs'],
    flavorText: 'Um único byte desprotegido pode abrir os portões da metrópole ao caos.',
    tags: ['Security', 'Zero-Day', 'Audit'],
  },
  {
    id: 'card-unit-tests-suite',
    title: 'Bateria de Testes Unitários',
    subtitle: 'Cobertura & Verificação Mock',
    category: 'TEST',
    rarity: 'COMMON',
    energyCost: 1,
    tokenEstimate: '1.5k tokens',
    targetModel: 'Codex / GPT-4.5',
    targetRole: 'DEVELOPER',
    description: 'Gera suíte completa de testes de regressão automatizados para cobrir casos extremos e edge cases.',
    prompt: 'Escreva testes unitários rigorosos cobrindo todas as variantes de status da máquina de estados dos agentes e erros de serialização Serde.',
    contextFiles: ['server/tests/unit_tests.rs'],
    flavorText: 'Se não foi testado, simplesmente não existe no reino da verdade binária.',
    tags: ['Testing', 'Quality', 'Mock'],
  },
];

// Stores reativas
export const architectEnergy = writable<number>(5);
export const maxArchitectEnergy = writable<number>(5);

export const hand = writable<ActionCard[]>([
  STARTER_DECK[0],
  STARTER_DECK[1],
  STARTER_DECK[2],
  STARTER_DECK[3],
]);

export const deck = writable<ActionCard[]>([
  STARTER_DECK[4],
  {
    id: 'card-ci-pipeline',
    title: 'Automação de CI/CD Cluster',
    subtitle: 'GitHub Actions & Containerização',
    category: 'CODE',
    rarity: 'COMMON',
    energyCost: 2,
    tokenEstimate: '1.8k tokens',
    targetModel: 'Codex / GPT-4.5',
    targetRole: 'DEVOPS',
    description: 'Configura pipelines de entrega contínua com compilação cruzada em Rust e bundle Vite.',
    prompt: 'Configure o pipeline de CI para rodar cargo clippy, cargo test e vite build em paralelo em menos de 45 segundos.',
    flavorText: 'O fluxo de entrega deve ser tão constante quanto a corrente que alimenta a cidade.',
    tags: ['DevOps', 'CI/CD', 'Rust'],
  },
]);

export const graveyard = writable<PlayedCardRecord[]>([]);

export const inspectingCard = writable<ActionCard | null>(null);
export const isGrimoireOpen = writable<boolean>(false);
export const isForgeModalOpen = writable<boolean>(false);

// Métodos de Ação do Card Game
export function drawCard(): boolean {
  const currentHand = get(hand);
  if (currentHand.length >= 6) {
    return false; // Mão cheia (máx 6 cartas)
  }

  const currentDeck = get(deck);
  if (currentDeck.length === 0) {
    // Reembaralha a partir dos starters
    deck.set([...STARTER_DECK].sort(() => Math.random() - 0.5));
  }

  const updatedDeck = get(deck);
  const nextCard = updatedDeck.shift();
  if (nextCard) {
    deck.set(updatedDeck);
    hand.update((h) => [...h, nextCard]);
    return true;
  }
  return false;
}

export function playCard(cardId: string, assignedAgentOverrideId?: string): boolean {
  const currentEnergy = get(architectEnergy);
  const currentHand = get(hand);
  const targetCard = currentHand.find((c) => c.id === cardId);

  if (!targetCard) return false;
  if (currentEnergy < targetCard.energyCost) {
    alert(`⚡ Energia insuficiente! Custo da carta: ${targetCard.energyCost} | Energia atual: ${currentEnergy}`);
    return false;
  }

  // Consome energia
  architectEnergy.update((e) => Math.max(0, e - targetCard.energyCost));

  // Remove da mão
  hand.update((h) => h.filter((c) => c.id !== cardId));

  // Localiza ou elege o melhor agente para o papel
  const currentAgents = get(agents);
  let eligibleAgent = currentAgents.find(
    (a) => a.role === targetCard.targetRole && a.status === 'IDLE'
  );
  if (!eligibleAgent) {
    eligibleAgent = currentAgents.find((a) => a.status === 'IDLE') || currentAgents[0];
  }

  const agentId = assignedAgentOverrideId || (eligibleAgent ? eligibleAgent.id : 'agent-codex-01');
  const agentName = eligibleAgent ? eligibleAgent.name : 'Rusty Hacker';

  const taskId = `task-${Math.floor(1000 + Math.random() * 9000)}`;

  // Registra no Grimório
  const record: PlayedCardRecord = {
    recordId: `rec-${Date.now()}`,
    card: targetCard,
    taskId,
    assignedAgentId: agentId,
    assignedAgentName: agentName,
    playedAt: new Date().toLocaleTimeString(),
    status: 'EXECUTING',
    reasoningSnippet: `[${targetCard.targetModel}] Iniciando interpretação da diretriz: "${targetCard.title}"...`,
    artifactsCreated: targetCard.contextFiles || [],
  };

  graveyard.update((g) => [record, ...g]);

  // Atualiza estado do agente na gridStore
  agents.update((list) =>
    list.map((a) =>
      a.id === agentId
        ? {
            ...a,
            status: 'WALKING',
            activeTaskId: taskId,
          }
        : a
    )
  );

  // Fecha modal de inspeção se estiver aberto com esta carta
  inspectingCard.set(null);

  // Simula progressão da IA após caminhada e resolução
  setTimeout(() => {
    agents.update((list) =>
      list.map((a) =>
        a.id === agentId
          ? {
              ...a,
              status: 'WORKING',
              metrics: {
                ...a.metrics,
                tokensConsumed: a.metrics.tokensConsumed + parseInt(targetCard.tokenEstimate) * 1000 || 2500,
              },
            }
          : a
      )
    );

    // Atualiza registro no Grimório para resolvido após alguns segundos
    setTimeout(() => {
      agents.update((list) =>
        list.map((a) =>
          a.id === agentId
            ? {
                ...a,
                status: 'IDLE',
                activeTaskId: undefined,
                metrics: { ...a.metrics, tasksCompleted: a.metrics.tasksCompleted + 1 },
              }
            : a
        )
      );

      graveyard.update((g) =>
        g.map((r) =>
          r.taskId === taskId
            ? {
                ...r,
                status: 'RESOLVED',
                completedAt: new Date().toLocaleTimeString(),
                tokensUsed: 2840,
                reasoningSnippet: `[${targetCard.targetModel}] Raciocínio finalizado com sucesso. Artefatos gerados e validados contra os requisitos do contrato.`,
              }
            : r
        )
      );

      // Regenera 1 ponto de energia do Arquiteto
      architectEnergy.update((e) => Math.min(get(maxArchitectEnergy), e + 1));
    }, 8000);
  }, 2500);

  return true;
}

export function forgeCustomCard(data: {
  title: string;
  subtitle: string;
  category: ActionCard['category'];
  rarity: ActionCard['rarity'];
  energyCost: number;
  targetModel: ActionCard['targetModel'];
  targetRole: ActionCard['targetRole'];
  prompt: string;
  description: string;
  flavorText?: string;
  tags?: string[];
}) {
  const newCard: ActionCard = {
    id: `custom-${Date.now()}`,
    title: data.title,
    subtitle: data.subtitle,
    category: data.category,
    rarity: data.rarity,
    energyCost: data.energyCost,
    tokenEstimate: `${data.energyCost * 1.5}k tokens`,
    targetModel: data.targetModel,
    targetRole: data.targetRole,
    description: data.description,
    prompt: data.prompt,
    flavorText: data.flavorText || 'Forjada pelo Arquiteto nos fornos de silício da metrópole.',
    tags: data.tags || ['Custom', 'User Prompt'],
  };

  hand.update((h) => [...h, newCard]);
  isForgeModalOpen.set(false);
}
