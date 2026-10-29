import Phaser from 'phaser';

export type AgentDirection = 'SW' | 'SE' | 'NW' | 'NE';

export interface AgentVisualTheme {
  bodyColor: string;       // Calça / techwear
  coatColor: string;       // Jaqueta cibernética principal
  coatTrimColor: string;   // Detalhes da jaqueta / ombreiras
  accentColor: string;     // Luz néon de assinatura
  hairColor: string;       // Cabelo / capuz / chassis
  skinColor: string;       // Tom de pele / metal
  visorColor: string;      // Visor / óculos HUD / LEDs ópticos
  shoeColor: string;       // Tênis cibernéticos de sola grossa
  hairStyle: 'hacker_spikes' | 'sleek_researcher' | 'tactical_helmet' | 'android_mesh' | 'neon_bob' | 'cyber_ponytail';
}

export const AGENT_THEMES: Record<string, AgentVisualTheme> = {
  // Rusty (Hacker Rust / Dev Lead)
  'agent-codex-01': {
    bodyColor: '#0c1626',
    coatColor: '#17253b',
    coatTrimColor: '#00f0ff',
    accentColor: '#00f0ff',
    hairColor: '#4a2c11',
    skinColor: '#f7cbab',
    visorColor: '#00f0ff',
    shoeColor: '#050a14',
    hairStyle: 'hacker_spikes',
  },
  // Dr. Cipher (Arquiteto & Pesquisador Claude)
  'agent-claude-02': {
    bodyColor: '#121224',
    coatColor: '#f0f4f8',    // Jaleco branco de laboratório
    coatTrimColor: '#9d00ff',
    accentColor: '#9d00ff',
    hairColor: '#d1d5db',    // Cabelo grisalho sofisticado
    skinColor: '#f3c19d',
    visorColor: '#9d00ff',   // HUD roxo
    shoeColor: '#1a1a2e',
    hairStyle: 'sleek_researcher',
  },
  // Nova (Desenvolvedora DeepSeek R1)
  'agent-nova-03': {
    bodyColor: '#1a1424',
    coatColor: '#2b1b3d',
    coatTrimColor: '#ff9900',
    accentColor: '#ff9900',
    hairColor: '#ff6b35',    // Cabelo ruivo néon
    skinColor: '#fcd5b8',
    visorColor: '#ffaa00',
    shoeColor: '#0f0a17',
    hairStyle: 'neon_bob',
  },
  // Kaelen (Pesquisador Gemini 2.0)
  'agent-kaelen-04': {
    bodyColor: '#101726',
    coatColor: '#1e293b',
    coatTrimColor: '#ff007f',
    accentColor: '#ff007f',
    hairColor: '#382247',
    skinColor: '#f5cba7',
    visorColor: '#ff007f',   // Visor magenta
    shoeColor: '#0b0f19',
    hairStyle: 'cyber_ponytail',
  },
  // Vanguard (Segurança & Red-Team)
  'agent-sec-05': {
    bodyColor: '#120707',
    coatColor: '#260f0f',    // Armadura balística escura
    coatTrimColor: '#ff3b30',
    accentColor: '#ff3b30',
    hairColor: '#1a1a1a',
    skinColor: '#d69e7a',
    visorColor: '#ff3b30',   // Visor tático vermelho
    shoeColor: '#080303',
    hairStyle: 'tactical_helmet',
  },
  // Unit-7 (Androide CI/CD Automaton)
  'agent-bot-01': {
    bodyColor: '#27303f',    // Chassis cibernético cinza chumbo
    coatColor: '#3b485d',
    coatTrimColor: '#39ff14',
    accentColor: '#39ff14',
    hairColor: '#1a202c',    // Placa metálica superior
    skinColor: '#718096',    // Rosto sintético
    visorColor: '#39ff14',   // Matriz ótica verde néon
    shoeColor: '#161d27',
    hairStyle: 'android_mesh',
  },
};

// Fallback por Role
export const ROLE_FALLBACKS: Record<string, string> = {
  DEVELOPER: 'agent-codex-01',
  RESEARCHER: 'agent-claude-02',
  SECURITY: 'agent-sec-05',
  BOT_RUNNER: 'agent-bot-01',
  DEVOPS: 'agent-codex-01',
};

/**
 * ProceduralSpriteGenerator: Gera spritesheets de alta fidelidade isométrica 2:1 no estilo
 * Habbo / Habblaud, com proporções substanciais para preencher adequadamente as cadeiras gamer,
 * mesas de monitores e mobília do Quartel-General Hacker.
 */
