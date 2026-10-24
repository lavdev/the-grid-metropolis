import Phaser from 'phaser';

interface SmokeParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  alpha: number;
  size: number;
  life: number;
  maxLife: number;
  color: number;
}

interface ConduitPulse {
  p1: { x: number; y: number };
  p2: { x: number; y: number };
  p3: { x: number; y: number };
  t: number;
  speed: number;
  color: number;
}

/**
 * LivingWorldOverlay: Adiciona vida dinâmica contínua sobre a arte mestra de The Grid Metropolis:
 * - Fumaça volumétrica contínua colorida nas 4 chaminés (rosa, verde, ciano)
 * - Projeção holográfica giratória sobre a mesa do Nexus
 * - Telas de código animado e monitores com estática no Glitch Alley
 * - Letreiro e balcão pulsantes no The Battery Lounge
 * - Pulsos bioluminescentes correndo pelos cabos grossos suspensos no ar
 */
export class LivingWorldOverlay {
  private scene: Phaser.Scene;

  private smokeGraphics: Phaser.GameObjects.Graphics;
  private holoGraphics: Phaser.GameObjects.Graphics;
  private screenGraphics: Phaser.GameObjects.Graphics;
  private cableGraphics: Phaser.GameObjects.Graphics;
  private pulseGraphics: Phaser.GameObjects.Graphics;

  private smokeParticles: SmokeParticle[] = [];
  private conduitPulses: ConduitPulse[] = [];

  private animTimer = 0;
  private holoAngle = 0;
  private neonFlicker = 1.0;
  private codeOffset = 0;
  private isDispatchPulsing = false;
  private dispatchRadius = 0;
  private dispatchAlpha = 0;

  // Posições exatas das chaminés na arte mestra
  private readonly chimneys = [
    { x: -442, y: -238, color: 0xff3b94 }, // Cúpula rosa superior esquerda
    { x: -470, y: -80, color: 0xff3b94 },  // Prédio inferior esquerdo
    { x: -70, y: 154, color: 0x39ff14 },   // Chaminé central inferior
    { x: 454, y: -30, color: 0x39ff14 },   // Fábrica direita
  ];

  constructor(scene: Phaser.Scene) {
    this.scene = scene;

    this.cableGraphics = scene.add.graphics();
    this.cableGraphics.setDepth(5);

    this.smokeGraphics = scene.add.graphics();
    this.smokeGraphics.setDepth(45);

    this.screenGraphics = scene.add.graphics();
    this.screenGraphics.setDepth(15);

    this.holoGraphics = scene.add.graphics();
    this.holoGraphics.setDepth(30);

    this.pulseGraphics = scene.add.graphics();
    this.pulseGraphics.setDepth(25);

    this.setupCablePulses();
  }

  private setupCablePulses() {
    // Curvas dos cabos de sustentação suspensos no abismo
    this.conduitPulses = [
      // Cabo do Nexus para a fábrica leste
      {
        p1: { x: 50, y: 35 },
        p2: { x: 180, y: 120 },
        p3: { x: 340, y: 140 },
        t: 0,
        speed: 0.008,
        color: 0x00f0ff,
      },
      // Cabo do Nexus para o Battery Lounge
      {
        p1: { x: 60, y: -30 },
        p2: { x: 190, y: -90 },
        p3: { x: 330, y: -130 },
        t: 0.5,
        speed: 0.007,
        color: 0xff007f,
      },
      // Cabo do Nexus para o laboratório oeste
      {
        p1: { x: -60, y: -10 },
        p2: { x: -180, y: -80 },
        p3: { x: -330, y: -120 },
        t: 0.25,
        speed: 0.009,
        color: 0x39ff14,
      },
      // Cabo inferior esquerdo
      {
        p1: { x: -50, y: 40 },
        p2: { x: -160, y: 110 },
        p3: { x: -320, y: 140 },
        t: 0.75,
        speed: 0.008,
        color: 0x00f0ff,
      },
    ];
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
    this.holoAngle += delta * 0.002;
    this.codeOffset = (this.codeOffset + delta * 0.04) % 12;

    // 1. Gera fumaça contínua nas 4 chaminés
    if (this.animTimer > 60) {
      this.animTimer = 0;
      this.neonFlicker = 0.8 + Math.random() * 0.2;

      for (const ch of this.chimneys) {
        this.smokeParticles.push({
          x: ch.x + Phaser.Math.FloatBetween(-2, 2),
          y: ch.y,
          vx: Phaser.Math.FloatBetween(-0.25, 0.25),
          vy: Phaser.Math.FloatBetween(-0.6, -1.1),
          alpha: 0.85,
          size: Phaser.Math.Between(4, 8),
          life: 0,
          maxLife: 75,
          color: ch.color,
        });
      }
    }

    // 2. Renderiza partículas de fumaça subindo e expandindo
    const smg = this.smokeGraphics;
    smg.clear();

    for (let i = this.smokeParticles.length - 1; i >= 0; i--) {
      const p = this.smokeParticles[i];
      p.x += p.vx;
      p.y += p.vy;
      p.size += 0.06;
      p.life++;
      p.alpha = Math.max(0, 0.85 * (1 - p.life / p.maxLife));

      if (p.life >= p.maxLife) {
        this.smokeParticles.splice(i, 1);
        continue;
      }

      smg.fillStyle(p.color, p.alpha);
      smg.fillCircle(p.x, p.y, p.size);
    }

    // 3. Projeção Holográfica Dinâmica no Nexus (Piscina central)
    this.renderNexusHologram();

    // 4. Telas vivas piscando código e estática
    this.renderDynamicScreens();

    // 5. Pulsos de dados fluindo pelos cabos suspensos
    this.renderCableDataPulses(delta);

    // 6. Onda concêntrica de despacho
    this.renderDispatchPulse(delta);
  }

