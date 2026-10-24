# AGENTS.md — The Grid Metropolis: Isometric Multi-Agent Hub

---

## 1. Visão Geral do Projeto

**The Grid Metropolis** (também referida como *The Core*) é um *Command Center* gamificado e simulador multi-agente em 2D isométrico com estética *cyberpunk* de alta fidelidade visual (estilo *pixel art* sofisticado, combinando elementos visuais inspirados em clássicos espaciais e de simulação como Habbo, SimCity e jogos táticos isométricos).

A proposta substitui painéis convencionais de tabelas e logs monótonos por uma megacidade viva suspensa sobre um mar de dados brutos (*Data Ocean*). Cada projeto ativo possui sua própria estação de trabalho (cúpula pressurizada, fábrica vertical ou laboratório tecnológico), e cada agente de inteligência artificial é representado por um personagem com comportamento autônomo, rotinas de trabalho, deslocamento físico por passarelas de néon e colaboração espacial em tempo real.

---

## 2. Worldbuilding & Geografia da Cidade

A metrópole é modular e dividida em distritos e setores conectados por cabos condutores grossos, tubulações luminosas com fluidos de dados bioluminescentes e passarelas suspensas de aço:

### 2.1 The Nexus (Terminal Central de Orquestração & Despacho)
* **Função:** Centro de comando e despacho de missões recebidas do usuário ("O Arquiteto") ou de sistemas externos (nuvem, IDEs, webhooks).
* **Visual:** Plataforma octogonal suspensa com uma antena parabólica massiva e uma mesa holográfica circular translúcida ao centro.
* **Dinâmica:** Quando uma nova tarefa chega da nuvem/IDE, o holograma central projeta o novo contrato e emite pulsos de luz ciano. Agentes se aproximam do terminal para recolher parâmetros de execução.

### 2.2 Project Pods & Domes (Casas dos Projetos)
* **Função:** Espaço de trabalho dedicado de cada agente ou projeto específico (ex: Notex, APIs, pipelines de dados, microsserviços).
* **Visual:** Estruturas modulares em estilo industrial futurista, cúpulas de observação em vidro reforçado, chaminés com vapor colorido e telas externas exibindo códigos e métricas em tempo real.
* **Estados Visuais Reativos:**
  * *Em execução pesada:* Néons pulsando em alta frequência, fumaça saindo das chaminés e monitores piscando dados ativamente.
  * *Ocioso / Concluído:* Iluminação suave em tons de azul ou âmbar, monitores em modo de descanso.
  * *Deploy / Sucesso:* Emissão de partículas e fumaça verde-néon fluorescente.
  * *Falha / Erro:* Iluminação piscando em vermelho-néon com alarme visual sobre a porta de acesso.

### 2.3 The Battery Lounge (Área de Ociosidade & Recarga)
* **Função:** Zona de descanso e socialização para agentes sem tarefas ativas (`IDLE`).
* **Visual:** Bar retrofuturista com balcão iluminado, letreiro luminoso vertical, banquetas e bebidas sintéticas fluorescentes.
* **Dinâmica:** Agentes ociosos sentam no balcão, descansam, recarregam baterias e trocam interações sociais com balões de conversa contendo piadas, rotinas ou trechos curtos de texto.

### 2.4 The Glitch Alleys (Área de Debugging & Exceções)
* **Função:** Setor de quarentena, análise de falhas, tratamento de exceções e recuperação de alucinações de LLMs.
* **Visual:** Beco lateral repleto de fios expostos, peças desconectadas, telas com efeito de estática/ruído e avisos luminosos de alerta (*warning*).
* **Dinâmica:** Se uma chamada de LLM estoura erro, gera código quebrado ou atinge timeout, o agente é deslocado para este beco até que a rotina de recuperação ou um agente de QA intervenha para reparar sua memória contextual.

### 2.5 Passarelas e Malha de Tráfego
* **Função:** Linhas de comunicação física navegáveis entre os distritos.
* **Visual:** Pisos de grade metálica com guias laterais iluminadas em magenta e ciano, conectando todos os pods ao Nexus e ao Lounge.
* **Tráfego Aéreo:** Mini-drones aéreos patrulhando o espaço e transportando pequenos pacotes de dados entre prédios distantes que não possuem passarela direta.

