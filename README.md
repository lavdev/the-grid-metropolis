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
* **Motor Isométrico 2D:** Phaser 4 (WebGL)
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

O servidor local inicia em `http://127.0.0.1:3000`, com health check em `http://127.0.0.1:3000/health`. Ele aceita conexões apenas da própria máquina.

## 🖥️ Aplicativo Desktop (Tauri)

O cliente desktop usa Tauri 2, abre maximizado e inicia o backend local incluído. Em desenvolvimento, `tauri:dev` compila o sidecar Rust e inicia Vite, Tauri e a Bridge; não é necessário abrir um terminal separado para o servidor:

```bash
cd client
npm install
npm run tauri:dev
```

Para gerar o instalador desktop, execute `npm run tauri:build` em `client`. O comando compila o backend como sidecar e o inclui no pacote. Os dados da versão instalada ficam no diretório de dados do aplicativo do sistema; ao primeiro início, um banco legado encontrado no workspace é migrado por backup SQLite. O comando `npm run dev` continua disponível para usar apenas o frontend; nesse caso, inicie o backend manualmente com `cd server && cargo run`.

---

## 📋 Documentação & Especificações

* [AGENTS.md](AGENTS.md) — Especificação completa do universo, dinâmicas de jogo, contratos de API e diretrizes técnicas (SOLID & Clean Code).
* [ROADMAP.md](ROADMAP.md) — Lista das 33 tarefas divididas por 7 fases e sincronizadas com o Notion.

---

## System Overview

The repository contains a Svelte/Vite client with a Tauri desktop package, a Rust server, and operational scripts. The client owns the interactive UI and simulation code; the server owns HTTP/WebSocket transport, SQLite persistence, domain models, and in-memory world state. The scripts directory contains Notion, bridge, browser capture, and source-maintenance utilities. See the package contracts below for verified interfaces and side effects.

## Subdirectories & System Boundaries

- [client](client/README.md) — Svelte/TypeScript application, Phaser simulation, and Tauri desktop integration.
- [scripts](scripts/README.md) — Notion and bridge utilities, browser captures, and one-off source editing tools.
- [server](server/README.md) — Rust API, SQLite persistence, domain types, and simulation state.

## Global Invariants

- The bridge dispatch endpoint checks Bearer authentication before dispatching a task (`server/src/api/auth.rs::RequireAuth::from_request_parts`, `server/src/api/dispatch.rs::dispatch_handler`).
- The server binds to the loopback interface and defaults to port 3000 (`server/src/main.rs::main`).

## Global Side Effects

- Server startup initializes local SQLite storage and serves HTTP; API and task operations can persist records or mutate and broadcast in-memory state (`server/README.md`).
- Operational scripts can make authenticated Notion and HTTP/WebSocket requests, write screenshots, and modify frontend source files (`scripts/README.md`).

## System Verification

- Client: run `npm run check` from `client/`; the package manifest also provides `npm run build`.
- Server: run `cargo test` from `server/`.
- The root `package.json` defines no scripts, so there is no root-level verification command.
