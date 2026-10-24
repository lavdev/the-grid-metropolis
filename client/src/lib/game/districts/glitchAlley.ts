import Phaser from 'phaser';
import { cartesianToIsometric } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

interface SparkParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  color: number;
}

/**
 * GlitchAlleySector: Setor de quarentena, depuração e exceções da metrópole (The Glitch Alleys).
 * Apresenta cabos rompidos soltando faíscas, telas com estática/ruído de sinal,
 * faixas de perigo (hazard stripes) e balizas vermelhas de advertência.
 */
export class GlitchAlleySector {
  private scene: Phaser.Scene;
  public readonly cartX: number;
  public readonly cartY: number;

  private structureGraphics: Phaser.GameObjects.Graphics;
  private staticScreenGraphics: Phaser.GameObjects.Graphics;
  private sparkGraphics: Phaser.GameObjects.Graphics;

  private sparks: SparkParticle[] = [];
  private animTimer = 0;
  private alertBlink = 0;

  constructor(scene: Phaser.Scene, cartX: number = 0, cartY: number = -12) {
    this.scene = scene;
    this.cartX = cartX;
    this.cartY = cartY;

    this.structureGraphics = this.scene.add.graphics();
    this.staticScreenGraphics = this.scene.add.graphics();
    this.sparkGraphics = this.scene.add.graphics();

    this.renderQuarantineStructures();
  }

  public renderQuarantineStructures() {
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const baseDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_BASE);
    const roofDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF);

    this.structureGraphics.setDepth(baseDepth);
    this.staticScreenGraphics.setDepth(baseDepth + 4);
    this.sparkGraphics.setDepth(roofDepth + 5);

    const g = this.structureGraphics;
    g.clear();

    // 1. Barreira de quarentena com faixas de advertência
    const barW = 28;
    const barH = 14;

    g.fillStyle(0x1a0808, 0.95);
    g.fillRect(isoX - barW, isoY - 16, barW * 2, 16);
    g.lineStyle(1.5, 0xff3b30, 0.9);
    g.strokeRect(isoX - barW, isoY - 16, barW * 2, 16);

    // Listras diagonais de perigo (Hazard stripes)
    g.lineStyle(2, 0xff9500, 0.7);
    for (let x = -barW + 4; x < barW; x += 8) {
      g.strokeLineShape(new Phaser.Geom.Line(isoX + x, isoY, isoX + x + 6, isoY - 16));
    }

    // 2. Fios e cabos expostos rompidos
    g.lineStyle(1.5, 0x334155, 1);
    g.beginPath();
    g.moveTo(isoX - 22, isoY);
    g.lineTo(isoX - 16, isoY - 6);
    g.lineTo(isoX - 10, isoY - 2);
    g.strokePath();

    g.beginPath();
    g.moveTo(isoX + 12, isoY);
    g.lineTo(isoX + 18, isoY - 4);
    g.lineTo(isoX + 24, isoY - 1);
    g.strokePath();

    // 3. Totem de monitor de diagnóstico com ruído
    const monX = isoX - 16;
    const monY = isoY - 34;
    g.fillStyle(0x0f0505, 1);
    g.fillRect(monX, monY, 18, 12);
    g.lineStyle(1, 0xff3b30, 0.8);
    g.strokeRect(monX, monY, 18, 12);

    // Baliza de alarme no topo do totem
    g.fillStyle(0xff3b30, 1);
    g.fillCircle(monX + 9, monY - 4, 3);
  }

  /**
   * Atualização contínua de faíscas elétricas e estática na tela
   */
  public update(delta: number) {
    this.animTimer += delta;
    this.alertBlink += delta * 0.005;

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);

    // 1. Gera faíscas aleatórias no cabo rompido
    if (this.animTimer > 150) {
      this.animTimer = 0;
      const sparkX = isoX - 10 + Phaser.Math.FloatBetween(-2, 2);
      const sparkY = isoY - 2;

      for (let s = 0; s < 3; s++) {
        this.sparks.push({
          x: sparkX,
          y: sparkY,
          vx: Phaser.Math.FloatBetween(-1.2, 1.2),
          vy: Phaser.Math.FloatBetween(-1.5, -0.2),
          life: 0,
          maxLife: 20,
          color: Math.random() > 0.5 ? 0x00f0ff : 0xffcc00,
        });
      }
    }

    // 2. Renderiza faíscas
    const spg = this.sparkGraphics;
    spg.clear();

    for (let i = this.sparks.length - 1; i >= 0; i--) {
      const sp = this.sparks[i];
      sp.x += sp.vx;
      sp.y += sp.vy;
      sp.vy += 0.08; // Gravidade
      sp.life++;

      if (sp.life >= sp.maxLife) {
        this.sparks.splice(i, 1);
        continue;
      }

      spg.fillStyle(sp.color, 1 - sp.life / sp.maxLife);
      spg.fillRect(sp.x, sp.y, 2, 2);
    }

    // 3. Renderiza estática na tela do monitor de diagnóstico
    const sc = this.staticScreenGraphics;
    sc.clear();

    const monX = isoX - 16;
    const monY = isoY - 34;
    const monW = 16;
    const monH = 10;

    // Fundo da tela
    sc.fillStyle(0x220909, 0.9);
    sc.fillRect(monX + 1, monY + 1, monW, monH);

    // Ruído e estática estocástica
    sc.fillStyle(0xff3b30, 0.7);
    for (let px = 0; px < monW; px += 3) {
      for (let py = 0; py < monH; py += 2) {
        if (Math.random() > 0.6) {
          sc.fillRect(monX + 1 + px, monY + 1 + py, 2, 1.5);
        }
      }
    }

    // Luz intermitente da baliza de alerta
    const beaconAlpha = 0.4 + Math.sin(this.alertBlink) * 0.5;
    sc.fillStyle(0xff3b30, Math.max(0.1, beaconAlpha));
    sc.fillCircle(monX + 9, monY - 4, 5);
  }

  public destroy() {
    this.structureGraphics.destroy();
    this.staticScreenGraphics.destroy();
    this.sparkGraphics.destroy();
    this.sparks = [];
  }
}
