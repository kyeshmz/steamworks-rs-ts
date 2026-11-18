const assert = require('node:assert/strict')
const path = require('node:path')
const { app } = require('electron')
const verify = require('./package-smoke')

app.whenReady().then(async () => {
    assert.equal(process.type, 'browser')
    assert.equal(process.platform, process.env.EXPECTED_PLATFORM)
    assert.equal(process.arch, process.env.EXPECTED_ARCH)
    await verify(path.resolve(process.argv[2]))
    console.log(`Electron ${process.versions.electron} verified the package on ${process.platform}-${process.arch}`)
    app.exit(0)
}).catch(error => {
    console.error(error)
    app.exit(1)
})
