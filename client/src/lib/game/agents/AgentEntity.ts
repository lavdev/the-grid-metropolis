import Phaser from 'phaser';
import { cartesianToIsometric, type CartesianPoint } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';
import type { AgentDirection } from './spritesheetGenerator';
import { GridPathfinder } from '../pathfinding';
import { gameBridge } from '../gameBridge';

export type AgentRole = 'DEVELOPER' | 'RESEARCHER' | 'DEVOPS' | 'SECURITY' | 'BOT_RUNNER';
export type AgentStatus = 'IDLE' | 'WALKING' | 'WORKING' | 'MEETING' | 'ERROR';

const WORKING_VISUAL_OFFSET_X = -24;
const WORKING_VISUAL_TRANSITION_MS = 200;

export interface AgentData {
  id: string;
  name: string;
  role: AgentRole;
  homePodId: string;
  cartX: number;
  cartY: number;
  status: AgentStatus;
  accentColor: string;
}

/**
 * AgentEntity: Personagem autônomo com máquina de estados finita (FSM),
 * navegação espacial suave sobre o grid A*, animação de passos direcional em 4 direções,
 * digitação realista sentado na cadeira gamer e balões de diálogo (Estilo Habbo / Habblaud).
 */
export class AgentEntity {
  public scene: Phaser.Scene;
  public data: AgentData;

  public cartX: number;
  public cartY: number;
  public status: AgentStatus;
  public direction: AgentDirection = 'SW';

  private sprite: Phaser.GameObjects.Sprite;
  private nameText: Phaser.GameObjects.Text;
  private selectionGraphics: Phaser.GameObjects.Graphics;
  private packetGraphics: Phaser.GameObjects.Graphics;
  private workSparkGraphics: Phaser.GameObjects.Graphics;
  private bubbleContainer: Phaser.GameObjects.Container;
  private bubbleBg: Phaser.GameObjects.Graphics;
  private bubbleText: Phaser.GameObjects.Text;

  private isSelected = false;
  public isCarryingData = false;
  public autonomousWandering = false;
  private wanderTimer = 0;
  private walkPath: CartesianPoint[] = [];
  private pathIndex = 0;
  private walkProgress = 0;
  private stepTimer = 0;
  private stepFrame = 1;
  private bubbleTimer = 0;
  private workSparkTimer = 0;
  private intendedWorkState = false;
  private workDirection: AgentDirection = 'NW';
  private visualOffsetX: number;
  private visualOffsetFromX: number;
  private visualOffsetTargetX: number;
  private visualOffsetElapsedMs = WORKING_VISUAL_TRANSITION_MS;

  private readonly walkSpeed = 0.075; // Velocidade ágil e suave de caminhada entre tiles

  constructor(scene: Phaser.Scene, data: AgentData) {
    this.scene = scene;
    this.data = data;
    this.cartX = data.cartX;
    this.cartY = data.cartY;
    this.status = data.status;
    this.visualOffsetX = data.status === 'WORKING' ? WORKING_VISUAL_OFFSET_X : 0;
    this.visualOffsetFromX = this.visualOffsetX;
    this.visualOffsetTargetX = this.visualOffsetX;

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);

    // 1. Sprite Principal do Personagem (Escala ajustada para corresponder perfeitamente à mobília)
    const specificTexture = `agent_${this.data.id}_sw_idle`;
    const fallbackTexture = `agent_${this.data.role.toLowerCase()}_sw_idle`;
    const initialTexture = scene.textures.exists(specificTexture) ? specificTexture : fallbackTexture;

    this.sprite = scene.add.sprite(isoX, isoY - 38, initialTexture);
    this.sprite.setScale(1.85);
    this.sprite.setOrigin(0.5, 0.5);
    this.sprite.setInteractive({ useHandCursor: true });

    // 2. Anel de Seleção Néon sob os pés
    this.selectionGraphics = scene.add.graphics();

    // 3. Pacote de Dados Luminoso (Cubo de Dados)
    this.packetGraphics = scene.add.graphics();