  private renderNexusHologram() {
    const hg = this.holoGraphics;
    hg.clear();

    const hx = 3;
    const hy = 2; // Mesa holográfica central na arte

    // Brilho da piscina circular
    hg.fillStyle(0x00f0ff, 0.2);
    hg.fillEllipse(hx, hy + 4, 76, 38);
    hg.lineStyle(1.5, 0x00f0ff, 0.8);
    hg.strokeEllipse(hx, hy + 4, 76, 38);

    // Feixes holográficos ascendentes
    const holoY = hy - 16;
    hg.fillStyle(0x00f0ff, 0.12);
    hg.beginPath();
    hg.moveTo(hx - 18, hy + 4);
    hg.lineTo(hx - 36, holoY - 14);
    hg.lineTo(hx + 36, holoY - 14);
    hg.lineTo(hx + 18, hy + 4);
    hg.closePath();
    hg.fillPath();

    // Telas holográficas rotativas
    const angle1 = this.holoAngle;
    const s1x = hx + Math.cos(angle1) * 16;
    const s1y = holoY - 6 + Math.sin(angle1) * 6;

    hg.fillStyle(0xff007f, 0.7);
    hg.fillRect(s1x - 12, s1y - 8, 24, 16);
    hg.lineStyle(1.5, 0x00f0ff, 0.95);
    hg.strokeRect(s1x - 12, s1y - 8, 24, 16);

    // Linhas de código na tela holográfica
    hg.fillStyle(0xffffff, 0.9);
    for (let l = 0; l < 3; l++) {
      hg.fillRect(s1x - 9, s1y - 5 + l * 4, 14 - l * 3, 1.5);
    }
  }

  private renderDynamicScreens() {
    const sc = this.screenGraphics;
    sc.clear();

    // Tela grande do Pod Notex (Fábrica Leste)
    const fScreenX = 330;
    const fScreenY = 46;
    sc.fillStyle(0x39ff14, 0.8);
    for (let row = 0; row < 4; row++) {
      const lineY = fScreenY + ((row * 4 + this.codeOffset) % 18);
      sc.fillRect(fScreenX + 3, lineY, 26 - (row % 3) * 4, 1.5);
    }

    // Tela de Estática / Ruído no Glitch Alley (Caverna superior)
    const gScreenX = 172;
    const gScreenY = -185;
    sc.fillStyle(0xff3b30, 0.75);
    for (let px = 0; px < 24; px += 3) {
      for (let py = 0; py < 12; py += 2) {
        if (Math.random() > 0.6) {
          sc.fillRect(gScreenX + px, gScreenY + py, 2, 1.5);
        }
      }
    }

    // Baliza de advertência piscando no Glitch Alley
    const beaconAlpha = 0.4 + Math.sin(this.holoAngle * 3) * 0.6;
    sc.fillStyle(0xff3b30, Math.max(0.1, beaconAlpha));
    sc.fillCircle(38, -204, 3.5);

    // Letreiro do The Battery Lounge
    sc.lineStyle(2, 0x00f0ff, this.neonFlicker);
    sc.strokeRect(338, -242, 126, 18);
  }

  private renderCableDataPulses(delta: number) {
    const cg = this.cableGraphics;
    cg.clear();

    for (const cp of this.conduitPulses) {
      cp.t += cp.speed * (delta * 0.06);
      if (cp.t > 1) cp.t = 0;

      // Ponto na curva de Bézier quadrática
      const u = 1 - cp.t;
      const tt = cp.t * cp.t;
      const uu = u * u;
      const x = uu * cp.p1.x + 2 * u * cp.t * cp.p2.x + tt * cp.p3.x;
      const y = uu * cp.p1.y + 2 * u * cp.t * cp.p2.y + tt * cp.p3.y;

      // Pacote de luz fluindo pelo cabo
      cg.fillStyle(cp.color, 0.95);
      cg.fillCircle(x, y, 3);
      cg.fillStyle(cp.color, 0.4);
      cg.fillCircle(x, y, 6);
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

    pg.lineStyle(2, 0x00f0ff, this.dispatchAlpha);
    pg.strokeEllipse(3, 2, this.dispatchRadius * 2, this.dispatchRadius);

    if (this.dispatchRadius > 35) {
      pg.lineStyle(1.5, 0x39ff14, this.dispatchAlpha * 0.7);
      pg.strokeEllipse(3, 2, (this.dispatchRadius - 20) * 2, this.dispatchRadius - 20);
    }
  }

  public destroy() {
    this.cableGraphics.destroy();
    this.smokeGraphics.destroy();
    this.screenGraphics.destroy();
    this.holoGraphics.destroy();
    this.pulseGraphics.destroy();
    this.smokeParticles = [];
  }
}
