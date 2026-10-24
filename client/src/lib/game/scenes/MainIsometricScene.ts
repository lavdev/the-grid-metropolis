import Phaser from 'phaser';
import { cartesianToIsometric, isometricToCartesian, TILE_WIDTH, TILE_HEIGHT } from '../isometric';
import { GridPathfinder, TileType, CITY_GRID_WIDTH, CITY_GRID_HEIGHT } from '../pathfinding';

export class MainIsometricScene extends Phaser.Scene {
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private cameraStartX = 0;
  private cameraStartY = 0;

  private pathfinder!: GridPathfinder;
  private pathGraphics!: Phaser.GameObjects.Graphics;
  private markerGraphics!: Phaser.GameObjects.Graphics;

  constructor() {
    super({ key: 'MainIsometricScene' });
  }

  preload() {
    // Assets serão carregados na T-2.4 e T-4.1
  }

  create() {
    this.pathfinder = new GridPathfinder();

    // Centraliza a câmera no Nexus com zoom padrão
    this.cameras.main.centerOn(0, 160);
    this.cameras.main.setZoom(0.85);

    // 1. Renderiza o Data Ocean no fundo
    this.renderDataOcean();

    // 2. Renderiza a geometria da cidade baseada no Grid de Pathfinding
    this.renderCityTopology();

    // 3. Camada gráfica para desenhar trajetórias A* e seleções
    this.pathGraphics = this.add.graphics();
    this.pathGraphics.setDepth(500);

    this.markerGraphics = this.add.graphics();
    this.markerGraphics.setDepth(501);

    // 4. Configura controles interativos de câmera e cliques no mapa
    this.setupCameraControls();
    this.setupMapInteractivity();

    // Redimensionamento responsivo automático
    this.scale.on('resize', this.handleResize, this);
  }

  private renderDataOcean() {
    const graphics = this.add.graphics();
    graphics.setDepth(-1000);

    // Fundo abissal do mar de dados
    graphics.fillStyle(0x04060a, 1);
    graphics.fillRect(-5000, -5000, 10000, 10000);

    // Ondas bioluminescentes no abismo
    graphics.lineStyle(1, 0x00f0ff, 0.03);
    for (let x = -4000; x < 4000; x += 160) {
      graphics.lineBetween(x, -4000, x, 4000);
    }
  }

  /**
   * Renderiza os pisos da cidade respeitando estritamente o layout do Pathfinding
   */
  private renderCityTopology() {
    const graphics = this.add.graphics();
    graphics.setDepth(1);

    const halfW = TILE_WIDTH / 2;
    const halfH = TILE_HEIGHT / 2;
    const matrix = this.pathfinder.getMatrix();

    for (let my = 0; my < CITY_GRID_HEIGHT; my++) {
      for (let mx = 0; mx < CITY_GRID_WIDTH; mx++) {
        const tile = matrix[my][mx];
        if (tile === TileType.Abyss) continue; // Pula o abismo vazio

        const cart = this.pathfinder.toCartesianCoord(mx, my);
        const { x: isoX, y: isoY } = cartesianToIsometric(cart.x, cart.y);

        let fillColor = 0x0d121d;
        let strokeColor = 0x1e293b;
        let strokeAlpha = 0.5;

        switch (tile) {
          case TileType.Nexus:
            fillColor = 0x0b1e33;
            strokeColor = 0x00f0ff; // Ciano néon
            strokeAlpha = 0.7;
            break;
          case TileType.Walkway:
            fillColor = 0x0f1522;
            strokeColor = 0x9d00ff; // Roxo néon
            strokeAlpha = 0.6;
            break;
          case TileType.Lounge:
            fillColor = 0x1f142b;
            strokeColor = 0xff007f; // Magenta néon (Battery Lounge)
            strokeAlpha = 0.8;
            break;
          case TileType.PodEntrance:
            fillColor = 0x0e241b;
            strokeColor = 0x39ff14; // Verde néon (Project Pods)
            strokeAlpha = 0.8;
            break;
          case TileType.GlitchAlley:
            fillColor = 0x241111;
            strokeColor = 0xff3b30; // Vermelho de alerta (Glitch Alley)
            strokeAlpha = 0.8;
            break;
          case TileType.Obstacle:
            fillColor = 0x331a00;
            strokeColor = 0xff9900;
            strokeAlpha = 0.9;
            break;
        }

        // Desenha losango isométrico
        graphics.fillStyle(fillColor, 0.9);
        graphics.beginPath();
        graphics.moveTo(isoX, isoY - halfH);
        graphics.lineTo(isoX + halfW, isoY);
        graphics.lineTo(isoX, isoY + halfH);
        graphics.lineTo(isoX - halfW, isoY);
        graphics.closePath();
        graphics.fillPath();

        graphics.lineStyle(1, strokeColor, strokeAlpha);
        graphics.strokePath();
      }
    }

    // Marcações e Labels temáticos dos distritos
    this.addDistrictMarker(0, 0, 'THE NEXUS', '#00f0ff');
    this.addDistrictMarker(-12, 0, 'BATTERY LOUNGE', '#ff007f');
    this.addDistrictMarker(12, 0, 'POD: NOTEX CORE', '#39ff14');
    this.addDistrictMarker(0, -12, 'GLITCH ALLEY', '#ff3b30');
    this.addDistrictMarker(0, 12, 'EXPANSION BAY', '#00f0ff');
  }

  private addDistrictMarker(cartX: number, cartY: number, title: string, color: string) {
    const { x: isoX, y: isoY } = cartesianToIsometric(cartX, cartY);

    const text = this.add.text(isoX, isoY - 30, title, {
      fontFamily: 'monospace',
      fontSize: '11px',
      color,
      backgroundColor: '#05070cee',
      padding: { x: 5, y: 2 },
    });
    text.setOrigin(0.5);
    text.setDepth(100);
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

    this.markerGraphics.lineStyle(2, 0x00f0ff, 0.9);
    this.markerGraphics.strokeCircle(isoX, isoY, 16);
    this.markerGraphics.fillStyle(0x00f0ff, 0.3);
    this.markerGraphics.fillCircle(isoX, isoY, 8);
  }

  /**
   * Desenha a trajetória calculada pelo A* com traçado néon conectando os pontos
   */
  private drawAStarPath(path: Array<{ x: number; y: number }>) {
    this.pathGraphics.clear();
    if (path.length < 2) return;

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
