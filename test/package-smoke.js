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
    assert.throws(() => native.networking.isP2PPacketAvailableOnChannel(1.5), /channel/)
    assert.throws(() => native.cloud.setSyncPlatforms('save', 128), /platform mask/)
    assert.throws(() => native.friends.inviteUserToGame(1n, 'a\0b'), /NUL/)
    assert.throws(() => native.friends.getRichPresence(1n, ''), /Rich presence keys/)
    assert.throws(() => native.screenshots.writeScreenshot(Buffer.alloc(4), 1, 1), /RGB bytes/)
    assert.throws(() => native.screenshots.tagUser(1, -1n), /64-bit ID/)
    assert.throws(() => native.voice.decompressVoice(Buffer.from([1]), 8000), /sample rate|sampleRate/i)
    assert.equal(sdk.SteamCallback.GameRichPresenceJoinRequested, 12)
    await assert.rejects(native.inventory.getItemsByID([1n << 64n]), /unsigned 64-bit/)
    assert.throws(() => native.networking_sockets.createListenSocketP2P(-1), /virtualPort/)
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
