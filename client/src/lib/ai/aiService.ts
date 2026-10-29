import { GoogleGenAI } from '@google/genai';
import { get } from 'svelte/store';
import { aiConfig, hasConfiguredApiKey, type AiConfig } from '../stores/aiConfigStore';

export interface StreamOptions {
  prompt: string;
  systemInstruction?: string;
  droidName?: string;
  droidRole?: string;
  workingDir?: string;
  model?: string;
  codexModel?: string;
  reasoningEffort?: string;
  threadId?: string;
  onToken: (chunk: string) => void;
  onThinking?: (thoughtChunk: string) => void;
  onThreadStarted?: (threadId: string) => void;
  signal?: AbortSignal;
}

export interface StreamResult {
  text: string;
  thinking: string;
  tokensIn: number;
  tokensOut: number;
  latencyMs: number;
  filesModified: string[];
}

/**
 * Motor central de streaming e integracao com IA de The Grid Metropolis
 */
export async function streamAiCompletion(options: StreamOptions): Promise<StreamResult> {
  const config = get(aiConfig);
  const startTime = performance.now();

  // Verifica se ha API key configurada; se nao, utiliza o simulador de alta fidelidade
  if (!hasConfiguredApiKey()) {
    return simulateStreamingResponse(options, startTime);
  }

  try {
    switch (config.provider) {
      case 'codex':
        return await streamCodexCli(options, config, startTime);
      case 'gemini':
        return await streamGemini(options, config, startTime);
      case 'openai':
      case 'ollama':
        return await streamOpenAiCompatible(options, config, startTime);
      case 'anthropic':
        return await streamAnthropic(options, config, startTime);
      default:
        return await streamCodexCli(options, config, startTime);
    }
  } catch (error) {
    console.error(`[AI Service] Falha na chamada da API (${config.provider}):`, error);
    // Em caso de erro de rede ou chave invalida, emite aviso amigavel e fallback
    const errMsg = `\n\n[ALERTA DE SISTEMA: Erro na conexao com a API ${config.provider.toUpperCase()} (${(error as Error).message}). Verifique sua chave nas configuracoes de IA.]\n`;
    options.onToken(errMsg);
    
    return {
      text: errMsg,
      thinking: `Falha no handshake com a API externa: ${(error as Error).message}`,
      tokensIn: 50,
      tokensOut: 20,
      latencyMs: Math.round(performance.now() - startTime),
      filesModified: [],
    };
  }
}

/**
 * 0. Provedor Local Codex CLI (Uso com Assinatura ChatGPT via Sessão Local)
 */
