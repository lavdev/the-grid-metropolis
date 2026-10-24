import Phaser from 'phaser';
import { cartesianToIsometric, type CartesianPoint } from '../isometric';
import { DepthLayer, calculateIsometricDepth } from '../depth';

/**
 * DroneEntity: Drone aéreo de mensageria que patrulha e transporta pacotes de dados
 * sobre as fendas e abismos da cidade entre os prédios e o Nexus.
 */
export class DroneEntity {
  private scene: Phaser.Scene;
  private graphics: Phaser.GameObjects.Graphics;
  private packetGraphics: Phaser.GameObjects.Graphics;

  public cartX: number;
  public cartY: number;
  private waypoints: CartesianPoint[];
  private currentWaypointIndex = 0;
  private flightProgress = 0;
  private hoverPhase = 0;
  private rotorAngle = 0;
  private isCarryingData = true;

  constructor(scene: Phaser.Scene, startX: number = 6, startY: number = 0) {
    this.scene = scene;
    this.cartX = startX;
    this.cartY = startY;

    this.graphics = scene.add.graphics();
    this.packetGraphics = scene.add.graphics();

    // Rota de patrulha aérea ligando Nexus -> Pod Notex -> Glitch Alley -> Battery Lounge
    this.waypoints = [
      { x: 0, y: 0 },    // Nexus
      { x: 12, y: 0 },   // Pod Notex
      { x: 0, y: -12 },  // Glitch Alley
      { x: -12, y: 0 },  // Battery Lounge
      { x: 0, y: 12 },   // Expansion Bay
    ];
  }

  public update(delta: number) {
    this.hoverPhase += delta * 0.004;
    this.rotorAngle += delta * 0.04;

    // 1. Navegação aérea contínua entre waypoints
    const pCurrent = this.waypoints[this.currentWaypointIndex];
    const nextIndex = (this.currentWaypointIndex + 1) % this.waypoints.length;
    const pNext = this.waypoints[nextIndex];

    this.flightProgress += 0.002 * (delta * 0.06);
    if (this.flightProgress >= 1) {
      this.flightProgress = 0;
      this.currentWaypointIndex = nextIndex;
      this.isCarryingData = !this.isCarryingData;
    }

    this.cartX = Phaser.Math.Linear(pCurrent.x, pNext.x, this.flightProgress);
    this.cartY = Phaser.Math.Linear(pCurrent.y, pNext.y, this.flightProgress);

    const { x: isoX, y: isoY } = cartesianToIsometric(this.cartX, this.cartY);
    const altitude = 42 + Math.sin(this.hoverPhase) * 4; // Flutuação suave no ar
    const droneY = isoY - altitude;

    const depth = calculateIsometricDepth(this.cartX, this.cartY, DepthLayer.DRONE_AIR);
    this.graphics.setDepth(depth);
    this.packetGraphics.setDepth(depth - 1);

    const g = this.graphics;
    g.clear();

    // Sombra do drone projetada na plataforma ou no abismo
    g.fillStyle(0x000000, 0.35);
    g.fillEllipse(isoX, isoY, 18, 9);

    // Chassis principal do drone (formato em X aerodinâmico)
    g.fillStyle(0x0a1622, 1);
    g.fillRect(isoX - 6, droneY - 4, 12, 8);
    g.lineStyle(1.5, 0x00f0ff, 0.9);
    g.strokeRect(isoX - 6, droneY - 4, 12, 8);

    // Braços dos 4 rotores
    g.lineStyle(1.5, 0x1f293d, 1);
    g.strokeLineShape(new Phaser.Geom.Line(isoX - 10, droneY - 6, isoX + 10, droneY + 6));
    g.strokeLineShape(new Phaser.Geom.Line(isoX - 10, droneY + 6, isoX + 10, droneY - 6));

    // 4 Hélices giratórias em alta velocidade
    const rotorOffsets = [
      { dx: -10, dy: -6 },
      { dx: 10, dy: -6 },
      { dx: -10, dy: 6 },
      { dx: 10, dy: 6 },
    ];

    for (const r of rotorOffsets) {
      const rx = isoX + r.dx;
      const ry = droneY + r.dy;

      // Disco semitransparente do rotor
      g.fillStyle(0x00f0ff, 0.25);
      g.fillCircle(rx, ry, 4);

      // Pás do rotor girando
      const rdx = Math.cos(this.rotorAngle) * 4;
      const rdy = Math.sin(this.rotorAngle) * 4;
      g.lineStyle(1, 0xffffff, 0.8);
      g.strokeLineShape(new Phaser.Geom.Line(rx - rdx, ry - rdy, rx + rdx, ry + rdy));
    }

    // Luz de sinalização do drone (LED frontal pulsante)
    g.fillStyle(0x39ff14, 1);
    g.fillCircle(isoX, droneY + 3, 2);

    // 2. Pacote de dados translúcido transportado
    const pg = this.packetGraphics;
    pg.clear();

    if (this.isCarryingData) {
      const pkgY = droneY + 8;
      pg.fillStyle(0x00f0ff, 0.85);
      pg.fillRect(isoX - 3, pkgY - 3, 6, 6);
      pg.lineStyle(1, 0xffffff, 0.9);
      pg.strokeRect(isoX - 3, pkgY - 3, 6, 6);

      // Brilho sob o pacote
      pg.fillStyle(0x00f0ff, 0.35);
      pg.fillCircle(isoX, pkgY, 6);
    }
  }

  public destroy() {
    this.graphics.destroy();
    this.packetGraphics.destroy();
  }
}
