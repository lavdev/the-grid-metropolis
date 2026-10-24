import Phaser from 'phaser';
import { MainIsometricScene } from './scenes/MainIsometricScene';

export function createGridMetropolisGame(parent: HTMLElement): Phaser.Game {
  const config: Phaser.Types.Core.GameConfig = {
    type: Phaser.WEBGL,
    parent,
    width: parent.clientWidth || window.innerWidth,
    height: parent.clientHeight || window.innerHeight,
    backgroundColor: '#05070c',
    pixelArt: true,
    roundPixels: true,
    scale: {
      mode: Phaser.Scale.RESIZE,
      autoCenter: Phaser.Scale.CENTER_BOTH,
    },
    scene: [MainIsometricScene],
  };

  return new Phaser.Game(config);
}

export function destroyGridMetropolisGame(game: Phaser.Game | null) {
  if (game) {
    game.destroy(true);
  }
}
