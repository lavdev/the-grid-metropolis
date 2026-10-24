/**
 * Constantes de Projeção Isométrica (Dimensões de Tile 2:1 padrão)
 */
export const TILE_WIDTH = 64;
export const TILE_HEIGHT = 32;

export interface IsometricPoint {
  x: number;
  y: number;
}

export interface CartesianPoint {
  x: number;
  y: number;
}

/**
 * Converte coordenadas cartesianas do grid (x, y) em coordenadas de tela isométrica (pixels).
 * Fórmula clássica de projeção 2:1:
 * isoX = (cartX - cartY) * (tileWidth / 2)
 * isoY = (cartX + cartY) * (tileHeight / 2)
 */
export function cartesianToIsometric(cartX: number, cartY: number): IsometricPoint {
  const halfWidth = TILE_WIDTH / 2;
  const halfHeight = TILE_HEIGHT / 2;

  return {
    x: (cartX - cartY) * halfWidth,
    y: (cartX + cartY) * halfHeight,
  };
}

/**
 * Converte coordenadas de tela isométrica (pixels) de volta para coordenadas cartesianas do grid.
 * cartX = (isoX / halfWidth + isoY / halfHeight) / 2
 * cartY = (isoY / halfHeight - isoX / halfWidth) / 2
 */
export function isometricToCartesian(screenX: number, screenY: number): CartesianPoint {
  const halfWidth = TILE_WIDTH / 2;
  const halfHeight = TILE_HEIGHT / 2;

  const cartX = (screenX / halfWidth + screenY / halfHeight) / 2;
  const cartY = (screenY / halfHeight - screenX / halfWidth) / 2;

  return {
    x: Math.round(cartX),
    y: Math.round(cartY),
  };
}

/**
 * Calcula a profundidade de renderização (z-index / depth) para z-sorting isométrico correto.
 * Elementos mais "ao sul" na tela precisam sobrepor elementos mais "ao norte".
 */
export function getIsometricDepth(cartX: number, cartY: number, layerOffset: number = 0): number {
  return (cartX + cartY) * 100 + layerOffset;
}
