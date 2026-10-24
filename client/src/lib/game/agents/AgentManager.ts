import Phaser from 'phaser';
import { AgentEntity, type AgentData } from './AgentEntity';
import { DroneEntity } from './DroneEntity';
import { ProceduralSpriteGenerator } from './spritesheetGenerator';
import { GridPathfinder } from '../pathfinding';

interface WaypointHub {
  name: string;
  x: number;
  y: number;
}

const HUBS: WaypointHub[] = [
  { name: 'The Nexus', x: 0, y: 0 },
  { name: 'The Battery Lounge', x: -11, y: 0 },
  { name: 'Pod Notex Core', x: 11, y: 0 },
  { name: 'Glitch Alleys', x: 0, y: -10 },
  { name: 'Expansion Bay', x: 0, y: 10 },
  { name: 'Passarela Leste', x: 6, y: 0 },
  { name: 'Passarela Oeste', x: -6, y: 0 },
];

const AMBIENT_PHRASES = [
  '☕ Partiu tomar algo no Battery Lounge!',
  '⚡ Bateria em 100%!',
  '💻 Compilando kernel Rust assíncrono...',
  '⚙️ Pipeline CI/CD totalmente verde!',
  '😊 Salve! O Grid está a todo vapor hoje!',
  '📦 Entregando pacote de dados ao Nexus!',
  '⚠️ Monitorando anomalias na rede...',
  '🍸 Experimentando o novo drink sintético!',
  '🔍 Auditando contratos de execução...',
  '🌐 Sincronizado com o Bridge Hub!',
];

/**
 * AgentManager: Orquestrador da população viva de agentes na metrópole.
 * Gerencia a deambulação autônoma, conversas reativas, transporte de dados
 * e seleção no Modo Diretor.
 */
export class AgentManager {
  private scene: Phaser.Scene;
  private pathfinder!: GridPathfinder;
  private agents: Map<string, AgentEntity> = new Map();
  private selectedAgentId: string | null = null;
  private drone!: DroneEntity;
  private ambientTimer = 0;

  constructor(scene: Phaser.Scene, pathfinder?: GridPathfinder) {
    this.scene = scene;
    if (pathfinder) {
      this.pathfinder = pathfinder;
    }
    ProceduralSpriteGenerator.ensureAgentTextures(scene);
    this.spawnLivingPopulation();
    this.drone = new DroneEntity(scene, 4, 0);
  }

  public setPathfinder(pathfinder: GridPathfinder) {
    this.pathfinder = pathfinder;
  }

  private spawnLivingPopulation() {
    const populationRoster: AgentData[] = [
      {
        id: 'agent-codex-01',
        name: 'Rusty',
        role: 'DEVELOPER',
        homePodId: 'pod-notex',
        cartX: 1,
        cartY: 1,
        status: 'IDLE',
        accentColor: '#00f0ff',
      },
      {
        id: 'agent-claude-02',
        name: 'Dr. Cipher',
        role: 'RESEARCHER',
        homePodId: 'pod-notex',
        cartX: -6,
        cartY: 0,
        status: 'IDLE',
        accentColor: '#9d00ff',
      },
      {
        id: 'agent-bot-01',
        name: 'Unit-7',
        role: 'BOT_RUNNER',
        homePodId: 'pod-notex',
        cartX: 8,
        cartY: 0,
        status: 'IDLE',
        accentColor: '#39ff14',
      },
      {
        id: 'agent-nova-03',
        name: 'Nova',
        role: 'DEVELOPER',
        homePodId: 'pod-notex',
        cartX: 0,
        cartY: 7,
        status: 'IDLE',
        accentColor: '#ff9900',
      },
      {
        id: 'agent-kaelen-04',
        name: 'Kaelen',
        role: 'RESEARCHER',
        homePodId: 'pod-notex',
        cartX: -11,
        cartY: 0,
        status: 'IDLE',
        accentColor: '#ff007f',
      },
      {
        id: 'agent-sec-05',
        name: 'Vanguard',
        role: 'SECURITY',
        homePodId: 'pod-notex',
        cartX: 0,
        cartY: -8,
        status: 'IDLE',
        accentColor: '#ff3b30',
      },
    ];

    for (const data of populationRoster) {
      const entity = new AgentEntity(this.scene, data);
      entity.autonomousWandering = true;
      this.agents.set(data.id, entity);
    }

    // Configura Unit-7 para carregar um pacote de dados luminoso
    const unit7 = this.agents.get('agent-bot-01');
    if (unit7) {
      unit7.isCarryingData = true;
    }

    // Seleciona Rusty inicialmente
    this.selectAgent('agent-codex-01');

    // Fala inicial de Rusty
    const rusty = this.agents.get('agent-codex-01');
    if (rusty) {
      this.scene.time.delayedCall(1200, () => {
        rusty.say('The Core online. População de agentes ativa!');
      });
    }
  }

