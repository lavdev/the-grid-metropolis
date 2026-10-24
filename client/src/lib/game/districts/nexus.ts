import Phaser from 'phaser';
import { cartesianToIsometric, TILE_WIDTH, TILE_HEIGHT } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

/**
 * TheNexusSector: A Cidadela Central de Orquestração & Despacho (The Nexus).
 * Uma imponente edificação octogonal de comando com múltiplos pavimentos,
 * paredes volumétricas de aço blindado, painéis de dados luminescentes,
 * praça superior elevada com a mesa holográfica translúcida e torre de transmissão.
 */
export class TheNexusSector {
  private scene: Phaser.Scene;
  private baseGraphics: Phaser.GameObjects.Graphics;
  private holoGraphics: Phaser.GameObjects.Graphics;
  private pulseGraphics: Phaser.GameObjects.Graphics;

  private holoAngle = 0;
  private pulseRadius = 0;
  private isPulsing = false;
  private pulseAlpha = 0;

  // Centro da Cidadela
  private readonly centerX = 0;
  private readonly centerY = 0;

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
    this.baseGraphics = this.scene.add.graphics();
    this.holoGraphics = this.scene.add.graphics();
    this.pulseGraphics = this.scene.add.graphics();

    this.renderCitadel();
  }

  /**
   * Renderiza a edificação volumétrica completa da Cidadela The Nexus
   */
  public renderCitadel() {
    const g = this.baseGraphics;
    const { x: isoX, y: isoY } = cartesianToIsometric(this.centerX, this.centerY);
    const depth = calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.STRUCTURE_BASE);

    g.setDepth(depth);
    g.clear();

    const halfW = 48; // Largura do bloco da cidadela
    const halfH = 24; // Profundidade do bloco
    const wallH = 34; // Altura volumétrica das paredes

    // ==========================================
    // 1. FUNDAÇÃO E PAREDES VOLUMÉTRICAS (TÉRREO)
    // ==========================================

    // Parede Sudoeste (Face Esquerda)
    g.fillStyle(0x0a1626, 0.98); // Aço escuro de comando
    g.beginPath();
    g.moveTo(isoX - halfW, isoY);
    g.lineTo(isoX, isoY + halfH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.lineTo(isoX - halfW, isoY - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x15314f, 0.9);
    g.strokePath();

    // Detalhes da Parede Esquerda: Pilares verticais e janelas de servidores
    for (let p = 1; p <= 3; p++) {
      const px = isoX - halfW + (p * halfW) / 4;
      const py = isoY + (p * halfH) / 4;
      // Pilar de reforço
      g.fillStyle(0x0e2035, 1);
      g.fillRect(px - 2, py - wallH, 4, wallH);
      // Janela horizontal de fluxo de dados
      g.fillStyle(0x00f0ff, 0.7);
      g.fillRect(px + 4, py - wallH + 10, 6, 2);
    }

    // Parede Sudeste (Face Direita - Fachada Principal)
    g.fillStyle(0x07111e, 0.98); // Sombra mais intensa
    g.beginPath();
    g.moveTo(isoX, isoY + halfH);
    g.lineTo(isoX + halfW, isoY);
    g.lineTo(isoX + halfW, isoY - wallH);
    g.lineTo(isoX, isoY + halfH - wallH);
    g.closePath();
    g.fillPath();

    g.lineStyle(1.5, 0x15314f, 0.9);
    g.strokePath();

    // Portal de acesso central da Cidadela
    const gateW = 18;
    const gateH = 20;
    g.fillStyle(0x040910, 1);
    g.fillRect(isoX + 10, isoY + halfH - 22, gateW, gateH);
    g.lineStyle(1.5, 0x00f0ff, 0.9);
    g.strokeRect(isoX + 10, isoY + halfH - 22, gateW, gateH);

    // Linha de energia do portal
    g.fillStyle(0x39ff14, 0.9);
    g.fillRect(isoX + 12, isoY + halfH - 24, gateW - 4, 2);

    // ==========================================
    // 2. PRAÇA ELEVADA (ROOFTOP PLAZA)
    // ==========================================
    const roofY = isoY - wallH;

    // Piso do terraço superior
    g.fillStyle(0x0d2036, 1);
    g.beginPath();
    g.moveTo(isoX, roofY - halfH);
    g.lineTo(isoX + halfW, roofY);
    g.lineTo(isoX, roofY + halfH);
    g.lineTo(isoX - halfW, roofY);
    g.closePath();
    g.fillPath();

    // Guarda-corpo e parapeito iluminado em néon ciano
    g.lineStyle(2, 0x00f0ff, 0.85);
    g.strokePath();

    // Detalhe geométrico octogonal interno no piso do terraço
    g.lineStyle(1, 0x1e456e, 0.6);
    g.beginPath();
    g.moveTo(isoX, roofY - halfH + 8);
    g.lineTo(isoX + halfW - 14, roofY);
    g.lineTo(isoX, roofY + halfH - 8);
    g.lineTo(isoX - halfW + 14, roofY);
    g.closePath();
    g.strokePath();

    // ==========================================
    // 3. MESA HOLOGRÁFICA ELEVADA NO CENTRO DO TERRAÇO
    // ==========================================
    const tableW = 20;
    const tableH = 10;
    const tableElevation = 8;
    const tCenterY = roofY;

    // Pedestal da mesa
    g.fillStyle(0x06111f, 1);
    g.fillRect(isoX - 8, tCenterY - tableElevation, 16, tableElevation);

    // Tampo circular da mesa holográfica
    g.fillStyle(0x0f2d4d, 1);
    g.fillEllipse(isoX, tCenterY - tableElevation, tableW * 2, tableH * 2);
    g.lineStyle(1.5, 0x00f0ff, 1);
    g.strokeEllipse(isoX, tCenterY - tableElevation, tableW * 2, tableH * 2);

    // Ponto emissor central
    g.fillStyle(0x00f0ff, 1);
    g.fillCircle(isoX, tCenterY - tableElevation, 3);

    // ==========================================
    // 4. TORRE DE TRANSMISSÃO E ANTENA PARABÓLICA
    // ==========================================
    const towerBaseX = isoX - 16;
    const towerBaseY = roofY - 10;
    const towerHeight = 56;
    const towerTopY = towerBaseY - towerHeight;

    // Treliça metálica estrutural da torre
    g.lineStyle(2, 0x182d45, 1);
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX - 6, towerBaseY, towerBaseX - 2, towerTopY));
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX + 6, towerBaseY, towerBaseX + 2, towerTopY));
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX - 5, towerBaseY - towerHeight * 0.4, towerBaseX + 5, towerBaseY - towerHeight * 0.4));
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX - 3, towerBaseY - towerHeight * 0.7, towerBaseX + 3, towerBaseY - towerHeight * 0.7));

    // Prato da super-antena parabólica
    g.fillStyle(0x0b1c2e, 1);
    g.fillEllipse(towerBaseX, towerTopY, 28, 14);
    g.lineStyle(2, 0x00f0ff, 0.9);
    g.strokeEllipse(towerBaseX, towerTopY, 28, 14);

    // Emitter focal da parabólica
    g.lineStyle(2, 0x9d00ff, 1);
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX, towerTopY, towerBaseX - 6, towerTopY - 12));

    // Luz de baliza aeronáutica de topo (Beacon ciano pulsante)
    g.fillStyle(0x00f0ff, 1);
    g.fillCircle(towerBaseX - 6, towerTopY - 12, 3);

    // Cabos tensores de ancoragem descendo para o terraço
    g.lineStyle(1, 0x00f0ff, 0.25);
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX, towerTopY + 10, isoX - halfW + 10, roofY));
    g.strokeLineShape(new Phaser.Geom.Line(towerBaseX, towerTopY + 10, isoX, roofY - halfH + 4));
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
    const wallH = 34;
    const tableElevation = 8;
    const tCenterY = isoY - wallH - tableElevation;

    const holoDepth = calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.STRUCTURE_ROOF);
    const g = this.holoGraphics;
    g.setDepth(holoDepth);
    g.clear();

    // 1. Projeção Holográfica Translúcida
    const baseW = 20;
    const baseH = 10;
    const holoY = tCenterY - 12;

    // Feixe cônico de luz ascendente
    g.fillStyle(0x00f0ff, 0.09);
    g.beginPath();
    g.moveTo(isoX - 6, tCenterY);
    g.lineTo(isoX - baseW, holoY - 14);
    g.lineTo(isoX + baseW, holoY - 14);
    g.lineTo(isoX + 6, tCenterY);
    g.closePath();
    g.fillPath();

    // Anel holográfico inferior
    g.lineStyle(1.5, 0x00f0ff, 0.85);
    g.strokeEllipse(isoX, holoY, baseW * 2, baseH * 2);

    // Anel intermediário oscilante
    const midH = baseH * (0.6 + Math.sin(this.holoAngle) * 0.35);
    g.lineStyle(1, 0x39ff14, 0.7);
    g.strokeEllipse(isoX, holoY - 8, baseW * 1.5, Math.abs(midH) * 2);

    // Cubo de dados giratório
    const cubeSize = 8;
    const offsetX = Math.cos(this.holoAngle) * 4;
    const offsetY = Math.sin(this.holoAngle) * 2;
    const cubeY = holoY - 14 + offsetY;

    g.lineStyle(1.5, 0x00f0ff, 0.95);
    g.strokeRect(isoX + offsetX - cubeSize / 2, cubeY - cubeSize / 2, cubeSize, cubeSize);

    // 2. Ondas concêntricas de despacho
    const pg = this.pulseGraphics;
    pg.setDepth(calculateIsometricDepth(this.centerX, this.centerY, DepthLayer.PATH_OVERLAY));
    pg.clear();

    if (this.isPulsing) {
      this.pulseRadius += delta * 0.18;
      this.pulseAlpha -= delta * 0.0012;

      if (this.pulseAlpha <= 0) {
        this.isPulsing = false;
        this.pulseAlpha = 0;
      } else {
        pg.lineStyle(2, 0x00f0ff, this.pulseAlpha);
        pg.strokeEllipse(isoX, isoY, this.pulseRadius * 2, this.pulseRadius);

        if (this.pulseRadius > 35) {
          pg.lineStyle(1.5, 0x39ff14, this.pulseAlpha * 0.7);
          pg.strokeEllipse(isoX, isoY, (this.pulseRadius - 24) * 2, this.pulseRadius - 24);
        }
      }
    }
  }

  public destroy() {
    this.baseGraphics.destroy();
    this.holoGraphics.destroy();
    this.pulseGraphics.destroy();
  }
}