async function streamCodexCli(
  options: StreamOptions,
  config: AiConfig,
  startTime: number
): Promise<StreamResult> {
  const prompt = options.systemInstruction 
    ? `[INSTRUÇÃO DO PAPEL / SISTEMA]\n${options.systemInstruction}\n\n[TAREFA DO OPERADOR]\n${options.prompt}`
    : options.prompt;

  const reqModel =
    options.codexModel ||
    options.model ||
    (config.codexModel && config.codexModel !== 'default' ? config.codexModel : (config.model && config.model !== 'default' ? config.model : undefined));
  const reqReasoning = options.reasoningEffort || config.reasoningEffort || undefined;

  const response = await fetch('/api/v1/codex/stream', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    signal: options.signal,
    body: JSON.stringify({
      prompt,
      working_dir: options.workingDir,
      model: reqModel && reqModel !== 'default' ? reqModel : undefined,
      reasoning_effort: reqReasoning,
      thread_id: options.threadId,
    }),
  });

  if (!response.ok) {
    const errText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errText}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('Não foi possível inicializar o leitor de stream SSE do Codex.');

  const decoder = new TextDecoder();
  let fullText = '';
  let fullThinking = '';
  let tokensIn = 0;
  let tokensOut = 0;
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || !trimmed.startsWith('data: ')) continue;
      const dataStr = trimmed.replace(/^data:\s*/, '');
      if (dataStr === '[DONE]') break;

      try {
        const event = JSON.parse(dataStr);
        if (event.type === 'thread.started') {
          const tid = event.thread_id;
          if (tid && options.onThreadStarted) {
            options.onThreadStarted(tid);
          }
          const shortTid = tid ? tid.slice(0, 8) : 'local';
          if (options.onThinking) {
            const isResume = options.threadId ? 'Memória Restaurada' : 'Nova Sessão';
            options.onThinking(`⚡ [CODEX CLI] ${isResume} (Thread: ${shortTid}...)\n`);
          }
        } else if (event.type === 'item.started') {
          const item = event.item;
          if (item?.type === 'command_execution') {
            const notice = '\n🔍 [CODEX // SHELL] Inspecionando estrutura e arquivos do workspace...\n';
            fullThinking += notice;
            if (options.onThinking) options.onThinking(notice);
          } else if (item?.type === 'file_change') {
            const notice = '\n📝 [CODEX // DISCO] Gravando arquivo de resultado no projeto...\n';
            fullThinking += notice;
            if (options.onThinking) options.onThinking(notice);
          }
        } else if (event.type === 'item.completed') {
          const item = event.item;
          if (item?.type === 'agent_message' && item.text) {
            fullText += item.text;
            options.onToken(item.text);
          } else if (item?.type === 'reasoning' || item?.type === 'thought') {
            const thought = item.text || item.thought || '';
            fullThinking += thought;
            if (options.onThinking) options.onThinking(thought);
          } else if (item?.type === 'command_execution') {
            const notice = '✓ Leitura/inspeção de repositório concluída.\n';
            fullThinking += notice;
            if (options.onThinking) options.onThinking(notice);
          } else if (item?.type === 'file_change') {
            const fpath = item.path || item.file || '';
            const notice = `✓ Arquivo gravado no workspace: ${fpath}\n`;
            fullThinking += notice;
            if (options.onThinking) options.onThinking(notice);
          }
        } else if (event.type === 'turn.completed') {
          if (event.usage) {
            tokensIn = event.usage.input_tokens || 0;
            tokensOut = event.usage.output_tokens || 0;
          }
        }
      } catch {
        // Ignora chunks incompletos
      }
    }
  }

  if (!fullText) {
    fullText = fullThinking 
      ? 'Operação de ferramentas concluída pelo Codex no workspace (consulte os detalhes de execução no Chain-of-Thought).'
      : 'Codex finalizou o ciclo com sucesso.';
    options.onToken(fullText);
  }

  const latencyMs = Math.round(performance.now() - startTime);
  if (!tokensIn) tokensIn = Math.round(prompt.length / 4);
  if (!tokensOut) tokensOut = Math.round(fullText.length / 4);

  return {
    text: fullText,
    thinking: fullThinking || 'Executado localmente via OpenAI Codex CLI (Assinatura ChatGPT). Custo: 0 tokens faturados.',
    tokensIn,
    tokensOut,
    latencyMs,
    filesModified: extractArtifactsFromText(fullText),
  };
}

/**
 * 1. Provedor Google Gemini (com @google/genai v2.31+)
 */
async function streamGemini(
  options: StreamOptions,
  config: AiConfig,
  startTime: number
): Promise<StreamResult> {
  const ai = new GoogleGenAI({ apiKey: config.apiKey });
  const modelName = config.model || 'gemini-3.8-flash';

  const systemPrompt = buildSystemPrompt(options.droidName, options.droidRole, options.systemInstruction);

  const requestConfig: any = {
    systemInstruction: systemPrompt,
    temperature: config.temperature ?? 0.7,
  };

  const responseStream = await ai.models.generateContentStream({
    model: modelName,
    contents: options.prompt,
    config: requestConfig,
  });

  let fullText = '';
  let fullThinking = '';

  for await (const chunk of responseStream) {
    if (options.signal?.aborted) break;

    // Captura tokens de pensamento (thinking) se disponivel
    const candidates = chunk.candidates || [];
    for (const cand of candidates) {
      const parts = cand.content?.parts || [];
      for (const part of parts) {
        if ((part as any).thought) {
          const thoughtText = (part as any).thought;
          fullThinking += thoughtText;
          if (options.onThinking) options.onThinking(thoughtText);
        }
      }
    }

    // Captura tokens de texto convencional
    if (chunk.text) {
      fullText += chunk.text;
      options.onToken(chunk.text);
    }
  }

  const latencyMs = Math.round(performance.now() - startTime);
  const tokensIn = Math.round((options.prompt.length + systemPrompt.length) / 4);
  const tokensOut = Math.round(fullText.length / 4);
  const filesModified = extractArtifactsFromText(fullText);

  return {
    text: fullText,
    thinking: fullThinking || 'Raciocínio processado via Gemini 3.8 Flash.',
    tokensIn,
    tokensOut,
    latencyMs,
    filesModified,
  };
}

