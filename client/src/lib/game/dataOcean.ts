import Phaser from 'phaser';
import { DepthLayer } from './depth';

interface DataPacket {
  x: number;
  y: number;
  speedY: number;
  speedX: number;
  alpha: number;
  size: number;
  color: number;
  pulseSpeed: number;
  pulseTimer: number;
}

/**
 * DataOcean: O abismo de dados brutos sobre o qual a metrópole está suspensa.
 * Renderiza um mar escuro e profundo com linhas de fluxo digital e partículas
 * de dados bioluminescentes flutuantes.
 */
export class DataOcean {
  private scene: Phaser.Scene;
  private backgroundGraphics: Phaser.GameObjects.Graphics;
  private particleGraphics: Phaser.GameObjects.Graphics;
  private packets: DataPacket[] = [];
  private pulsePhase = 0;

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
    this.backgroundGraphics = this.scene.add.graphics();
    this.backgroundGraphics.setDepth(DepthLayer.DATA_OCEAN);

    this.particleGraphics = this.scene.add.graphics();
    this.particleGraphics.setDepth(DepthLayer.DATA_PACKETS);

    this.initializeDataPackets();
    this.renderStaticGrid();
  }

  private initializeDataPackets() {
    const packetCount = 80;
    const colors = [0x00f0ff, 0x9d00ff, 0x39ff14, 0x0088ff];

    for (let i = 0; i < packetCount; i++) {
      this.packets.push({
        x: Phaser.Math.Between(-3000, 3000),
        y: Phaser.Math.Between(-2500, 2500),
        speedY: Phaser.Math.FloatBetween(-0.25, -0.75),
        speedX: Phaser.Math.FloatBetween(-0.15, 0.15),
        alpha: Phaser.Math.FloatBetween(0.2, 0.7),
        size: Phaser.Math.Between(2, 4),
        color: colors[Phaser.Math.Between(0, colors.length - 1)],
        pulseSpeed: Phaser.Math.FloatBetween(0.02, 0.05),
        pulseTimer: Phaser.Math.FloatBetween(0, Math.PI * 2),
      });
    }
  }

  private renderStaticGrid() {
    const g = this.backgroundGraphics;
    g.clear();

    // Fundo do abismo: gradiente espacial profundo
    g.fillStyle(0x020409, 1);
    g.fillRect(-5000, -5000, 10000, 10000);

    // Grid cibernético do abismo (Data Streams primários)
    g.lineStyle(1, 0x071b30, 0.4);
    for (let x = -4000; x <= 4000; x += 128) {
      g.lineBetween(x, -4000, x, 4000);
    }
    for (let y = -4000; y <= 4000; y += 64) {
      g.lineBetween(-4000, y, 4000, y);
    }

    // Linhas de pulso néon ocasionais
    g.lineStyle(1, 0x00f0ff, 0.08);
    for (let x = -3500; x <= 3500; x += 512) {
      g.lineBetween(x, -4000, x, 4000);
    }
  }

  /**
   * Atualização contínua das correntes do Data Ocean (partículas e pulsos)
   */
  public update(delta: number) {
    this.pulsePhase += delta * 0.001;
    const g = this.particleGraphics;
    g.clear();

    // Renderiza e anima partículas de dados flutuantes
    for (const p of this.packets) {
      p.y += p.speedY * (delta * 0.06);
      p.x += p.speedX * (delta * 0.06);
      p.pulseTimer += p.pulseSpeed;

      // Reseta partículas que saem do campo de visão
      if (p.y < -2500) {
        p.y = 2500;
        p.x = Phaser.Math.Between(-3000, 3000);
      }
      if (p.x < -3000) p.x = 3000;
      if (p.x > 3000) p.x = -3000;

      const dynamicAlpha = Math.max(0.1, p.alpha + Math.sin(p.pulseTimer) * 0.2);

      g.fillStyle(p.color, dynamicAlpha);
      g.fillRect(p.x, p.y, p.size, p.size);

      // Leve rastro vertical
      g.fillStyle(p.color, dynamicAlpha * 0.35);
      g.fillRect(p.x, p.y + p.size, p.size, p.size * 2);
    }
  }

  public destroy() {
    this.backgroundGraphics.destroy();
    this.particleGraphics.destroy();
    this.packets = [];
  }
}