export class ProceduralSpriteGenerator {
  public static ensureAgentTextures(scene: Phaser.Scene) {
    const directions: AgentDirection[] = ['SW', 'SE', 'NW', 'NE'];
    const frames = ['idle', 'walk_1', 'walk_2', 'work'];

    for (const [agentId, theme] of Object.entries(AGENT_THEMES)) {
      for (const dir of directions) {
        for (const frame of frames) {
          const textureKey = `agent_${agentId}_${dir.toLowerCase()}_${frame}`;
          if (!scene.textures.exists(textureKey)) {
            this.generateCharacterFrame(scene, textureKey, theme, dir, frame);
          }
        }
      }
    }

    // Gera também aliases genéricos por role
    const roles = ['DEVELOPER', 'RESEARCHER', 'SECURITY', 'BOT_RUNNER', 'DEVOPS'];
    for (const role of roles) {
      const fallbackId = ROLE_FALLBACKS[role] || 'agent-codex-01';
      const theme = AGENT_THEMES[fallbackId];
      for (const dir of directions) {
        for (const frame of frames) {
          const aliasKey = `agent_${role.toLowerCase()}_${dir.toLowerCase()}_${frame}`;
          if (!scene.textures.exists(aliasKey)) {
            this.generateCharacterFrame(scene, aliasKey, theme, dir, frame);
          }
        }
      }
    }
  }

