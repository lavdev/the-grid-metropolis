<script lang="ts">
  import { Activity, Bot, CheckCircle2, CircleAlert, Clock3, Cpu, FileCode2, Gauge, Radio, Users } from '@lucide/svelte';
  import type { ProjectTerminal } from '../../lib/stores/projectStore';
  import type { ConnectionState } from '../../lib/socket/client';

  export let project: ProjectTerminal;
  export let connectionStatus: ConnectionState;

  $: activeDroid = project.droids.find((droid) => droid.id === project.pipeline.currentDroidId);
  $: latestMessages = [...project.messages].slice(-5).reverse();
  $: executionLabel = project.pipeline.isActive
    ? 'Em execução'
    : project.pipeline.stage === 'COMPLETED'
      ? 'Concluída'
      : project.pipeline.stage === 'IDLE'
        ? 'Pronta'
        : project.pipeline.stage;
  $: statusColor = project.pipeline.isActive
    ? 'text-cyan-200'
    : project.pipeline.stage === 'COMPLETED'
      ? 'text-emerald-300'
      : 'text-slate-300';

  function messageLabel(message: ProjectTerminal['messages'][number]): string {
    if (message.sender === 'user') return 'Você';
    if (message.sender === 'system') return 'Sistema';
    return message.agentName || 'Droid';
  }

  function messageIcon(message: ProjectTerminal['messages'][number]) {
    if (message.sender === 'system') return Activity;
    if (message.sender === 'user') return Radio;
    return Bot;
  }
</script>

