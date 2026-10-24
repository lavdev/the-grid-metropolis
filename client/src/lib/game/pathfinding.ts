import EasyStar from 'easystarjs';
import type { CartesianPoint } from './isometric';

export enum TileType {
  Abyss = 0,        // Mar de dados / abismo (intransponível)
  Walkway = 1,      // Passarela metálica iluminada (navegável)
  Nexus = 2,        // Plataforma central The Nexus (navegável)
  Lounge = 3,       // The Battery Lounge (navegável)
  GlitchAlley = 4,  // The Glitch Alleys (navegável)
  PodEntrance = 5,  // Entrada e estações do Project Pod (navegável)
  Obstacle = 9,     // Paredes, maquinários e servidores (bloqueado)
}

export const CITY_GRID_WIDTH = 36;
export const CITY_GRID_HEIGHT = 36;
const HALF_GRID_W = Math.floor(CITY_GRID_WIDTH / 2);
const HALF_GRID_H = Math.floor(CITY_GRID_HEIGHT / 2);

/**
 * Constrói a topologia oficial da cidade The Grid Metropolis sobre a matriz 2D.
 */
export function buildCityGridMatrix(): number[][] {
  const matrix: number[][] = [];

  for (let y = 0; y < CITY_GRID_HEIGHT; y++) {
    const row: number[] = [];
    for (let x = 0; x < CITY_GRID_WIDTH; x++) {
      row.push(TileType.Abyss);
    }
    matrix.push(row);
  }

  // 1. Constrói The Nexus (Plataforma octogonal central)
  for (let cy = -4; cy <= 4; cy++) {
    for (let cx = -4; cx <= 4; cx++) {
      if (Math.abs(cx) + Math.abs(cy) <= 6) {
        setTile(matrix, cx, cy, TileType.Nexus);
      }
    }
  }

  // 2. Passarelas Cardeais Principais (Eixo X e Eixo Y com 2 tiles de largura para tráfego duplo)
  for (let x = -15; x <= 15; x++) {
    setTile(matrix, x, 0, TileType.Walkway);
    setTile(matrix, x, 1, TileType.Walkway);
  }
  for (let y = -15; y <= 15; y++) {
    setTile(matrix, 0, y, TileType.Walkway);
    setTile(matrix, 1, y, TileType.Walkway);
  }

  // 3. Anel Periférico de Conexão (Passarela circular conectando os 4 distritos)
  for (let i = -12; i <= 12; i++) {
    setTile(matrix, i, -12, TileType.Walkway);
    setTile(matrix, i, 12, TileType.Walkway);
    setTile(matrix, -12, i, TileType.Walkway);
    setTile(matrix, 12, i, TileType.Walkway);
  }

  // 4. Distrito Oeste (-12, 0): The Battery Lounge (Plataforma 5x5)
  for (let ly = -2; ly <= 2; ly++) {
    for (let lx = -14; lx <= -10; lx++) {
      setTile(matrix, lx, ly, TileType.Lounge);
    }
  }

  // 5. Distrito Leste (+12, 0): Project Pod Notex Core (Plataforma 5x5)
  for (let py = -2; py <= 2; py++) {
    for (let px = 10; px <= 14; px++) {
      setTile(matrix, px, py, TileType.PodEntrance);
    }
  }

  // 6. Distrito Norte (0, -12): The Glitch Alleys (Área de quarentena com obstáculos)
  for (let gy = -14; gy <= -10; gy++) {
    for (let gx = -2; gx <= 2; gx++) {
      setTile(matrix, gx, gy, TileType.GlitchAlley);
    }
  }
  // Coloca alguns obstáculos intencionais (peças soltas / fios) no Glitch Alley
  setTile(matrix, -1, -13, TileType.Obstacle);
  setTile(matrix, 1, -11, TileType.Obstacle);

  // 7. Distrito Sul (0, +12): Plataforma Modular de Expansão Contínua
  for (let sy = 10; sy <= 14; sy++) {
    for (let sx = -2; sx <= 2; sx++) {
      setTile(matrix, sx, sy, TileType.PodEntrance);
    }
  }

  return matrix;
}

