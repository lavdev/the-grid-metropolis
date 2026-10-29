import Phaser from 'phaser';

interface SteamParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  alpha: number;
  size: number;
  life: number;
  maxLife: number;
}

/**
 * LivingWorldOverlay: Adiciona vida dinâmica contínua sobre o Quartel-General Hacker:
 * - Vapor suave e contínuo subindo da máquina de café expresso no Breakroom
 * - Brilho pulsante e telas CRT nos arcades retrô
 * - Luzes de dados e leds piscando nos racks de servidores no Server Vault
 * - Pulso concêntrico de dados na mesa holográfica central do Nexus
 */
export class LivingWorldOverlay {
  private scene: Phaser.Scene;

  private steamGraphics: Phaser.GameObjects.Graphics;
  private glowGraphics: Phaser.GameObjects.Graphics;
  private serverLedGraphics: Phaser.GameObjects.Graphics;
  private pulseGraphics: Phaser.GameObjects.Graphics;

  private steamParticles: SteamParticle[] = [];
  private animTimer = 0;
  private pulseAngle = 0;
  private isDispatchPulsing = false;
  private dispatchRadius = 0;
  private dispatchAlpha = 0;

  constructor(scene: Phaser.Scene) {
    this.scene = scene;

    this.glowGraphics = scene.add.graphics();
    this.glowGraphics.setDepth(20);

    this.serverLedGraphics = scene.add.graphics();
    this.serverLedGraphics.setDepth(22);

    this.steamGraphics = scene.add.graphics();
    this.steamGraphics.setDepth(45);

    this.pulseGraphics = scene.add.graphics();
    this.pulseGraphics.setDepth(25);
  }

  /**
   * Dispara um pulso concêntrico a partir da mesa central do Nexus
   */
  public triggerNexusPulse() {
    this.isDispatchPulsing = true;
    this.dispatchRadius = 10;
    this.dispatchAlpha = 1.0;
  }

  public update(delta: number) {
    this.animTimer += delta;
    this.pulseAngle += delta * 0.003;

    // 1. Gera vapor suave da máquina de café expresso no Breakroom (canto superior direito ~ x: 450, y: -70)
    if (this.animTimer > 120) {
      this.animTimer = 0;

      this.steamParticles.push({
        x: 448 + Phaser.Math.FloatBetween(-3, 3),
        y: -75,
        vx: Phaser.Math.FloatBetween(-0.15, 0.15),
        vy: Phaser.Math.FloatBetween(-0.4, -0.7),
        alpha: 0.6,
        size: Phaser.Math.Between(2, 4),
        life: 0,
        maxLife: 45,
      });
    }

    this.renderSteam();
    this.renderArcadeAndHoloGlow();
    this.renderServerLeds();
    this.renderDispatchPulse(delta);
  }

  private renderSteam() {
    const sg = this.steamGraphics;
    sg.clear();

    for (let i = this.steamParticles.length - 1; i >= 0; i--) {
      const p = this.steamParticles[i];
      p.x += p.vx;
      p.y += p.vy;
      p.life++;

      const progress = p.life / p.maxLife;
      const currentAlpha = p.alpha * (1 - progress);
      const currentSize = p.size * (1 + progress * 0.8);

      sg.fillStyle(0x00f0ff, currentAlpha * 0.7);
      sg.fillCircle(p.x, p.y, currentSize);

      if (p.life >= p.maxLife) {
        this.steamParticles.splice(i, 1);
      }
    }
  }

  private renderArcadeAndHoloGlow() {
    const gg = this.glowGraphics;
    gg.clear();

    const flicker = 0.85 + Math.sin(this.pulseAngle * 4) * 0.15;

    // 1. Brilho da tela dos Fliperamas no Breakroom
    // Arcade 1
    gg.fillStyle(0x00f0ff, 0.15 * flicker);
    gg.fillRect(235, -85, 24, 28);
    // Arcade 2
    gg.fillStyle(0x9d00ff, 0.15 * flicker);
    gg.fillRect(290, -115, 24, 28);
    // Arcade 3
    gg.fillStyle(0xff007f, 0.15 * flicker);
    gg.fillRect(360, -85, 24, 28);

    // 2. Halo azul brilhante em torno da mesa holográfica do War Room (x: 0, y: -190)
    const holoFlicker = 0.2 + Math.sin(this.pulseAngle * 2) * 0.08;
    gg.fillStyle(0x00f0ff, holoFlicker);
    gg.fillEllipse(0, -188, 120, 60);
  }

  private renderServerLeds() {
    const lg = this.serverLedGraphics;
    lg.clear();

    // Cascata de luzes verdes Matrix nos racks do Server Vault (embaixo: ~ x: -60 a 60, y: 120 a 240)
    const timeSec = this.animTimer * 0.01;

    // Piscar aleatório mas com padrão sutil de datacenter
    if (Math.random() > 0.3) {
      lg.fillStyle(0x39ff14, 0.9);
      // Racks esquerdos
      lg.fillRect(-52, 130 + (Math.floor(Math.sin(timeSec) * 3) + 3) * 6, 2, 2);
      lg.fillRect(-38, 145 + (Math.floor(Math.cos(timeSec) * 4) + 4) * 5, 2, 2);

      // Racks direitos
      lg.fillStyle(0x00f0ff, 0.9);
      lg.fillRect(40, 140 + (Math.floor(Math.sin(timeSec * 1.5) * 3) + 3) * 6, 2, 2);
      lg.fillRect(58, 155 + (Math.floor(Math.cos(timeSec * 1.2) * 4) + 4) * 5, 2, 2);
    }
  }

  private renderDispatchPulse(delta: number) {
    const pg = this.pulseGraphics;
    pg.clear();

    if (!this.isDispatchPulsing) return;

    this.dispatchRadius += delta * 0.18;
    this.dispatchAlpha -= delta * 0.0014;

    if (this.dispatchAlpha <= 0) {
      this.isDispatchPulsing = false;
      this.dispatchAlpha = 0;
      return;
    }

    // Pulso concêntrico a partir da mesa holográfica (x: 0, y: -190)
    pg.lineStyle(2, 0x00f0ff, this.dispatchAlpha);
    pg.strokeEllipse(0, -188, this.dispatchRadius * 2, this.dispatchRadius);

    if (this.dispatchRadius > 35) {
      pg.lineStyle(1.5, 0x39ff14, this.dispatchAlpha * 0.7);
      pg.strokeEllipse(0, -188, (this.dispatchRadius - 20) * 2, this.dispatchRadius - 20);
    }
  }

  public destroy() {
    this.steamGraphics.destroy();
    this.glowGraphics.destroy();
    this.serverLedGraphics.destroy();
    this.pulseGraphics.destroy();
    this.steamParticles = [];
  }
}