### 2.6 War Room / Sala de Reunião Central
* **Função:** Orquestração de tarefas que exigem deliberação conjunta entre múltiplos agentes.
* **Dinâmica:** Ao disparar demandas complexas que exigem alinhamento estratégico, os agentes envolvidos deixam suas estações e se reúnem ao redor da mesa holográfica central do Nexus para sincronizar o plano de execução.

### 2.7 Expansão Modular Contínua
* **Dinâmica:** Quando um novo projeto ou agente é registrado na rede, uma grua/plataforma de construção desce do topo da tela e constrói dinamicamente um novo Pod modular nas bordas navegáveis da cidade.

---

## 3. Atmosfera & Estética Visual

* **Paleta de Cores Cyberpunk:**
  * Base: Metal cinzento, ardósia industrial e escuridão do abismo (*Data Ocean*).
  * Acentos Néon: Roxo-elétrico (`#9d00ff`), Ciano/Turquesa (`#00f0ff`), Rosa-Magenta (`#ff007f`) e Verde-Lima fluorescente (`#39ff14`).
* **Tubulações Luminescentes:** Cabos e dutos transparentes pelo piso transportando fluidos bioluminescentes que pulsam quando há tráfego de dados ativo entre estações.
* **Perspectiva & Escala:** Vista isométrica clássica (projeção 2:1), com sensação de profundidade através de *z-sorting*, plataformas em múltiplos níveis e escadas metálicas.
* **Ambiente Reativo e Vivo:** Ecrãs exteriores exibindo linhas de código em fluxo, gráficos de consumo de tokens em tempo real e sensores atmosféricos.

---

## 4. Tipologia e Papéis Visuais dos Agentes

Cada agente possui sprite, vestimenta, animações e comportamentos condizentes com a sua especialidade:

1. **Hacker / Engenheiro de Software:**
   * *Aparência:* Roupas reforçadas, jaqueta com detalhes em néon, mochila técnica e headsets integrados.
   * *Ação:* Opera consoles com múltiplos monitores, digita freneticamente e manipula blocos de código flutuantes.
2. **Cientista / Analista de Dados:**
   * *Aparência:* Bata de laboratório branca sobre trajes cibernéticos, óculos HUD e luvas técnicas.
   * *Ação:* Opera interfaces circulares holográficas, avalia gráficos de dispersão e métricas de treinamento.
3. **Androide / Robô de Automação (DevOps / Bot Runner):**
   * *Aparência:* Chassis polido metálico com linhas luminosas coloridas.
   * *Ação:* Executa rotinas repetitivas, rotas de entrega de pacotes digitais e transporta artefatos entre estações.
4. **Drones de Mensageria:**
   * *Aparência:* Quadricópteros compactos com iluminação de sinalização.
   * *Ação:* Sobrevoam as fendas da cidade levando respostas rápidas de volta ao Nexus ou disparando notificações.

---

## 5. Metáforas Espaciais e Mecânicas de Jogo

* **Pipeline Handover (Delegação de Tarefas):** Quando o Agente Pesquisador conclui um resumo e o envia para o Agente Desenvolvedor, o sprite do pesquisador sai de sua cúpula segurando uma maleta de dados translúcida (ou holograma flutuante), caminha pelas passarelas e entrega o artefato na bancada do colega.
* **Colaboração Inter-Casas (Pair Programming / QA / Reuniões):** Dois agentes podem se encontrar no mesmo Pod ou no meio de uma passarela. Balões de diálogo sincronizados surgem com ícones de engrenagens, lâmpadas, códigos ou snippets para ilustrar a troca de contexto via API.
* **Modo Diretor (Interatividade do Usuário):**
  * *Clique e Inspeção:* Clicar em qualquer personagem em trânsito abre uma gaveta/modal lateral com:
    * Status atual (ex: *"Caminhando para o Lab 02"*).
    * Scratchpad / Chain-of-Thought (último log de raciocínio da LLM).
    * Tokens consumidos e tempo de atividade (*uptime*).
    * Arquivos e artefatos sob sua custódia no momento.
  * *Drag & Drop (Arremesso de Tarefas):* O usuário pode arrastar um agente ocioso do Battery Lounge e soltá-lo diretamente dentro do laboratório de um projeto para forçar assistência imediata.
  * *Visão em Corte (X-Ray Cutaway):* Clicar no domo ou telhado de um Pod oculta a cobertura externa e exibe os monitores internos, servidores em rack e consoles daquele projeto.