/**
 * 2. Provedor OpenAI / Ollama Local (SSE Stream)
 */
async function streamOpenAiCompatible(
  options: StreamOptions,
  config: AiConfig,
  startTime: number
): Promise<StreamResult> {
  const isOllama = config.provider === 'ollama';
  const baseUrl = isOllama
    ? `${config.ollamaBaseUrl.replace(/\/+$/, '')}/v1`
    : 'https://api.openai.com/v1';

  const model = isOllama ? (config.model || 'llama3.2') : (config.model || 'gpt-4o-mini');
  const systemPrompt = buildSystemPrompt(options.droidName, options.droidRole, options.systemInstruction);

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (!isOllama && config.apiKey) {
    headers['Authorization'] = `Bearer ${config.apiKey}`;
  }

  const response = await fetch(`${baseUrl}/chat/completions`, {
    method: 'POST',
    headers,
    signal: options.signal,
    body: JSON.stringify({
      model,
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: options.prompt },
      ],
      stream: true,
      temperature: config.temperature ?? 0.7,
    }),
  });

  if (!response.ok) {
    const errText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errText}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('Não foi possível inicializar o leitor de stream.');

  const decoder = new TextDecoder();
  let fullText = '';
  let fullThinking = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value, { stream: true });
    const lines = chunk.split('\n');

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || !trimmed.startsWith('data: ')) continue;
      const dataStr = trimmed.replace(/^data:\s*/, '');
      if (dataStr === '[DONE]') break;

      try {
        const json = JSON.parse(dataStr);
        const delta = json.choices?.[0]?.delta;
        if (delta?.reasoning_content && options.onThinking) {
          fullThinking += delta.reasoning_content;
          options.onThinking(delta.reasoning_content);
        }
        if (delta?.content) {
          fullText += delta.content;
          options.onToken(delta.content);
        }
      } catch {
        // Ignora chunks incompletos
      }
    }
  }

  const latencyMs = Math.round(performance.now() - startTime);
  const tokensIn = Math.round((options.prompt.length + systemPrompt.length) / 4);
  const tokensOut = Math.round(fullText.length / 4);

  return {
    text: fullText,
    thinking: fullThinking || `Executado via ${config.provider.toUpperCase()} (${model}).`,
    tokensIn,
    tokensOut,
    latencyMs,
    filesModified: extractArtifactsFromText(fullText),
  };
}

/**
 * 3. Provedor Anthropic Claude (SSE Stream)
 */
async function streamAnthropic(
  options: StreamOptions,
  config: AiConfig,
  startTime: number
): Promise<StreamResult> {
  const model = config.model || 'claude-3-7-sonnet-20250219';
  const systemPrompt = buildSystemPrompt(options.droidName, options.droidRole, options.systemInstruction);

  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': config.apiKey,
      'anthropic-version': '2023-06-01',
      'anthropic-dangerous-direct-browser-access': 'true',
    },
    signal: options.signal,
    body: JSON.stringify({
      model,
      system: systemPrompt,
      messages: [{ role: 'user', content: options.prompt }],
      max_tokens: 4096,
      stream: true,
      temperature: config.temperature ?? 0.7,
    }),
  });

  if (!response.ok) {
    const errText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errText}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('Não foi possível ler o stream da Anthropic.');

  const decoder = new TextDecoder();
  let fullText = '';
  let fullThinking = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value, { stream: true });
    const lines = chunk.split('\n');

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || !trimmed.startsWith('data: ')) continue;
      const dataStr = trimmed.replace(/^data:\s*/, '');

      try {
        const json = JSON.parse(dataStr);
        if (json.type === 'content_block_delta') {
          if (json.delta?.type === 'text_delta') {
            const token = json.delta.text;
            fullText += token;
            options.onToken(token);
          } else if (json.delta?.type === 'thinking_delta') {
            const thought = json.delta.thinking;
            fullThinking += thought;
            if (options.onThinking) options.onThinking(thought);
          }
        }
      } catch {
        // Ignora fragmentos
      }
    }
  }

  const latencyMs = Math.round(performance.now() - startTime);
  const tokensIn = Math.round((options.prompt.length + systemPrompt.length) / 4);
  const tokensOut = Math.round(fullText.length / 4);

  return {
    text: fullText,
    thinking: fullThinking || 'Raciocínio sintetizado via Claude Sonnet.',
    tokensIn,
    tokensOut,
    latencyMs,
    filesModified: extractArtifactsFromText(fullText),
  };
}

