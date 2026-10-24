import Phaser from 'phaser';
import { cartesianToIsometric, isometricToCartesian } from '../isometric';
import { GridPathfinder } from '../pathfinding';
import { DataOcean } from '../dataOcean';
import { CityTilesetRenderer } from '../tileset';
import { DepthLayer, calculateIsometricDepth } from '../depth';

export class MainIsometricScene extends Phaser.Scene {
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private cameraStartX = 0;
  private cameraStartY = 0;

  private pathfinder!: GridPathfinder;
  private dataOcean!: DataOcean;
  private tilesetRenderer!: CityTilesetRenderer;
  private pathGraphics!: Phaser.GameObjects.Graphics;
  private markerGraphics!: Phaser.GameObjects.Graphics;

  constructor() {
    super({ key: 'MainIsometricScene' });
  }

  preload() {
    // Assets e Spritesheets adicionais
  }

  create() {
    this.pathfinder = new GridPathfinder();

    // Centraliza a câmera no Nexus com zoom padrão
    this.cameras.main.centerOn(0, 160);
    this.cameras.main.setZoom(0.85);

    // 1. Data Ocean no fundo abissal (-1000 depth) com partículas animadas
    this.dataOcean = new DataOcean(this);

    // 2. Renderizador modular de tileset com laje 3D, bordas de neon e dutos bioluminescentes
    this.tilesetRenderer = new CityTilesetRenderer(this);
    this.tilesetRenderer.renderGrid(
      this.pathfinder.getMatrix(),
      (mx, my) => this.pathfinder.toCartesianCoord(mx, my)
    );

    // 3. Camada gráfica para desenhar trajetórias A* e seleções com Z-sorting correto
    this.pathGraphics = this.add.graphics();
    this.pathGraphics.setDepth(DepthLayer.PATH_OVERLAY);

    this.markerGraphics = this.add.graphics();
    this.markerGraphics.setDepth(DepthLayer.SELECTION_MARKER);

    // 4. Marcações dos Distritos da Cidade
    this.setupDistrictMarkers();

    // 5. Configura controles interativos de câmera e cliques no mapa
    this.setupCameraControls();
    this.setupMapInteractivity();

    // Redimensionamento responsivo automático
    this.scale.on('resize', this.handleResize, this);
  }

  override update(_time: number, delta: number) {
    // Atualiza correntes do Data Ocean e pulso dos fluidos nos dutos
    if (this.dataOcean) {
      this.dataOcean.update(delta);
    }
    if (this.tilesetRenderer) {
      this.tilesetRenderer.update(delta);
    }
  }

  private setupDistrictMarkers() {
    this.addDistrictMarker(0, 0, 'THE NEXUS', '#00f0ff');
    this.addDistrictMarker(-12, 0, 'BATTERY LOUNGE', '#ff007f');
    this.addDistrictMarker(12, 0, 'POD: NOTEX CORE', '#39ff14');
    this.addDistrictMarker(0, -12, 'GLITCH ALLEY', '#ff3b30');
    this.addDistrictMarker(0, 12, 'EXPANSION BAY', '#00f0ff');
  }

  private addDistrictMarker(cartX: number, cartY: number, title: string, color: string) {
    const { x: isoX, y: isoY } = cartesianToIsometric(cartX, cartY);

    const text = this.add.text(isoX, isoY - 32, title, {
      fontFamily: 'monospace',
      fontSize: '11px',
      color,
      backgroundColor: '#040711ee',
      padding: { x: 6, y: 3 },
    });
    text.setOrigin(0.5);
    text.setDepth(calculateIsometricDepth(cartX, cartY, DepthLayer.FLOOR_MARKINGS));
  }

  /**
   * Configura clique no mapa para demonstrar o cálculo de rota A* em tempo real
   */
  private setupMapInteractivity() {
    this.input.on('pointerdown', async (pointer: Phaser.Input.Pointer) => {
      // Se estava arrastando, ignora cálculo de rota
      if (pointer.rightButtonDown() || pointer.getDistance() > 10) return;

      // Converte coordenadas do clique para o mundo e em seguida para cartesianas
      const worldPoint = this.cameras.main.getWorldPoint(pointer.x, pointer.y);
      const cartTarget = isometricToCartesian(worldPoint.x, worldPoint.y);

      if (this.pathfinder.isWalkable(cartTarget.x, cartTarget.y)) {
        // Marca o destino com feedback visual néon
        this.drawTargetMarker(cartTarget.x, cartTarget.y);

        // Calcula a rota A* a partir do Nexus (0, 0) até o ponto clicado
        const path = await this.pathfinder.findPath({ x: 0, y: 0 }, cartTarget);
        if (path) {
          this.drawAStarPath(path);
        }
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

  /**
   * Desenha a trajetória calculada pelo A* com traçado néon conectando os pontos
   */
  private drawAStarPath(path: Array<{ x: number; y: number }>) {
    this.pathGraphics.clear();
    if (path.length < 2) return;

    this.pathGraphics.setDepth(DepthLayer.PATH_OVERLAY);
    this.pathGraphics.lineStyle(3, 0x39ff14, 0.85); // Linha verde néon

    const start = cartesianToIsometric(path[0].x, path[0].y);
    this.pathGraphics.beginPath();
    this.pathGraphics.moveTo(start.x, start.y);

    for (let i = 1; i < path.length; i++) {
      const p = cartesianToIsometric(path[i].x, path[i].y);
      this.pathGraphics.lineTo(p.x, p.y);
    }

    this.pathGraphics.strokePath();

    // Desenha pequenas luzes em cada nó do caminho
    this.pathGraphics.fillStyle(0x00f0ff, 0.9);
    for (const node of path) {
      const p = cartesianToIsometric(node.x, node.y);
      this.pathGraphics.fillCircle(p.x, p.y, 3);
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
      const newZoom = Phaser.Math.Clamp(this.cameras.main.zoom * zoomFactor, 0.35, 2.2);
      this.cameras.main.setZoom(newZoom);
    });
  }

  private handleResize(gameSize: Phaser.Structs.Size) {
    this.cameras.main.setSize(gameSize.width, gameSize.height);
  }
}
