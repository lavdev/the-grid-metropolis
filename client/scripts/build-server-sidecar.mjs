import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const clientDirectory = path.resolve(scriptDirectory, '..');
const repositoryDirectory = path.resolve(clientDirectory, '..');
const profile = process.argv[2] === 'release' ? 'release' : 'debug';

function resolveHostTarget() {
  const configuredTarget = process.env.TAURI_ENV_TARGET_TRIPLE;
  if (configuredTarget) return configuredTarget;

  const rustcVersion = execFileSync('rustc', ['-vV'], { encoding: 'utf8' });
  const hostTarget = rustcVersion.match(/^host: (.+)$/m)?.[1];
  if (!hostTarget) throw new Error('Não foi possível identificar a plataforma Rust atual.');
  return hostTarget;
}

const targetTriple = resolveHostTarget();
const buildArguments = [
  'build',
  '--manifest-path',
  path.join(repositoryDirectory, 'server', 'Cargo.toml'),
  '--bin',
  'server',
  '--target',
  targetTriple,
];

if (profile === 'release') buildArguments.push('--release');

const cargoResult = spawnSync('cargo', buildArguments, { cwd: repositoryDirectory, stdio: 'inherit' });
if (cargoResult.error) throw cargoResult.error;
if (cargoResult.status !== 0) process.exit(cargoResult.status ?? 1);

const platformExtension = targetTriple.includes('windows') ? '.exe' : '';
const builtServer = path.join(repositoryDirectory, 'server', 'target', targetTriple, profile, `server${platformExtension}`);
const sidecarDirectory = path.join(clientDirectory, 'src-tauri', 'binaries');
const sidecarPath = path.join(sidecarDirectory, `grid-metropolis-server-${targetTriple}${platformExtension}`);

mkdirSync(sidecarDirectory, { recursive: true });
copyFileSync(builtServer, sidecarPath);
process.stdout.write(`Backend preparado para Tauri: ${sidecarPath}\n`);
