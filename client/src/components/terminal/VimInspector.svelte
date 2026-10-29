<script lang="ts">
  import {
    workspaceFiles,
    activeFilePath,
    closeVimInspector,
    workspaceFileLoading,
    type WorkspaceFile,
  } from '../../lib/stores/workspaceFilesStore';
  import { activeProject } from '../../lib/stores/projectStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { highlightCode } from '../../lib/markdown';
  import { FileCode2, X, Terminal, Bot, Check, ArrowLeft } from '@lucide/svelte';

  $: currentProjectId = $activeProject?.id || 'proj-alfa';
  $: files = $workspaceFiles[currentProjectId] || [];
  $: currentFile = ($activeFilePath ? files.find((f) => f.path === $activeFilePath) : files[0]) || null;

  function handleSelectFile(path: string) {
    activeFilePath.set(path);
    soundFX.playKeyClick();
  }

  function handleClose() {
    soundFX.playKeyClick();
    closeVimInspector();
  }

  // O mesmo tokenizer do terminal mantém o editor coerente com a linguagem do arquivo.
  function highlightLine(line: string, lang: string): string {
    return highlightCode(line, lang) || '&nbsp;';
  }
</script>

<!-- Container Principal do Inspetor Vim Retrô -->
<div class="relative w-full h-full flex flex-col bg-[#050912] font-mono select-none overflow-hidden text-sm">
  <!-- 1. Top Bar de Abas de Arquivos (Estilo Vim Buffer / Neovim Bufferline) -->
  <div class="px-3 py-2 bg-[#090f1d] border-b border-cyan-500/30 flex items-center justify-between gap-3 overflow-x-auto">
    <div class="flex items-center gap-1.5 overflow-x-auto">
      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-emerald-400 font-ui text-[12px] tracking-normal shrink-0 mr-1">
        <FileCode2 size={12} />
        <span>VIM v9.1</span>
      </div>

      {#each files as f}
        {@const isActive = currentFile?.path === f.path}
        <button
          type="button"
          on:click={() => handleSelectFile(f.path)}
          class="flex items-center gap-2 px-3 py-1 rounded-t-lg transition-all cursor-pointer shrink-0 border-t border-x text-sm {isActive
            ? 'bg-[#0f172a] border-cyan-500/60 text-cyan-200 shadow-[0_-2px_8px_rgba(0,240,255,0.2)]'
            : 'bg-transparent border-transparent text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'}"
        >
          <span class="font-bold">{f.name}</span>
          <span
            class="text-[12px] font-ui px-1 rounded uppercase tracking-normal"
            style="color: {f.accentColor}; background: {f.accentColor}18; border: 1px solid {f.accentColor}40"
          >
            @{f.authorDroid}
          </span>
        </button>
      {/each}
    </div>

    <!-- Botão Fechar Vim (:q) -->
    <button
      type="button"
      on:click={handleClose}
      class="px-2.5 py-1 rounded-lg bg-rose-950/70 hover:bg-rose-900 border border-rose-500/50 text-rose-300 font-ui text-[12px] tracking-normal flex items-center gap-1 transition cursor-pointer shrink-0"
      title="Fechar Inspetor de Código (:q)"
    >
      <ArrowLeft size={12} />
      <span>VOLTAR AO TERMINAL (:q)</span>
    </button>
  </div>

  <!-- 2. Área do Código com Numeração de Linhas -->
  <div class="flex-1 overflow-y-auto p-4 flex custom-scrollbar bg-[#03060c]">
    {#if $workspaceFileLoading && !$workspaceFiles[currentProjectId]?.some((file) => file.path === $workspaceFileLoading)}
      <div class="flex-1 flex items-center justify-center text-cyan-300 font-mono text-sm animate-pulse">
        Lendo artefato <span class="ml-2 text-slate-400">{$workspaceFileLoading}</span>…
      </div>
    {:else if currentFile?.error}
      <div class="flex-1 flex flex-col items-center justify-center gap-2 text-rose-300 font-mono text-sm text-center">
        <Terminal size={18} />
        <strong>Artefato indisponível</strong>
        <span class="text-slate-500 max-w-md">{currentFile.error}</span>
        <span class="text-slate-600 text-xs">O caminho foi preservado, mas o conteúdo não está acessível neste workspace.</span>
      </div>
    {:else if currentFile}
      <!-- Coluna de Números de Linha -->
      <div class="pr-4 border-r border-slate-800/80 select-none text-right font-mono text-sm text-slate-600 leading-relaxed font-semibold">
        {#each currentFile.content.split('\n') as _, index}
          <div>{String(index + 1).padStart(2, '0')}</div>
        {/each}
      </div>

      <!-- Conteúdo do Arquivo -->
      <div class="readable-text pl-4 flex-1 min-w-0 font-mono text-sm leading-relaxed overflow-x-auto text-slate-200">
        {#if currentFile.content}
          {#each currentFile.content.split('\n') as line}
            <div class="whitespace-pre">{@html highlightLine(line, currentFile.language)}</div>
          {/each}
        {:else}
          <div class="text-slate-500 italic">Arquivo vazio.</div>
        {/if}
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center text-slate-500 font-mono text-sm">
        Nenhum arquivo de código selecionado.
      </div>
    {/if}
  </div>

  <!-- 3. Powerline Footer Retrô (Estilo Vim Statusline) -->
  <footer class="px-3 py-1.5 bg-[#080d19] border-t border-cyan-500/30 flex flex-wrap gap-2 items-center justify-between text-[12px] font-code select-none text-slate-300">
    <div class="flex items-center gap-2">
      <span class="px-2 py-0.5 rounded bg-cyan-400 text-black font-bold uppercase tracking-normal">
        NORMAL
      </span>
      <span class="text-cyan-300 font-mono font-bold">
        {currentFile?.path || 'buffer'} [RO]
      </span>
      <span class="text-slate-500 hidden sm:inline">•</span>
      <span class="text-slate-400 font-mono hidden sm:inline">
        Autor: <strong style="color: {currentFile?.accentColor || '#00f0ff'}">{currentFile?.authorDroid}</strong>
      </span>
    </div>

    <div class="flex items-center gap-3 font-mono text-slate-400">
      <span class="hidden md:inline">utf-8</span>
      <span class="hidden md:inline">|</span>
      <span class="uppercase text-emerald-400">{currentFile?.language || 'text'}</span>
      <span>|</span>
      <span>{currentFile?.linesCount || 0} lines</span>
      <span>|</span>
      <span class="text-cyan-400 font-bold">100%</span>
      <span class="hidden lg:inline text-slate-600">(:q para fechar)</span>
    </div>
  </footer>
</div>

<style>
  :global(.code-token-keyword) { color: #d8b4fe; font-weight: 700; }
  :global(.code-token-string) { color: #86efac; }
  :global(.code-token-comment) { color: #718096; font-style: italic; }
  :global(.code-token-number) { color: #fbbf24; }
  :global(.code-token-tag) { color: #67e8f9; }
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: #03060c;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #182236;
    border-radius: 3px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #00f0ff40;
  }
</style>
