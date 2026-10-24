# The Grid Metropolis: Isometric Multi-Agent Hub 🌆🤖

> **Command Center gamificado e simulador multi-agente em 2D isométrico com estética cyberpunk de alta fidelidade visual.**

A proposta substitui painéis convencionais de tabelas e logs monótonos por uma megacidade viva suspensa sobre um mar de dados (*Data Ocean*). Cada projeto ativo possui sua própria estação de trabalho (cúpula pressurizada, fábrica vertical ou laboratório tecnológico), e cada agente de inteligência artificial é representado por um personagem com comportamento autônomo, rotinas de trabalho, deslocamento físico por passarelas de néon e colaboração espacial em tempo real.

---

## 🏛️ Worldbuilding & Setores

* **The Nexus (Terminal Central de Despacho):** Centro de comando e despacho de tarefas com mesa holográfica translúcida.
* **Project Pods & Domes (Casas dos Projetos):** Estações modulares temáticas com chaminés de fumaça colorida e telas reativas em tempo real.
* **The Battery Lounge (Área de Ociosidade & Recarga):** Bar retrofuturista onde agentes em repouso (`IDLE`) descansam e interagem socialmente.
* **The Glitch Alleys (Área de Debugging & Exceções):** Setor de quarentena com monitores de estática e fios expostos para agentes em recuperação de erros ou alucinações.
* **Passarelas e Malha de Tráfego:** Linhas navegáveis com guias luminosas em ciano e magenta conectando todos os setores, patrulhadas por drones aéreos de dados.

---

## ⚙️ Arquitetura do Sistema

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
                    │ Tokio Broadcast Channel / Arc<RwLock<WorldState>>
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

---

## 🛠️ Stack Tecnológica

* **Backend:** Rust (`axum 0.8`, `tokio 1.54`, `serde`, `tower-http`, `tracing`, `thiserror`)
* **Frontend UI / HUD:** Svelte 5, TypeScript, Tailwind CSS v4
* **Motor Isométrico 2D:** Phaser 3 (WebGL)
* **Pathfinding:** EasyStar.js / A* Isométrico (projeção 2:1)
* **Gestão:** Task-Driven Development sincronizado com o Notion Workspace via API

---

## 🚀 Como Executar o Backend

```bash
# 1. Navegue até o diretório do backend
cd server

# 2. Execute os testes unitários da camada de domínio e estado
cargo test

# 3. Inicie o servidor
cargo run
```

O servidor iniciará por padrão em `http://0.0.0.0:3000` com endpoint de health check em `http://localhost:3000/health`.

---

## 📋 Documentação & Especificações

* [AGENTS.md](AGENTS.md) — Especificação completa do universo, dinâmicas de jogo, contratos de API e diretrizes técnicas (SOLID & Clean Code).
* [ROADMAP.md](ROADMAP.md) — Lista das 33 tarefas divididas por 7 fases e sincronizadas com o Notion.
