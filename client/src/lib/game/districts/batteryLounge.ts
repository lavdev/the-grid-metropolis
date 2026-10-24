import Phaser from 'phaser';
import { cartesianToIsometric } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

/**
 * BatteryLoungeSector: Edificação do The Battery Lounge (Club & Recharge Hub).
 * Um edifício retrofuturista de 2 pavimentos com fachada escura e frisos magenta,
 * janelas fumê com iluminação interna, marquise de entrada com luz néon,
 * terraço superior com o balcão iluminado e grande totem vertical luminoso.
 */
export class BatteryLoungeSector {
  private scene: Phaser.Scene;
  public readonly cartX: number;
  public readonly cartY: number;

  private buildingGraphics: Phaser.GameObjects.Graphics;
  private rooftopGraphics: Phaser.GameObjects.Graphics;
  private neonSignGraphics: Phaser.GameObjects.Graphics;
  private signText?: Phaser.GameObjects.Text;
  private animTimer = 0;
  private neonFlicker = 1.0;

  constructor(scene: Phaser.Scene, cartX: number = -12, cartY: number = 0) {
    this.scene = scene;
    this.cartX = cartX;
    this.cartY = cartY;

    this.buildingGraphics = this.scene.add.graphics();
    this.rooftopGraphics = this.scene.add.graphics();
    this.neonSignGraphics = this.scene.add.graphics();

    this.renderBuilding();
  }

