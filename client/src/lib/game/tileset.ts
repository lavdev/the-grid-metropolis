import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from './isometric';
import { TileType, CITY_GRID_WIDTH, CITY_GRID_HEIGHT } from './pathfinding';
import { DepthLayer, calculateIsometricDepth } from './depth';

export const PLATFORM_SLAB_HEIGHT = 16; // Espessura 3D da plataforma suspensa sobre o abismo

interface ConduitSegment {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  color: number;
}

/**
 * CityTilesetRenderer: Responsável pela renderização modular em alta fidelidade
 * das passarelas metálicas, lajes com espessura 3D, dutos bioluminescentes e distritos.
 */
export class CityTilesetRenderer {
  private scene: Phaser.Scene;
  private slabGraphics: Phaser.GameObjects.Graphics;
  private floorGraphics: Phaser.GameObjects.Graphics;
  private conduitGraphics: Phaser.GameObjects.Graphics;
  private pulseTimer = 0;
  private conduitBeads: Array<{ t: number; speed: number; segmentIndex: number }> = [];
  private conduitSegments: ConduitSegment[] = [];

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
    this.slabGraphics = this.scene.add.graphics();
    this.floorGraphics = this.scene.add.graphics();
    this.conduitGraphics = this.scene.add.graphics();
  }

  /**
   * Renderiza a metrópole inteira com base na matriz do mapa
   */
  public renderGrid(matrix: TileType[][], toCartesian: (mx: number, my: number) => { x: number; y: number }) {
    this.slabGraphics.clear();
    this.floorGraphics.clear();
    this.conduitSegments = [];

    const halfW = TILE_WIDTH / 2;
    const halfH = TILE_HEIGHT / 2;

    for (let my = 0; my < CITY_GRID_HEIGHT; my++) {
      for (let mx = 0; mx < CITY_GRID_WIDTH; mx++) {
        const tile = matrix[my][mx];
        if (tile === TileType.Abyss) continue;

        const cart = toCartesian(mx, my);
        const { x: isoX, y: isoY } = cartesianToIsometric(cart.x, cart.y);
        const tileDepth = calculateIsometricDepth(cart.x, cart.y, DepthLayer.FLOOR_TILE);

        // 1. Verifica se tem vizinhos no abismo a sul/sudoeste/sudeste para desenhar borda/laje 3D
        const southAbyss = my + 1 >= CITY_GRID_HEIGHT || matrix[my + 1][mx] === TileType.Abyss;
        const eastAbyss = mx + 1 >= CITY_GRID_WIDTH || matrix[my][mx + 1] === TileType.Abyss;
        const westAbyss = mx - 1 < 0 || matrix[my][mx - 1] === TileType.Abyss;

        // Renderiza laje 3D (espessura suspensa sobre o Data Ocean)
        if (southAbyss || eastAbyss || westAbyss) {
          this.drawPlatformSlab(isoX, isoY, halfW, halfH, eastAbyss, southAbyss, westAbyss);
        }

        // 2. Renderiza a superfície do piso de acordo com a tipologia
        this.drawTileSurface(isoX, isoY, halfW, halfH, tile);

        // 3. Se for passarela, registra segmento de duto bioluminescente
        if (tile === TileType.Walkway) {
          this.registerConduitPipes(cart.x, cart.y, mx, my, matrix, toCartesian);
        }
      }
    }

    // Inicializa pulso de dados nos dutos
    this.initializeConduitPulses();
  }

  /**
   * Desenha a laje chanfrada vertical 3D dando a sensação de altura suspensa sobre o mar abissal
   */
  private drawPlatformSlab(
    isoX: number,
    isoY: number,
    halfW: number,
    halfH: number,
    eastBorder: boolean,
    southBorder: boolean,
    _westBorder: boolean
  ) {
    const g = this.slabGraphics;
    const slabH = PLATFORM_SLAB_HEIGHT;

    // Face Direita (Sudeste) da laje metálica
    if (eastBorder || southBorder) {
      g.fillStyle(0x080c14, 0.95); // Aço escuro industrial
      g.beginPath();
      g.moveTo(isoX, isoY + halfH);
      g.lineTo(isoX + halfW, isoY);
      g.lineTo(isoX + halfW, isoY + slabH);
      g.lineTo(isoX, isoY + halfH + slabH);
      g.closePath();
      g.fillPath();

      // Borda néon inferior da laje
      g.lineStyle(1.5, 0x00f0ff, 0.4);
      g.beginPath();
      g.moveTo(isoX, isoY + halfH + slabH);
      g.lineTo(isoX + halfW, isoY + slabH);
      g.strokePath();
    }

    // Face Esquerda (Sudoeste) da laje metálica
    if (southBorder) {
      g.fillStyle(0x05080f, 0.95); // Sombra mais escura da laje
      g.beginPath();
      g.moveTo(isoX - halfW, isoY);
      g.lineTo(isoX, isoY + halfH);
      g.lineTo(isoX, isoY + halfH + slabH);
      g.lineTo(isoX - halfW, isoY + slabH);
      g.closePath();
      g.fillPath();

      // Borda néon inferior da laje
      g.lineStyle(1.5, 0x9d00ff, 0.4);
      g.beginPath();
      g.moveTo(isoX - halfW, isoY + slabH);
      g.lineTo(isoX, isoY + halfH + slabH);
      g.strokePath();
    }
  }

  /**
   * Desenha a textura e acabamento de cada tipo de piso
   */
  private drawTileSurface(isoX: number, isoY: number, halfW: number, halfH: number, tile: TileType) {
    const g = this.floorGraphics;

    let fillColor = 0x0c121e;
    let strokeColor = 0x1e2a3f;
    let accentColor = 0x00f0ff;
    let hasGlowEdge = false;

    switch (tile) {
      case TileType.Nexus:
        fillColor = 0x0a1c30;
        strokeColor = 0x00f0ff;
        accentColor = 0x00f0ff;
        hasGlowEdge = true;
        break;

      case TileType.Walkway:
        fillColor = 0x0b101c;
        strokeColor = 0x22324e;
        accentColor = 0x9d00ff;
        hasGlowEdge = true;
        break;

      case TileType.Lounge:
        fillColor = 0x1a0f26;
        strokeColor = 0xff007f;
        accentColor = 0xff007f;
        hasGlowEdge = true;
        break;

      case TileType.PodEntrance:
        fillColor = 0x0b2419;
        strokeColor = 0x39ff14;
        accentColor = 0x39ff14;
        hasGlowEdge = true;
        break;

      case TileType.GlitchAlley:
        fillColor = 0x240d0d;
        strokeColor = 0xff3b30;
        accentColor = 0xff3b30;
        hasGlowEdge = true;
        break;

      case TileType.Obstacle:
        fillColor = 0x1c1307;
        strokeColor = 0xff9500;
        accentColor = 0xff9500;
        break;
    }

    // 1. Superfície Losangular Isométrica
    g.fillStyle(fillColor, 0.95);
    g.beginPath();
    g.moveTo(isoX, isoY - halfH);
    g.lineTo(isoX + halfW, isoY);
    g.lineTo(isoX, isoY + halfH);
    g.lineTo(isoX - halfW, isoY);
    g.closePath();
    g.fillPath();

    // 2. Traço da Borda Metálica
    g.lineStyle(1, strokeColor, 0.7);
    g.strokePath();

    // 3. Guias de Néon Laterais para Passarelas e Distritos
    if (hasGlowEdge) {
      g.lineStyle(1.5, accentColor, 0.6);
      g.beginPath();
      // Borda sul iluminada
      g.moveTo(isoX - halfW, isoY);
      g.lineTo(isoX, isoY + halfH);
      g.lineTo(isoX + halfW, isoY);
      g.strokePath();
    }

    // 4. Detalhes em Pixel Art / Rebites nos cantos dos painéis
    g.fillStyle(accentColor, 0.5);
    g.fillRect(isoX - 1, isoY - halfH + 2, 2, 2);
    g.fillRect(isoX - 1, isoY + halfH - 3, 2, 2);

    // Linha central para passarela
    if (tile === TileType.Walkway) {
      g.lineStyle(1, 0x00f0ff, 0.25);
      g.beginPath();
      g.moveTo(isoX, isoY - halfH * 0.5);
      g.lineTo(isoX, isoY + halfH * 0.5);
      g.strokePath();
    }
  }

  private registerConduitPipes(
    cartX: number,
    cartY: number,
    mx: number,
    my: number,
    matrix: TileType[][],
    toCartesian: (mx: number, my: number) => { x: number; y: number }
  ) {
    const current = cartesianToIsometric(cartX, cartY);

    // Conecta dutos com vizinhos que também são passarelas ou nexus
    const neighbors = [
      { dx: 1, dy: 0 },
      { dx: 0, dy: 1 },
    ];

    for (const n of neighbors) {
      const nx = mx + n.dx;
      const ny = my + n.dy;
      if (nx < CITY_GRID_WIDTH && ny < CITY_GRID_HEIGHT) {
        const neighborTile = matrix[ny][nx];
        if (neighborTile === TileType.Walkway || neighborTile === TileType.Nexus || neighborTile === TileType.PodEntrance) {
          const neighborCart = toCartesian(nx, ny);
          const neighborIso = cartesianToIsometric(neighborCart.x, neighborCart.y);
          this.conduitSegments.push({
            startX: current.x,
            startY: current.y,
            endX: neighborIso.x,
            endY: neighborIso.y,
            color: 0x00f0ff,
          });
        }
      }
    }
  }

  private initializeConduitPulses() {
    this.conduitBeads = [];
    for (let i = 0; i < this.conduitSegments.length; i++) {
      if (i % 2 === 0) {
        this.conduitBeads.push({
          segmentIndex: i,
          t: Math.random(),
          speed: 0.008 + Math.random() * 0.008,
        });
      }
    }
  }

  /**
   * Atualiza a animação de pulso dos fluidos nos dutos bioluminescentes
   */
  public update(delta: number) {
    this.pulseTimer += delta * 0.002;
    const g = this.conduitGraphics;
    g.clear();
    g.setDepth(calculateIsometricDepth(0, 0, DepthLayer.CONDUIT));

    // Renderiza linha do duto bioluminescente
    g.lineStyle(1.5, 0x00f0ff, 0.35);
    for (const seg of this.conduitSegments) {
      g.beginPath();
      g.moveTo(seg.startX, seg.startY);
      g.lineTo(seg.endX, seg.endY);
      g.strokePath();
    }

    // Renderiza pacotes de luz fluindo pelos tubos
    for (const bead of this.conduitBeads) {
      bead.t += bead.speed * (delta * 0.06);
      if (bead.t > 1) bead.t = 0;

      const seg = this.conduitSegments[bead.segmentIndex];
      if (seg) {
        const curX = Phaser.Math.Linear(seg.startX, seg.endX, bead.t);
        const curY = Phaser.Math.Linear(seg.startY, seg.endY, bead.t);

        // Brilho fluorescente do pulso
        g.fillStyle(0x39ff14, 0.9);
        g.fillCircle(curX, curY, 2.5);

        g.fillStyle(0x00f0ff, 0.4);
        g.fillCircle(curX, curY, 5);
      }
    }
  }

  public destroy() {
    this.slabGraphics.destroy();
    this.floorGraphics.destroy();
    this.conduitGraphics.destroy();
    this.conduitSegments = [];
    this.conduitBeads = [];
  }
}