---

## 6. Arquitetura da Ponte de Integração (Bridge Architecture)

A ponte permite conectar o mundo isométrico com serviços externos, tais como instâncias na nuvem, Codex/Claude, webhooks do GitHub, pipelines de CI/CD ou CLIs locais.

### 6.1 Diagrama de Fluxo de Dados

```text
┌────────────────────────────────────────┐
│     Fontes Externas (Cloud / IDE)      │
│ (Codex, Claude, GitHub Webhooks, CLI) │
└───────────────────┬────────────────────┘
                    │ HTTP POST /api/v1/bridge/dispatch
                    ▼
┌────────────────────────────────────────┐
│         Bridge Ingestion API           │
│         (Rust / Axum + Tokio)          │
└───────────────────┬────────────────────┘
                    │ Tokio Broadcast Channel / Arc<RwLock<WorldState>> (ou Redis)
                    ▼
┌────────────────────────────────────────┐
│   WebSocket Hub (Axum WebSockets)      │
└───────────────────┬────────────────────┘
                    │ Mensagens WS em tempo real (JSON)
                    ▼
┌────────────────────────────────────────┐
│      Client 2D Isométrico (Browser)    │
│  - Renderização (Phaser / PixiJS)      │
│  - Pathfinding das Passarelas (A*)     │
│  - Animação, Z-Sorting e Balões        │
└────────────────────────────────────────┘
```

### 6.2 Especificação do Endpoint de Ingestão HTTP

* **Rota:** `POST /api/v1/bridge/dispatch`
* **Headers:** 
  * `Content-Type: application/json`
  * `Authorization: Bearer <TOKEN>`
* **Payload de Exemplo:**

```json
{
  "source": "codex-cloud-runner",
  "project_id": "proj-notex-backend",
  "target_agent_id": "agent-rust-dev",
  "task": {
    "task_id": "task-8841",
    "title": "Implementar driver SQLite OPFS",
    "priority": "HIGH",
    "payload": {
      "prompt": "Escreva a camada de persistência local usando SQLite WASM...",
      "context_files": ["src/db/schema.rs"]
    }
  },
  "callback_webhook": "https://cloud.engine.io/tasks/task-8841/callback"
}
```

* **Respostas:**
  * `202 Accepted`: Evento enfileirado e despachado para a simulação com sucesso.
  * `400 Bad Request`: Payload malformado ou campos obrigatórios ausentes.
  * `401 Unauthorized`: Token de autenticação inválido ou expirado.
  * `404 Not Found`: Agente ou Pod de destino não cadastrado no mapa.

### 6.3 Protocolo de Eventos WebSocket (Frontend)

O frontend assina o canal WebSocket e renderiza os comportamentos espaciais com base nos seguintes tópicos:

| Tópico do Evento | Parâmetros Principais | Reação Visual no Canvas |
| :--- | :--- | :--- |
| `AGENT_SPAWN` | `agent_id, role, start_pod_id` | Agente surge por teletransporte ou elevador na base indicada. |
| `TASK_ASSIGNED` | `agent_id, source_pod, target_pod` | Agente se levanta, traça a rota via pathfinding e caminha pela passarela. |
| `TASK_PROGRESS` | `agent_id, step, tokens_used` | Agente digita freneticamente; telas do Pod piscam; fumaça sobe da chaminé. |
| `AGENT_COLLABORATE` | `source_agent, target_agent, pod_id` | Dois agentes se aproximam e exibem balões de diálogo sincronizados com ícones. |
| `COLLABORATION_REQUEST`| `source_agent, target_agent` | O agente de origem caminha até a casa do colega para pedir revisão. |
| `TASK_COMPLETED` | `agent_id, task_id, artifacts[]` | Duto emite fumaça verde fluorescente; drone decola para entregar dados ao Nexus; agente comemora. |
| `TASK_ERROR` | `agent_id, error_code, details` | Iluminação pisca em vermelho-néon; agente é escoltado para o Glitch Alley. |
| `AGENT_REST` | `agent_id` | Agente encerra a jornada e caminha de volta ao The Battery Lounge. |

