import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

export interface PodVisualState {
  powerLevel: number;
  chimneySmokeColor: number;
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
 * ProjectPodStructure: Edificação Industrial Volumétrica de Projeto (ex: Notex Core).
 * Um laboratório/fábrica cibernética de 2 pavimentos com paredes de aço escuro,
 * janelas de datacenters iluminadas, porta de hangar pressurizada, chaminés industriais duplas
 * e cúpula de observação geodésica com suporte a Visão em Corte (X-Ray Cutaway).
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
    powerLevel: 85,
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
    this.smokeGraphics.setDepth(roofDepth + 6);

    this.renderVolumetricBuilding(isoX, isoY);
    this.renderInteriorLab(isoX, isoY);
    this.renderRooftopAndDome(isoX, isoY);
  }

  /**
   * Renderiza a edificação de 2 pavimentos (paredes volumétricas e fachadas)
   */
  private renderVolumetricBuilding(isoX: number, isoY: number) {
    const g = this.baseGraphics;
    g.clear();

    const halfW = 46;
    const halfH = 23;
    const wallH = 46; // Edificação de 2 andares imponente

    // ==========================================
    // 1. PAREDE SUDOESTE (FACE ESQUERDA)
    // ==========================================
    g.fillStyle(0x0a1622, 0.98); // Aço industrial escuro
    g.beginPath();
    g.moveTo(isoX - halfW, isoY);
    g.lineTo(isoX, isoY + halfH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.lineTo(isoX - halfW, isoY - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x162c3f, 0.9);
    g.strokePath();

    // Faixas horizontais de janelas de servidores (Andar 1 e Andar 2)
    const windowColors = [0x39ff14, 0x00f0ff];
    for (let floor = 0; floor < 2; floor++) {
      const fOffsetY = floor * 18 + 10;
      for (let w = 0; w < 3; w++) {
        const wx = isoX - halfW + 10 + w * 12;
        const wy = isoY + (w * 6) + 4 - fOffsetY;
        g.fillStyle(windowColors[floor % 2], 0.85);
        g.fillRect(wx, wy, 8, 3);
      }
    }

    // Pilares verticais e tubos industriais na parede esquerda
    g.fillStyle(0x112335, 1);
    g.fillRect(isoX - halfW + 4, isoY - wallH + 2, 4, wallH - 4);
    g.fillRect(isoX - 6, isoY + halfH - wallH + 2, 4, wallH - 4);

    // ==========================================
    // 2. PAREDE SUDESTE (FACE DIREITA - ENTRADA)
    // ==========================================
    g.fillStyle(0x060f17, 0.98);
    g.beginPath();
    g.moveTo(isoX, isoY + halfH);
    g.lineTo(isoX + halfW, isoY);
    g.lineTo(isoX + halfW, isoY - wallH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x162c3f, 0.9);
    g.strokePath();

    // Porta de Hangar Pressurizada
    const doorW = 20;
    const doorH = 24;
    const doorX = isoX + 12;
    const doorY = isoY + halfH - 26;

    g.fillStyle(0x04090e, 1);
    g.fillRect(doorX, doorY, doorW, doorH);
    g.lineStyle(2, 0x39ff14, 0.9);
    g.strokeRect(doorX, doorY, doorW, doorH);

    // Faixa diagonal de advertência na porta
    g.lineStyle(1, 0xff9900, 0.7);
    g.strokeLineShape(new Phaser.Geom.Line(doorX + 2, doorY + doorH - 4, doorX + doorW - 2, doorY + 4));

    // Luz de status da porta
    g.fillStyle(0x39ff14, 1);
    g.fillCircle(doorX + doorW / 2, doorY - 4, 3);

    // ==========================================
    // 3. LAJE SUPERIOR / TELHADO (ROOFTOP SLAB)
    // ==========================================
    const roofY = isoY - wallH;

    g.fillStyle(0x0d1f2d, 1);
    g.beginPath();
    g.moveTo(isoX, roofY - halfH);
    g.lineTo(isoX + halfW, roofY);
    g.lineTo(isoX, roofY + halfH);
    g.lineTo(isoX - halfW, roofY);
    g.closePath();
    g.fillPath();

    g.lineStyle(2, 0x39ff14, 0.85); // Borda néon verde no terraço
    g.strokePath();

    // ==========================================
    // 4. DUAS CHAMINÉS INDUSTRIAIS
    // ==========================================
    const chimWidth = 7;
    const chimH = 20;

    // Chaminé 1 (Noroeste)
    const chim1X = isoX - 18;
    const chim1Y = roofY - 8;
    g.fillStyle(0x142533, 1);
    g.fillRect(chim1X - chimWidth / 2, chim1Y - chimH, chimWidth, chimH);
    g.lineStyle(1.5, 0x00f0ff, 0.8);
    g.strokeRect(chim1X - chimWidth / 2, chim1Y - chimH, chimWidth, chimH);

    // Chaminé 2 (Nordeste)
    const chim2X = isoX + 22;
    const chim2Y = roofY - 4;
    g.fillStyle(0x142533, 1);
    g.fillRect(chim2X - chimWidth / 2, chim2Y - chimH, chimWidth, chimH);
    g.lineStyle(1.5, 0x39ff14, 0.8);
    g.strokeRect(chim2X - chimWidth / 2, chim2Y - chimH, chimWidth, chimH);
  }

  /**
   * Interior revelado em corte (Visão X-Ray) com estações, racks de servidores e computadores
   */
  private renderInteriorLab(isoX: number, isoY: number) {
    const g = this.interiorGraphics;
    g.clear();

    if (!this.isXRayOpen) return;

    const wallH = 46;
    const floorY = isoY - wallH;

    // Piso do laboratório
    g.fillStyle(0x061810, 0.95);
    g.fillEllipse(isoX, floorY, 52, 26);
    g.lineStyle(1, 0x39ff14, 0.6);
    g.strokeEllipse(isoX, floorY, 52, 26);

    // Racks de Servidores Principais
    g.fillStyle(0x040c07, 1);
    g.fillRect(isoX - 20, floorY - 24, 14, 22);
    g.lineStyle(1.5, 0x39ff14, 0.9);
    g.strokeRect(isoX - 20, floorY - 24, 14, 22);

    // LEDs pulsando no rack
    for (let ly = 0; ly < 5; ly++) {
      g.fillStyle(ly % 2 === 0 ? 0x39ff14 : 0x00f0ff, 1);
      g.fillRect(isoX - 18, floorY - 21 + ly * 4, 3, 2);
    }

    // Estação de trabalho do Engenheiro de Software
    g.fillStyle(0x092015, 1);
    g.fillRect(isoX + 2, floorY - 14, 18, 10);
    g.lineStyle(1, 0x00f0ff, 0.9);
    g.strokeRect(isoX + 2, floorY - 14, 18, 10);

    // Monitores duplos na bancada
    g.fillStyle(0x00f0ff, 0.9);
    g.fillRect(isoX + 4, floorY - 20, 6, 5);
    g.fillRect(isoX + 12, floorY - 20, 6, 5);
  }

  /**
   * Cúpula Geodésica de Vidro Reforçado no topo da edificação
   */
  private renderRooftopAndDome(isoX: number, isoY: number) {
    const g = this.domeGraphics;
    g.clear();

    const wallH = 46;
    const domeY = isoY - wallH;
    const radiusX = 30;
    const radiusY = 15;

    if (this.isXRayOpen) {
      // Modo X-Ray: Cúpula transparente em aramado holográfico tênue
      g.lineStyle(1, 0x00f0ff, 0.3);
      g.strokeEllipse(isoX, domeY, radiusX * 2, radiusY * 2);
      g.lineStyle(1, 0x00f0ff, 0.15);
      g.strokeEllipse(isoX, domeY - 10, radiusX * 1.4, radiusY * 1.4);
      return;
    }

    // Cúpula em vidro pressurizado com volume translúcido
    g.fillStyle(0x09261a, 0.72);
    g.fillEllipse(isoX, domeY, radiusX * 2, radiusY * 2);

    // Estrutura geodésica em néon verde
    g.lineStyle(2, 0x39ff14, 0.85);
    g.strokeEllipse(isoX, domeY, radiusX * 2, radiusY * 2);

    g.lineStyle(1.5, 0x00f0ff, 0.6);
    g.strokeEllipse(isoX, domeY - 8, radiusX * 1.4, radiusY * 1.4);

    // Brilho especular do vidro curvo
    g.lineStyle(2, 0xffffff, 0.6);
    g.beginPath();
    g.arc(isoX - 8, domeY - 6, 8, Math.PI * 1.1, Math.PI * 1.6);
    g.strokePath();
  }

  /**
   * Atualização contínua das partículas de fumaça das chaminés e tela de código
   */
  public update(delta: number) {
    this.animTimer += delta;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const wallH = 46;
    const roofY = isoY - wallH;
    const chim1X = isoX - 18;
    const chim1Y = roofY - 8 - 20;

    // Emite partículas de vapor da chaminé
    if (this.animTimer > 70) {
      this.animTimer = 0;
      this.smokeParticles.push({
        x: chim1X + Phaser.Math.FloatBetween(-2, 2),
        y: chim1Y,
        speedY: Phaser.Math.FloatBetween(-0.5, -1.0),
        speedX: Phaser.Math.FloatBetween(-0.25, 0.25),
        alpha: 0.85,
        size: Phaser.Math.Between(3, 7),
        maxLife: 80,
        life: 0,
        color: this.visualState.chimneySmokeColor,
      });
    }

    const sg = this.smokeGraphics;
    sg.clear();

    for (let i = this.smokeParticles.length - 1; i >= 0; i--) {
      const p = this.smokeParticles[i];
      p.x += p.speedX;
      p.y += p.speedY;
      p.size += 0.06;
      p.life++;
      p.alpha = Math.max(0, 0.85 * (1 - p.life / p.maxLife));

      if (p.life >= p.maxLife) {
        this.smokeParticles.splice(i, 1);
        continue;
      }

      sg.fillStyle(p.color, p.alpha);
      sg.fillCircle(p.x, p.y, p.size);
    }

    // Tela com linhas de código
    this.renderScreenCode(isoX, isoY, delta);
  }

  private renderScreenCode(isoX: number, isoY: number, delta: number) {
    const sc = this.screenGraphics;
    sc.clear();

    const screenX = isoX - 38;
    const screenY = isoY - 14;
    const screenW = 24;
    const screenH = 14;

    sc.fillStyle(0x040810, 0.95);
    sc.fillRect(screenX, screenY, screenW, screenH);
    sc.lineStyle(1.5, 0x00f0ff, 0.85);
    sc.strokeRect(screenX, screenY, screenW, screenH);

    this.codeLinesOffset = (this.codeLinesOffset + delta * 0.03) % 4;

    sc.fillStyle(this.visualState.chimneySmokeColor, 0.9);
    for (let i = 0; i < 3; i++) {
      const lineY = screenY + 3 + ((i * 3 + this.codeLinesOffset) % (screenH - 4));
      const lineW = 6 + ((i * 6) % 14);
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
