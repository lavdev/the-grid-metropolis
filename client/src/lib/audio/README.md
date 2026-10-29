# Audio Effects

## Overview
Provides synthesized retro sound effects for client interactions.

## Interface
- `soundFX` is the exported `RetroAudioSynthesizer` instance (`soundFX.ts:RetroAudioSynthesizer`, `soundFX.ts:soundFX`).

## Invariants
- None explicitly declared (`soundFX.ts:RetroAudioSynthesizer`).

## Side Effects
- Creates and uses the browser `AudioContext` when effect methods are invoked (`soundFX.ts:RetroAudioSynthesizer.getContext`, `soundFX.ts:RetroAudioSynthesizer.playHandoff`, `soundFX.ts:RetroAudioSynthesizer.playPipelineComplete`, `soundFX.ts:RetroAudioSynthesizer.playKeyClick`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
