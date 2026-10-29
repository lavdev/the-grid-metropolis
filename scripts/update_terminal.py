import sys

path = 'client/src/components/terminal/CrtTerminal.svelte'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Normalize
text = text.replace('\r\n', '\n')

old_button = """          <!-- Botão de Configuração de IA e Modelo Ativo -->
          <button
            type="button"
            on:click={openAiSettings}
            class="flex items-center gap-2 px-3 py-1 rounded border text-cyan-300 font-bold text-[11px] shadow-sm hover:border-cyan-400 transition cursor-pointer shrink-0 {hasConfiguredApiKey()
              ? 'bg-cyan-950/50 border-cyan-500/50 hover:bg-cyan-900/60 shadow-[0_0_10px_rgba(0,240,255,0.25)]'
              : 'bg-amber-950/40 border-amber-500/50 text-amber-300 hover:bg-amber-900/40'}"
            title="Abrir Configuração de IA e Chaves de API (Gemini / Claude / OpenAI / Ollama)"
          >
            <Key class="w-3.5 h-3.5 text-cyan-400" />
            <span>
              {$aiConfig.provider === 'codex' ? 'CODEX CLI: ' + ($aiConfig.model === 'default' ? 'ASSINATURA' : $aiConfig.model.toUpperCase()) : 'MODELO: ' + $aiConfig.model.toUpperCase()}
            </span>
            <span class="w-1.5 h-1.5 rounded-full {hasConfiguredApiKey() ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'}"></span>
          </button>"""

new_button = """          <!-- Botão Dropdown Rápido de Seleção de Modelo (Codex / Reasoning) -->
          <div class="relative">
            <button
              type="button"
              on:click={() => (isModelMenuOpen = !isModelMenuOpen)}
              class="flex items-center gap-1.5 px-2.5 py-1 rounded border text-cyan-300 font-bold text-[11px] shadow-sm hover:border-cyan-400 transition cursor-pointer shrink-0 {hasConfiguredApiKey()
                ? 'bg-cyan-950/50 border-cyan-500/50 hover:bg-cyan-900/60 shadow-[0_0_10px_rgba(0,240,255,0.25)]'
                : 'bg-amber-950/40 border-amber-500/50 text-amber-300 hover:bg-amber-900/40'}"
              title="Selecionar Modelo & Reasoning Effort"
            >
              <Cpu class="w-3.5 h-3.5 text-cyan-400" />
              <span>
                {$aiConfig.provider === 'codex'
                  ? ($aiConfig.model === 'default' ? 'DEFAULT' : $aiConfig.model.toUpperCase())
                  : $aiConfig.model.toUpperCase()}
              </span>
              {#if $aiConfig.provider === 'codex'}
                <span class="text-[8.5px] px-1 py-0.2 rounded bg-black/50 text-emerald-400 font-mono border border-emerald-500/30">
                  {$aiConfig.reasoningEffort === 'low' ? 'LIGHT' : $aiConfig.reasoningEffort === 'medium' ? 'MED' : $aiConfig.reasoningEffort === 'high' ? 'HIGH' : 'EXTRA'}
                </span>
              {/if}
              <ChevronDown size={11} class="text-cyan-400" />
            </button>

            <!-- Dropdown Popover Idêntico ao Codex -->
            <ModelSelectorDropdown isOpen={isModelMenuOpen} on:close={() => (isModelMenuOpen = false)} />
          </div>

          <!-- Botão de Configuração de Chaves & Provedor -->
          <button
            type="button"
            on:click={openAiSettings}
            class="p-1 rounded border border-slate-700 bg-slate-900/80 text-slate-400 hover:text-cyan-300 hover:border-cyan-500 transition cursor-pointer"
            title="Abrir Configurações Globais de IA e Chaves de API"
          >
            <Key class="w-3.5 h-3.5" />
          </button>"""

if old_button in text:
    text = text.replace(old_button, new_button)
    print("Replaced model button")
else:
    print("WARNING: old_button not found")

old_metrics = """          <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-900/70 border border-slate-800">
            <span class="text-slate-400">LATÊNCIA:</span>
            <span class="font-bold text-emerald-400 font-pixel-terminal text-sm">{$activeProject.metrics.latencyMs}ms</span>
          </div>"""

new_metrics = """          <div class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-slate-900/70 border border-slate-800">
            <span class="text-slate-400">LATÊNCIA:</span>
            <span class="font-bold text-emerald-400 font-pixel-terminal text-sm">{$activeProject.metrics.latencyMs}ms</span>
          </div>

          <!-- Botão Limpar Chat / Tarefas do Workspace -->
          <button
            type="button"
            on:click={handleClearMessages}
            class="flex items-center gap-1.5 px-2.5 py-1 rounded border border-rose-500/30 bg-rose-950/30 hover:bg-rose-950/60 text-rose-300 text-[10px] font-pixel-title transition cursor-pointer"
            title="Limpar histórico de mensagens deste workspace"
          >
            <Trash2 size={12} />
            <span class="hidden sm:inline">LIMPAR CHAT</span>
          </button>"""

if old_metrics in text:
    text = text.replace(old_metrics, new_metrics)
    print("Added clear messages button")
else:
    print("WARNING: old_metrics not found")

old_user_hdr = """                  <div class="text-[10px] font-pixel-title text-emerald-400 font-bold uppercase tracking-wider mb-1 flex items-center justify-between gap-4">
                    <span>[ARQUITETO@{$activeProject.name.toUpperCase()}]: ~</span>
                    <span class="text-emerald-500/70 text-[8.5px] font-pixel-terminal text-xs">{msg.timestamp}</span>
                  </div>"""

new_user_hdr = """                  <div class="text-[10px] font-pixel-title text-emerald-400 font-bold uppercase tracking-wider mb-1 flex items-center justify-between gap-4">
                    <span>[ARQUITETO@{$activeProject.name.toUpperCase()}]: ~</span>
                    <div class="flex items-center gap-1.5">
                      <span class="text-emerald-500/70 text-[8.5px] font-pixel-terminal text-xs">{msg.timestamp}</span>
                      <button
                        type="button"
                        on:click={() => handleDeleteMessage(msg.id)}
                        class="opacity-50 hover:opacity-100 hover:text-rose-400 text-slate-400 transition p-0.5 cursor-pointer"
                        title="Excluir mensagem"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  </div>"""

if old_user_hdr in text:
    text = text.replace(old_user_hdr, new_user_hdr)
    print("Added user message delete button")
else:
    print("WARNING: old_user_hdr not found")

old_agent_hdr = """                    <span class="opacity-70 text-[8.5px] font-pixel-terminal text-xs">[{msg.timestamp}]</span>"""

new_agent_hdr = """                    <div class="flex items-center gap-1.5">
                      <span class="opacity-70 text-[8.5px] font-pixel-terminal text-xs">[{msg.timestamp}]</span>
                      <button
                        type="button"
                        on:click={() => handleDeleteMessage(msg.id)}
                        class="opacity-50 hover:opacity-100 hover:text-rose-400 text-slate-400 transition p-0.5 cursor-pointer"
                        title="Excluir mensagem"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>"""

if old_agent_hdr in text:
    text = text.replace(old_agent_hdr, new_agent_hdr)
    print("Added agent message delete button")
else:
    print("WARNING: old_agent_hdr not found")

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Finished updating CrtTerminal.svelte")
