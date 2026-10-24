import { writable } from 'svelte/store';
import type { Agent, Pod, Task, WebSocketEvent } from '../../types';

export const agents = writable<Agent[]>([]);
export const pods = writable<Pod[]>([]);
export const tasks = writable<Task[]>([]);
export const selectedAgent = writable<Agent | null>(null);
export const eventLogs = writable<Array<{ id: string; time: string; text: string; topic: string }>>([]);

export function handleIncomingEvent(event: WebSocketEvent) {
  const timeStr = new Date(event.timestamp).toLocaleTimeString();

  switch (event.topic) {
    case 'WORLD_SNAPSHOT': {
      const { agents: initialAgents, pods: initialPods, tasks: initialTasks } = event.payload;
      if (initialAgents) agents.set(initialAgents);
      if (initialPods) pods.set(initialPods);
      if (initialTasks) tasks.set(initialTasks);
      addLog(timeStr, 'Sincronização inicial do mundo concluída', 'WORLD_SNAPSHOT');
      break;
    }

    case 'AGENT_SPAWN': {
      const { agentId, role, startPodId } = event.payload;
      addLog(timeStr, `Agente ${agentId} (${role}) surgiu na base ${startPodId}`, 'AGENT_SPAWN');
      break;
    }

    case 'TASK_ASSIGNED': {
      const { agentId, taskId, targetPod } = event.payload;
      agents.update((list) =>
        list.map((a) => (a.id === agentId ? { ...a, status: 'WALKING', activeTaskId: taskId } : a))
      );
      pods.update((list) =>
        list.map((p) =>
          p.id === targetPod
            ? {
                ...p,
                visualState: {
                  ...p.visualState,
                  powerLevel: 100,
                  screenStatus: 'CODE_STREAM',
                  chimneySmokeColor: '#00f0ff',
                },
              }
            : p
        )
      );
      addLog(timeStr, `Tarefa ${taskId} despachada para ${agentId} no Pod ${targetPod}`, 'TASK_ASSIGNED');
      break;
    }

    case 'TASK_PROGRESS': {
      const { agentId, step, tokensUsed } = event.payload;
      agents.update((list) =>
        list.map((a) =>
          a.id === agentId
            ? {
                ...a,
                status: 'WORKING',
                metrics: {
                  ...a.metrics,
                  tokensConsumed: a.metrics.tokensConsumed + tokensUsed,
                },
              }
            : a
        )
      );
      addLog(timeStr, `[${agentId}] ${step} (${tokensUsed} tokens)`, 'TASK_PROGRESS');
      break;
    }

    case 'TASK_COMPLETED': {
      const { agentId, taskId } = event.payload;
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
      addLog(timeStr, `Tarefa ${taskId} finalizada com sucesso por ${agentId}!`, 'TASK_COMPLETED');
      break;
    }

    case 'TASK_ERROR': {
      const { agentId, errorCode, details } = event.payload;
      agents.update((list) =>
        list.map((a) => (a.id === agentId ? { ...a, status: 'ERROR' } : a))
      );
      addLog(timeStr, `⚠️ Erro no agente ${agentId}: [${errorCode}] ${details}`, 'TASK_ERROR');
      break;
    }
  }
}

function addLog(time: string, text: string, topic: string) {
  eventLogs.update((logs) => [
    { id: Math.random().toString(36).substring(2, 9), time, text, topic },
    ...logs.slice(0, 49),
  ]);
}