---

## 7. Modelagem de Dados & Schemas (TypeScript)

### 7.1 Schema do Agente (`Agent`)

```typescript
export interface Agent {
  id: string;                    // Identificador único (ex: 'agent-codex-01')
  name: string;                  // Nome exibido (ex: 'Rusty', 'Dr. Cipher')
  role: 'DEVELOPER' | 'RESEARCHER' | 'DEVOPS' | 'SECURITY' | 'BOT_RUNNER';
  spriteConfig: {
    textureKey: string;          // Referência da spritesheet de animação
    skinTheme: 'cyber_punk_lab' | 'street_hacker' | 'android_unit';
    colorAccent: string;         // Cor hexadecimal do néon do agente
  };
  homePodId: string;             // Base de origem do agente
  currentLocation: {
    x: number;
    y: number;
    currentZone: 'NEXUS' | 'HOME' | 'LOUNGE' | 'GLITCH_ALLEY' | 'TRANSIT';
  };
  status: 'IDLE' | 'WALKING' | 'WORKING' | 'MEETING' | 'ERROR';
  activeTaskId?: string;
  metrics: {
    tasksCompleted: number;
    tokensConsumed: number;
    uptimeSeconds: number;
  };
}
```

### 7.2 Schema do Pod / Estação de Projeto (`Pod`)

```typescript
export interface Pod {
  id: string;                    // ex: 'pod-notex'
  title: string;                 // Nome do projeto
  repositoryUrl?: string;        // Link do repositório Git
  maxCapacity: number;           // Quantidade máxima de estações simultâneas
  assignedAgents: string[];      // Lista de agentes vinculados
  visualState: {
    powerLevel: number;          // 0 a 100 (intensidade das luzes)
    chimneySmokeColor: string;   // Ex: '#39ff14' (verde), '#ff007f' (magenta)
    screenStatus: 'OFF' | 'CODE_STREAM' | 'DASHBOARD' | 'ALERT_RED';
  };
  interiorStations: Array<{
    stationId: string;
    isOccupied: boolean;
    coordX: number;
    coordY: number;
  }>;
}
```

### 7.3 Schema de Tarefa (`Task`) & Evento WebSocket (`WebSocketEvent`)

```typescript
export interface Task {
  taskId: string;
  projectId: string;
  targetAgentId: string;
  title: string;
  priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  status: 'PENDING' | 'IN_PROGRESS' | 'REVIEW' | 'COMPLETED' | 'FAILED';
  payload: {
    prompt: string;
    contextFiles?: string[];
    scratchpad?: string;
  };
  createdAt: string;
  completedAt?: string;
}

export interface WebSocketEvent {
  topic: 
    | 'AGENT_SPAWN'
    | 'TASK_ASSIGNED'
    | 'TASK_PROGRESS'
    | 'AGENT_COLLABORATE'
    | 'COLLABORATION_REQUEST'
    | 'TASK_COMPLETED'
    | 'TASK_ERROR'
    | 'AGENT_REST';
  timestamp: number;
  payload: Record<string, unknown>;
}
```

