import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

export interface PodVisualState {
  powerLevel: number; // 0 a 100
  chimneySmokeColor: number; // Ex: 0x39ff14, 0x00f0ff, 0xff3b30
  screenStatus: 'OFF' | 'CODE_STREAM' | 'DASHBOARD' | 'ALERT_RED';
}

interface SmokeParticle {
  x: number;
  y: number;
  speedY: number;
  speedX: number;
  alpha: number;
  size: number;
  maxLife: number;
  life: number;
  color: number;
}

/**
 * ProjectPodStructure: Representa uma cúpula/fábrica modular de projeto (ex: Notex Core).
 * Inclui paredes industriais, cúpula de observação com modo X-Ray, chaminé com fumaça
 * e telas externas com código em tempo real.
 */
export class ProjectPodStructure {
  private scene: Phaser.Scene;
  public readonly id: string;
  public readonly title: string;
  public readonly cartX: number;
  public readonly cartY: number;

  private baseGraphics: Phaser.GameObjects.Graphics;
  private domeGraphics: Phaser.GameObjects.Graphics;
  private interiorGraphics: Phaser.GameObjects.Graphics;
  private smokeGraphics: Phaser.GameObjects.Graphics;
  private screenGraphics: Phaser.GameObjects.Graphics;

  private isXRayOpen = false;
  private visualState: PodVisualState = {
    powerLevel: 80,
    chimneySmokeColor: 0x39ff14, // Verde néon fluorescente
    screenStatus: 'CODE_STREAM',
  };

  private smokeParticles: SmokeParticle[] = [];
  private codeLinesOffset = 0;
  private animTimer = 0;

  constructor(scene: Phaser.Scene, id: string, title: string, cartX: number, cartY: number) {
    this.scene = scene;
    this.id = id;
    this.title = title;
    this.cartX = cartX;
    this.cartY = cartY;

    this.baseGraphics = this.scene.add.graphics();
    this.interiorGraphics = this.scene.add.graphics();
    this.domeGraphics = this.scene.add.graphics();
    this.screenGraphics = this.scene.add.graphics();
    this.smokeGraphics = this.scene.add.graphics();

    this.renderPod();
  }

  public setVisualState(newState: Partial<PodVisualState>) {
    this.visualState = { ...this.visualState, ...newState };
  }

  public toggleXRay() {
    this.isXRayOpen = !this.isXRayOpen;
    this.renderPod();
  }

  public getIsXRayOpen(): boolean {
    return this.isXRayOpen;
  }

