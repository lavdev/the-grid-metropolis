<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TopBarHUD from './components/TopBarHUD.svelte';
  import CanvasContainer from './components/CanvasContainer.svelte';
  import AgentDrawer from './components/AgentDrawer.svelte';
  import { GridWebSocketClient, type ConnectionState } from './lib/socket/client';
  import { handleIncomingEvent } from './lib/stores/gridStore';

  let client: GridWebSocketClient;
  let connectionStatus: ConnectionState = 'CONNECTING';

  onMount(() => {
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
      client.disconnect();
    };
  });
</script>

<div class="w-screen h-screen overflow-hidden flex flex-col bg-[#050608] relative font-sans">
  <TopBarHUD {connectionStatus} />
  <CanvasContainer />
  <AgentDrawer />
  <div class="cyber-scanline fixed inset-0 z-50 pointer-events-none"></div>
</div>
