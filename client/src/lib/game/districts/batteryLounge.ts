import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

/**
 * BatteryLoungeSector: A área de recarga e ociosidade retrofuturista para agentes (The Battery Lounge).
 * Contém o balcão iluminado em magenta/néon, banquetas, dispensadores de fluidos sintéticos
 * e o letreiro vertical luminoso.
 */
export class BatteryLoungeSector {
  private scene: Phaser.Scene;
  public readonly cartX: number;
  public readonly cartY: number;

  private barGraphics: Phaser.GameObjects.Graphics;
  private neonSignGraphics: Phaser.GameObjects.Graphics;
  private signText?: Phaser.GameObjects.Text;
  private animTimer = 0;
  private neonFlicker = 1.0;

  constructor(scene: Phaser.Scene, cartX: number = -12, cartY: number = 0) {
    this.scene = scene;
    this.cartX = cartX;
    this.cartY = cartY;

    this.barGraphics = this.scene.add.graphics();
    this.neonSignGraphics = this.scene.add.graphics();

    this.renderLounge();
  }

  public renderLounge() {
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const baseDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_BASE);
    const signDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF);

    this.barGraphics.setDepth(baseDepth);
    this.neonSignGraphics.setDepth(signDepth);

    this.renderBarCounter(isoX, isoY);
    this.renderNeonSign(isoX, isoY);
  }

  /**
   * Renderiza o balcão retrô-cyberpunk, banquetas e garrafas sintéticas fluorescentes
   */
  private renderBarCounter(isoX: number, isoY: number) {
    const g = this.barGraphics;
    g.clear();

    const barW = 32;
    const barH = 16;
    const barHeight = 14;

    // 1. Base do balcão iluminado em magenta/roxo néon
    g.fillStyle(0x150b20, 0.95);
    g.fillRect(isoX - barW, isoY - barHeight, barW * 2, barHeight);

    // Borda néon inferior e superior do balcão
    g.lineStyle(2, 0xff007f, 0.9);
    g.strokeRect(isoX - barW, isoY - barHeight, barW * 2, barHeight);

    // Tampo superior do balcão
    g.fillStyle(0x2d113f, 0.95);
    g.beginPath();
    g.ellipse(isoX, isoY - barHeight, barW, barH * 0.7, 0, 0, Math.PI * 2);
    g.fillPath();

    g.lineStyle(1.5, 0x9d00ff, 0.8);
    g.strokeEllipse(isoX, isoY - barHeight, barW, barH * 0.7);

    // 2. Banquetas metálicas diante do balcão
    const stoolOffsets = [-18, 0, 18];
    for (const offset of stoolOffsets) {
      const stoolX = isoX + offset;
      const stoolY = isoY + 8;

      // Haste da banqueta
      g.lineStyle(2, 0x334155, 1);
      g.strokeLineShape(new Phaser.Geom.Line(stoolX, stoolY, stoolX, stoolY - 8));

      // Assento da banqueta com detalhe em néon magenta
      g.fillStyle(0x38123c, 1);
      g.fillCircle(stoolX, stoolY - 8, 4);
      g.lineStyle(1, 0xff007f, 0.9);
      g.strokeCircle(stoolX, stoolY - 8, 4);
    }

    // 3. Garrafas e dispensadores de bebidas sintéticas fluorescentes
    const bottles = [
      { dx: -14, color: 0x00f0ff },
      { dx: -8, color: 0x39ff14 },
      { dx: 6, color: 0xff007f },
      { dx: 12, color: 0x00f0ff },
    ];

    for (const b of bottles) {
      g.fillStyle(b.color, 0.9);
      g.fillRect(isoX + b.dx, isoY - barHeight - 7, 3, 7);
      g.lineStyle(1, 0xffffff, 0.6);
      g.strokeRect(isoX + b.dx, isoY - barHeight - 7, 3, 7);
    }
  }

  /**
   * Renderiza o letreiro vertical retrofuturista ("BATTERY")
   */
  private renderNeonSign(isoX: number, isoY: number) {
    const sg = this.neonSignGraphics;
    sg.clear();

    const signX = isoX - 36;
    const signY = isoY - 14;
    const signW = 10;
    const signH = 48;

    // Haste de suporte vertical
    sg.lineStyle(2, 0x1a2332, 1);
    sg.strokeLineShape(new Phaser.Geom.Line(signX + signW / 2, signY + signH, signX + signW / 2, isoY));

    // Placa de fundo do letreiro
    sg.fillStyle(0x0c0614, 0.95);
    sg.fillRect(signX, signY, signW, signH);

    // Contorno néon pulsante
    sg.lineStyle(1.5, 0xff007f, this.neonFlicker);
    sg.strokeRect(signX, signY, signW, signH);

    // Texto vertical do letreiro
    if (!this.signText) {
      this.signText = this.scene.add.text(signX + 2, signY + 2, 'B\nA\nT\nT\nE\nR\nY', {
        fontFamily: 'monospace',
        fontSize: '6px',
        color: '#ff007f',
        align: 'center',
        lineSpacing: 0,
      });
      this.signText.setDepth(calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF) + 1);
    }
  }

  /**
   * Atualização contínua de oscilação do letreiro néon
   */
  public update(delta: number) {
    this.animTimer += delta;

    // Efeito de oscilação e flicker sutil no letreiro néon
    if (this.animTimer > 120) {
      this.animTimer = 0;
      this.neonFlicker = 0.75 + Math.random() * 0.25;

      const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
      const signX = isoX - 36;
      const signY = isoY - 14;
      const signW = 10;
      const signH = 48;

      const sg = this.neonSignGraphics;
      sg.clear();

      sg.lineStyle(2, 0x1a2332, 1);
      sg.strokeLineShape(new Phaser.Geom.Line(signX + signW / 2, signY + signH, signX + signW / 2, isoY));

      sg.fillStyle(0x0c0614, 0.95);
      sg.fillRect(signX, signY, signW, signH);

      sg.lineStyle(1.5, 0xff007f, this.neonFlicker);
      sg.strokeRect(signX, signY, signW, signH);

      if (this.signText) {
        this.signText.setAlpha(this.neonFlicker);
      }
    }
  }

  public destroy() {
    this.barGraphics.destroy();
    this.neonSignGraphics.destroy();
    if (this.signText) {
      this.signText.destroy();
    }
  }
}