  public renderBuilding() {
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const baseDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_BASE);
    const roofDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF);

    this.buildingGraphics.setDepth(baseDepth);
    this.rooftopGraphics.setDepth(roofDepth);
    this.neonSignGraphics.setDepth(roofDepth + 5);

    const halfW = 44;
    const halfH = 22;
    const wallH = 40; // Edificação de 2 andares

    const bg = this.buildingGraphics;
    bg.clear();

    // ==========================================
    // 1. FACHADAS VOLUMÉTRICAS (PAREDES)
    // ==========================================

    // Parede Sudoeste (Face Esquerda)
    bg.fillStyle(0x130a1c, 0.98); // Obsidiana com tom roxo
    bg.beginPath();
    bg.moveTo(isoX - halfW, isoY);
    bg.lineTo(isoX, isoY + halfH);
    bg.lineTo(isoX, isoY + halfH - wallH);
    bg.lineTo(isoX - halfW, isoY - wallH);
    bg.closePath();
    bg.fillPath();

    bg.lineStyle(1.5, 0x2b153d, 0.9);
    bg.strokePath();

    // Janelas de vidro fumê com néon violeta/magenta (2 andares)
    for (let floor = 0; floor < 2; floor++) {
      const fOffsetY = floor * 16 + 8;
      for (let w = 0; w < 3; w++) {
        const wx = isoX - halfW + 10 + w * 11;
        const wy = isoY + (w * 5.5) + 3 - fOffsetY;
        bg.fillStyle(0xff007f, 0.85);
        bg.fillRect(wx, wy, 7, 4);
      }
    }

    // Parede Sudeste (Face Direita - Fachada Principal)
    bg.fillStyle(0x0c0612, 0.98);
    bg.beginPath();
    bg.moveTo(isoX, isoY + halfH);
    bg.lineTo(isoX + halfW, isoY);
    bg.lineTo(isoX + halfW, isoY - wallH);
    bg.lineTo(isoX, isoY + halfH - wallH);
    bg.closePath();
    bg.fillPath();

    bg.lineStyle(1.5, 0x2b153d, 0.9);
    bg.strokePath();

    // Marquise e Entrada do Lounge
    const entranceW = 20;
    const entranceH = 22;
    const entX = isoX + 10;
    const entY = isoY + halfH - 24;

    bg.fillStyle(0x050208, 1);
    bg.fillRect(entX, entY, entranceW, entranceH);
    bg.lineStyle(2, 0xff007f, 0.95);
    bg.strokeRect(entX, entY, entranceW, entranceH);

    // Marquise projetada para frente
    bg.fillStyle(0x2d103f, 0.9);
    bg.fillRect(entX - 4, entY - 4, entranceW + 8, 4);
    bg.lineStyle(1.5, 0x9d00ff, 1);
    bg.strokeRect(entX - 4, entY - 4, entranceW + 8, 4);

    // ==========================================
    // 2. TERRAÇO SUPERIOR (ROOFTOP BAR & LOUNGE)
    // ==========================================
    const roofY = isoY - wallH;
    const rg = this.rooftopGraphics;
    rg.clear();

    // Piso do terraço
    rg.fillStyle(0x1a0d26, 1);
    rg.beginPath();
    rg.moveTo(isoX, roofY - halfH);
    rg.lineTo(isoX + halfW, roofY);
    rg.lineTo(isoX, roofY + halfH);
    rg.lineTo(isoX - halfW, roofY);
    rg.closePath();
    rg.fillPath();

    // Parapeito com néon magenta
    rg.lineStyle(2, 0xff007f, 0.9);
    rg.strokePath();

    // Balcão de Bebidas Sintéticas no terraço
    const barW = 22;
    const barH = 10;
    const barHeight = 8;
    const barY = roofY - 2;

    rg.fillStyle(0x100618, 1);
    rg.fillRect(isoX - barW / 2, barY - barHeight, barW, barHeight);
    rg.lineStyle(1.5, 0xff007f, 0.9);
    rg.strokeRect(isoX - barW / 2, barY - barHeight, barW, barHeight);

    // Tampo superior do balcão
    rg.fillStyle(0x351249, 1);
    rg.fillEllipse(isoX, barY - barHeight, barW, barH);
    rg.lineStyle(1, 0x9d00ff, 0.9);
    rg.strokeEllipse(isoX, barY - barHeight, barW, barH);

    // Garrafas sintéticas fluorescentes no balcão
    const bottles = [
      { dx: -6, color: 0x00f0ff },
      { dx: -2, color: 0x39ff14 },
      { dx: 3, color: 0xff007f },
    ];
    for (const b of bottles) {
      rg.fillStyle(b.color, 1);
      rg.fillRect(isoX + b.dx, barY - barHeight - 5, 2, 5);
    }

    // Banquetas do terraço
    const stoolOffsets = [-10, 0, 10];
    for (const offset of stoolOffsets) {
      const sx = isoX + offset;
      const sy = barY + 6;
      rg.lineStyle(1.5, 0x475569, 1);
      rg.strokeLineShape(new Phaser.Geom.Line(sx, sy, sx, sy - 5));
      rg.fillStyle(0xff007f, 1);
      rg.fillCircle(sx, sy - 5, 2.5);
    }

    // ==========================================
    // 3. GRANDE TOTEM LUMINOSO VERTICAL ("BATTERY")
    // ==========================================
    this.renderNeonSign(isoX, roofY);
  }

  private renderNeonSign(isoX: number, roofY: number) {
    const sg = this.neonSignGraphics;
    sg.clear();

    const signX = isoX - 32;
    const signY = roofY - 44;
    const signW = 12;
    const signH = 46;

    // Haste metálica do totem
    sg.lineStyle(2, 0x1e1528, 1);
    sg.strokeLineShape(new Phaser.Geom.Line(signX + signW / 2, signY + signH, signX + signW / 2, roofY));

    // Corpo do totem
    sg.fillStyle(0x0c0414, 0.95);
    sg.fillRect(signX, signY, signW, signH);

    // Borda néon pulsante
    sg.lineStyle(2, 0xff007f, this.neonFlicker);
    sg.strokeRect(signX, signY, signW, signH);

    // Texto do letreiro
    if (!this.signText) {
      this.signText = this.scene.add.text(signX + 2, signY + 3, 'B\nA\nT\nT\nE\nR\nY', {
        fontFamily: 'monospace',
        fontSize: '6px',
        color: '#ff007f',
        align: 'center',
        lineSpacing: 0,
      });
      this.signText.setDepth(calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF) + 6);
    }
  }

  public update(delta: number) {
    this.animTimer += delta;

    if (this.animTimer > 100) {
      this.animTimer = 0;
      this.neonFlicker = 0.75 + Math.random() * 0.25;

      const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
      const wallH = 40;
      const roofY = isoY - wallH;
      const signX = isoX - 32;
      const signY = roofY - 44;
      const signW = 12;
      const signH = 46;

      const sg = this.neonSignGraphics;
      sg.clear();

      sg.lineStyle(2, 0x1e1528, 1);
      sg.strokeLineShape(new Phaser.Geom.Line(signX + signW / 2, signY + signH, signX + signW / 2, roofY));

      sg.fillStyle(0x0c0414, 0.95);
      sg.fillRect(signX, signY, signW, signH);

      sg.lineStyle(2, 0xff007f, this.neonFlicker);
      sg.strokeRect(signX, signY, signW, signH);

      if (this.signText) {
        this.signText.setAlpha(this.neonFlicker);
      }
    }
  }

  public destroy() {
    this.buildingGraphics.destroy();
    this.rooftopGraphics.destroy();
    this.neonSignGraphics.destroy();
    if (this.signText) {
      this.signText.destroy();
    }
  }
}
