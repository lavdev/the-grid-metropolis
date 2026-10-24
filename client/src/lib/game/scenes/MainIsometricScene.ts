import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from '../isometric';

export class MainIsometricScene extends Phaser.Scene {
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private cameraStartX = 0;
  private cameraStartY = 0;

  constructor() {
    super({ key: 'MainIsometricScene' });
  }

  preload() {
    // Assets serão carregados conforme as próximas tarefas (T-2.4 e T-4.1)
  }

  create() {
    // Centraliza a câmera no ponto de origem (Nexus)
    this.cameras.main.centerOn(0, 200);
    this.cameras.main.setZoom(0.85);

    // Desenha a malha inicial do grid isométrico e o mar de dados
    this.renderDataOcean();
    this.renderInitialGrid();

    // Configura os controles interativos de câmera (Pan & Zoom)
    this.setupCameraControls();

    // Redimensionamento responsivo automático
    this.scale.on('resize', this.handleResize, this);
  }

  /**
   * Renderiza o fundo sutil do Data Ocean (abismo sob as passarelas)
   */
  private renderDataOcean() {
    const graphics = this.add.graphics();
    graphics.setDepth(-1000);

    // Gradiente e feixes do Data Ocean
    graphics.fillStyle(0x05070c, 1);
    graphics.fillRect(-4000, -4000, 8000, 8000);

    // Linhas de dados no abismo
    graphics.lineStyle(1, 0x00f0ff, 0.04);
    for (let x = -3000; x < 3000; x += 120) {
      graphics.lineBetween(x, -3000, x, 3000);
    }
  }

  /**
   * Renderiza a malha geométrica de base isométrica (demonstração da projeção 2:1)
   */
  private renderInitialGrid() {
    const graphics = this.add.graphics();
    graphics.setDepth(0);

    const gridSize = 24; // 24x24 tiles
    const halfWidth = TILE_WIDTH / 2;
    const halfHeight = TILE_HEIGHT / 2;

    for (let x = -gridSize / 2; x <= gridSize / 2; x++) {
      for (let y = -gridSize / 2; y <= gridSize / 2; y++) {
        const { x: isoX, y: isoY } = cartesianToIsometric(x, y);

        // Identifica zonas principais para acentuação de cores neon
        const isCenter = Math.abs(x) <= 3 && Math.abs(y) <= 3; // The Nexus
        const isMainAvenue = x === 0 || y === 0; // Passarelas centrais

        let fillColor = 0x0a0e17;
        let strokeColor = 0x1a2638;
        let strokeAlpha = 0.4;

        if (isCenter) {
          fillColor = 0x0f1c2e;
          strokeColor = 0x00f0ff;
          strokeAlpha = 0.6;
        } else if (isMainAvenue) {
          fillColor = 0x0d1522;
          strokeColor = 0x9d00ff;
          strokeAlpha = 0.5;
        }

        // Desenha o losango isométrico (projeção 2:1)
        graphics.fillStyle(fillColor, 0.85);
        graphics.beginPath();
        graphics.moveTo(isoX, isoY - halfHeight);
        graphics.lineTo(isoX + halfWidth, isoY);
        graphics.lineTo(isoX, isoY + halfHeight);
        graphics.lineTo(isoX - halfWidth, isoY);
        graphics.closePath();
        graphics.fillPath();

        // Linha de contorno neon
        graphics.lineStyle(1, strokeColor, strokeAlpha);
        graphics.strokePath();
      }
    }

    // Marca holográfica central do Nexus
    const nexusHolo = this.add.graphics();
    nexusHolo.setDepth(10);
    nexusHolo.lineStyle(2, 0x00f0ff, 0.8);
    nexusHolo.strokeCircle(0, 0, 48);
    nexusHolo.lineStyle(1, 0x39ff14, 0.5);
    nexusHolo.strokeCircle(0, 0, 72);

    // Texto flutuante no Nexus
    const label = this.add.text(0, -60, 'THE NEXUS // CENTRAL PLATFORM', {
      fontFamily: 'monospace',
      fontSize: '12px',
      color: '#00f0ff',
      backgroundColor: '#05070caa',
      padding: { x: 6, y: 3 },
    });
    label.setOrigin(0.5);
    label.setDepth(20);
  }

  /**
   * Configura os controles de câmera: Pan arrastando e Zoom com a roda do mouse
   */
  private setupCameraControls() {
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      if (pointer.leftButtonDown()) {
        this.isDragging = true;
        this.dragStartX = pointer.x;
        this.dragStartY = pointer.y;
        this.cameraStartX = this.cameras.main.scrollX;
        this.cameraStartY = this.cameras.main.scrollY;
      }
    });

    this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
      if (this.isDragging) {
        const dx = (pointer.x - this.dragStartX) / this.cameras.main.zoom;
        const dy = (pointer.y - this.dragStartY) / this.cameras.main.zoom;
        this.cameras.main.scrollX = this.cameraStartX - dx;
        this.cameras.main.scrollY = this.cameraStartY - dy;
      }
    });

    this.input.on('pointerup', () => {
      this.isDragging = false;
    });

    this.input.on('wheel', (pointer: Phaser.Input.Pointer, gameObjects: any, deltaX: number, deltaY: number) => {
      const zoomFactor = deltaY > 0 ? 0.9 : 1.1;
      const newZoom = Phaser.Math.Clamp(this.cameras.main.zoom * zoomFactor, 0.35, 2.2);
      this.cameras.main.setZoom(newZoom);
    });
  }

  private handleResize(gameSize: Phaser.Structs.Size) {
    this.cameras.main.setSize(gameSize.width, gameSize.height);
  }
}