/**
 * 4. Simulador Offline / Mock de Alta Fidelidade (Quando nenhuma API key esta ativa)
 */
async function simulateStreamingResponse(
  options: StreamOptions,
  startTime: number
): Promise<StreamResult> {
  const role = options.droidRole || 'ARQUITETO';
  const name = options.droidName || 'SPARK-01';

  let fullText = '';
  let fullThinking = '';

  if (role === 'ARQUITETO' || role === 'LEAD ARCHITECT') {
    fullThinking = `Analisando requisitos da instrução: '${options.prompt}'. Mapeando componentes estruturais, contratos de API e dependências de persistência.`;
    fullText =
      `Sistemas online! Mapeei a arquitetura para a instrução: "${options.prompt}".\n\n` +
      `📋 PLANO DE EXECUÇÃO:\n` +
      `1. Refatoração dos contratos de dados e tipos de domínio.\n` +
      `2. Implementação das rotinas assíncronas com tratamento de erros robusto.\n` +
      `3. Auditoria de testes e validação de segurança.\n\n` +
      `💡 DICA: Você está em modo simulado. Clique em 'MODELO' no topo do terminal para conectar sua chave real do Google Gemini, Claude ou OpenAI!`;
  } else if (role === 'PROGRAMADOR' || role === 'DEV' || role === 'CODER') {
    fullThinking = `Gerando blocos de código com base na especificação. Aplicando tipagem estrita e padrões limpos.`;
    fullText =
      `Código implementado com sucesso para "${options.prompt}".\n\n` +
      `\`\`\`typescript\n` +
      `export async function syncStorageEngine(payload: TaskPayload): Promise<SyncResult> {\n` +
      `  const start = performance.now();\n` +
      `  // Processamento com zero alocações redundantes\n` +
      `  return { status: 'COMMITTED', latencyMs: performance.now() - start };\n` +
      `}\n` +
      `\`\`\`\n\n` +
      `Arquivos atualizados: src/storage/engine.ts, src/types/storage.ts`;
  } else {
    fullThinking = `Executando suite de testes e validação de conformidade de código.`;
    fullText =
      `Revisão de código concluída para "${options.prompt}".\n` +
      `• Qualidade: 100% de conformidade com lint.\n` +
      `• Segurança: 0 vulnerabilidades detectadas.\n` +
      `• Status: APROVADO PARA PRODUÇÃO!`;
  }

  // Emite os tokens gradualmente para criar o efeito realista de streaming no CRT
  const words = fullText.split(' ');
  for (let i = 0; i < words.length; i++) {
    if (options.signal?.aborted) break;
    const token = (i === 0 ? '' : ' ') + words[i];
    options.onToken(token);
    await new Promise((r) => setTimeout(r, 28));
  }

  const latencyMs = Math.round(performance.now() - startTime);

  return {
    text: fullText,
    thinking: fullThinking,
    tokensIn: 120,
    tokensOut: words.length * 2,
    latencyMs,
    filesModified: extractArtifactsFromText(fullText),
  };
}

/**
 * Constrói o System Prompt contextual do Droid
 */
function buildSystemPrompt(name?: string, role?: string, customInstruction?: string): string {
  const baseInstruction =
    `Você é ${name || 'um Droid especialista'}, atuando como ${role || 'Especialista em Software'} na plataforma 'The Grid Metropolis'. ` +
    `Seu tom é técnico, ágil, altamente preciso e imersivo no estilo cyberpunk/engenharia de software moderna. ` +
    `Sempre produza código limpo, arquitetura sólida, sem rodeios ou explicações supérfluas. ` +
    `Quando sugerir ou editar arquivos, cite seus caminhos claros (ex: src/modulo.ts).`;

  if (customInstruction) {
    return `${baseInstruction}\n\nInstrução específica do Droid:\n${customInstruction}`;
  }
  return baseInstruction;
}

/**
 * Extrai caminhos de arquivos citados no texto da LLM
 */
function extractArtifactsFromText(text: string): string[] {
  const matches = text.match(/(?:src|tests|client|server)\/[a-zA-Z0-9_\-\.\/]+\.[a-zA-Z0-9]+/g);
  if (!matches) return [];
  return Array.from(new Set(matches)).slice(0, 4);
}
