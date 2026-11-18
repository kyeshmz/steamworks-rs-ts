const assert = require('node:assert/strict')
const steam = require('..')

const appId = Number(process.argv[2])
if (!Number.isInteger(appId) || appId < 1 || appId > 0xffffffff) {
    throw new Error('Usage: node test/steam-readonly.js <Steam App ID>')
}
const client = steam.init(appId)
assert.equal(client.utils.getAppId(), appId)
assert.equal(typeof client.utils.isOverlayEnabled(), 'boolean')
assert.equal(typeof client.utils.uiLanguage(), 'string')
assert.ok(Array.isArray(client.friends.getFriends()))
assert.ok(Array.isArray(client.achievement.names()))
assert.equal(typeof client.apps.launchCommandLine(), 'string')
assert.equal(typeof client.localplayer.isLoggedOn(), 'boolean')
console.log(`Read-only Steam checks passed for App ${appId}`)
process.exit(0)
