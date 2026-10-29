import { writable, derived, get } from 'svelte/store';

export interface WorkspaceFile {
  path: string;
  name: string;
  language: 'rust' | 'typescript' | 'json' | 'markdown' | 'sql';
  authorDroid: string;
  accentColor: string;
  content: string;
  linesCount: number;
  lastModified: string;
}

const DEFAULT_FILES: Record<string, WorkspaceFile[]> = {
  'proj-alfa': [
    {
      path: 'src/storage/engine.rs',
      name: 'engine.rs',
      language: 'rust',
      authorDroid: 'Rusty',
      accentColor: '#39ff14',
      linesCount: 38,
      lastModified: '18:15:20',
      content: `//! Persistence Engine - The Grid Metropolis
//! Driver assíncrono com suporte a WAL, Tokio MPSC e OPFS.

use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Debug, PartialEq)]
pub enum JournalMode {
    Wal,
    Memory,
}

pub trait StorageDriver: Send + Sync {
    async fn commit(&self, tx: Transaction) -> Result<(), String>;
    async fn rollback(&self, tx_id: u64) -> Result<(), String>;
}

pub struct PersistenceEngine {
    pub journal_mode: JournalMode,
    sync_channel: Sender<Transaction>,
}

impl PersistenceEngine {
    pub fn new(capacity: usize) -> (Self, Receiver<Transaction>) {
        let (tx, rx) = channel(capacity);
        (
            Self {
                journal_mode: JournalMode::Wal,
                sync_channel: tx,
            },
            rx,
        )
    }
}

impl StorageDriver for PersistenceEngine {
    async fn commit(&self, tx: Transaction) -> Result<(), String> {
        self.sync_channel
            .send(tx)
            .await
            .map_err(|e| format!("ChannelClosed: {}", e))
    }

    async fn rollback(&self, tx_id: u64) -> Result<(), String> {
        tracing::warn!("Executando rollback atômico para tx_id: {}", tx_id);
        Ok(())
    }
}`,
    },
    {
      path: 'src/architecture/contracts.md',
      name: 'contracts.md',
      language: 'markdown',
      authorDroid: 'SPARK-01',
      accentColor: '#00f0ff',
      linesCount: 22,
      lastModified: '18:14:10',
      content: `# Especificação de Arquitetura // Projeto Alfa

## 1. Contrato da Camada de Persistência
- Padrão: Trait-based storage driver
- Assincronismo: Tokio non-blocking channels
- Concorrência: MPSC com backpressure configurável

## 2. Garantias do Sistema
- Zero alocações redundantes no hot-path de transações.
- Journal mode configurado para WAL para tolerância a falhas.
- Isolamento de erros com tipos Result estritos.

## 3. Matriz de Responsabilidades
- Arquiteto (SPARK-01): Definição de traits e invariantes.
- Programador (Rusty): Implementação do driver e testes.
- Revisor (Vanguard): Auditoria de AST e verificador clippy.`,
    },
    {
      path: 'tests/storage_test.rs',
      name: 'storage_test.rs',
      language: 'rust',
      authorDroid: 'Rusty',
      accentColor: '#39ff14',
      linesCount: 29,
      lastModified: '18:15:35',
      content: `#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_commit_transaction() {
        let (engine, mut rx) = PersistenceEngine::new(10);
        let tx = Transaction {
            id: 101,
            payload: b"METROPOLIS_STATE_V1".to_vec(),
            timestamp: 1718000000,
        };

        let result = engine.commit(tx.clone()).await;
        assert!(result.is_ok(), "Commit deve ser bem sucedido");

        let received = rx.recv().await.expect("Mensagem deve chegar no canal");
        assert_eq!(received.id, 101);
    }

    #[tokio::test]
    async fn test_rollback_atomicity() {
        let (engine, _rx) = PersistenceEngine::new(5);
        let res = engine.rollback(999).await;
        assert!(res.is_ok());
    }
}`,
    },
    {
      path: 'reports/qa_audit.json',
      name: 'qa_audit.json',
      language: 'json',
      authorDroid: 'Vanguard',
      accentColor: '#ff007f',
      linesCount: 20,
      lastModified: '18:16:02',
      content: `{
  "auditor": "Vanguard (Claude 3.7 Sonnet)",
  "project": "proj-alfa",
  "review_status": "APPROVED",
  "metrics": {
    "unsafe_blocks": 0,
    "memory_leaks_detected": 0,
    "concurrency_deadlocks": 0,
    "unit_tests_passed": 4,
    "coverage_percent": 94.2
  },
  "clippy_warnings": [],
  "recommendations": [
    "Adicionar métricas OpenTelemetry para latência de commit",
    "Configurar bounded channel dinâmico para picos de I/O"
  ]
}`,
    },
  ],
  'proj-notex': [
    {
      path: 'src/crdt/vector_clock.rs',
      name: 'vector_clock.rs',
      language: 'rust',
      authorDroid: 'Dr. Cipher',
      accentColor: '#9d00ff',
      linesCount: 24,
      lastModified: '18:08:00',
      content: `use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    clocks: BTreeMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { clocks: BTreeMap::new() }
    }

    pub fn increment(&mut self, peer_id: &str) {
        let counter = self.clocks.entry(peer_id.to_string()).or_insert(0);
        *counter += 1;
    }
}`,
    },
  ],
};

export const workspaceFiles = writable<Record<string, WorkspaceFile[]>>(DEFAULT_FILES);
export const activeFilePath = writable<string | null>('src/storage/engine.rs');
export const terminalViewTab = writable<'CONSOLE' | 'VIM_INSPECTOR'>('CONSOLE');
export const droidViewFilter = writable<string | null>(null); // null = Todos, string = ID do Droid selecionado

export function openFileInVim(path: string) {
  activeFilePath.set(path);
  terminalViewTab.set('VIM_INSPECTOR');
}

export function closeVimInspector() {
  terminalViewTab.set('CONSOLE');
}

export function setDroidFilter(droidId: string | null) {
  droidViewFilter.set(droidId);
}
