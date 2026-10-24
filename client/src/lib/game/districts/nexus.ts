import Phaser from 'phaser';
import { cartesianToIsometric } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

/**
 * TheNexusSector: Representa o Centro de Comando e Despacho de Missões (The Nexus).
 * Contém a mesa holográfica circular central e a antena parabólica de transmissão massiva.
 */
export class TheNexusSector {
  private scene: Phaser.Scene;
  private baseGraphics: Phaser.GameObjects.Graphics;
  private holoGraphics: Phaser.GameObjects.Graphics;
  private antennaGraphics: Phaser.GameObjects.Graphics;
  private pulseGraphics: Phaser.GameObjects.Graphics;

  private holoAngle = 0;
  private pulseRadius = 0;
  private isPulsing = false;
  private pulseAlpha = 0;

  // Posição central do Nexus
  private readonly centerX = 0;
  private readonly centerY = 0;

  // Posição da antena no quadrante norte do Nexus
  private readonly antennaX = 0;
  private readonly antennaY = -2;

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
    this.baseGraphics = this.scene.add.graphics();
    this.holoGraphics = this.scene.add.graphics();
    this.antennaGraphics = this.scene.add.graphics();
    this.pulseGraphics = this.scene.add.graphics();

    this.renderBaseStation();
    this.renderAntennaArray();
  }

  /**
   * Renderiza a base física da mesa holográfica central
   */
  private renderBaseStation() {
    const g = this.baseGraphics;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.centerX, this.centerY);
    const depth = calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.STRUCTURE_BASE);

    g.setDepth(depth);
    g.clear();

    // Pedestal cilíndrico metálico (base isométrica)
    const baseW = 34;
    const baseH = 17;
    const height = 12;

    // Face lateral cilíndrica da mesa
    g.fillStyle(0x06111f, 0.95);
    g.fillEllipse(isoX, isoY + height, baseW * 2, baseH * 2);

    // Paredes verticais do pedestal
    g.fillStyle(0x0a1c30, 0.95);
    g.fillRect(isoX - baseW, isoY, baseW * 2, height);

    // Tampo superior da mesa holográfica
    g.fillStyle(0x0d2847, 0.95);
    g.fillEllipse(isoX, isoY, baseW * 2, baseH * 2);

    // Anel externo iluminado da mesa
    g.lineStyle(2, 0x00f0ff, 0.9);
    g.strokeEllipse(isoX, isoY, baseW * 2, baseH * 2);

    // Emitter central do projetor
    g.fillStyle(0x00f0ff, 1);
    g.fillCircle(isoX, isoY, 4);
  }

  /**
   * Renderiza a torre e antena parabólica de transmissão massiva
   */
  private renderAntennaArray() {
    const g = this.antennaGraphics;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.antennaX, this.antennaY);
    const depth = calculateIsometricDepth(this.antennaX, this.antennaY, DepthLayer.STRUCTURE_BASE);

    g.setDepth(depth);
    g.clear();

    // Pylon de treliça metálica (Torre)
    const towerH = 50;
    g.lineStyle(2, 0x1b2d42, 0.9);
    g.strokeLineShape(new Phaser.Geom.Line(isoX - 8, isoY, isoX - 4, isoY - towerH));
    g.strokeLineShape(new Phaser.Geom.Line(isoX + 8, isoY, isoX + 4, isoY - towerH));
    g.strokeLineShape(new Phaser.Geom.Line(isoX - 6, isoY - towerH * 0.5, isoX + 6, isoY - towerH * 0.5));

    // Antena Parabólica no topo
    const topY = isoY - towerH;
    g.fillStyle(0x0d2238, 0.95);
    g.lineStyle(2, 0x00f0ff, 0.85);

    // Prato parabólico
    g.fillEllipse(isoX, topY, 40, 20);
    g.strokeEllipse(isoX, topY, 40, 20);

    // Receptor central focal da parabólica
    g.lineStyle(2, 0x9d00ff, 0.9);
    g.strokeLineShape(new Phaser.Geom.Line(isoX, topY, isoX - 5, topY - 14));

    // Luz de alerta da antena (Beacon vermelho/ciano)
    g.fillStyle(0x00f0ff, 1);
    g.fillCircle(isoX - 5, topY - 14, 3);
  }

  /**
   * Dispara um pulso concêntrico de emissão ao receber nova tarefa da ponte
   */
  public triggerDispatchPulse() {
    this.isPulsing = true;
    this.pulseRadius = 10;
    this.pulseAlpha = 1.0;
  }

  /**
   * Atualização contínua do holograma giratório e ondas concêntricas
   */
  public update(delta: number) {
    this.holoAngle += delta * 0.002;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.centerX, this.centerY);
    const holoDepth = calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.STRUCTURE_ROOF);

    const g = this.holoGraphics;
    g.setDepth(holoDepth);
    g.clear();

    // 1. Projeção Holográfica Translúcida (Anéis em rotação vertical e horizontal)
    const baseW = 28;
    const baseH = 14;
    const holoY = isoY - 16;

    // Cone de luz ascendente
    g.fillStyle(0x00f0ff, 0.08);
    g.beginPath();
    g.moveTo(isoX - 10, isoY);
    g.lineTo(isoX - baseW, holoY - 12);
    g.lineTo(isoX + baseW, holoY - 12);
    g.lineTo(isoX + 10, isoY);
    g.closePath();
    g.fillPath();

    // Anel holográfico inferior
    g.lineStyle(1.5, 0x00f0ff, 0.75);
    g.strokeEllipse(isoX, holoY, baseW * 2, baseH * 2);

    // Anel intermediário oscilante
    const midH = baseH * (0.6 + Math.sin(this.holoAngle) * 0.3);
    g.lineStyle(1, 0x39ff14, 0.65);
    g.strokeEllipse(isoX, holoY - 8, baseW * 1.5, Math.abs(midH) * 2);

    // Cubo / Diagrama de dados giratório no centro do holograma
    const cubeSize = 8;
    const offsetX = Math.cos(this.holoAngle) * 4;
    const offsetY = Math.sin(this.holoAngle) * 2;
    const cubeY = holoY - 14 + offsetY;

    g.lineStyle(1.5, 0x00f0ff, 0.9);
    g.strokeRect(isoX + offsetX - cubeSize / 2, cubeY - cubeSize / 2, cubeSize, cubeSize);

    // 2. Ondas concêntricas de despacho
    const pg = this.pulseGraphics;
    pg.setDepth(calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.PATH_OVERLAY));
    pg.clear();

    if (this.isPulsing) {
      this.pulseRadius += delta * 0.15;
      this.pulseAlpha -= delta * 0.0012;

      if (this.pulseAlpha <= 0) {
        this.isPulsing = false;
        this.pulseAlpha = 0;
      } else {
        // Onda isométrica concêntrica expandindo sobre a cidade
        pg.lineStyle(2, 0x00f0ff, this.pulseAlpha);
        pg.strokeEllipse(isoX, isoY, this.pulseRadius * 2, this.pulseRadius);

        // Onda secundária ligeiramente defasada
        if (this.pulseRadius > 30) {
          pg.lineStyle(1, 0x39ff14, this.pulseAlpha * 0.7);
          pg.strokeEllipse(isoX, isoY, (this.pulseRadius - 20) * 2, this.pulseRadius - 20);
        }
      }
    }
  }

  public destroy() {
    this.baseGraphics.destroy();
    this.holoGraphics.destroy();
    this.antennaGraphics.destroy();
    this.pulseGraphics.destroy();
  }
}