<aside class="execution-inspector" aria-label="Resumo da execução">
  <section class="inspector-card run-card">
    <div class="inspector-heading">
      <div>
        <p class="inspector-kicker">VISÃO GERAL</p>
        <h2>Execução</h2>
      </div>
      <span class="run-indicator" class:running={project.pipeline.isActive} class:complete={project.pipeline.stage === 'COMPLETED'}>
        {#if project.pipeline.isActive}<Activity size={14} />{:else if project.pipeline.stage === 'COMPLETED'}<CheckCircle2 size={14} />{:else}<Clock3 size={14} />{/if}
      </span>
    </div>

    <div class="run-status {statusColor}">{executionLabel}</div>
    <p class="run-description">{project.pipeline.statusText || 'Aguardando uma nova instrução.'}</p>

    {#if project.pipeline.isActive}
      <div class="progress-track" role="progressbar" aria-label="Progresso da execução" aria-valuemin="0" aria-valuemax="100" aria-valuenow={project.pipeline.progressPercent}>
        <span style={`width: ${Math.max(0, Math.min(100, project.pipeline.progressPercent))}%`}></span>
      </div>
      <div class="progress-caption"><span>{activeDroid?.name || 'Preparando Droid'}</span><span>{project.pipeline.progressPercent}%</span></div>
    {:else if activeDroid}
      <div class="current-agent"><span class="agent-dot" style={`background:${activeDroid.accentColor}`}></span><span>Último responsável</span><strong>{activeDroid.name}</strong></div>
    {/if}
  </section>

  <section class="inspector-card metrics-card" aria-label="Métricas do workspace">
    <div class="inspector-heading compact-heading"><div><p class="inspector-kicker">WORKSPACE</p><h2>Atividade</h2></div><Gauge size={16} class="text-cyan-300" /></div>
    <div class="metric-grid">
      <div><span><Cpu size={12} /> Tokens</span><strong>{(project.metrics.tokensIn + project.metrics.tokensOut).toLocaleString()}</strong></div>
      <div><span><Users size={12} /> Droids</span><strong>{project.droids.length}</strong></div>
      <div><span><Clock3 size={12} /> Latência</span><strong>{project.metrics.latencyMs}<small>ms</small></strong></div>
    </div>
  </section>

  <section class="inspector-card squad-card">
    <div class="inspector-heading compact-heading"><div><p class="inspector-kicker">EQUIPE ALOCADA</p><h2>Droids</h2></div><span class="count-pill">{project.droids.length}</span></div>
    {#if project.droids.length === 0}
      <div class="empty-state"><CircleAlert size={15} /> Nenhum Droid alocado neste workspace.</div>
    {:else}
      <ul class="squad-list">
        {#each project.droids as droid (droid.id)}
          <li class:active-agent={droid.id === project.activeDroidId || droid.id === project.pipeline.currentDroidId}>
            <span class="agent-dot" style={`background:${droid.accentColor}`}></span>
            <span class="agent-copy"><strong>{droid.name}</strong><small>{droid.planBadge || droid.role}</small></span>
            <span class="agent-state" title={droid.statusDetail}>{droid.status === 'WORKING' || droid.status === 'THINKING' ? 'Ativo' : droid.status === 'COMPLETED' ? 'Concluído' : droid.status === 'IDLE' ? 'Ocioso' : 'Pronto'}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="inspector-card activity-card">
    <div class="inspector-heading compact-heading"><div><p class="inspector-kicker">SESSÃO</p><h2>Atividade recente</h2></div><Activity size={15} class="text-slate-400" /></div>
    {#if latestMessages.length === 0}
      <p class="activity-empty">As ações da sessão aparecerão aqui.</p>
    {:else}
      <ol class="activity-list">
        {#each latestMessages as message (message.id)}
          {@const MessageIcon = messageIcon(message)}
          <li>
            <span class="activity-icon"><MessageIcon size={12} /></span>
            <div class="activity-copy"><div><strong>{messageLabel(message)}</strong><time>{message.timestamp}</time></div><p>{message.content}</p>
              {#if message.filesModified?.length}
                <span class="artifact-count"><FileCode2 size={11} /> {message.filesModified.length} artefato(s)</span>
              {/if}
            </div>
          </li>
        {/each}
      </ol>
    {/if}
  </section>

  <div class="bridge-footer" class:bridge-connected={connectionStatus === 'CONNECTED'}>
    <span class="bridge-dot"></span>
    <span>{connectionStatus === 'CONNECTED' ? 'Bridge conectada' : connectionStatus === 'CONNECTING' ? 'Conectando à Bridge' : 'Bridge desconectada'}</span>
  </div>
</aside>

<style>
  .execution-inspector { width: 294px; flex: 0 0 294px; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; padding: 18px 16px 20px 0; background: #0a111c; }
  .inspector-card { border: 1px solid #26364b; border-radius: 10px; background: linear-gradient(145deg, #111d2b, #0d1622); padding: 15px; }
  .run-card { border-color: #315267; background: radial-gradient(ellipse at 100% 0%, #0e749022, transparent 55%), #101a28; }
  .inspector-heading { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .inspector-heading h2 { margin: 3px 0 0; color: #edf4fc; font-size: 15px; font-weight: 650; letter-spacing: -.015em; }
  .compact-heading { margin-bottom: 12px; }
  .inspector-kicker { margin: 0; color: #7f95ad; font: 600 9px/1.2 var(--font-mono); letter-spacing: .12em; }
  .run-indicator { display: grid; place-items: center; width: 30px; height: 30px; border: 1px solid #3a4c63; border-radius: 8px; color: #a7b7ca; background: #142235; }
  .run-indicator.running { color: #67e8f9; border-color: #2d7180; background: #0d2934; }
  .run-indicator.complete { color: #86efac; border-color: #28634d; background: #102b22; }
  .run-status { margin-top: 18px; font-size: 20px; font-weight: 650; letter-spacing: -.035em; }
  .run-description { margin: 5px 0 0; color: #a4b2c8; font-size: 12px; line-height: 1.55; overflow-wrap: anywhere; }
  .progress-track { height: 5px; margin-top: 16px; overflow: hidden; border-radius: 999px; background: #1d2b3c; }
  .progress-track span { display: block; height: 100%; border-radius: inherit; background: linear-gradient(90deg, #22d3ee, #3b82f6); transition: width .25s ease; }
  .progress-caption { display: flex; justify-content: space-between; gap: 8px; margin-top: 7px; color: #96a8bc; font: 10px var(--font-mono); }
  .current-agent { display: flex; align-items: center; gap: 7px; margin-top: 15px; color: #8293a9; font-size: 11px; }
  .current-agent strong { margin-left: auto; color: #dce8f5; font-size: 11px; }
  .agent-dot { width: 8px; height: 8px; flex: 0 0 8px; border: 1px solid #ffffff35; border-radius: 50%; box-shadow: 0 0 9px currentColor; }
  .metric-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 7px; }
  .metric-grid > div { min-width: 0; padding: 9px 8px; border: 1px solid #233349; border-radius: 7px; background: #0b1420; }
  .metric-grid span { display: flex; align-items: center; gap: 4px; color: #8498af; font-size: 9px; }
  .metric-grid strong { display: block; margin-top: 8px; overflow: hidden; color: #e5edf7; font: 600 13px var(--font-mono); text-overflow: ellipsis; white-space: nowrap; }
  .metric-grid small { margin-left: 2px; color: #8fa0b4; font-size: 9px; }
  .count-pill { display: grid; min-width: 24px; height: 24px; place-items: center; padding: 0 6px; border: 1px solid #2a5363; border-radius: 999px; color: #8de8f5; background: #0d2731; font: 11px var(--font-mono); }
  .squad-list, .activity-list { display: flex; flex-direction: column; gap: 6px; margin: 0; padding: 0; list-style: none; }
  .squad-list li { display: flex; align-items: center; gap: 9px; min-width: 0; padding: 8px; border: 1px solid transparent; border-radius: 7px; background: #0b1420; }
  .squad-list li.active-agent { border-color: #3b6875; background: #10212d; }
  .agent-copy { display: flex; min-width: 0; flex: 1; flex-direction: column; gap: 3px; }
  .agent-copy strong { overflow: hidden; color: #e4edf7; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
  .agent-copy small { overflow: hidden; color: #8395aa; font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
  .agent-state { flex: 0 0 auto; color: #98a9bc; font-size: 9px; }
  .activity-list { gap: 0; }
  .activity-list li { display: flex; gap: 9px; padding: 9px 0; border-top: 1px solid #202d3d; }
  .activity-icon { display: grid; width: 23px; height: 23px; flex: 0 0 23px; place-items: center; border: 1px solid #2b3f56; border-radius: 6px; color: #8ddde9; background: #111f2e; }
  .activity-copy { min-width: 0; flex: 1; }
  .activity-copy > div { display: flex; justify-content: space-between; gap: 6px; }
  .activity-copy strong { overflow: hidden; color: #cbd7e5; font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
  .activity-copy time { flex: 0 0 auto; color: #72849a; font: 9px var(--font-mono); }
  .activity-copy p { display: -webkit-box; overflow: hidden; margin: 4px 0 0; color: #8496aa; font-size: 10px; line-height: 1.45; overflow-wrap: anywhere; -webkit-box-orient: vertical; -webkit-line-clamp: 2; line-clamp: 2; }
  .artifact-count { display: inline-flex; align-items: center; gap: 4px; margin-top: 5px; color: #7ed5e7; font-size: 9px; }
  .empty-state, .activity-empty { display: flex; align-items: center; gap: 7px; margin: 0; color: #8496aa; font-size: 10px; line-height: 1.5; }
  .bridge-footer { display: flex; align-items: center; gap: 8px; margin-top: auto; padding: 10px 2px 0; color: #f2c77e; font-size: 10px; }
  .bridge-footer.bridge-connected { color: #83dcb0; }
  .bridge-dot { width: 7px; height: 7px; border-radius: 50%; background: currentColor; }
  @media (max-width: 1200px) { .execution-inspector { width: 270px; flex-basis: 270px; } }
  @media (max-width: 980px) { .execution-inspector { display: none; } }
</style>
