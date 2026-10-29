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

  // 1. The War Room / Nexus (Plataforma elevada ao Norte)
  for (let cy = -9; cy <= -1; cy++) {
    for (let cx = -9; cx <= -1; cx++) {
      setTile(matrix, cx, cy, TileType.Nexus);
    }
  }

  // 2. The Dev Lab (Sala Oeste com bancadas de computadores e cadeiras gamer)
  for (let cy = 2; cy <= 13; cy++) {
    for (let cx = -10; cx <= 0; cx++) {
      setTile(matrix, cx, cy, TileType.PodEntrance);
    }
  }

  // 3. The Cyber Breakroom & Arcades (Sala Leste com fliperamas, sofá e máquina de café)
  for (let cy = -11; cy <= 0; cy++) {
    for (let cx = 1; cx <= 10; cx++) {
      setTile(matrix, cx, cy, TileType.Lounge);
    }
  }

  // 4. The Server Vault (Sala Sul com racks de servidores e iluminação cibernética)
  for (let cy = 2; cy <= 8; cy++) {
    for (let cx = 1; cx <= 8; cx++) {
      setTile(matrix, cx, cy, TileType.GlitchAlley);
    }
  }

  // 5. Passarelas e eixos de tráfego que interligam todos os distritos
  for (let x = -10; x <= 10; x++) {
    setTile(matrix, x, 0, TileType.Walkway);
    setTile(matrix, x, 1, TileType.Walkway);
  }
  for (let y = -11; y <= 11; y++) {
    setTile(matrix, 0, y, TileType.Walkway);
    setTile(matrix, 1, y, TileType.Walkway);
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
