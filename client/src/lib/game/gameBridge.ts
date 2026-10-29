import type { AgentEntity } from './agents/AgentEntity';
import type { MainIsometricScene } from './scenes/MainIsometricScene';

export interface GameBridgeEvents {
  onAgentSelect?: (agentId: string) => void;
}

class GameBridge {
  private scene: MainIsometricScene | null = null;
  private listeners: Map<string, Array<(...args: any[]) => void>> = new Map();

  public registerScene(scene: MainIsometricScene) {
    this.scene = scene;
  }

  public getScene(): MainIsometricScene | null {
    return this.scene;
  }

  public on(event: string, callback: (...args: any[]) => void) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  public emit(event: string, ...args: any[]) {
    const list = this.listeners.get(event);
    if (list) {
      for (const cb of list) {
        cb(...args);
      }
    }
  }

  /**
   * Ordena um agente a responder via balão, caminhar pelas passarelas e sentar na cadeira gamer
   * da sua estação de trabalho no Dev Lab / War Room para executar a tarefa (Estilo Habbo / Habblaud).
   */
  public orderAgentInstruction(agentId: string, speech: string) {
    if (!this.scene) {
      console.warn('[gameBridge] Scene not registered!');
      return;
    }

    const manager = (this.scene as any).agentManager;
    if (!manager) {
      console.warn('[gameBridge] agentManager not found!');
      return;
    }

    const agent: AgentEntity = manager.getAgent(agentId);
    if (!agent) {
      console.warn(`[gameBridge] Agent ${agentId} not found!`);
      return;
    }

    // 1. Balão de fala imediato no mundo
    agent.say(speech, 4500);

    // 2. Caminha pelas passarelas até a estação de trabalho e senta na cadeira virado para os monitores
    const pathfinder = (this.scene as any).pathfinder;
    if (pathfinder) {
      console.log(`[gameBridge] Ordering ${agentId} to desk station via pathfinder`);
      if (agentId === 'agent-codex-01') {
        // Rusty: senta na cadeira gamer da bancada principal de monitores do Dev Lab (-4, 11), virado para NW
        agent.workAtDesk(-4, 11, pathfinder, 'NW');
      } else if (agentId === 'agent-nova-03') {
        // Nova: senta na cadeira da bancada secundária do Dev Lab (-5, 8), virado para NW
        agent.workAtDesk(-5, 8, pathfinder, 'NW');
      } else if (agentId === 'agent-claude-02') {
        // Dr. Cipher: opera a mesa holográfica no War Room Nexus (-4, -4), virado para NW
        agent.workAtDesk(-4, -4, pathfinder, 'NW');
      } else if (agentId === 'agent-sec-05') {
        // Vanguard: opera os consoles do Server Vault (4, 5), virado para SW
        agent.workAtDesk(4, 5, pathfinder, 'SW');
      } else if (agentId === 'agent-kaelen-04') {
        // Kaelen: estação do whiteboard no Dev Lab (-6, 4), virado para NW
        agent.workAtDesk(-6, 4, pathfinder, 'NW');
      } else {
        // Unit-7: despacha no Breakroom & Arcades (3, -7), virado para NE
        agent.workAtDesk(3, -7, pathfinder, 'NE');
      }
    } else {
      console.warn('[gameBridge] Pathfinder not found on scene!');
    }
  }

  /**
   * Mecânica de Pipeline Handover (T-5.3):
   * Um agente recebe a missão, caminha até o colega carregando o cubo holográfico de dados,
   * entrega as especificações com balões de conversa sincronizados, e o colega senta na cadeira
   * gamer para executar no Dev Lab!
   */
  public async orderHandoverInstruction(fromAgentId: string, toAgentId: string, taskTitle: string) {
    if (!this.scene) return;

    const manager = (this.scene as any).agentManager;
    const pathfinder = (this.scene as any).pathfinder;
    if (!manager || !pathfinder) return;

    const fromAgent: AgentEntity = manager.getAgent(fromAgentId);
    const toAgent: AgentEntity = manager.getAgent(toAgentId);
    if (!fromAgent || !toAgent) return;

    // 1. Agente de origem recebe a demanda e avisa que vai encaminhar
    fromAgent.say(`Encaminhando demanda "${taskTitle.slice(0, 24)}..." para @${toAgent.data.name}! 📦`, 4000);
    fromAgent.isCarryingData = true;

    // 2. Destino de encontro: próximo ao agente alvo
    const meetX = toAgent.cartX > 0 ? toAgent.cartX - 1 : toAgent.cartX + 1;
    const meetY = toAgent.cartY;

    await fromAgent.walkTo(meetX, meetY, pathfinder);

    // 3. Chegando no local: entrega de dados e diálogo sincronizado
    if (this.scene.time) {
      this.scene.time.delayedCall(1200, () => {
        fromAgent.isCarryingData = false;
        fromAgent.say(`@${toAgent.data.name}, specs da tarefa entregues!`, 3500);

        // Agente de destino reage, recebe os dados e senta na estação
        this.scene!.time.delayedCall(800, () => {
          toAgent.say(`Entendido! Assumindo execução no Dev Lab 💻`, 4000);
          this.orderAgentInstruction(toAgentId, `Compilando diretriz no cluster...`);
        });
      });
    }
  }
}

export const gameBridge = new GameBridge();
if (typeof window !== 'undefined') {
  (window as any).__gridGameBridge = gameBridge;
}
