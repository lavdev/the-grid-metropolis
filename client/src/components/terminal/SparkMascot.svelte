<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';

  export let state: 'idle' | 'speaking' | 'thinking' | 'celebrate' = 'idle';
  export let label: string = 'SPARK-01';
  export let interactive: boolean = true;
  export let wander: boolean = true;
  export let inactivityTimeoutMs: number = 5 * 60 * 1000; // 5 minutos padrão

  const dispatch = createEventDispatcher();
  let isHovered = false;
  let clickReactionTimer: number | null = null;
  let localOverrideState: 'idle' | 'speaking' | 'thinking' | 'celebrate' | 'sleep' | null = null;

  // Posição 2D (X e Y) dentro do monitor CRT
  const BASE_X = 6;
  const BASE_Y = 76; // Próximo ao rodapé CLI
  let currentXPercent = BASE_X;
  let currentYPercent = BASE_Y;
  let direction: 1 | -1 = 1; // 1 = direita, -1 = esquerda
  let pitchDeg = 0; // Inclinação de voo (subir/descer)
  let isFlying = false;
  let moveDurationSec = 3;
  let wanderTimer: number | null = null;

  // Inatividade / Modo de Voo e Sono
  // Idle ativo = pousado na base CLI
  // Após 2 minutos sem interação = voo/patrulha aérea
  // Após 5 minutos sem interação = descida e sono profundo (Zzzz e recarga)
  let isSleeping = false;
  let isPatrolAllowed = false; // Só decola após inatividade
  let patrolInactivityTimer: number | null = null;
  let inactivityTimer: number | null = null;
  let batteryLevel = 38; // Porcentagem que sobe durante a recarga
  let batteryInterval: number | null = null;
  const PATROL_TIMEOUT_MS = 2 * 60 * 1000; // 2 minutos para liberar voo

  // Balão de Piadas Geek em Idle
  const GEEK_JOKES = [
    "Existem 10 tipos de pessoas: as que entendem binário e as que não! 🤖",
    "Não é bug, é uma feature não documentada! 🐞",
    "O que o dev foi fazer na praia? Surfar no buffer e pescar sockets! 🌊",
    "Compilando piada em Rust... Zero overhead e sem data races! 🦀",
    "sudo apt-get install mais_cafe ☕",
    "A IA não vai dominar o mundo... a menos que o git push --force passe! 😅",
    "Ping 12ms... Conexão mais rápida que a entrega de pizza do laboratório! 🍕",
    "Por que o droid atravessou a rede? Para atualizar o firmware do outro lado! ⚡",
    "Limpei 4MB de logs da memória. Mereço um cookie sintético! 🍪",
    "Status: 100% de CPU dedicado a calcular o sentido da vida: 42. 🌌",
    "Cuidado com o garbage collector, ele quase me pegou! 🗑️",
    "Warning: Nível de cafeína no pod abaixo do limite operacional! ⚡",
    "SELECT * FROM brain WHERE thoughts IS NOT NULL; 0 rows returned. 😂",
    "Quem precisa de sono quando se tem Tokio async runtime rodando a 60 FPS? 🚀",
  ];

  let currentJoke: string | null = null;
  let jokeTimer: number | null = null;

  $: activeState = localOverrideState || (isSleeping ? 'sleep' : state);

  // Monitora mudanças de estado externo para cancelar sono ou voo
  $: {
    if (state !== 'idle') {
      wakeUp();
      stopWandering();
      dismissJoke();
      // Retorna suavemente para a posição base ao lado do prompt
      if (currentXPercent !== BASE_X || currentYPercent !== BASE_Y) {
        currentXPercent = BASE_X;
        currentYPercent = BASE_Y;
        direction = 1;
        pitchDeg = 0;
        moveDurationSec = 1.8;
      }
    } else if (wander && isPatrolAllowed && !wanderTimer && !isFlying && !isSleeping) {
      scheduleNextFlight(3000);
    }
  }

  // --- Gerenciador de Inatividade, Voo e Sono ---
  function resetInactivityTimer() {
    // Se o usuário interagiu, pousa o droid na base
    if (isFlying || isPatrolAllowed || isSleeping) {
      isPatrolAllowed = false;
      stopWandering();
      dismissJoke();
      if (isSleeping) wakeUp();
      currentXPercent = BASE_X;
      currentYPercent = BASE_Y;
      direction = 1;
      pitchDeg = 0;
      moveDurationSec = 1.8;
    }

    // Timer 1: Inicia voo e patrulha apenas após 2 minutos de inatividade
    if (patrolInactivityTimer) clearTimeout(patrolInactivityTimer);
    patrolInactivityTimer = window.setTimeout(() => {
      if (state === 'idle' && !isSleeping) {
        isPatrolAllowed = true;
        scheduleNextFlight(1000);
      }
    }, PATROL_TIMEOUT_MS);

    // Timer 2: Modo sono profundo após 5 minutos de inatividade
    if (inactivityTimer) clearTimeout(inactivityTimer);
    inactivityTimer = window.setTimeout(() => {
      enterSleepMode();
    }, inactivityTimeoutMs);
  }

  function enterSleepMode() {
    if (state !== 'idle' || isSleeping) return;
    isSleeping = true;
    stopWandering();
    dismissJoke();

    // Desce suavemente até a base de pouso
    currentXPercent = BASE_X;
    currentYPercent = BASE_Y;
    direction = 1;
    pitchDeg = 0;
    moveDurationSec = 2.8;

    // Simulação do carregamento da bateria enquanto dorme
    batteryLevel = 45;
    if (batteryInterval) clearInterval(batteryInterval);
    batteryInterval = window.setInterval(() => {
      if (batteryLevel < 100) batteryLevel += 1;
    }, 1800);
  }

  function wakeUp() {
    if (!isSleeping) return;
    isSleeping = false;
    if (batteryInterval) {
      clearInterval(batteryInterval);
      batteryInterval = null;
    }
    // Efeito de despertar
    scheduleNextFlight(3000);
  }

  // --- Sistema de Voo 2D Autônomo ---
  function scheduleNextFlight(delayMs: number = 3000) {
    if (!wander || activeState !== 'idle' || isSleeping) return;
    if (wanderTimer) clearTimeout(wanderTimer);

    wanderTimer = window.setTimeout(() => {
      startFlightStep();
    }, delayMs);
  }

  function startFlightStep() {
    if (!wander || activeState !== 'idle' || isSleeping) return;

    // Sorteia novo destino 2D (X entre 5% e 74%, Y entre 18% e 76%)
    const minX = 5;
    const maxX = 74;
    const minY = 18; // Limite superior
    const maxY = 76; // Limite inferior

    let nextX = minX + Math.random() * (maxX - minX);
    let nextY = minY + Math.random() * (maxY - minY);

    const dx = nextX - currentXPercent;
    const dy = nextY - currentYPercent;
    const distance = Math.sqrt(dx * dx + dy * dy);

    if (distance < 18) {
      nextX = currentXPercent > 40 ? minX + 8 : maxX - 8;
      nextY = currentYPercent > 45 ? minY + 6 : maxY - 6;
    }

    const finalDx = nextX - currentXPercent;
    const finalDy = nextY - currentYPercent;
    const finalDist = Math.sqrt(finalDx * finalDx + finalDy * finalDy);

    moveDurationSec = Math.max(2.4, Math.min(6.5, finalDist * 0.12));
    direction = finalDx >= 0 ? 1 : -1;
    pitchDeg = finalDy < -5 ? -6 : finalDy > 5 ? 6 : 0;
    isFlying = true;
    currentXPercent = nextX;
    currentYPercent = nextY;

    wanderTimer = window.setTimeout(() => {
      isFlying = false;
      pitchDeg = 0;

      // 55% de chance de soltar uma piada geek ao pousar/pairar
      if (Math.random() < 0.6 && !currentJoke && !isSleeping) {
        triggerGeekJoke();
      }

      const pauseDuration = 4000 + Math.random() * 5000;
      scheduleNextFlight(pauseDuration);
    }, moveDurationSec * 1000);
  }

  function triggerGeekJoke() {
    if (activeState !== 'idle' || isSleeping) return;
    const joke = GEEK_JOKES[Math.floor(Math.random() * GEEK_JOKES.length)];
    currentJoke = joke;

    if (jokeTimer) clearTimeout(jokeTimer);
    // Duração ajustada: 8.5 segundos (+3s como solicitado pelo Arquiteto)
    jokeTimer = window.setTimeout(() => {
      dismissJoke();
    }, 8500);
  }

  function dismissJoke() {
    currentJoke = null;
    if (jokeTimer) {
      clearTimeout(jokeTimer);
      jokeTimer = null;
    }
  }

  function stopWandering() {
    if (wanderTimer) {
      clearTimeout(wanderTimer);
      wanderTimer = null;
    }
    isFlying = false;
    pitchDeg = 0;
  }

  function handleClick() {
    if (!interactive) return;
    resetInactivityTimer();
    dismissJoke();
    localOverrideState = 'celebrate';
    dispatch('click');
    if (clickReactionTimer) clearTimeout(clickReactionTimer);
    clickReactionTimer = window.setTimeout(() => {
      localOverrideState = null;
    }, 2800);
  }

  // Permite disparar o modo sono programaticamente para testes
  export function triggerSleepNow() {
    enterSleepMode();
  }

  onMount(() => {
    // Monitora interações globais para resetar inatividade
    const handleActivity = () => resetInactivityTimer();
    window.addEventListener('mousemove', handleActivity, { passive: true });
    window.addEventListener('keydown', handleActivity, { passive: true });
    window.addEventListener('click', handleActivity, { passive: true });
    const handleForceSleep = () => enterSleepMode();
    const handleForceJoke = () => triggerGeekJoke();
    window.addEventListener('test-force-sleep', handleForceSleep);
    window.addEventListener('test-force-joke', handleForceJoke);

    resetInactivityTimer();

    return () => {
      window.removeEventListener('mousemove', handleActivity);
      window.removeEventListener('keydown', handleActivity);
      window.removeEventListener('click', handleActivity);
      window.removeEventListener('test-force-sleep', handleForceSleep);
      window.removeEventListener('test-force-joke', handleForceJoke);
      if (patrolInactivityTimer) clearTimeout(patrolInactivityTimer);
      if (wanderTimer) clearTimeout(wanderTimer);
      if (jokeTimer) clearTimeout(jokeTimer);
      if (inactivityTimer) clearTimeout(inactivityTimer);
      if (batteryInterval) clearInterval(batteryInterval);
      if (clickReactionTimer) clearTimeout(clickReactionTimer);
    };
  });