  public selectAgent(agentId: string) {
    for (const [id, agent] of this.agents.entries()) {
      agent.setSelected(id === agentId);
    }
    this.selectedAgentId = agentId;
  }

  public getSelectedAgent(): AgentEntity | undefined {
    if (this.selectedAgentId) {
      return this.agents.get(this.selectedAgentId);
    }
    return this.agents.get('agent-codex-01');
  }

  public getAgent(agentId: string): AgentEntity | undefined {
    return this.agents.get(agentId);
  }

  public getAllAgents(): AgentEntity[] {
    return Array.from(this.agents.values());
  }

  /**
   * Envia o agente selecionado para caminhar até o destino via pathfinding A*
   */
  public async dispatchTo(cartX: number, cartY: number, pathfinder: GridPathfinder) {
    const agent = this.getSelectedAgent();
    if (!agent) return;

    await agent.walkTo(cartX, cartY, pathfinder);

    // Mensagens contextuais dependendo do distrito
    if (Math.hypot(cartX - (-12), cartY - 0) <= 2) {
      agent.say('Indo relaxar no Battery Lounge... 🍸');
    } else if (Math.hypot(cartX - 12, cartY - 0) <= 2) {
      agent.say('A caminho do Pod Notex Core! ⚙️');
    } else if (Math.hypot(cartX - 0, cartY - (-12)) <= 2) {
      agent.say('Setor de Quarentena: depurando erros! ⚠️');
    } else if (Math.hypot(cartX - 0, cartY - 0) <= 2) {
      agent.say('Retornando ao Nexus para sincronizar! 🌐');
    }
  }

  public update(delta: number) {
    // 1. Atualiza cada agente
    for (const agent of this.agents.values()) {
      agent.update(delta);
    }

    // 2. Atualiza drone aéreo
    if (this.drone) {
      this.drone.update(delta);
    }

    // 3. Sistema de vida autônoma e deambulação ambiente
    this.processAmbientLife(delta);
  }

  private processAmbientLife(delta: number) {
    if (!this.pathfinder) return;

    this.ambientTimer += delta;
    if (this.ambientTimer > 5000) {
      this.ambientTimer = 0;

      // Escolhe um agente aleatório (que não esteja selecionado pelo jogador) para deambular
      const agentList = Array.from(this.agents.values()).filter(
        (a) => a.data.id !== this.selectedAgentId && a.status === 'IDLE'
      );

      if (agentList.length > 0) {
        const luckyAgent = agentList[Phaser.Math.Between(0, agentList.length - 1)];
        const randomHub = HUBS[Phaser.Math.Between(0, HUBS.length - 1)];

        luckyAgent.walkTo(randomHub.x, randomHub.y, this.pathfinder);

        // Ocasionalmente diz algo enquanto caminha
        if (Math.random() > 0.5) {
          const phrase = AMBIENT_PHRASES[Phaser.Math.Between(0, AMBIENT_PHRASES.length - 1)];
          luckyAgent.say(phrase, 3500);
        }
      }
    }
  }

  public destroy() {
    for (const agent of this.agents.values()) {
      agent.destroy();
    }
    this.agents.clear();
    if (this.drone) {
      this.drone.destroy();
    }
  }
}
