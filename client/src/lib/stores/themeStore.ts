import { writable } from 'svelte/store';

export const isArcadeMode = writable<boolean>(true);

export function toggleArcadeMode() {
  isArcadeMode.update((v) => !v);
}
