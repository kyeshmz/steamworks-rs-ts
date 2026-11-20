const assert = require('node:assert/strict')
const { execFileSync } = require('node:child_process')
const path = require('node:path')

if (process.argv[2] === '--inspect') {
    const base = path.resolve(process.argv[3])
    const binary = {
        'linux-x64': 'linux64/steamworksjs.linux-x64-gnu.node',
        'win32-x64': 'win64/steamworksjs.win32-x64-msvc.node',
        'darwin-x64': 'osx/steamworksjs.darwin-x64.node',
        'darwin-arm64': 'osx/steamworksjs.darwin-arm64.node',
    }[`${process.platform}-${process.arch}`]
    const surface = {}
    function inspect(value, prefix, prototype = false) {
        for (const name of Object.getOwnPropertyNames(value)) {
            if (prototype && name === 'constructor') continue
            const key = `${prefix}.${name}`
            const { value: member, get, set } = Object.getOwnPropertyDescriptor(value, name)
            if (get || set) {
                surface[key] = [get && 'get', set && 'set'].filter(Boolean)
            } else if (typeof member === 'function') {
                surface[key] = 'function'
                if (member.prototype) inspect(member.prototype, key + '.prototype', true)
            } else if (member && typeof member === 'object') {
                surface[key] = 'object'
                inspect(member, key)
            } else {
                surface[key] = member
            }
        }
    }
    inspect(require(base), 'package')
    inspect(require(path.join(base, 'dist', binary)), 'native')
    process.stdout.write(JSON.stringify(surface))
} else {
    const inspect = root => JSON.parse(execFileSync(process.execPath, [__filename, '--inspect', root]))
    assert.ok(process.argv[2], 'Pass the installed upstream steamworks.js package directory')
    const upstream = inspect(process.argv[2])
    const candidate = inspect(process.argv[3] || path.resolve(__dirname, '../..'))
    for (const [key, expected] of Object.entries(upstream)) assert.deepEqual(candidate[key], expected, key)
    console.log(`Matched all ${Object.keys(upstream).length} root and native paths, including enum values and class members`)
}
