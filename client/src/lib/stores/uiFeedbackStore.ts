import { writable } from 'svelte/store';

export type ToastTone = 'success' | 'error' | 'info';

export interface AppToast {
  id: number;
  title: string;
  message: string;
  tone: ToastTone;
}

export interface ConfirmationOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  tone?: 'danger' | 'normal';
}

interface ActiveConfirmation extends Required<ConfirmationOptions> {
  resolve: (confirmed: boolean) => void;
}

export const appToasts = writable<AppToast[]>([]);
export const activeConfirmation = writable<ActiveConfirmation | null>(null);

let nextToastId = 0;
let confirmationResolver: ((confirmed: boolean) => void) | null = null;
const toastTimers = new Map<number, ReturnType<typeof setTimeout>>();

export function notify(title: string, message: string, tone: ToastTone = 'info'): void {
  const id = ++nextToastId;
  appToasts.update((toasts) => [...toasts, { id, title, message, tone }]);
  toastTimers.set(id, setTimeout(() => dismissToast(id), 5000));
}

export function dismissToast(id: number): void {
  const timer = toastTimers.get(id);
  if (timer) clearTimeout(timer);
  toastTimers.delete(id);
  appToasts.update((toasts) => toasts.filter((toast) => toast.id !== id));
}

export function confirmAction(options: ConfirmationOptions): Promise<boolean> {
  confirmationResolver?.(false);

  return new Promise((resolve) => {
    confirmationResolver = resolve;
    activeConfirmation.set({
      title: options.title,
      message: options.message,
      confirmLabel: options.confirmLabel ?? 'Confirmar',
      cancelLabel: options.cancelLabel ?? 'Cancelar',
      tone: options.tone ?? 'normal',
      resolve,
    });
  });
}

export function resolveConfirmation(confirmed: boolean): void {
  const resolve = confirmationResolver;
  confirmationResolver = null;
  activeConfirmation.set(null);
  resolve?.(confirmed);
}
