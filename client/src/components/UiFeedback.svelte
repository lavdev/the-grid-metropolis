<script lang="ts">
  import { CheckCircle2, CircleAlert, Info, X } from '@lucide/svelte';
  import {
    activeConfirmation,
    appToasts,
    dismissToast,
    resolveConfirmation,
  } from '../lib/stores/uiFeedbackStore';

  let confirmationDialog: HTMLElement;

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && $activeConfirmation) {
      event.preventDefault();
      resolveConfirmation(false);
    }
  }

  function trapDialogFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    const buttons = confirmationDialog?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)');
    if (!buttons?.length) return;

    const first = buttons[0];
    const last = buttons[buttons.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function focusDialogControl(node: HTMLButtonElement) {
    node.focus();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="pointer-events-none fixed right-5 top-5 z-[100] flex w-[min(24rem,calc(100vw-2.5rem))] flex-col gap-3" aria-live="polite" aria-relevant="additions removals">
  {#each $appToasts as toast (toast.id)}
    <div
      class="pointer-events-auto flex gap-3 rounded-xl border bg-[#080d18]/95 p-4 shadow-[0_14px_50px_rgba(0,0,0,0.55)] backdrop-blur-xl {toast.tone === 'error' ? 'border-rose-500/50' : toast.tone === 'success' ? 'border-emerald-500/50' : 'border-cyan-500/40'}"
      role={toast.tone === 'error' ? 'alert' : 'status'}
    >
      <div class="mt-0.5 shrink-0 {toast.tone === 'error' ? 'text-rose-300' : toast.tone === 'success' ? 'text-emerald-300' : 'text-cyan-300'}">
        {#if toast.tone === 'error'}<CircleAlert size={18} />{:else if toast.tone === 'success'}<CheckCircle2 size={18} />{:else}<Info size={18} />{/if}
      </div>
      <div class="min-w-0 flex-1">
        <div class="text-sm font-bold text-slate-100">{toast.title}</div>
        <p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-slate-300">{toast.message}</p>
      </div>
      <button type="button" class="h-fit rounded p-1 text-slate-500 transition hover:bg-slate-800 hover:text-white" aria-label="Fechar aviso" on:click={() => dismissToast(toast.id)}>
        <X size={15} />
      </button>
    </div>
  {/each}
</div>

{#if $activeConfirmation}
  <div
    class="fixed inset-0 z-[110] flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
    role="presentation"
    on:click={(event) => event.target === event.currentTarget && resolveConfirmation(false)}
  >
    <div
      class="w-full max-w-md rounded-2xl border border-cyan-500/40 bg-[#080d18] p-6 shadow-[0_0_50px_rgba(0,0,0,0.75)]"
      bind:this={confirmationDialog}
      on:keydown={trapDialogFocus}
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirmation-title"
      aria-describedby="confirmation-message"
      tabindex="-1"
    >
      <div class="mb-3 flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-xl border { $activeConfirmation.tone === 'danger' ? 'border-rose-500/40 bg-rose-950/60 text-rose-300' : 'border-amber-500/40 bg-amber-950/50 text-amber-300'}">
          <CircleAlert size={20} />
        </div>
        <h2 id="confirmation-title" class="text-base font-bold text-slate-100">{$activeConfirmation.title}</h2>
      </div>
      <p id="confirmation-message" class="whitespace-pre-line text-sm leading-relaxed text-slate-300">{$activeConfirmation.message}</p>
      <div class="mt-6 flex justify-end gap-2">
        <button
          type="button"
          use:focusDialogControl
          class="rounded-lg border border-slate-700 px-4 py-2 text-sm font-semibold text-slate-300 transition hover:border-slate-500 hover:bg-slate-800 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-cyan-400"
          on:click={() => resolveConfirmation(false)}
        >{$activeConfirmation.cancelLabel}</button>
        <button
          type="button"
          class="rounded-lg border px-4 py-2 text-sm font-bold transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 { $activeConfirmation.tone === 'danger' ? 'border-rose-400/60 bg-rose-600/20 text-rose-200 hover:bg-rose-600/35 focus-visible:outline-rose-400' : 'border-cyan-400/60 bg-cyan-600/20 text-cyan-100 hover:bg-cyan-600/35 focus-visible:outline-cyan-400'}"
          on:click={() => resolveConfirmation(true)}
        >{$activeConfirmation.confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}