  private static generateCharacterFrame(
    scene: Phaser.Scene,
    key: string,
    theme: AgentVisualTheme,
    dir: AgentDirection,
    frame: string
  ) {
    // Dimensões do canvas: 36px largura x 52px altura (proporção substancial Habbo)
    const width = 36;
    const height = 52;
    const canvas = scene.textures.createCanvas(key, width, height);
    if (!canvas) return;

    const ctx = canvas.context;
    ctx.clearRect(0, 0, width, height);

    const isBack = dir === 'NW' || dir === 'NE';
    const isFacingRight = dir === 'SE' || dir === 'NE';
    const isWorking = frame === 'work';

    // 1. Sombra suave sob os pés / cadeira
    ctx.fillStyle = 'rgba(0, 0, 0, 0.55)';
    ctx.beginPath();
    ctx.ellipse(18, 48, 12, 5.5, 0, 0, Math.PI * 2);
    ctx.fill();

    // 2. Pernas & Tênis (Sentado na cadeira gamer em 'work' ou em pé/caminhando)
    if (isWorking) {
      // POSTURA SENTADA:
      // Pés e coxas dobrados em 90 graus apoiados no assento da cadeira
      ctx.fillStyle = theme.bodyColor;
      ctx.fillRect(10, 32, 7, 10);
      ctx.fillRect(19, 32, 7, 10);

      // Joelhos projetados para a frente em direção à mesa
      ctx.fillStyle = theme.coatTrimColor;
      ctx.fillRect(9, 36, 3, 4);
      ctx.fillRect(24, 36, 3, 4);

      // Tênis cibernéticos recolhidos
      ctx.fillStyle = theme.shoeColor;
      ctx.fillRect(10, 42, 7, 4);
      ctx.fillRect(19, 42, 7, 4);

      // Sola iluminada em néon dos tênis
      ctx.fillStyle = theme.accentColor;
      ctx.fillRect(10, 45, 7, 2);
      ctx.fillRect(19, 45, 7, 2);
    } else {
      // POSTURA EM PÉ / CAMINHADA:
      const legOffset = frame === 'walk_1' ? -4 : frame === 'walk_2' ? 4 : 0;

      // Perna Esquerda
      ctx.fillStyle = theme.bodyColor;
      ctx.fillRect(11, 30, 6, 14 + legOffset);
      // Perna Direita
      ctx.fillRect(19, 30, 6, 14 - legOffset);

      // Tiras e detalhes techwear nas pernas
      ctx.fillStyle = theme.coatTrimColor;
      ctx.fillRect(11, 35 + Math.floor(legOffset / 2), 6, 2);
      ctx.fillRect(19, 35 - Math.floor(legOffset / 2), 6, 2);

      // Tênis cibernéticos de cano alto
      ctx.fillStyle = theme.shoeColor;
      ctx.fillRect(10, 43 + legOffset, 7, 5);
      ctx.fillRect(19, 43 - legOffset, 7, 5);

      // Solado de luz néon assinatura nos tênis
      ctx.fillStyle = theme.accentColor;
      ctx.fillRect(10, 47 + legOffset, 7, 2);
      ctx.fillRect(19, 47 - legOffset, 7, 2);
    }

    // Rebaixamento do tronco quando sentado: 5 pixels para encaixar na almofada da cadeira
    const yOff = isWorking ? 5 : 0;

    // 3. Tronco e Jaqueta Cyberpunk
    ctx.fillStyle = theme.coatColor;
    ctx.fillRect(9, 16 + yOff, 18, 16);

    // Ombreiras com recorte angular
    ctx.fillStyle = theme.coatTrimColor;
    ctx.fillRect(7, 17 + yOff, 3, 6);
    ctx.fillRect(26, 17 + yOff, 3, 6);

    if (!isBack) {
      // Visão Frontal (SW, SE): zíper central aberto com camiseta de dados luminosa
      ctx.fillStyle = '#050a14';
      ctx.fillRect(16, 17 + yOff, 4, 14);

      // Linha néon luminosa do reator ou zíper
      ctx.fillStyle = theme.accentColor;
      ctx.fillRect(17, 18 + yOff, 2, 12);
    } else {
      // Visão Dorsal (NW, NE): Mochila técnica / Unidade de processamento neural
      ctx.fillStyle = '#090f1d';
      ctx.fillRect(11, 18 + yOff, 14, 11);

      // LEDs verticais de status da mochila dorsal
      ctx.fillStyle = theme.accentColor;
      ctx.fillRect(13, 20 + yOff, 2, 6);
      ctx.fillRect(21, 20 + yOff, 2, 6);

      // Grade de exaustão térmica
      ctx.fillStyle = '#1e293b';
      ctx.fillRect(16, 22 + yOff, 4, 4);
    }

    // 4. Braços e Digitação no Teclado
    ctx.fillStyle = theme.coatColor;
    if (isWorking) {
      if (isBack) {
        // Visto de costas virado para os monitores (NW, NE):
        // Braços estendidos para a frente na mesa
        ctx.fillRect(7, 18 + yOff, 4, 7);
        ctx.fillRect(25, 18 + yOff, 4, 7);
        ctx.fillRect(8, 23 + yOff, 20, 3);

        // Teclado mecânico RGB à frente na mesa
        ctx.fillStyle = '#0a0f1d';
        ctx.fillRect(8, 20 + yOff, 20, 3);
        ctx.fillStyle = '#00f0ff';
        ctx.fillRect(10, 21 + yOff, 4, 1);
        ctx.fillStyle = '#39ff14';
        ctx.fillRect(16, 21 + yOff, 4, 1);
        ctx.fillStyle = '#ff007f';
        ctx.fillRect(22, 21 + yOff, 4, 1);
      } else {
        // Visto de frente (SW, SE):
        ctx.fillRect(6, 18 + yOff, 4, 9);
        ctx.fillRect(26, 18 + yOff, 4, 9);
        // Antebraços projetados sobre a mesa
        ctx.fillRect(7, 26 + yOff, 22, 4);

        // Mãos com luvas técnicas
        ctx.fillStyle = theme.skinColor;
        ctx.fillRect(11, 29 + yOff, 4, 3);
        ctx.fillRect(21, 29 + yOff, 4, 3);

        // TECLADO MECÂNICO RGB ILUMINADO:
        ctx.fillStyle = '#0a0f1d';
        ctx.fillRect(8, 32 + yOff, 20, 3);
        ctx.fillStyle = '#00f0ff'; // Teclas ciano
        ctx.fillRect(10, 33 + yOff, 4, 1);
        ctx.fillStyle = '#39ff14'; // Teclas verde
        ctx.fillRect(16, 33 + yOff, 4, 1);
        ctx.fillStyle = '#ff007f'; // Teclas magenta
        ctx.fillRect(22, 33 + yOff, 4, 1);
      }
    } else {
      // BRAÇOS EM PÉ / BALANÇO DE CAMINHADA:
      const armSwing = frame === 'walk_1' ? 3 : frame === 'walk_2' ? -3 : 0;
      ctx.fillRect(6, 18 - armSwing, 4, 12);
      ctx.fillRect(26, 18 + armSwing, 4, 12);

      // Mãos
      ctx.fillStyle = theme.skinColor;
      ctx.fillRect(6, 29 - armSwing, 4, 3);
      ctx.fillRect(26, 29 + armSwing, 4, 3);
    }

    // 5. Cabeça, Rosto e Cabelo (Estilo Habbo / Habblaud)
    const headX = 11;
    const headY = 6 + yOff;
    const headW = 14;
    const headH = 12;

    if (!isBack) {
      // Visão Frontal (SW, SE): Rosto (Tom de pele)
      ctx.fillStyle = theme.skinColor;
      ctx.fillRect(headX, headY, headW, headH);
    } else {
      // Visão Dorsal (NW, NE): Nuca e cabelo de trás
      ctx.fillStyle = theme.hairColor;
      ctx.fillRect(headX, headY, headW, headH);
    }

    // Cabelo / Capacete / Penteado de cada personagem
    ctx.fillStyle = theme.hairColor;
    switch (theme.hairStyle) {
      case 'hacker_spikes':
        // Cabelo espetado com pontas cyberpunk
        ctx.fillRect(headX - 1, headY - 3, headW + 2, 5);
        ctx.fillRect(headX, headY - 5, 4, 3);
        ctx.fillRect(headX + 5, headY - 6, 4, 4);
        ctx.fillRect(headX + 10, headY - 4, 4, 3);
        ctx.fillRect(headX - 2, headY, 3, 8); // Mechas laterais
        ctx.fillRect(headX + headW - 1, headY, 3, 8);
        if (isBack) {
          // Mechas pontudas na parte de trás da cabeça
          ctx.fillRect(headX + 3, headY + 3, 8, 8);
        }
        break;

      case 'sleek_researcher':
        // Cabelo alinhado com mechas futuristas
        ctx.fillRect(headX - 1, headY - 3, headW + 2, 5);
        ctx.fillRect(headX - 2, headY, 3, 10);
        ctx.fillRect(headX + headW - 1, headY, 3, 10);
        if (isBack) {
          ctx.fillRect(headX, headY, headW, headH);
        }
        break;

      case 'tactical_helmet':
        // Capacete tático reforçado cobrindo a cabeça
        ctx.fillRect(headX - 2, headY - 4, headW + 4, 7);
        ctx.fillRect(headX - 2, headY, 3, 10);
        ctx.fillRect(headX + headW - 1, headY, 3, 10);
        if (isBack) {
          ctx.fillStyle = '#080808';
          ctx.fillRect(headX, headY, headW, headH);
          ctx.fillStyle = theme.accentColor;
          ctx.fillRect(headX + 4, headY + 4, 6, 2); // LED tático dorsal do capacete
        } else {
          ctx.fillStyle = '#080808';
          ctx.fillRect(headX, headY - 2, headW, 3);
        }
        break;

      case 'neon_bob':
        // Corte bob com franja moderna
        ctx.fillRect(headX - 2, headY - 3, headW + 4, 5);
        ctx.fillRect(headX - 2, headY, 4, 11);
        ctx.fillRect(headX + headW - 2, headY, 4, 11);
        if (!isBack) {
          ctx.fillRect(headX + 2, headY - 1, headW - 4, 3); // Franja
        } else {
          ctx.fillRect(headX, headY, headW, headH); // Parte de trás volumosa
        }
        break;

      case 'cyber_ponytail':
        // Rabo de cavalo alto
        ctx.fillRect(headX - 1, headY - 3, headW + 2, 5);
        ctx.fillRect(headX - 2, headY, 3, 8);
        ctx.fillRect(headX + headW - 1, headY, 3, 8);
        if (isBack) {
          ctx.fillRect(headX + 4, headY - 5, 6, 12); // Rabo de cavalo dorsal
        } else {
          ctx.fillRect(headX + (isFacingRight ? -4 : headW + 1), headY - 4, 4, 10);
        }
        break;

      case 'android_mesh':
      default:
        // Placa cibernética lisa
        ctx.fillRect(headX - 1, headY - 3, headW + 2, 5);
        ctx.fillRect(headX - 2, headY, 3, 8);
        ctx.fillRect(headX + headW - 1, headY, 3, 8);
        if (isBack) {
          ctx.fillStyle = theme.accentColor;
          ctx.fillRect(headX + 3, headY + 3, 8, 2); // Porta de dados USB/cibernética
        }
        break;
    }

    // Visor Cibernético / Olhos HUD (quando virado de frente)
    if (!isBack) {
      ctx.fillStyle = theme.visorColor;
      const eyeY = headY + 4;
      if (isFacingRight) {
        ctx.fillRect(headX + 4, eyeY, 8, 3);
      } else {
        ctx.fillRect(headX + 2, eyeY, 8, 3);
      }

      // Brilho do visor
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(headX + (isFacingRight ? 8 : 4), eyeY, 2, 1);
    } else {
      // Fone / Headset de comunicação na lateral da cabeça
      ctx.fillStyle = theme.accentColor;
      ctx.fillRect(headX + (isFacingRight ? headW - 1 : -1), headY + 3, 2, 4);
    }

    canvas.refresh();
  }
}