function setTile(matrix: number[][], cartX: number, cartY: number, tileType: TileType) {
  const mx = cartX + HALF_GRID_W;
  const my = cartY + HALF_GRID_H;
  if (my >= 0 && my < CITY_GRID_HEIGHT && mx >= 0 && mx < CITY_GRID_WIDTH) {
    matrix[my][mx] = tileType;
  }
}

export class GridPathfinder {
  private easystar: EasyStar.js;
  private gridMatrix: number[][];

  constructor(matrix?: number[][]) {
    this.gridMatrix = matrix || buildCityGridMatrix();
    this.easystar = new EasyStar.js();
    this.easystar.setGrid(this.gridMatrix);

    // Tiles transitáveis pelos agentes
    this.easystar.setAcceptableTiles([
      TileType.Walkway,
      TileType.Nexus,
      TileType.Lounge,
      TileType.GlitchAlley,
      TileType.PodEntrance,
    ]);

    // Movimentação em 4 direções cardeais isométricas (produz rotas muito mais limpas em pixel art 2:1)
    this.easystar.disableDiagonals();
    this.easystar.enableSync(); // Cálculo síncrono para respostas com latência zero
  }

  /**
   * Converte coordenadas cartesianas relativas ao centro (-15..+15) para coordenadas do array matriz (0..35).
   */
  public toMatrixCoord(p: CartesianPoint): { mx: number; my: number } {
    return {
      mx: p.x + HALF_GRID_W,
      my: p.y + HALF_GRID_H,
    };
  }

  /**
   * Converte coordenadas da matriz de volta para cartesianas relativas.
   */
  public toCartesianCoord(mx: number, my: number): CartesianPoint {
    return {
      x: mx - HALF_GRID_W,
      y: my - HALF_GRID_H,
    };
  }

  /**
   * Verifica se uma célula cartesiana é transitável.
   */
  public isWalkable(cartX: number, cartY: number): boolean {
    const { mx, my } = this.toMatrixCoord({ x: cartX, y: cartY });
    if (my < 0 || my >= CITY_GRID_HEIGHT || mx < 0 || mx >= CITY_GRID_WIDTH) {
      return false;
    }
    const tile = this.gridMatrix[my][mx];
    return tile !== TileType.Abyss && tile !== TileType.Obstacle;
  }

  /**
   * Obtém o tipo de piso de uma determinada coordenada cartesiana.
   */
  public getTileType(cartX: number, cartY: number): TileType {
    const { mx, my } = this.toMatrixCoord({ x: cartX, y: cartY });
    if (my < 0 || my >= CITY_GRID_HEIGHT || mx < 0 || mx >= CITY_GRID_WIDTH) {
      return TileType.Abyss;
    }
    return this.gridMatrix[my][mx] as TileType;
  }

  /**
   * Calcula o caminho A* mais curto entre dois pontos cartesianos.
   * Retorna um array de pontos cartesianos ou null se a rota estiver bloqueada/impossível.
   */
  public findPath(start: CartesianPoint, end: CartesianPoint): Promise<CartesianPoint[] | null> {
    return new Promise((resolve) => {
      const startM = this.toMatrixCoord(start);
      const endM = this.toMatrixCoord(end);

      // Verificação de limites
      if (!this.isWalkable(start.x, start.y) || !this.isWalkable(end.x, end.y)) {
        return resolve(null);
      }

      this.easystar.findPath(startM.mx, startM.my, endM.mx, endM.my, (path) => {
        if (!path) {
          return resolve(null);
        }
        const cartesianPath: CartesianPoint[] = path.map((node) =>
          this.toCartesianCoord(node.x, node.y)
        );
        resolve(cartesianPath);
      });

      this.easystar.calculate();
    });
  }

  public getMatrix(): number[][] {
    return this.gridMatrix;
  }
}
