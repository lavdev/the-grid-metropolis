<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectSidebar from './components/sidebar/ProjectSidebar.svelte';
  import CrtTerminal from './components/terminal/CrtTerminal.svelte';
  import MissionBacklogModal from './components/tasks/MissionBacklogModal.svelte';
  import DroidForgeModal from './components/droids/DroidForgeModal.svelte';
  import AiSettingsModal from './components/terminal/AiSettingsModal.svelte';
  import UiFeedback from './components/UiFeedback.svelte';
  import { GridWebSocketClient, type ConnectionState } from './lib/socket/client';
  import { handleIncomingEvent } from './lib/stores/gridStore';
  import { loadProjectsFromDatabase } from './lib/stores/projectStore';

  let client: GridWebSocketClient | null = null;
  let connectionStatus: ConnectionState = 'CONNECTING';

  function reconnectBridge() {
    client?.connect();
  }

  onMount(() => {
    loadProjectsFromDatabase();
    client = new GridWebSocketClient();
    const unsubState = client.onStateChange((state) => {
      connectionStatus = state;
    });

    const unsubEvents = client.subscribe((event) => {
      handleIncomingEvent(event);
    });

    client.connect();

    return () => {
      unsubState();
      unsubEvents();
      client?.disconnect();
    };
  });
</script>

<div class="w-screen h-screen overflow-hidden flex bg-[#03060d] relative font-sans text-slate-100 select-none">
  <!-- 1. Sidebar de Projetos / Workspaces e Droids (Estilo Habblaud / Cyber-Office) -->
  <ProjectSidebar />

  <!-- 2. Área Central: Cyber-Deck Terminal CRT com Fundo Temático The Grid Metropolis -->
  <main class="flex-1 min-w-0 h-full overflow-hidden flex flex-col relative bg-[#04060c]">
    <CrtTerminal {connectionStatus} onReconnect={reconnectBridge} />
  </main>

  <!-- Modais Compartilhados (Painel de Missões & Backlog Linear & Configuração de IA) -->
  <MissionBacklogModal />
  <DroidForgeModal />
  <AiSettingsModal />
  <UiFeedback />
</div>
