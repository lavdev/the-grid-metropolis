import Phaser from 'phaser';
import { cartesianToIsometric, isometricToCartesian } from '../isometric';
import { GridPathfinder } from '../pathfinding';
import { DataOcean } from '../dataOcean';
import { LivingWorldOverlay } from '../world/LivingWorldOverlay';
import { AgentManager } from '../agents/AgentManager';
import { DepthLayer, calculateIsometricDepth } from '../depth';
import { gameBridge } from '../gameBridge';

export class MainIsometricScene extends Phaser.Scene {
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private cameraStartX = 0;
  private cameraStartY = 0;

  private pathfinder!: GridPathfinder;
  private dataOcean!: DataOcean;
  private cityMasterImage!: Phaser.GameObjects.Image;
  private livingOverlay!: LivingWorldOverlay;
  private agentManager!: AgentManager;
  private pathGraphics!: Phaser.GameObjects.Graphics;
  private markerGraphics!: Phaser.GameObjects.Graphics;

  constructor() {
    super({ key: 'MainIsometricScene' });
  }

  preload() {
    // Carrega a arte conceitual mestra oficial de alta fidelidade
    this.load.image('city_master', 'assets/the_grid_metropolis_master.jpg');
  }

  create() {
    this.pathfinder = new GridPathfinder();

    // Centraliza a câmera no Nexus com zoom imersivo
    this.cameras.main.centerOn(0, 0);
    this.cameras.main.setZoom(1.15);

    // 1. Data Ocean no fundo abissal (-100000 depth) com partículas animadas
    this.dataOcean = new DataOcean(this);

    // 2. Base Arquitetônica Mestra em Alta Fidelidade (Arte Oficial da Metrópole)
    this.cityMasterImage = this.add.image(0, 0, 'city_master');
    this.cityMasterImage.setOrigin(0.5, 0.5);
    this.cityMasterImage.setDepth(0); // Acima do mar abissal, abaixo dos efeitos dinâmicos

    // 3. Camada de Cenografia Viva (Fumaça nas chaminés, holograma giratório, telas e cabos)
    this.livingOverlay = new LivingWorldOverlay(this);

    // 4. População de Agentes Autônomos e Drones Aéreos (Fase 4)
    this.agentManager = new AgentManager(this, this.pathfinder);
    gameBridge.registerScene(this);

    // 5. Camadas gráficas de trajetórias e seleções
    this.pathGraphics = this.add.graphics();
    this.pathGraphics.setDepth(DepthLayer.PATH_OVERLAY);

    this.markerGraphics = this.add.graphics();
    this.markerGraphics.setDepth(DepthLayer.SELECTION_MARKER);

    // 6. Controles interativos de câmera e cliques no mapa
    this.setupCameraControls();
    this.setupMapInteractivity();

    // Redimensionamento responsivo automático
    this.scale.on('resize', this.handleResize, this);
  }

  override update(_time: number, delta: number) {
    // Atualiza correntes do Data Ocean
    if (this.dataOcean) {
      this.dataOcean.update(delta);
    }

    // Atualiza vida da cidade (fumaça, hologramas, luzes)
    if (this.livingOverlay) {
      this.livingOverlay.update(delta);
    }

    // Atualiza agentes e drones
    if (this.agentManager) {
      this.agentManager.update(delta);
    }
  }

  private setupMapInteractivity() {
    this.input.on('pointerdown', async (pointer: Phaser.Input.Pointer) => {
      if (pointer.rightButtonDown() || pointer.getDistance() > 10) return;

      const worldPoint = this.cameras.main.getWorldPoint(pointer.x, pointer.y);
      const cartTarget = isometricToCartesian(worldPoint.x, worldPoint.y);

      // Clicar no Nexus (centro) dispara o pulso holográfico de despacho
      if (Math.hypot(worldPoint.x - 3, worldPoint.y - 2) <= 45) {
        this.livingOverlay.triggerNexusPulse();
      }

      if (this.pathfinder.isWalkable(cartTarget.x, cartTarget.y)) {
        this.drawTargetMarker(cartTarget.x, cartTarget.y);

        const activeAgent = this.agentManager.getSelectedAgent();
        const startPoint = activeAgent
          ? { x: Math.round(activeAgent.cartX), y: Math.round(activeAgent.cartY) }
          : { x: 0, y: 0 };

        const path = await this.pathfinder.findPath(startPoint, cartTarget);
        if (path) {
          this.drawAStarPath(path);
        }

        await this.agentManager.dispatchTo(cartTarget.x, cartTarget.y, this.pathfinder);
      }
    });
  }

  private drawTargetMarker(cartX: number, cartY: number) {
    this.markerGraphics.clear();
    const { x: isoX, y: isoY } = cartesianToIsometric(cartX, cartY);
    this.markerGraphics.setDepth(calculateIsometricDepth(cartX, cartY, DepthLayer.SELECTION_MARKER));

    this.markerGraphics.lineStyle(2, 0x00f0ff, 0.9);
    this.markerGraphics.strokeCircle(isoX, isoY, 18);
    this.markerGraphics.fillStyle(0x00f0ff, 0.35);
    this.markerGraphics.fillCircle(isoX, isoY, 7);
  }

  private drawAStarPath(path: Array<{ x: number; y: number }>) {
    this.pathGraphics.clear();
    if (path.length < 2) return;

    this.pathGraphics.setDepth(DepthLayer.PATH_OVERLAY);
    this.pathGraphics.lineStyle(2.5, 0x39ff14, 0.85);

    const start = cartesianToIsometric(path[0].x, path[0].y);
    this.pathGraphics.beginPath();
    this.pathGraphics.moveTo(start.x, start.y);

    for (let i = 1; i < path.length; i++) {
      const p = cartesianToIsometric(path[i].x, path[i].y);
      this.pathGraphics.lineTo(p.x, p.y);
    }
    this.pathGraphics.strokePath();

    this.pathGraphics.fillStyle(0x00f0ff, 0.95);
    for (const node of path) {
      const p = cartesianToIsometric(node.x, node.y);
      this.pathGraphics.fillCircle(p.x, p.y, 2.5);
    }
  }

  private setupCameraControls() {
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      this.isDragging = true;
      this.dragStartX = pointer.x;
      this.dragStartY = pointer.y;
      this.cameraStartX = this.cameras.main.scrollX;
      this.cameraStartY = this.cameras.main.scrollY;
    });

    this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
      if (this.isDragging && pointer.isDown) {
        const dx = (pointer.x - this.dragStartX) / this.cameras.main.zoom;
        const dy = (pointer.y - this.dragStartY) / this.cameras.main.zoom;
        this.cameras.main.scrollX = this.cameraStartX - dx;
        this.cameras.main.scrollY = this.cameraStartY - dy;
      }
    });

    this.input.on('pointerup', () => {
      this.isDragging = false;
    });

    this.input.on('wheel', (_pointer: Phaser.Input.Pointer, _gameObjects: any, _deltaX: number, deltaY: number) => {
      const zoomFactor = deltaY > 0 ? 0.9 : 1.1;
      const newZoom = Phaser.Math.Clamp(this.cameras.main.zoom * zoomFactor, 0.45, 2.5);
      this.cameras.main.setZoom(newZoom);
    });
  }

  private handleResize(gameSize: Phaser.Structs.Size) {
    this.cameras.main.setSize(gameSize.width, gameSize.height);
  }
}
