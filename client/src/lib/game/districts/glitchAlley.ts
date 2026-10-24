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
 * GlitchAlleySector: Edificação do The Glitch Alleys (Bunkers de Quarentena e Depuração).
 * Um complexo denso de blocos de servidores modulares sobrepostos em estilo industrial,
 * com faixas de advertência, subestação elétrica com transformadores soltando faíscas,
 * telas de diagnóstico quebradas com estática e balizas de alerta.
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

    this.renderQuarantineComplex();
  }

  public renderQuarantineComplex() {
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const baseDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_BASE);
    const roofDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF);

    this.structureGraphics.setDepth(baseDepth);
    this.staticScreenGraphics.setDepth(baseDepth + 5);
    this.sparkGraphics.setDepth(roofDepth + 6);

    const g = this.structureGraphics;
    g.clear();

    const halfW = 44;
    const halfH = 22;
    const wallH = 34; // Bloco volumétrico principal

    // ==========================================
    // 1. BLOCO PRINCIPAL DO BUNKER (PAREDES)
    // ==========================================

    // Parede Sudoeste (Face Esquerda)
    g.fillStyle(0x190a0a, 0.98); // Metal escuro oxidado
    g.beginPath();
    g.moveTo(isoX - halfW, isoY);
    g.lineTo(isoX, isoY + halfH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.lineTo(isoX - halfW, isoY - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x3d1414, 0.9);
    g.strokePath();

    // Faixas de Perigo Diagonais (Hazard Stripes) na parede esquerda
    g.lineStyle(2, 0xff9900, 0.75);
    for (let x = -halfW + 8; x < 0; x += 10) {
      const p1y = isoY + (x + halfW) * 0.5;
      g.strokeLineShape(new Phaser.Geom.Line(isoX + x, p1y - 8, isoX + x + 6, p1y - 20));
    }

    // Parede Sudeste (Face Direita - Acesso)
    g.fillStyle(0x100505, 0.98);
    g.beginPath();
    g.moveTo(isoX, isoY + halfH);
    g.lineTo(isoX + halfW, isoY);
    g.lineTo(isoX + halfW, isoY - wallH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x3d1414, 0.9);
    g.strokePath();

    // Porta Blindada de Quarentena
    const doorW = 18;
    const doorH = 22;
    const doorX = isoX + 10;
    const doorY = isoY + halfH - 24;

    g.fillStyle(0x050202, 1);
    g.fillRect(doorX, doorY, doorW, doorH);
    g.lineStyle(1.5, 0xff3b30, 0.95);
    g.strokeRect(doorX, doorY, doorW, doorH);

    // ==========================================
    // 2. LAJE SUPERIOR DO BUNKER E BLOCO SOBREPOSTO
    // ==========================================
    const roofY = isoY - wallH;

    // Laje principal
    g.fillStyle(0x220e0e, 1);
    g.beginPath();
    g.moveTo(isoX, roofY - halfH);
    g.lineTo(isoX + halfW, roofY);
    g.lineTo(isoX, roofY + halfH);
    g.lineTo(isoX - halfW, roofY);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0xff3b30, 0.8);
    g.strokePath();

    // Bloco Modular Superior (Container Sobreposto)
    const cW = 24;
    const cH = 12;
    const cWallH = 16;
    const cX = isoX - 10;
    const cY = roofY - 4;

    // Parede esquerda do container sobreposto
    g.fillStyle(0x2d1010, 1);
    g.beginPath();
    g.moveTo(cX - cW, cY);
    g.lineTo(cX, cY + cH);
    g.lineTo(cX, cY + cH - cWallH);
    g.lineTo(cX - cW, cY - cWallH);
    g.closePath();
    g.fillPath();

    // Parede direita do container
    g.fillStyle(0x1a0808, 1);
    g.beginPath();
    g.moveTo(cX, cY + cH);
    g.lineTo(cX + cW, cY);
    g.lineTo(cX + cW, cY - cWallH);
    g.lineTo(cX, cY + cH - cWallH);
    g.closePath();
    g.fillPath();

    // Teto do container
    g.fillStyle(0x361414, 1);
    g.beginPath();
    g.moveTo(cX, cY - cWallH - cH);
    g.lineTo(cX + cW, cY - cWallH);
    g.lineTo(cX, cY + cH - cWallH);
    g.lineTo(cX - cW, cY - cWallH);
    g.closePath();
    g.fillPath();
    g.lineStyle(1, 0xff3b30, 0.7);
    g.strokePath();

    // ==========================================
    // 3. TRANSFORMADOR ELÉTRICO E CABOS ROMPIDOS
    // ==========================================
    const transX = isoX + 16;
    const transY = roofY + 2;

    // Subestação
    g.fillStyle(0x1c1a1a, 1);
    g.fillRect(transX - 6, transY - 14, 12, 14);
    g.lineStyle(1.5, 0xffcc00, 0.9);
    g.strokeRect(transX - 6, transY - 14, 12, 14);

    // Cabos soltos descendo do teto
    g.lineStyle(1.5, 0x475569, 1);
    g.beginPath();
    g.moveTo(transX, transY);
    g.lineTo(isoX + 6, isoY + halfH - 8);
    g.lineTo(isoX - 8, isoY + halfH - 2);
    g.strokePath();
  }

  public update(delta: number) {
    this.animTimer += delta;
    this.alertBlink += delta * 0.006;

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const wallH = 34;
    const roofY = isoY - wallH;
    const transX = isoX + 16;
    const transY = roofY - 14;

    // 1. Gera faíscas elétricas saindo do transformador
    if (this.animTimer > 110) {
      this.animTimer = 0;
      for (let s = 0; s < 3; s++) {
        this.sparks.push({
          x: transX + Phaser.Math.FloatBetween(-2, 2),
          y: transY,
          vx: Phaser.Math.FloatBetween(-1.5, 1.5),
          vy: Phaser.Math.FloatBetween(-2.0, -0.4),
          life: 0,
          maxLife: 22,
          color: Math.random() > 0.4 ? 0x00f0ff : 0xffcc00,
        });
      }
    }

    const spg = this.sparkGraphics;
    spg.clear();

    for (let i = this.sparks.length - 1; i >= 0; i--) {
      const sp = this.sparks[i];
      sp.x += sp.vx;
      sp.y += sp.vy;
      sp.vy += 0.09;
      sp.life++;

      if (sp.life >= sp.maxLife) {
        this.sparks.splice(i, 1);
        continue;
      }

      spg.fillStyle(sp.color, 1 - sp.life / sp.maxLife);
      spg.fillRect(sp.x, sp.y, 2, 2);
    }

    // 2. Monitor CRT de diagnóstico com estática na fachada
    const sc = this.staticScreenGraphics;
    sc.clear();

    const monX = isoX - 28;
    const monY = isoY - 16;
    const monW = 16;
    const monH = 11;

    sc.fillStyle(0x180505, 0.95);
    sc.fillRect(monX, monY, monW, monH);
    sc.lineStyle(1.5, 0xff3b30, 0.9);
    sc.strokeRect(monX, monY, monW, monH);

    sc.fillStyle(0xff3b30, 0.75);
    for (let px = 0; px < monW - 2; px += 3) {
      for (let py = 0; py < monH - 2; py += 2) {
        if (Math.random() > 0.55) {
          sc.fillRect(monX + 1 + px, monY + 1 + py, 2, 1.5);
        }
      }
    }

    // Baliza de alarme piscante no topo do bunker
    const beaconAlpha = 0.35 + Math.sin(this.alertBlink) * 0.65;
    sc.fillStyle(0xff3b30, Math.max(0.1, beaconAlpha));
    sc.fillCircle(isoX - 10, roofY - 24, 4);
  }

  public destroy() {
    this.structureGraphics.destroy();
    this.staticScreenGraphics.destroy();
    this.sparkGraphics.destroy();
    this.sparks = [];
  }
}
