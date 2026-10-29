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
  { name: 'The War Room (Nexus)', x: -4, y: -4 },
  { name: 'The Dev Lab (Mesa Principal)', x: -3, y: 7 },
  { name: 'The Dev Lab (Mesa Secundária)', x: -4, y: 5 },
  { name: 'The Dev Lab (Whiteboard)', x: -5, y: 3 },
  { name: 'Cyber Breakroom & Arcades', x: 3, y: -7 },
  { name: 'Máquina de Café', x: 2, y: -9 },
  { name: 'Cyber Sofa Lounge', x: 8, y: -4 },
  { name: 'The Server Vault', x: 4, y: 5 },
  { name: 'Passarela Central', x: 0, y: 0 },
];

const AMBIENT_PHRASES = [
  '☕ Pegando um expresso duplo no Breakroom...',
  '🕹️ Quem topa um round no fliperama retrô?',
  '💻 Analisando o whiteboard de arquitetura...',
  '⚡ Datacenter com refrigeração líquida em 100%!',
  '🔍 Debugando memory leak no Dev Lab...',
  '⚙️ Pipeline CI/CD totalmente verde no cluster!',
  '🍸 Sofá do lounge tá excelente pra pensar!',
  '📦 Sincronizando pacote de dados com o Nexus!',
  '🛡️ Monitorando anomalias no Server Vault...',
  '🌐 Pronto para a próxima instrução do Arquiteto!',
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
        cartX: 0,
        cartY: 0,
        status: 'IDLE',
        accentColor: '#00f0ff',
      },
      {
        id: 'agent-claude-02',
        name: 'Dr. Cipher',
        role: 'RESEARCHER',
        homePodId: 'pod-notex',
        cartX: -4,
        cartY: -4,
        status: 'IDLE',
        accentColor: '#9d00ff',
      },
      {
        id: 'agent-bot-01',
        name: 'Unit-7',
        role: 'BOT_RUNNER',
        homePodId: 'pod-notex',
        cartX: 3,
        cartY: -7,
        status: 'IDLE',
        accentColor: '#39ff14',
      },
      {
        id: 'agent-nova-03',
        name: 'Nova',
        role: 'DEVELOPER',
        homePodId: 'pod-notex',
        cartX: -4,
        cartY: 5,
        status: 'IDLE',
        accentColor: '#ff9900',
      },
      {
        id: 'agent-kaelen-04',
        name: 'Kaelen',
        role: 'RESEARCHER',
        homePodId: 'pod-notex',
        cartX: -5,
        cartY: 3,
        status: 'IDLE',
        accentColor: '#ff007f',
      },
      {
        id: 'agent-sec-05',
        name: 'Vanguard',
        role: 'SECURITY',
        homePodId: 'pod-notex',
        cartX: 4,
        cartY: 5,
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

    // Mensagens contextuais dependendo da sala
    if (cartX <= -4) {
      agent.say('A caminho da bancada no Dev Lab! 💻');
    } else if (cartX >= 4) {
      agent.say('Pausa rápida no Breakroom & Arcades! 🕹️');
    } else if (cartY >= 3) {
      agent.say('Inspecionando os racks no Server Vault! ⚡');
    } else {
      agent.say('Reunindo na mesa holográfica do War Room! 🌐');
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
