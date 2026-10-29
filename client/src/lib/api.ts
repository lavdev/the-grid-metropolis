const isTauriDesktop =
  typeof window !== 'undefined' &&
  (window.location.protocol === 'tauri:' || window.location.hostname === 'tauri.localhost');
const API_ORIGIN = isTauriDesktop ? 'http://127.0.0.1:3000' : '';

export function apiUrl(path: string): string {
  return `${API_ORIGIN}${path}`;
}
