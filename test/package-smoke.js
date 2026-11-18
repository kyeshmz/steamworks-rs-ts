const assert = require('node:assert/strict')
const path = require('node:path')

async function verify(root) {
    const sdk = require(root)
    assert.equal(sdk.SteamCallback.PersonaStateChange, 0)
    assert.equal(sdk.SteamCallback.GameOverlayActivated, 10)

    const { platform, arch } = process
    const target = {
        'linux-x64': 'linux64/steamworksjs.linux-x64-gnu.node',
        'win32-x64': 'win64/steamworksjs.win32-x64-msvc.node',
        'darwin-arm64': 'osx/steamworksjs.darwin-arm64.node',
        'darwin-x64': 'osx/steamworksjs.darwin-x64.node',
    }[`${platform}-${arch}`]
    const native = require(path.join(root, 'dist', target))
    await assert.rejects(native.cloud.readFileBytes(''), /non-empty cloud filename/)
    await assert.rejects(native.cloud.writeFileBytes('a\0b', Buffer.alloc(0)), /NUL bytes/)
    assert.throws(() => native.networking.readP2PPacket(-1), /packet size/)
    assert.equal(typeof native.workshop.deleteItem, 'function')
    assert.equal(typeof native.utils.isOverlayEnabled, 'function')
    await assert.rejects(native.leaderboards.findLeaderboard(''), /Leaderboard name/)
    assert.throws(() => native.friends.getPersonaName(-1n), /Steam ID/)
    assert.throws(() => native.inventory.getItemDefinitionProperty(1, 'a\0b'), /NUL byte/)
    assert.equal(native.stats.setFloat('speed', Infinity), false)
    assert.throws(() => native.input.getGlyphSvgForActionOrigin(-1), /action origin/)
    assert.throws(() => native.networking_messages.receiveMessagesOnChannel(0, 33), /maxMessages/)
    assert.throws(() => native.networking_messages.acceptSessionWithUser(-1n), /SteamID64/)
    assert.throws(() => native.screenshots.addScreenshotToLibrary('test.jpg', null, 0, 1), /positive/)
    console.log(`Native package verified on ${platform}-${arch}`)
}

verify(path.resolve(process.argv[2] || '.')).catch(error => {
    console.error(error)
    process.exitCode = 1
})
