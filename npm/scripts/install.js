const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');

const VERSION = require('../package.json').version;
const REPO = 'anamuthink/tabinal';
const MAX_REDIRECTS = 5;

function getPlatformBinary() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32' && arch === 'x64') return 'tabinal-windows-x64.exe';
  if (platform === 'darwin' && arch === 'arm64') return 'tabinal-macos-arm64';
  if (platform === 'darwin' && arch === 'x64') return 'tabinal-macos-x64';
  if (platform === 'linux' && arch === 'x64') return 'tabinal-linux-x64';

  console.error(`Unsupported platform: ${platform}-${arch}`);
  process.exit(1);
}

function download(url, dest, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > MAX_REDIRECTS) {
      reject(new Error(`Too many redirects (max ${MAX_REDIRECTS})`));
      return;
    }
    https.get(url, { headers: { 'User-Agent': 'tabinal-installer' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        download(res.headers.location, dest, redirects + 1).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`Download failed: HTTP ${res.statusCode}`));
        return;
      }
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', reject);
  });
}

function fetchText(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > MAX_REDIRECTS) {
      reject(new Error(`Too many redirects (max ${MAX_REDIRECTS})`));
      return;
    }
    https.get(url, { headers: { 'User-Agent': 'tabinal-installer' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        fetchText(res.headers.location, redirects + 1).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`Fetch failed: HTTP ${res.statusCode}`));
        return;
      }
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => resolve(data));
    }).on('error', reject);
  });
}

function sha256(filePath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash('sha256');
    const stream = fs.createReadStream(filePath);
    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('end', () => resolve(hash.digest('hex')));
    stream.on('error', reject);
  });
}

async function main() {
  const binaryName = getPlatformBinary();
  const baseUrl = `https://github.com/${REPO}/releases/download/v${VERSION}`;
  const url = `${baseUrl}/${binaryName}`;
  const binDir = path.join(__dirname, '..', 'bin');
  const isWindows = process.platform === 'win32';
  const dest = path.join(binDir, isWindows ? 'tabinal.exe' : 'tabinal');

  console.log(`Downloading tabinal v${VERSION} for ${process.platform}-${process.arch}...`);

  try {
    await download(url, dest);

    // Verify SHA-256 checksum
    try {
      const checksums = await fetchText(`${baseUrl}/checksums.txt`);
      const actual = await sha256(dest);
      const expected = checksums
        .split('\n')
        .find((line) => line.includes(binaryName));

      if (expected) {
        const expectedHash = expected.trim().split(/\s+/)[0];
        if (actual !== expectedHash) {
          fs.unlinkSync(dest);
          console.error('Checksum verification FAILED — downloaded binary does not match.');
          console.error(`  Expected: ${expectedHash}`);
          console.error(`  Actual:   ${actual}`);
          process.exit(1);
        }
        console.log('Checksum verified.');
      } else {
        console.warn('Warning: binary not found in checksums.txt, skipping verification.');
      }
    } catch (e) {
      // Checksums file may not exist for older releases — warn but continue
      console.warn(`Warning: could not verify checksum (${e.message})`);
    }

    if (!isWindows) {
      fs.chmodSync(dest, 0o755);
    }

    const BLUE = '\x1b[38;2;88;166;255m';
    const DIM = '\x1b[38;2;110;118;129m';
    const RESET = '\x1b[0m';
    console.log('');
    console.log(`${BLUE} ██████╗ ██████╗███╗   ███╗██╗   ██╗██╗  ██╗${RESET}`);
    console.log(`${BLUE}██╔════╝██╔════╝████╗ ████║██║   ██║╚██╗██╔╝${RESET}`);
    console.log(`${BLUE}██║     ██║     ██╔████╔██║██║   ██║ ╚███╔╝${RESET}`);
    console.log(`${BLUE}██║     ██║     ██║╚██╔╝██║██║   ██║ ██╔██╗${RESET}`);
    console.log(`${BLUE}╚██████╗╚██████╗██║ ╚═╝ ██║╚██████╔╝██╔╝ ██╗${RESET}`);
    console.log(`${BLUE} ╚═════╝ ╚═════╝╚═╝     ╚═╝ ╚═════╝╚═╝  ╚═╝${RESET}`);
    console.log('');
    console.log(`${DIM}  Claude Code Multiplexer v${VERSION}${RESET}`);
    console.log(`${DIM}  Run 'tabinal' to start.${RESET}`);
    console.log('');
  } catch (err) {
    console.error(`Failed to download tabinal: ${err.message}`);
    console.error(`URL: ${url}`);
    console.error('You can download manually from: https://github.com/anamuthink/tabinal/releases');
    process.exit(1);
  }
}

main();
