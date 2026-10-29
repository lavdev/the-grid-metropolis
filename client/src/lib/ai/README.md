# AI Service

## Overview
Streams assistant completions through the configured provider and returns completion metrics and extracted artifact paths.

## Interface
- `StreamOptions`, `StreamResult`, and `streamAiCompletion` define the provider-neutral request and result boundary (`aiService.ts:StreamOptions`, `aiService.ts:StreamResult`, `aiService.ts:streamAiCompletion`).
- Provider implementations are `streamCodexCli`, `streamGemini`, `streamOpenAiCompatible`, `streamAnthropic`, and `simulateStreamingResponse` (`aiService.ts:streamCodexCli`, `aiService.ts:streamGemini`, `aiService.ts:streamOpenAiCompatible`, `aiService.ts:streamAnthropic`, `aiService.ts:simulateStreamingResponse`).

## Invariants
- Provider selection and credentials are read from `aiConfig`; the result reports the selected provider and completion metrics (`aiService.ts:streamAiCompletion`, `aiService.ts:StreamResult`).

## Side Effects
- Sends HTTP requests to the local Codex endpoint or configured AI provider endpoints and consumes streaming responses (`aiService.ts:streamCodexCli`, `aiService.ts:streamOpenAiCompatible`, `aiService.ts:streamAnthropic`).

## Verification
- Run `npm run check` from `client/` (`package.json`).
