/**
 * Camadas de Profundidade (Depth Layers) para o Renderizador Isométrico.
 * Segue a hierarquia visual do mundo cyberpunk de The Grid Metropolis.
 */
export enum DepthLayer {
  DATA_OCEAN = -1000,
  DATA_PACKETS = -900,
  PLATFORM_SLAB = 5,
  FLOOR_TILE = 10,
  CONDUIT = 20,
  FLOOR_MARKINGS = 30,
  PATH_OVERLAY = 40,
  STRUCTURE_BASE = 50,
  ENTITY = 60,            // Agentes, Bots, Drones terrestres
  DRONE_AIR = 65,         // Drones aéreos sobrevoando passarelas
  STRUCTURE_ROOF = 70,    // Coberturas e cúpulas dos Pods
  BALLOON_UI = 80,        // Balões de diálogo e pensamento
  SELECTION_MARKER = 90,  // Círculo de seleção e foco
  HUD_OVERLAY = 100,      // Elementos fixos da viewport
}

/**
 * Calcula a profundidade isométrica z-index exata para qualquer coordenada e camada.
 * No grid isométrico 2:1, a profundidade visual cresce com (cartX + cartY).
 */
export function calculateIsometricDepth(cartX: number, cartY: number, layer: DepthLayer = DepthLayer.ENTITY): number {
  const baseDepth = (cartX + cartY) * 100;
  return baseDepth + layer;
}

/**
 * Utilitário para gerenciar e reordenar a profundidade de GameObjects dinâmicos (Agentes, Drones)
 */
export class DepthManager {
  /**
   * Atualiza a profundidade de um GameObject de acordo com sua posição cartesiana atual.
   */
  public static sortEntity(gameObject: Phaser.GameObjects.GameObject & { depth: number }, cartX: number, cartY: number, layer: DepthLayer = DepthLayer.ENTITY) {
    gameObject.depth = calculateIsometricDepth(cartX, cartY, layer);
  }

  /**
   * Ordena um array de GameObjects com base nas suas coordenadas cartesianas
   */
  public static sortEntities(entities: Array<{ gameObject: Phaser.GameObjects.GameObject & { depth: number }; cartX: number; cartY: number; layer?: DepthLayer }>) {
    for (const item of entities) {
      item.gameObject.depth = calculateIsometricDepth(item.cartX, item.cartY, item.layer ?? DepthLayer.ENTITY);
    }
  }
}