### 7.4 Modelagem de Dados no Backend (Rust + Serde)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentRole {
    Developer,
    Researcher,
    Devops,
    Security,
    BotRunner,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentStatus {
    Idle,
    Walking,
    Working,
    Meeting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteConfig {
    pub texture_key: String,
    pub skin_theme: String,
    pub color_accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLocation {
    pub x: f64,
    pub y: f64,
    pub current_zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tokens_consumed: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub sprite_config: SpriteConfig,
    pub home_pod_id: String,
    pub current_location: AgentLocation,
    pub status: AgentStatus,
    pub active_task_id: Option<String>,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Review,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPayload {
    pub prompt: String,
    pub context_files: Option<Vec<String>>,
    pub scratchpad: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub task_id: String,
    pub project_id: String,
    pub target_agent_id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub payload: TaskPayload,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventTopic {
    AgentSpawn,
    TaskAssigned,
    TaskProgress,
    AgentCollaborate,
    CollaborationRequest,
    TaskCompleted,
    TaskError,
    AgentRest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketEvent {
    pub topic: EventTopic,
    pub timestamp: i64,
    pub payload: serde_json::Value,
}
```

---

## 8. Ciclo de Vida Visual de uma Tarefa (Workflow Completo)

1. **Recepção:**
   * A ponte recebe a requisição via webhook de uma fonte externa (Codex, Cloud, GitHub).
   * O farol holográfico central do Nexus pisca em cor ciano e emite ondas concêntricas de luz.
2. **Despacho:**
   * O agente designado levanta do banco do Battery Lounge (onde estava descansando) ou do Nexus.
   * O motor de pathfinding calcula a rota pelas passarelas metálicas.
   * O personagem caminha com animação direcional até o seu Project Pod.
3. **Execução:**
   * Ao entrar no laboratório, as portas se abrem e o personagem senta na estação de trabalho.
   * Os servidores do rack ligam, as chaminés emitem vapor e balões de código/engrenagens flutuam sobre ele.
4. **Verificação / Colaboração (Opcional):**
   * Em caso de necessidade de QA ou pair programming, o agente se levanta, visita o Pod vizinho carregando a maleta de dados e inicia uma rodada de discussão com balões sincronizados.
5. **Finalização:**
   * Concluída a computação, a fumaça de exaustão do Pod assume tons verdes fluorescentes.
   * Um mini-drone transporta o pacote de dados de volta ao Nexus para entrega à nuvem via callback webhook.
   * O agente retorna para descansar no Battery Lounge ou aguarda a próxima chamada.

---

## 9. Stack Técnica Completa & Justificativas

| Camada | Tecnologia Sugerida | Justificativa |
| :--- | :--- | :--- |
| **Renderizador 2D / Canvas** | **Phaser.js** ou **Pixi.js** | Motores com suporte maduro a WebGL, camadas de profundidade (*z-sorting* isométrico), sistemas de partículas para fumaça de néon e troca dinâmica de spritesheets. |
| **Pathfinding Isométrico** | **EasyStar.js** ou **A\* Customizado** | Cálculo de rotas sobre a matriz de grid 2D sem travamentos, respeitando largura de passarelas, paredes e colisões. |
| **Camada de Interface UI / HUD** | **Svelte (Vite + TypeScript)** | Reatividade compilada sem Virtual DOM (zero overhead sobre o WebGL), ciclo de vida limpo para instanciar o Canvas, animações nativas fluidas para HUD cyberpunk e bundle ultraleve. |
| **Backend de Integração (Bridge)** | **Rust (Axum + Tokio)** | Zero pausas de GC, latência de microssegundos para WebSockets, concorrência assíncrona robusta (`tokio::sync::broadcast`), validação de contratos em tempo de compilação com `serde` e binário único ultraleve. |
| **Comunicação em Tempo Real** | **WebSockets nativos (`axum::extract::ws`)** | Transmissão bidirecional de eventos de baixa latência da nuvem para o navegador do usuário. |
| **Fila / Gerenciamento de Estado** | **In-Memory (`Arc<RwLock<WorldState>>` + `tokio::sync::broadcast`)** (Redis opcional para clusters multi-nó) | Desacoplamento entre a ingestão das APIs externas e a distribuição de estados visuais na cidade com latência quase nula. |
| **Assets Gráficos** | **Pixel Art Isométrica (32x32 / 64x64)** | Blocos modulares reutilizáveis de piso metálico, barreiras de neon, maquinários e folhas de sprites com 4/8 direções de movimento. |

---

## 10. Protocolo Operacional: Task-Driven Development (Notion First)

> [!IMPORTANT]
> **Regra Fundamental de Gestão: "Nada é feito sem tarefa."**
> Toda e qualquer implementação, refatoração ou funcionalidade solicitada pelo Owner deve obrigatoriamente estar vinculada a uma tarefa registrada no Notion.

### Fluxo Obrigatório Pré-Execução:
1. **Verificação no Notion:**
   - Antes de iniciar qualquer trabalho de código, o agente/desenvolvedor deve consultar o banco de dados do **ROADMAP no Notion** (`ROADMAP — The Grid Metropolis`).
2. **Cenário A — Tarefa Existente:**
   - Se a demanda já estiver mapeada (ex: `T-1.1`), ela é referenciada, e seu status é atualizado para `In Progress` no início e para `Done` ao concluir.
3. **Cenário B — Demanda Nova (Ausente no Notion):**
   - Se a solicitação não constar no Notion, ela **deve ser criada imediatamente via API do Notion** antes de qualquer código ser escrito, com:
     - Identificador único sequencial.
     - Título claro e objetivo.
     - Fase e Prioridade.
     - Dependências explícitas.
     - Descrição detalhada dos critérios de aceite.
4. **Sincronização:**
   - O arquivo local [ROADMAP.md](file:///c:/Workspace/the-grid-metropolis/ROADMAP.md) deve ser mantido em sincronia com o Notion.

---

## 11. Diretrizes Técnicas de Engenharia: Clean Code & SOLID

O projeto preza por alta manutenibilidade, separação estrita de responsabilidades e clareza arquitetural, tanto no backend em **Rust** quanto no frontend em **TypeScript**:

### 11.1 Princípios SOLID na Prática
* **Single Responsibility Principle (SRP):**
  - Módulos, structs, traits e componentes possuem uma única razão para mudar.
  - *Exemplo:* O roteador HTTP (`dispatch.rs`) apenas valida e despacha; ele não gerencia a máquina de estados nem o broadcast WebSocket. O hub WebSocket (`hub.rs`) cuida de conexões e sockets, sem acoplamento a detalhes de persistência.
* **Open/Closed Principle (OCP):**
  - A arquitetura é aberta para extensão (novos tipos de agentes, novas reações de Pods, novos tópicos de eventos), mas fechada para modificação de código crítico existente. Uso de enums extensíveis e traits desacopladas.
* **Liskov Substitution Principle (LSP):**
  - Contratos e traits (como provedores de mensageria ou motores de pathfinding) podem ter suas implementações trocadas (ex: In-Memory Broadcast para Redis Pub/Sub) sem quebrar o comportamento do consumidor.
* **Interface Segregation Principle (ISP):**
  - Interfaces TypeScript e traits Rust granulares e coesas. Nenhum cliente ou componente deve ser forçado a depender de métodos que não utiliza.
* **Dependency Inversion Principle (DIP):**
  - Módulos de alto nível de lógica de simulação dependem de abstrações (traits / interfaces), e não de detalhes de transporte de rede ou I/O concreto.

### 11.2 Padrões de Clean Code
* **Nomenclatura Semântica:** Identificadores, funções e tipos autoexplicativos que revelam intenção sem abreviações ambíguas.
* **Funções Pequenas e Focadas:** Funções com propósito único, baixo aninhamento e sem efeitos colaterais ocultos.
* **Tratamento de Erros Seguro & Explícito em Rust:**
  - Proibido o uso de `.unwrap()` e `.expect()` em fluxos de execução de produção.
  - Utilização estrita de `Result<T, AppError>`, `Option<T>` e enums de erro tipados com `thiserror`.
* **Separação em Camadas Arquiteturais:**
  1. **Domain / State:** Entidades puras, invariantes de regras e modelos (`Agent`, `Pod`, `Task`).
  2. **Service / Engine:** Lógica de negócio, pathfinding e orquestração do ciclo de vida.
  3. **Transport / Infrastructure:** Endpoints HTTP Axum, handlers de WebSockets e adaptadores de webhooks externos.
  4. **Presentation (Frontend):** Renderizador WebGL isolado das camadas de UI/HUD e controle de sessão.
