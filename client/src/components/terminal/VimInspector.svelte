<script lang="ts">
  import {
    workspaceFiles,
    activeFilePath,
    closeVimInspector,
    openFileInVim,
    type WorkspaceFile,
  } from '../../lib/stores/workspaceFilesStore';
  import { activeProject } from '../../lib/stores/projectStore';
  import { soundFX } from '../../lib/audio/soundFX';
  import { FileCode2, X, Terminal, Bot, Check, ArrowLeft } from '@lucide/svelte';

  $: currentProjectId = $activeProject?.id || 'proj-alfa';
  $: files = $workspaceFiles[currentProjectId] || [];
  $: currentFile = files.find((f) => f.path === $activeFilePath) || files[0] || null;

  function handleSelectFile(path: string) {
    activeFilePath.set(path);
    soundFX.playKeyClick();
  }

  function handleClose() {
    soundFX.playKeyClick();
    closeVimInspector();
  }

  // Sintaxe retro simples com coloração de tokens
  function highlightLine(line: string, lang: string): string {
    if (!line) return '&nbsp;';

    // Comentários
    if (line.trim().startsWith('//') || line.trim().startsWith('//!')) {
      return `<span class="text-slate-500 italic">${escapeHtml(line)}</span>`;
    }
    if (line.trim().startsWith('#')) {
      return `<span class="text-cyan-400 font-bold">${escapeHtml(line)}</span>`;
    }

    let escaped = escapeHtml(line);

    // Palavras-chave Rust / TS
    const keywords = ['pub', 'fn', 'struct', 'enum', 'impl', 'async', 'await', 'trait', 'let', 'mut', 'match', 'return', 'use', 'mod', 'const', 'export', 'import', 'from', 'interface', 'type'];
    const types = ['Self', 'String', 'Vec', 'Result', 'Option', 'Sender', 'Receiver', 'Arc', 'Transaction', 'PersistenceEngine', 'StorageDriver', 'JournalMode', 'u64', 'u32', 'usize', 'i64', 'bool'];

    // Strings
    escaped = escaped.replace(/(["'`].*?["'`])/g, '<span class="text-emerald-300 font-medium">$1</span>');

    // Keywords
    keywords.forEach((kw) => {
      const reg = new RegExp(`\\b(${kw})\\b`, 'g');
      escaped = escaped.replace(reg, '<span class="text-fuchsia-400 font-bold">$1</span>');
    });

    // Types
    types.forEach((t) => {
      const reg = new RegExp(`\\b(${t})\\b`, 'g');
      escaped = escaped.replace(reg, '<span class="text-cyan-300 font-semibold">$1</span>');
    });

    return escaped;
  }

  function escapeHtml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }
</script>

<!-- Container Principal do Inspetor Vim Retrô -->
<div class="relative w-full h-full flex flex-col bg-[#050912] font-mono select-none overflow-hidden text-xs">
  <!-- 1. Top Bar de Abas de Arquivos (Estilo Vim Buffer / Neovim Bufferline) -->
  <div class="px-3 py-2 bg-[#090f1d] border-b border-cyan-500/30 flex items-center justify-between gap-3 overflow-x-auto">
    <div class="flex items-center gap-1.5 overflow-x-auto">
      <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-950/80 border border-emerald-500/40 text-emerald-400 font-pixel-title text-[10px] tracking-wider shrink-0 mr-1">
        <FileCode2 size={12} />
        <span>VIM v9.1</span>
      </div>

      {#each files as f}
        {@const isActive = currentFile?.path === f.path}
        <button
          type="button"
          on:click={() => handleSelectFile(f.path)}
          class="flex items-center gap-2 px-3 py-1 rounded-t-lg transition-all cursor-pointer shrink-0 border-t border-x text-xs {isActive
            ? 'bg-[#0f172a] border-cyan-500/60 text-cyan-200 shadow-[0_-2px_8px_rgba(0,240,255,0.2)]'
            : 'bg-transparent border-transparent text-slate-400 hover:text-slate-200 hover:bg-[#0c1424]'}"
        >
          <span class="font-bold">{f.name}</span>
          <span
            class="text-[8px] font-pixel-title px-1 rounded uppercase tracking-wider"
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
      class="px-2.5 py-1 rounded-lg bg-rose-950/70 hover:bg-rose-900 border border-rose-500/50 text-rose-300 font-pixel-title text-[10px] tracking-wider flex items-center gap-1 transition cursor-pointer shrink-0"
      title="Fechar Inspetor de Código (:q)"
    >
      <ArrowLeft size={12} />
      <span>VOLTAR AO TERMINAL (:q)</span>
    </button>
  </div>

  <!-- 2. Área do Código com Numeração de Linhas -->
  <div class="flex-1 overflow-y-auto p-4 flex custom-scrollbar bg-[#03060c]">
    {#if currentFile}
      <!-- Coluna de Números de Linha -->
      <div class="pr-4 border-r border-slate-800/80 select-none text-right font-mono text-[11px] text-slate-600 leading-relaxed font-semibold">
        {#each currentFile.content.split('\n') as _, index}
          <div>{String(index + 1).padStart(2, '0')}</div>
        {/each}
      </div>

      <!-- Conteúdo do Arquivo -->
      <div class="pl-4 flex-1 font-mono text-[11px] leading-relaxed overflow-x-auto text-slate-200">
        {#each currentFile.content.split('\n') as line}
          <div class="whitespace-pre">{@html highlightLine(line, currentFile.language)}</div>
        {/each}
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center text-slate-500 font-mono text-xs">
        Nenhum arquivo de código selecionado.
      </div>
    {/if}
  </div>

  <!-- 3. Powerline Footer Retrô (Estilo Vim Statusline) -->
  <footer class="px-3 py-1.5 bg-[#080d19] border-t border-cyan-500/30 flex items-center justify-between text-[10px] font-pixel-terminal select-none text-slate-300">
    <div class="flex items-center gap-2">
      <span class="px-2 py-0.5 rounded bg-cyan-400 text-black font-bold uppercase tracking-wider">
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
