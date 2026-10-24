import Phaser from 'phaser';
import { cartesianToIsometric, type CartesianPoint } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';
import type { AgentDirection } from './spritesheetGenerator';
import { GridPathfinder } from '../pathfinding';

export type AgentRole = 'DEVELOPER' | 'RESEARCHER' | 'DEVOPS' | 'SECURITY' | 'BOT_RUNNER';
export type AgentStatus = 'IDLE' | 'WALKING' | 'WORKING' | 'MEETING' | 'ERROR';

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
 * balões de diálogo e seleção interativa (Modo Diretor).
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

  private readonly walkSpeed = 0.045; // Velocidade de transição entre tiles

  constructor(scene: Phaser.Scene, data: AgentData) {
    this.scene = scene;
    this.data = data;
    this.cartX = data.cartX;
    this.cartY = data.cartY;
    this.status = data.status;

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);

    // 1. Sprite Principal do Personagem
    const initialTexture = `agent_${this.data.role.toLowerCase()}_sw_idle`;
    this.sprite = scene.add.sprite(isoX, isoY - 18, initialTexture);
    this.sprite.setOrigin(0.5, 0.5);
    this.sprite.setInteractive({ useHandCursor: true });

    // 2. Anel de Seleção Néon sob os pés
    this.selectionGraphics = scene.add.graphics();

    // 3. Pacote de Dados Luminoso (Cubo de Dados)
    this.packetGraphics = scene.add.graphics();

    // 4. Label de Nome Flutuante
    this.nameText = scene.add.text(isoX, isoY - 38, this.data.name, {
      fontFamily: 'monospace',
      fontSize: '9px',
      color: this.data.accentColor,
      backgroundColor: '#040810dd',
      padding: { x: 4, y: 1 },
    });
    this.nameText.setOrigin(0.5);

    // 5. Balão de Diálogo e Pensamento (T-4.4)
    this.bubbleContainer = scene.add.container(isoX, isoY - 54);
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
    this.sprite.on('pointerdown', () => {
      this.setSelected(!this.isSelected);
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
    this.selectionGraphics.lineStyle(1.5, Phaser.Display.Color.HexStringToColor(this.data.accentColor).color, 0.9);
    this.selectionGraphics.strokeEllipse(isoX, isoY, 26, 13);
  }

  /**
   * Ordena a caminhada até o destino calculando a rota A* pelas passarelas
   */
  public async walkTo(targetCartX: number, targetCartY: number, pathfinder: GridPathfinder) {
    const start = { x: Math.round(this.cartX), y: Math.round(this.cartY) };
    const path = await pathfinder.findPath(start, { x: targetCartX, y: targetCartY });

    if (path && path.length > 1) {
      this.walkPath = path;
      this.pathIndex = 0;
      this.walkProgress = 0;
      this.setStatus('WALKING');
    }
  }

  public setStatus(newStatus: AgentStatus) {
    this.status = newStatus;
    this.updateFrameTexture();
  }

  /**
   * Exibe um balão de diálogo sincronizado sobre a cabeça do agente
   */
  public say(text: string, durationMs: number = 4000) {
    this.bubbleText.setText(text);
    const bounds = this.bubbleText.getBounds();
    const padX = 6;
    const padY = 4;
    const bw = bounds.width + padX * 2;
    const bh = bounds.height + padY * 2;

    this.bubbleBg.clear();
    // Fundo do balão
    this.bubbleBg.fillStyle(0x00f0ff, 0.95);
    this.bubbleBg.fillRoundedRect(-bw / 2, -bh / 2, bw, bh, 4);

    // Rabicho do balão apontando para o agente
    this.bubbleBg.beginPath();
    this.bubbleBg.moveTo(-3, bh / 2);
    this.bubbleBg.lineTo(0, bh / 2 + 5);
    this.bubbleBg.lineTo(3, bh / 2);
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

    // 2. Animação de digitação em 'WORKING' ou respiração em 'IDLE'
    if (this.status === 'WORKING') {
      this.updateFrameTexture('work');
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
    this.sprite.setPosition(isoX, isoY - 18);
    this.nameText.setPosition(isoX, isoY - 38);
    this.bubbleContainer.setPosition(isoX, isoY - 54);
    if (this.isSelected) {
      this.renderSelectionRing();
    }
    this.renderDataPacket(isoX, isoY);

    // 5. Z-Sorting dinâmico por tile
    this.updateZDepth();
  }

  private renderDataPacket(isoX: number, isoY: number) {
    this.packetGraphics.clear();
    if (!this.isCarryingData) return;

    const offsetDir = this.direction === 'SE' ? 6 : this.direction === 'SW' ? -6 : 0;
    const px = isoX + offsetDir;
    const py = isoY - 14;

    this.packetGraphics.fillStyle(0x00f0ff, 0.95);
    this.packetGraphics.fillRect(px - 3, py - 3, 6, 6);
    this.packetGraphics.lineStyle(1, 0xffffff, 0.9);
    this.packetGraphics.strokeRect(px - 3, py - 3, 6, 6);

    this.packetGraphics.fillStyle(0x00f0ff, 0.4);
    this.packetGraphics.fillCircle(px, py, 6);
  }

  private processWalking(delta: number) {
    if (this.pathIndex >= this.walkPath.length - 1) {
      this.setStatus('IDLE');
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
        this.setStatus('IDLE');
        this.walkPath = [];
        return;
      }
    } else {
      this.cartX = Phaser.Math.Linear(currentPoint.x, nextPoint.x, this.walkProgress);
      this.cartY = Phaser.Math.Linear(currentPoint.y, nextPoint.y, this.walkProgress);
    }

    // Alterna quadros de caminhada
    this.stepTimer += delta;
    if (this.stepTimer > 160) {
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

    const textureKey = `agent_${roleKey}_${dirKey}_${frameKey}`;
    if (this.scene.textures.exists(textureKey)) {
      this.sprite.setTexture(textureKey);
    }
  }

  private updateZDepth() {
    const depth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.ENTITY);
    this.sprite.setDepth(depth);
    this.nameText.setDepth(depth + 2);
    this.selectionGraphics.setDepth(depth - 1);
    this.packetGraphics.setDepth(depth + 3);
    this.bubbleContainer.setDepth(depth + 10);
  }

  public destroy() {
    this.sprite.destroy();
    this.nameText.destroy();
    this.selectionGraphics.destroy();
    this.packetGraphics.destroy();
    this.bubbleContainer.destroy();
  }
}
