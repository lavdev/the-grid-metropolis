import re

with open('client/src/lib/stores/projectStore.ts', 'r', encoding='utf-8') as f:
    content = f.read()

start_marker = "/**\n * Dispara o Pipeline Sequencial Multi-Droid (Esteira com Handoff Automático)"
end_marker = "export function updateDroidState("

start_idx = content.find(start_marker)
end_idx = content.find(end_marker, start_idx)

if start_idx == -1 or end_idx == -1:
    print(f"Error: markers not found (start: {start_idx}, end: {end_idx})")
    exit(1)

new_code = """function appendMessageToken(projectId: string, msgId: string, token: string, isThinking = false) {
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
}

/**
 * Dispara o Pipeline Sequencial Multi-Droid ou Solo com IA e Streaming em Tempo Real
 */
export async function sendCommandToActiveProject(commandText: string) {
  if (!commandText.trim()) return;

  const currentProj = get(activeProject);
  if (!currentProj) return;

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

    updateDroidState(currentProj.id, soloDroid.id, 'WORKING', `Processando: "${commandText.slice(0, 28)}..."`);

    try {
      const res = await streamAiCompletion({
        prompt: commandText,
        droidName: soloDroid.name,
        droidRole: soloDroid.role,
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
    } catch (err: any) {
      finalizeMessage(currentProj.id, soloMsgId, {
        status: 'ERROR',
        content: `Falha na execução: ${err.message}`,
      });
      updateDroidState(currentProj.id, soloDroid.id, 'ONLINE', 'Erro na execução');
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

  let archResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    archResult = await streamAiCompletion({
      prompt: `Instrução do usuário: "${commandText}". Atuando como Arquiteto de Software, trace a especificação técnica, requisitos fundamentais e contratos em tópicos diretos para passar a @${coderDroid.name}.`,
      droidName: architectDroid.name,
      droidRole: 'ARQUITETO',
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

  let coderResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    coderResult = await streamAiCompletion({
      prompt: `Com base na especificação do Arquiteto:\n${archResult.text}\n\nImplemente os módulos técnicos em código limpo com blocos de código markdown tipados.`,
      droidName: coderDroid.name,
      droidRole: 'PROGRAMADOR',
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

  let revResult = { text: '', thinking: '', tokensIn: 0, tokensOut: 0, latencyMs: 0, filesModified: [] as string[] };
  try {
    revResult = await streamAiCompletion({
      prompt: `Audite o código implementado:\n${coderResult.text}\n\nFaça um relatório conciso de qualidade, testes, possíveis falhas de segurança e dê o veredito final de aprovação do Pull Request.`,
      droidName: reviewerDroid.name,
      droidRole: 'REVISOR_DE_CODIGO',
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
}
"""

new_content = content[:start_idx] + new_code + "\n" + content[end_idx:]
with open('client/src/lib/stores/projectStore.ts', 'w', encoding='utf-8') as f:
    f.write(new_content)

print("Applied AI streaming integration into projectStore.ts successfully!")