  public renderPod() {
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const baseDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_BASE);
    const roofDepth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.STRUCTURE_ROOF);

    this.baseGraphics.setDepth(baseDepth);
    this.interiorGraphics.setDepth(baseDepth + 2);
    this.screenGraphics.setDepth(baseDepth + 5);
    this.domeGraphics.setDepth(roofDepth);
    this.smokeGraphics.setDepth(roofDepth + 5);

    this.renderIndustrialBase(isoX, isoY);
    this.renderInterior(isoX, isoY);
    this.renderDomeAndChimney(isoX, isoY);
  }

  /**
   * Paredes externas metálicas e fundações modulares
   */
  private renderIndustrialBase(isoX: number, isoY: number) {
    const g = this.baseGraphics;
    g.clear();

    const w = TILE_WIDTH * 1.5;
    const h = TILE_HEIGHT * 1.5;
    const wallH = 32;

    // Face Sudoeste da parede
    g.fillStyle(0x0a1422, 0.95);
    g.beginPath();
    g.moveTo(isoX - w / 2, isoY);
    g.lineTo(isoX, isoY + h / 2);
    g.lineTo(isoX, isoY + h / 2 - wallH);
    g.lineTo(isoX - w / 2, isoY - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x1e3a5f, 0.8);
    g.strokePath();

    // Face Sudeste da parede (com porta de acesso pressurizada)
    g.fillStyle(0x070e17, 0.95);
    g.beginPath();
    g.moveTo(isoX, isoY + h / 2);
    g.lineTo(isoX + w / 2, isoY);
    g.lineTo(isoX + w / 2, isoY - wallH);
    g.lineTo(isoX, isoY + h / 2 - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x1e3a5f, 0.8);
    g.strokePath();

    // Porta pressurizada iluminada
    g.fillStyle(0x0f293d, 0.9);
    g.fillRect(isoX + 4, isoY + h / 2 - 20, 16, 18);
    g.lineStyle(1.5, 0x39ff14, 0.9); // Luz verde néon sobre a porta
    g.strokeRect(isoX + 4, isoY + h / 2 - 20, 16, 18);

    // Chaminé de exaustão lateral no canto leste
    const chimX = isoX + w / 2 - 6;
    const chimY = isoY - wallH;
    g.fillStyle(0x111c2a, 1);
    g.fillRect(chimX - 4, chimY - 22, 8, 22);
    g.lineStyle(1.5, 0x00f0ff, 0.7);
    g.strokeRect(chimX - 4, chimY - 22, 8, 22);
  }

  /**
   * Interior revelado em corte (Visão X-Ray) com estações, racks de servidores e monitores
   */
  private renderInterior(isoX: number, isoY: number) {
    const g = this.interiorGraphics;
    g.clear();

    if (!this.isXRayOpen) return; // Só renderiza se a cúpula estiver transparente/oculta

    // Piso interno em grade cibernética
    g.fillStyle(0x081d13, 0.95);
    g.beginPath();
    g.ellipse(isoX, isoY - 20, 36, 18, 0, 0, Math.PI * 2);
    g.fillPath();

    // Servidores em Rack com LEDs pulsando
    g.fillStyle(0x050a10, 1);
    g.fillRect(isoX - 24, isoY - 38, 12, 20);
    g.lineStyle(1, 0x00f0ff, 0.9);
    g.strokeRect(isoX - 24, isoY - 38, 12, 20);

    // LEDs do rack
    for (let ly = 0; ly < 4; ly++) {
      g.fillStyle(ly % 2 === 0 ? 0x39ff14 : 0x00f0ff, 1);
      g.fillRect(isoX - 22, isoY - 35 + ly * 4, 3, 2);
    }

    // Mesa de desenvolvimento com monitor duplo
    g.fillStyle(0x0b2438, 1);
    g.fillRect(isoX - 2, isoY - 26, 16, 8);
    g.lineStyle(1, 0x00f0ff, 0.8);
    g.strokeRect(isoX - 2, isoY - 26, 16, 8);
  }

  /**
   * Cúpula de observação em vidro reforçado e telhado translúcido
   */
  private renderDomeAndChimney(isoX: number, isoY: number) {
    const g = this.domeGraphics;
    g.clear();

    const domeY = isoY - 32;
    const radiusX = 42;
    const radiusY = 22;

    if (this.isXRayOpen) {
      // Modo X-Ray: Cúpula em aramado holográfico translúcido tênue
      g.lineStyle(1, 0x00f0ff, 0.25);
      g.strokeEllipse(isoX, domeY, radiusX, radiusY);
      g.lineStyle(1, 0x00f0ff, 0.15);
      g.strokeEllipse(isoX, domeY - 14, radiusX * 0.7, radiusY * 0.7);
      return;
    }

    // Cúpula em vidro reforçado com brilho translúcido
    g.fillStyle(0x0a2638, 0.7);
    g.beginPath();
    g.ellipse(isoX, domeY, radiusX, radiusY, 0, 0, Math.PI * 2);
    g.fillPath();

    // Nervuras estruturais da cúpula
    g.lineStyle(2, 0x00f0ff, 0.8);
    g.strokeEllipse(isoX, domeY, radiusX, radiusY);

    g.lineStyle(1.5, 0x00f0ff, 0.5);
    g.strokeEllipse(isoX, domeY - 10, radiusX * 0.65, radiusY * 0.65);

    // Reflexo de luz / Brilho especular no vidro reforçado
    g.lineStyle(2, 0xffffff, 0.5);
    g.beginPath();
    g.arc(isoX - 12, domeY - 8, 12, Math.PI * 1.1, Math.PI * 1.6);
    g.strokePath();
  }

  /**
   * Atualização contínua das partículas da chaminé e monitor de código
   */
  public update(delta: number) {
    this.animTimer += delta;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const chimX = isoX + TILE_WIDTH * 0.75 - 6;
    const chimY = isoY - 32 - 22;

    // 1. Gera fumaça contínua saindo da chaminé
    if (this.animTimer > 80) {
      this.animTimer = 0;
      this.smokeParticles.push({
        x: chimX + Phaser.Math.FloatBetween(-2, 2),
        y: chimY,
        speedY: Phaser.Math.FloatBetween(-0.4, -0.9),
        speedX: Phaser.Math.FloatBetween(-0.2, 0.3),
        alpha: 0.8,
        size: Phaser.Math.Between(3, 6),
        maxLife: 90,
        life: 0,
        color: this.visualState.chimneySmokeColor,
      });
    }

    // 2. Anima e renderiza as partículas de vapor colorido
    const sg = this.smokeGraphics;
    sg.clear();

    for (let i = this.smokeParticles.length - 1; i >= 0; i--) {
      const p = this.smokeParticles[i];
      p.x += p.speedX;
      p.y += p.speedY;
      p.size += 0.05;
      p.life++;
      p.alpha = Math.max(0, 0.8 * (1 - p.life / p.maxLife));

      if (p.life >= p.maxLife) {
        this.smokeParticles.splice(i, 1);
        continue;
      }

      sg.fillStyle(p.color, p.alpha);
      sg.fillCircle(p.x, p.y, p.size);
    }

    // 3. Renderiza tela externa com linhas de código fluindo
    this.renderScreenCode(isoX, isoY, delta);
  }

  private renderScreenCode(isoX: number, isoY: number, delta: number) {
    const sc = this.screenGraphics;
    sc.clear();

    const screenX = isoX - 32;
    const screenY = isoY - 24;
    const screenW = 22;
    const screenH = 14;

    // Moldura do Monitor Externo
    sc.fillStyle(0x040810, 0.95);
    sc.fillRect(screenX, screenY, screenW, screenH);
    sc.lineStyle(1, 0x00f0ff, 0.7);
    sc.strokeRect(screenX, screenY, screenW, screenH);

    // Linhas de código em fluxo
    this.codeLinesOffset = (this.codeLinesOffset + delta * 0.03) % 4;

    sc.fillStyle(this.visualState.chimneySmokeColor, 0.85);
    for (let i = 0; i < 3; i++) {
      const lineY = screenY + 3 + ((i * 3 + this.codeLinesOffset) % (screenH - 4));
      const lineW = 6 + ((i * 5) % 12);
      sc.fillRect(screenX + 2, lineY, lineW, 1.5);
    }
  }

  public destroy() {
    this.baseGraphics.destroy();
    this.domeGraphics.destroy();
    this.interiorGraphics.destroy();
    this.smokeGraphics.destroy();
    this.screenGraphics.destroy();
    this.smokeParticles = [];
  }
}
