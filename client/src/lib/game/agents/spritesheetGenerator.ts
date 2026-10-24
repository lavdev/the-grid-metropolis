import Phaser from 'phaser';

export type AgentDirection = 'SW' | 'SE' | 'NW' | 'NE';

export interface RoleTheme {
  bodyColor: number;
  coatColor: number;
  visorColor: number;
  accentColor: number;
  skinColor: number;
}

export const ROLE_THEMES: Record<string, RoleTheme> = {
  DEVELOPER: {
    bodyColor: 0x0e1726,  // Traje cibernético escuro
    coatColor: 0x1f293d,  // Jaqueta hacker
    visorColor: 0x00f0ff, // Visor holográfico ciano
    accentColor: 0x00f0ff,// Luz néon ciano
    skinColor: 0xf5d0b0,
  },
  RESEARCHER: {
    bodyColor: 0x101b2b,
    coatColor: 0xedf2f7,  // Jaleco branco de laboratório
    visorColor: 0x9d00ff, // Óculos HUD roxo
    accentColor: 0x9d00ff,
    skinColor: 0xf5c29d,
  },
  DEVOPS: {
    bodyColor: 0x1a202c,
    coatColor: 0xd69e2e,  // Traje industrial âmbar
    visorColor: 0x39ff14, // Visor verde néon
    accentColor: 0x39ff14,
    skinColor: 0xd4a373,
  },
  BOT_RUNNER: {
    bodyColor: 0x2d3748,  // Chassis metálico
    coatColor: 0x4a5568,
    visorColor: 0x00f0ff, // LED óptico
    accentColor: 0x39ff14,
    skinColor: 0x718096,  // Face sintética cinza
  },
};

/**
 * ProceduralSpriteGenerator: Gera texturas em pixel art para agentes em 4 direções
 * e múltiplos quadros de animação (Idle, Walk A, Walk B, Work) sem depender de arquivos externos.
 */
export class ProceduralSpriteGenerator {
  public static ensureAgentTextures(scene: Phaser.Scene) {
    const roles = ['DEVELOPER', 'RESEARCHER', 'DEVOPS', 'BOT_RUNNER'];
    const directions: AgentDirection[] = ['SW', 'SE', 'NW', 'NE'];
    const frames = ['idle', 'walk_1', 'walk_2', 'work'];

    for (const role of roles) {
      const theme = ROLE_THEMES[role] || ROLE_THEMES.DEVELOPER;

      for (const dir of directions) {
        for (const frame of frames) {
          const textureKey = `agent_${role.toLowerCase()}_${dir}_${frame}`;
          if (!scene.textures.exists(textureKey)) {
            this.generateCharacterFrame(scene, textureKey, theme, dir, frame);
          }
        }
      }
    }
  }

  private static generateCharacterFrame(
    scene: Phaser.Scene,
    key: string,
    theme: RoleTheme,
    dir: AgentDirection,
    frame: string
  ) {
    const width = 24;
    const height = 36;
    const canvas = scene.textures.createCanvas(key, width, height);
    if (!canvas) return;

    const ctx = canvas.context;
    ctx.clearRect(0, 0, width, height);

    const isBack = dir === 'NW' || dir === 'NE';
    const isFacingRight = dir === 'SE' || dir === 'NE';

    // 1. Sombra circular suave sob os pés
    ctx.fillStyle = 'rgba(0, 0, 0, 0.4)';
    ctx.beginPath();
    ctx.ellipse(12, 33, 8, 4, 0, 0, Math.PI * 2);
    ctx.fill();

    // 2. Pernas (Animação de Caminhada)
    const legOffset = frame === 'walk_1' ? -3 : frame === 'walk_2' ? 3 : 0;
    ctx.fillStyle = this.colorToHex(theme.bodyColor);

    // Perna esquerda
    ctx.fillRect(8, 22, 3, 10 + legOffset);
    // Perna direita
    ctx.fillRect(13, 22, 3, 10 - legOffset);

    // Sapatos reforçados
    ctx.fillStyle = '#05080f';
    ctx.fillRect(8, 31 + legOffset, 4, 3);
    ctx.fillRect(13, 31 - legOffset, 4, 3);

    // 3. Tronco e Jaqueta/Jaleco
    ctx.fillStyle = this.colorToHex(theme.coatColor);
    ctx.fillRect(7, 11, 10, 12);

    // Faixa/zíper néon de acento
    ctx.fillStyle = this.colorToHex(theme.accentColor);
    if (!isBack) {
      ctx.fillRect(11, 12, 2, 10);
    } else {
      // Mochila técnica / bateria dorsal quando visto de costas
      ctx.fillStyle = '#0a101d';
      ctx.fillRect(8, 12, 8, 8);
      ctx.fillStyle = this.colorToHex(theme.accentColor);
      ctx.fillRect(11, 14, 2, 4); // LED da mochila
    }

    // 4. Braços (com animação de digitação em 'work')
    ctx.fillStyle = this.colorToHex(theme.coatColor);
    if (frame === 'work') {
      // Braços projetados para frente operando console holográfico
      ctx.fillRect(5, 12, 3, 6);
      ctx.fillRect(16, 12, 3, 6);
      ctx.fillRect(7, 17, 10, 2);

      // Console holográfico flutuante
      ctx.fillStyle = 'rgba(0, 240, 255, 0.85)';
      ctx.fillRect(6, 15, 12, 2);
    } else {
      const armSwing = frame === 'walk_1' ? 2 : frame === 'walk_2' ? -2 : 0;
      ctx.fillRect(4, 12 - armSwing, 3, 9);
      ctx.fillRect(17, 12 + armSwing, 3, 9);
    }

    // 5. Cabeça e Rosto
    ctx.fillStyle = this.colorToHex(theme.skinColor);
    ctx.fillRect(8, 4, 8, 8);

    // Cabelo / Capuz / Capacete
    ctx.fillStyle = isBack ? '#1a202c' : '#2d3748';
    ctx.fillRect(8, 3, 8, 3);
    ctx.fillRect(7, 4, 2, 4);
    ctx.fillRect(15, 4, 2, 4);

    // Visor Cibernético / Olhos HUD (quando virado de frente)
    if (!isBack) {
      ctx.fillStyle = this.colorToHex(theme.visorColor);
      if (isFacingRight) {
        ctx.fillRect(10, 6, 5, 2);
      } else {
        ctx.fillRect(9, 6, 5, 2);
      }
    }

    canvas.refresh();
  }

  private static colorToHex(num: number): string {
    return '#' + num.toString(16).padStart(6, '0');
  }
}
