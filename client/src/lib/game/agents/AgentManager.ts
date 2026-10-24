import Phaser from 'phaser';
import { AgentEntity, type AgentData } from './AgentEntity';
import { DroneEntity } from './DroneEntity';
import { ProceduralSpriteGenerator } from './spritesheetGenerator';
import { GridPathfinder } from '../pathfinding';

/**
 * AgentManager: Orquestrador da população de agentes na metrópole.
 * Gerencia ciclo de vida, despacho, seleção e reações espaciais autônomas.
 */
export class AgentManager {
  private scene: Phaser.Scene;
  private agents: Map<string, AgentEntity> = new Map();
  private selectedAgentId: string | null = null;
  private drone!: DroneEntity;

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
    ProceduralSpriteGenerator.ensureAgentTextures(scene);
    this.spawnDefaultAgents();
    this.drone = new DroneEntity(scene, 4, 0);
  }

  private spawnDefaultAgents() {
    const defaultRoster: AgentData[] = [
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
        cartX: -11,
        cartY: 0,
        status: 'IDLE',
        accentColor: '#9d00ff',
      },
      {
        id: 'agent-bot-01',
        name: 'Unit-7',
        role: 'BOT_RUNNER',
        homePodId: 'pod-notex',
        cartX: 10,
        cartY: 0,
        status: 'WORKING',
        accentColor: '#39ff14',
      },
    ];

    for (const data of defaultRoster) {
      const entity = new AgentEntity(this.scene, data);
      this.agents.set(data.id, entity);
    }

    // Seleciona Rusty inicialmente
    this.selectAgent('agent-codex-01');

    // Fala inicial de boas-vindas
    const rusty = this.agents.get('agent-codex-01');
    if (rusty) {
      this.scene.time.delayedCall(1200, () => {
        rusty.say('The Core online. Pronto para despachar missões!');
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
   * Envia o agente selecionado (ou Rusty) para caminhar até o destino via pathfinding A*
   */
  public async dispatchTo(cartX: number, cartY: number, pathfinder: GridPathfinder) {
    const agent = this.getSelectedAgent();
    if (!agent) return;

    await agent.walkTo(cartX, cartY, pathfinder);

    // Mensagens contextuais dependendo do distrito de destino
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
    for (const agent of this.agents.values()) {
      agent.update(delta);
    }
    if (this.drone) {
      this.drone.update(delta);
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