    // 4. Faíscas/Partículas de Digitação e Código quando em 'WORKING'
    this.workSparkGraphics = scene.add.graphics();

    // 5. Label de Nome Flutuante
    this.nameText = scene.add.text(isoX, isoY - 68, this.data.name, {
      fontFamily: 'monospace',
      fontSize: '10px',
      color: this.data.accentColor,
      backgroundColor: '#040810ee',
      padding: { x: 5, y: 2 },
    });
    this.nameText.setOrigin(0.5);

    // 6. Balão de Diálogo e Pensamento (Estilo Habbo / Habblaud)
    this.bubbleContainer = scene.add.container(isoX, isoY - 96);
    this.bubbleBg = scene.add.graphics();
    this.bubbleText = scene.add.text(0, 0, '', {
      fontFamily: 'monospace',
      fontSize: '9px',
      color: '#050914',
      align: 'center',
    });
    this.bubbleText.setOrigin(0.5);
    this.bubbleContainer.add([this.bubbleBg, this.bubbleText]);
    this.bubbleContainer.setVisible(false);

    // Atualiza profundidade inicial
    this.updateZDepth();

    // Clique para seleção (Modo Diretor)
    this.sprite.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      if (pointer.rightButtonDown()) return;
      this.setSelected(true);
      gameBridge.emit('agent_clicked', this.data);
    });
  }

  public setSelected(selected: boolean) {
    this.isSelected = selected;
    this.renderSelectionRing();
  }

  private renderSelectionRing() {
    this.selectionGraphics.clear();
    if (!this.isSelected) return;

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    this.selectionGraphics.lineStyle(2, Phaser.Display.Color.HexStringToColor(this.data.accentColor).color, 0.95);
    this.selectionGraphics.strokeEllipse(isoX + this.visualOffsetX, isoY, 44, 22);
  }

  /**
   * Ordena a caminhada até o destino calculando a rota A* pelas passarelas
   */
  public async walkTo(targetCartX: number, targetCartY: number, pathfinder: GridPathfinder): Promise<boolean> {
    const start = { x: Math.round(this.cartX), y: Math.round(this.cartY) };
    const path = await pathfinder.findPath(start, { x: targetCartX, y: targetCartY });
    console.log(`[AgentEntity ${this.data.id}] walkTo from (${start.x}, ${start.y}) to (${targetCartX}, ${targetCartY}) - path:`, path ? path.length : 'null');

    if (path && path.length > 1) {
      this.walkPath = path;
      this.pathIndex = 0;
      this.walkProgress = 0;
      this.setStatus('WALKING');
      return true;
    } else if (path && path.length === 1) {
      this.cartX = targetCartX;
      this.cartY = targetCartY;
      return true;
    }
    return false;
  }

  /**
   * Caminha até uma estação de trabalho e senta na cadeira gamer para executar tarefas
   */
  public async workAtDesk(targetCartX: number, targetCartY: number, pathfinder: GridPathfinder, faceDir: AgentDirection = 'NW') {
    this.intendedWorkState = true;
    this.workDirection = faceDir;

    const success = await this.walkTo(targetCartX, targetCartY, pathfinder);
    if (!success) {
      // Se já estava no local ou rota muito próxima, alinha direto
      this.cartX = targetCartX;
      this.cartY = targetCartY;
    }

    if (this.walkPath.length <= 1) {
      this.direction = faceDir;
      this.setStatus('WORKING');
      this.intendedWorkState = false;
    }
  }

  public setStatus(newStatus: AgentStatus) {
    if (this.status !== newStatus) {
      this.visualOffsetFromX = this.visualOffsetX;
      this.visualOffsetTargetX = newStatus === 'WORKING' ? WORKING_VISUAL_OFFSET_X : 0;
      this.visualOffsetElapsedMs = 0;
    }

    this.status = newStatus;
    this.updateFrameTexture();
    if (newStatus !== 'WORKING') {
      this.workSparkGraphics.clear();
    }
  }

  /**
   * Exibe um balão de diálogo sincronizado sobre a cabeça do agente
   */
  public say(text: string, durationMs: number = 4500) {
    this.bubbleText.setText(text);
    const bounds = this.bubbleText.getBounds();
    const padX = 8;
    const padY = 5;
    const bw = Math.max(bounds.width + padX * 2, 40);
    const bh = bounds.height + padY * 2;

    this.bubbleBg.clear();
    // Fundo do balão com borda brilhante
    this.bubbleBg.fillStyle(0x00f0ff, 0.95);
    this.bubbleBg.fillRoundedRect(-bw / 2, -bh / 2, bw, bh, 5);

    // Rabicho do balão apontando para a cabeça do agente
    this.bubbleBg.beginPath();
    this.bubbleBg.moveTo(-4, bh / 2);
    this.bubbleBg.lineTo(0, bh / 2 + 6);
    this.bubbleBg.lineTo(4, bh / 2);
    this.bubbleBg.closePath();
    this.bubbleBg.fillPath();

    this.bubbleContainer.setVisible(true);
    this.bubbleTimer = durationMs;
  }

  /**
   * Atualização contínua de movimento, animação e profundidade
   */
  public update(delta: number) {
    // 1. Atualização da caminhada pelo pathfinding
    if (this.status === 'WALKING' && this.walkPath.length > 1) {
      this.processWalking(delta);
    }
    this.updateVisualOffset(delta);

    // 2. Animação de digitação em 'WORKING' e partículas de código
    if (this.status === 'WORKING') {
      this.updateFrameTexture('work');
      this.renderWorkSparks(delta);
    }

    // 3. Contador do balão de fala
    if (this.bubbleTimer > 0) {
      this.bubbleTimer -= delta;
      if (this.bubbleTimer <= 0) {
        this.bubbleContainer.setVisible(false);
      }
    }

    // 4. Posiciona elementos visuais
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const visualX = isoX + this.visualOffsetX;
    this.sprite.setPosition(visualX, isoY - 38);
    this.nameText.setPosition(visualX, isoY - 68);
    this.bubbleContainer.setPosition(visualX, isoY - 96);
    if (this.isSelected) {
      this.renderSelectionRing();
    }
    this.renderDataPacket(visualX, isoY);

    // 5. Z-Sorting dinâmico por tile
    this.updateZDepth();
  }

  private updateVisualOffset(delta: number) {
    if (this.visualOffsetX === this.visualOffsetTargetX) return;

    this.visualOffsetElapsedMs = Math.min(
      this.visualOffsetElapsedMs + delta,
      WORKING_VISUAL_TRANSITION_MS,
    );
    const progress = this.visualOffsetElapsedMs / WORKING_VISUAL_TRANSITION_MS;
    const easedProgress = progress * progress * (3 - 2 * progress);
    this.visualOffsetX = Phaser.Math.Linear(
      this.visualOffsetFromX,
      this.visualOffsetTargetX,
      easedProgress,
    );
  }

  private renderWorkSparks(delta: number) {
    this.workSparkTimer += delta;
    if (this.workSparkTimer < 150) return;
    this.workSparkTimer = 0;

    this.workSparkGraphics.clear();
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const visualX = isoX + this.visualOffsetX;

    // Partículas de código verde e ciano saindo do teclado
    const colors = [0x00f0ff, 0x39ff14, 0xff007f];
    for (let i = 0; i < 3; i++) {
      const px = visualX + Phaser.Math.Between(-12, 12);
      const py = isoY - 38 + Phaser.Math.Between(-8, 8);
      const col = colors[i % colors.length];
      this.workSparkGraphics.fillStyle(col, 0.8);
      this.workSparkGraphics.fillCircle(px, py, Phaser.Math.Between(1, 2));
    }
  }

  private renderDataPacket(isoX: number, isoY: number) {
    this.packetGraphics.clear();
    if (!this.isCarryingData) return;

    const offsetDir = this.direction === 'SE' ? 10 : this.direction === 'SW' ? -10 : 0;
    const px = isoX + offsetDir;
    const py = isoY - 26;

    this.packetGraphics.fillStyle(0x00f0ff, 0.95);
    this.packetGraphics.fillRect(px - 4, py - 4, 8, 8);
    this.packetGraphics.lineStyle(1, 0xffffff, 0.9);
    this.packetGraphics.strokeRect(px - 4, py - 4, 8, 8);

    this.packetGraphics.fillStyle(0x00f0ff, 0.4);
    this.packetGraphics.fillCircle(px, py, 8);
  }

  private processWalking(delta: number) {
    if (this.pathIndex >= this.walkPath.length - 1) {
      if (this.intendedWorkState) {
        this.direction = this.workDirection;
        this.setStatus('WORKING');
        this.intendedWorkState = false;
      } else {
        this.setStatus('IDLE');
      }
      this.walkPath = [];
      return;
    }

    const currentPoint = this.walkPath[this.pathIndex];
    const nextPoint = this.walkPath[this.pathIndex + 1];

    // Calcula direção cardeal
    const dx = nextPoint.x - currentPoint.x;
    const dy = nextPoint.y - currentPoint.y;

    if (dx > 0) this.direction = 'SE';
    else if (dx < 0) this.direction = 'NW';
    else if (dy > 0) this.direction = 'SW';
    else if (dy < 0) this.direction = 'NE';

    // Interpolação suave
    this.walkProgress += this.walkSpeed * (delta * 0.06);

    if (this.walkProgress >= 1) {
      this.walkProgress = 0;
      this.pathIndex++;
      this.cartX = nextPoint.x;
      this.cartY = nextPoint.y;

      if (this.pathIndex >= this.walkPath.length - 1) {
        if (this.intendedWorkState) {
          this.direction = this.workDirection;
          this.setStatus('WORKING');
          this.intendedWorkState = false;
        } else {
          this.setStatus('IDLE');
        }
        this.walkPath = [];
        return;
      }
    } else {
      this.cartX = Phaser.Math.Linear(currentPoint.x, nextPoint.x, this.walkProgress);
      this.cartY = Phaser.Math.Linear(currentPoint.y, nextPoint.y, this.walkProgress);
    }

    // Alterna quadros de caminhada
    this.stepTimer += delta;
    if (this.stepTimer > 150) {
      this.stepTimer = 0;
      this.stepFrame = this.stepFrame === 1 ? 2 : 1;
      this.updateFrameTexture(`walk_${this.stepFrame}`);
    }
  }

  private updateFrameTexture(customFrame?: string) {
    const roleKey = this.data.role.toLowerCase();
    const dirKey = this.direction.toLowerCase();
    let frameKey = 'idle';

    if (customFrame) {
      frameKey = customFrame;
    } else if (this.status === 'WALKING') {
      frameKey = `walk_${this.stepFrame}`;
    } else if (this.status === 'WORKING') {
      frameKey = 'work';
    }

    const specificKey = `agent_${this.data.id}_${dirKey}_${frameKey}`;
    const fallbackKey = `agent_${roleKey}_${dirKey}_${frameKey}`;

    if (this.scene.textures.exists(specificKey)) {
      this.sprite.setTexture(specificKey);
    } else if (this.scene.textures.exists(fallbackKey)) {
      this.sprite.setTexture(fallbackKey);
    }
  }

  private updateZDepth() {
    const depth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.ENTITY);
    this.sprite.setDepth(depth);
    this.nameText.setDepth(depth + 2);
    this.selectionGraphics.setDepth(depth - 1);
    this.packetGraphics.setDepth(depth + 3);
    this.workSparkGraphics.setDepth(depth + 4);
    this.bubbleContainer.setDepth(depth + 10);
  }

  public destroy() {
    this.sprite.destroy();
    this.nameText.destroy();
    this.selectionGraphics.destroy();
    this.packetGraphics.destroy();
    this.workSparkGraphics.destroy();
    this.bubbleContainer.destroy();
  }
}