</script>

<!-- Contêiner do Mascote com Posicionamento 2D (X e Y) e Transição Suave de Voo -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="absolute flex items-end gap-2 select-none {interactive ? 'cursor-pointer' : ''} group z-40 pointer-events-auto"
  style="left: {currentXPercent}%; top: {currentYPercent}%; transition: left {moveDurationSec}s cubic-bezier(0.35, 0, 0.25, 1), top {moveDurationSec}s cubic-bezier(0.35, 0, 0.25, 1);"
  on:mouseenter={() => (isHovered = true)}
  on:mouseleave={() => (isHovered = false)}
  on:click={handleClick}
  title="{label} - Droid Companheiro ({isSleeping ? 'Dormindo / Recarregando' : 'Clique para interagir!'})"
>
  <!-- Balão de Fala com Piada Geek (Posicionado BEM ACIMA da cabeça, sem cobrir o robô) -->
  {#if currentJoke && activeState === 'idle' && !isSleeping}
    <div
      class="absolute bottom-[125px] left-0 sm:-left-6 z-50 min-w-[240px] max-w-[320px] sm:max-w-[360px] p-3 rounded-2xl bg-[#070e1b]/98 border-2 border-cyan-400 text-cyan-200 font-pixel-terminal text-base shadow-[0_0_25px_rgba(0,240,255,0.45)] animate-in fade-in slide-in-from-bottom-2 duration-300 pointer-events-auto"
    >
      <div class="relative">
        <div class="flex items-center justify-between text-[9px] font-pixel-title text-cyan-400 font-bold uppercase tracking-wider mb-1">
          <span class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping"></span>
            <span>💬 {label} TRANSMITINDO:</span>
          </span>
          <span class="text-cyan-500/80 font-mono text-[8px]">8.5s</span>
        </div>

        <div class="leading-snug text-white font-mono text-xs sm:text-[13px] tracking-wide py-0.5">
          "{currentJoke}"
        </div>

        <!-- Rabicho do Balão apontando diretamente para a antena/cabeça do robô -->
        <div class="absolute -bottom-[19px] left-10 w-0 h-0 border-l-[7px] border-l-transparent border-r-[7px] border-r-transparent border-t-[10px] border-t-cyan-400"></div>
        <div class="absolute -bottom-[17px] left-[41px] w-0 h-0 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[8px] border-t-[#070e1b]"></div>
      </div>
    </div>
  {/if}

  <!-- Modo de Sono: Animação Z z z Flutuante e Indicador de Recarga -->
  {#if isSleeping}
    <div class="absolute -top-12 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center pointer-events-none">
      <!-- Partículas Zzz Subindo -->
      <div class="relative h-10 w-16 overflow-visible font-pixel-title text-fuchsia-400 font-black">
        <span class="absolute text-xs animate-zzz-1">Z</span>
        <span class="absolute text-sm animate-zzz-2">z</span>
        <span class="absolute text-base animate-zzz-3">z</span>
      </div>

      <!-- Badge de Bateria Recarregando -->
      <div class="px-2 py-0.5 rounded-full bg-[#160e29]/90 border border-fuchsia-500/60 text-[9px] text-fuchsia-300 font-mono font-bold flex items-center gap-1 shadow-[0_0_12px_rgba(217,70,239,0.4)] whitespace-nowrap">
        <span>🔋 RECARGA:</span>
        <span class="text-cyan-300 font-bold">{batteryLevel}%</span>
        <span class="animate-pulse text-amber-300">⚡</span>
      </div>
    </div>
  {/if}

  <!-- Frame do Sprite com Animação Steps(6) e Orientação Direcional -->
  <div
    class="relative {isFlying ? 'spark-flying' : isSleeping ? 'spark-sleeping' : 'spark-float'}"
    style="transform: rotate({pitchDeg}deg); transition: transform 0.4s ease-out;"
  >
    <!-- Sprite do Robô (inverte apenas o sprite quando voa para a esquerda) -->
    <div
      class="w-[100px] h-[110px] spark-sprite spark-{isSleeping ? 'idle' : activeState} drop-shadow-[0_0_15px_rgba(6,182,212,0.6)] {isSleeping ? 'opacity-80 saturate-75' : ''}"
      style="image-rendering: pixelated; transform: scaleX({direction});"
    ></div>

    <!-- Efeito de propulsão a jato / néon nos pés durante o voo -->
    {#if isFlying && !isSleeping}
      <div class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-9 h-2.5 bg-gradient-to-r from-cyan-400 via-blue-500 to-fuchsia-400 rounded-full blur-[3px] animate-pulse"></div>
      <div class="absolute -bottom-3 left-1/2 -translate-x-1/2 w-4 h-4 bg-cyan-300/60 rounded-full blur-[4px] animate-ping"></div>
    {/if}

    <!-- Indicador de estado flutuante -->
    {#if activeState === 'thinking'}
      <div class="absolute -top-3 left-1/2 -translate-x-1/2 px-2 py-0.5 rounded-full bg-cyan-950/90 border border-cyan-400 text-[10px] text-cyan-300 font-mono font-bold animate-pulse shadow-[0_0_10px_#06b6d4] whitespace-nowrap">
        ⚙️ PENSANDO...
      </div>
    {:else if activeState === 'speaking'}
      <div class="absolute -top-3 left-1/2 -translate-x-1/2 px-2 py-0.5 rounded-full bg-fuchsia-950/90 border border-fuchsia-400 text-[10px] text-fuchsia-300 font-mono font-bold animate-pulse shadow-[0_0_10px_#d946ef] whitespace-nowrap">
        🔊 TRANSMITINDO...
      </div>
    {:else if activeState === 'celebrate'}
      <div class="absolute -top-3 left-1/2 -translate-x-1/2 px-2 py-0.5 rounded-full bg-emerald-950/90 border border-emerald-400 text-[10px] text-emerald-300 font-mono font-bold animate-bounce shadow-[0_0_10px_#10b981] whitespace-nowrap">
        ✨ 100% SUCESSO!
      </div>
    {:else if isFlying && !isSleeping}
      <div class="absolute -top-2 left-1/2 -translate-x-1/2 px-1.5 py-0.2 rounded bg-slate-900/80 border border-cyan-500/30 text-[8px] text-cyan-400/90 font-mono tracking-wider opacity-80 whitespace-nowrap">
        VOANDO...
      </div>
    {/if}
  </div>

  <!-- Tag de Nome / Status do Robô (Mantém orientação legível) -->
  <div class="mb-4 flex flex-col shrink-0">
    <div class="flex items-center gap-1.5 px-2 py-0.5 rounded bg-slate-900/90 border border-cyan-500/40 text-cyan-300 font-mono font-black text-xs tracking-wider shadow-[0_0_10px_rgba(6,182,212,0.4)]">
      <span class="w-1.5 h-1.5 rounded-full {isSleeping ? 'bg-fuchsia-400 animate-pulse' : isFlying ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400 animate-ping'}"></span>
      <span>{label}</span>
    </div>
    <span class="text-[9px] text-slate-400 font-mono mt-0.5 pl-0.5">
      {#if isSleeping}
        STANDBY // SONO
      {:else if activeState === 'idle'}
        {isFlying ? 'VOANDO // PATRULHA' : 'ONLINE // PRONTO'}
      {:else if activeState === 'thinking'}
        PROCESSANDO DADOS
      {:else if activeState === 'speaking'}
        FORMA DE ONDA ATIVA
      {:else if activeState === 'celebrate'}
        TAREFA CONCLUÍDA
      {/if}
    </span>
  </div>
</div>

<style>
  /* Flutuação suave em repouso */
  .spark-float {
    animation: spark-hover 3s ease-in-out infinite;
  }

  /* Respiração suave e lenta em modo sono */
  .spark-sleeping {
    animation: spark-sleep-breath 3.5s ease-in-out infinite;
  }

  /* Movimento de voo dinâmico com propulsão */
  .spark-flying {
    animation: spark-fly 0.5s ease-in-out infinite alternate;
  }

  @keyframes spark-hover {
    0%, 100% {
      transform: translateY(0px);
    }
    50% {
      transform: translateY(-8px);
    }
  }

  @keyframes spark-sleep-breath {
    0%, 100% {
      transform: translateY(2px) scale(0.97);
    }
    50% {
      transform: translateY(6px) scale(1.01);
    }
  }

  @keyframes spark-fly {
    0% {
      transform: translateY(0px);
    }
    100% {
      transform: translateY(-6px);
    }
  }

  /* Animações Z z z em Cascata */
  @keyframes zzz-drift-1 {
    0% { transform: translate(0, 8px) scale(0.6); opacity: 0; }
    30% { opacity: 0.9; }
    80% { opacity: 0.6; }
    100% { transform: translate(12px, -18px) scale(1.1); opacity: 0; }
  }

  @keyframes zzz-drift-2 {
    0% { transform: translate(6px, 12px) scale(0.5); opacity: 0; }
    40% { opacity: 0.9; }
    85% { opacity: 0.6; }
    100% { transform: translate(22px, -24px) scale(1.3); opacity: 0; }
  }

  @keyframes zzz-drift-3 {
    0% { transform: translate(14px, 14px) scale(0.6); opacity: 0; }
    50% { opacity: 1; }
    90% { opacity: 0.7; }
    100% { transform: translate(32px, -30px) scale(1.5); opacity: 0; }
  }

  .animate-zzz-1 {
    animation: zzz-drift-1 2.6s ease-in-out infinite;
  }
  .animate-zzz-2 {
    animation: zzz-drift-2 2.6s ease-in-out infinite 0.7s;
  }
  .animate-zzz-3 {
    animation: zzz-drift-3 2.6s ease-in-out infinite 1.4s;
  }

  .spark-sprite {
    background-repeat: no-repeat;
    animation: spark-cycle 0.8s steps(6) infinite;
    transition: transform 0.25s ease-out;
  }

  .spark-idle {
    background-image: url('/assets/spark_idle_strip.png');
  }

  .spark-speaking {
    background-image: url('/assets/spark_speaking_strip.png');
    animation-duration: 0.6s;
  }

  .spark-thinking {
    background-image: url('/assets/spark_thinking_strip.png');
    animation-duration: 1.0s;
  }

  .spark-celebrate {
    background-image: url('/assets/spark_celebrate_strip.png');
    animation-duration: 0.7s;
  }

  @keyframes spark-cycle {
    from {
      background-position: 0px 0px;
    }
    to {
      background-position: -600px 0px;
    }
  }
</style>
