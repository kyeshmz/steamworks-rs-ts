const path = require('node:path');
const fs = require('node:fs');
const child_process = require('node:child_process');

const targets = {
    ['x86_64-pc-windows-msvc']: {
        folder: 'win64',
        files: ['steam_api64.dll', 'steam_api64.lib'],
        platform: 'win32',
        arch: 'x64'
    },
    ['x86_64-unknown-linux-gnu']: {
        folder: 'linux64',
        files: ['libsteam_api.so'],
        platform: 'linux',
        arch: 'x64'
    },
    ['x86_64-apple-darwin']: {
        folder: 'osx',
        files: ['libsteam_api.dylib'],
        platform: 'darwin',
        arch: 'x64'
    },
    ['aarch64-apple-darwin']: {
        folder: 'osx',
        files: ['libsteam_api.dylib'],
        platform: 'darwin',
        arch: 'arm64'
    }
}

const args = process.argv.slice(2)
const targetArg = args.findIndex(arg => arg === '--target' || arg === '-t' || arg.startsWith('--target='))
const triple = targetArg === -1 ? undefined : args[targetArg].split('=')[1] || args[targetArg + 1]
const target = targetArg === -1
    ? Object.values(targets).find(t => t.platform === process.platform && t.arch === process.arch)
    : targets[triple]
if (!target) throw new Error(`No packaged Steam SDK for ${triple || `${process.platform}-${process.arch}`}`)

const dist = path.join(__dirname, 'dist', target.folder)
const metadata = JSON.parse(child_process.execFileSync('cargo', ['metadata', '--locked', '--format-version=1'], { cwd: __dirname, encoding: 'utf8' }))
const [bindings] = metadata.packages.filter(pkg => pkg.name === 'steamworks-sys')
const sdk = path.join(path.dirname(bindings.manifest_path), 'lib', 'steam')
const headers = path.join(__dirname, 'native', 'valve')
for (const header of fs.readdirSync(headers)) {
    if (!fs.readFileSync(path.join(headers, header)).equals(fs.readFileSync(path.join(sdk, 'public', 'steam', header)))) {
        throw new Error(`Vendored header ${header} differs from steamworks-sys ${bindings.version}`)
    }
}
const vendor = path.join(__dirname, 'sdk/redistributable_bin', target.folder)
const redist = path.join(sdk, 'redistributable_bin', target.folder)
fs.mkdirSync(dist, { recursive: true })
target.files.forEach(file => {
    const [source, dest] = [path.join(redist, file), path.join(dist, file)]
    if (!fs.readFileSync(source).equals(fs.readFileSync(path.join(vendor, file)))) {
        throw new Error(`Vendored ${target.folder}/${file} differs from steamworks-sys ${bindings.version}`)
    }
    fs.copyFileSync(source, dest)
})

function packageBin(pkg, executable) {
    const manifest = require.resolve(`${pkg}/package.json`)
    return path.resolve(path.dirname(manifest), require(manifest).bin[executable])
}
const params = [
    'build',
    '--platform',
    '--dts-header', '/// <reference types="node" />',
    '--no-js',
    '--const-enum',
    '--dts', '../../client.d.ts',
    '--output-dir', dist,
    ...args
]

child_process.execFileSync(process.execPath, [packageBin('@napi-rs/cli', 'napi'), ...params], {
    stdio: 'inherit',
    env: { ...process.env, STEAM_SDK_LOCATION: sdk }
})
