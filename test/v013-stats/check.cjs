const assert = require('node:assert/strict')
const sdk = require(process.argv[2])
const api = sdk.init(3104020)
async function main() {
const id = 76561198012345678n
assert.deepEqual(await api.leaderboards.downloadLeaderboardEntries('scores', api.leaderboards.LeaderboardDataRequest.GlobalAroundUser, -1, 1, 2), [
    { steamId: id, globalRank: 3, score: 50, details: [-5, 23] },
])
for (const days of [-1, 1.5, 61, Infinity]) await assert.rejects(api.stats.requestGlobalStats(days), /History days/)
const [first, second] = await Promise.all([
    api.stats.requestGlobalStats(1),
    api.stats.requestGlobalStats(2),
])
assert.equal(first, 9007199254740995n)
assert.equal(second, 9007199254740996n)
await assert.rejects(api.stats.requestGlobalStats(3), /io/i)
assert.equal(await api.stats.requestGlobalStats(4), 9007199254740998n)
assert.equal(api.stats.getGlobalInt('value'), -9007199254740993n)
assert.equal(api.stats.getGlobalInt('missing'), null)
assert.equal(api.stats.getGlobalFloat('value'), 1.25)
assert.equal(api.stats.getGlobalFloat('missing'), null)
assert.deepEqual(api.stats.getGlobalIntHistory('value', 2), [-9007199254740993n, 9007199254740993n])
assert.deepEqual(api.stats.getGlobalFloatHistory('value', 2), [1.25, -3.5])
assert.equal(api.stats.getGlobalIntHistory('missing', 2), null)
assert.equal(api.stats.getGlobalFloatHistory('missing', 2), null)
assert.throws(() => api.stats.getGlobalIntHistory('value', 61), /History days/)
for (const days of [-1, 1.5, Infinity]) assert.throws(() => api.stats.getGlobalFloatHistory('value', days), /History days/)
assert.equal(api.stats.getGlobalInt('a\0b'), null)
assert.deepEqual(api.achievement.getAchievementAndUnlockTime('value'), { achieved: true, unlockTime: 123456 })
assert.equal(api.achievement.getAchievementAndUnlockTime('missing'), null)
assert.deepEqual(api.achievement.getIconRgba('value'), { width: 2, height: 2, rgba: Buffer.from(Array.from({ length: 16 }, (_, i) => i)) })
assert.equal(api.achievement.getIconRgba('missing'), null)
assert.equal(api.achievement.getIconRgba('large'), null)
assert.equal(api.achievement.getIconRgba('size-error'), null)
assert.equal(api.achievement.getIconRgba('rgba-error'), null)
assert.equal(api.achievement.getIconRgba('a\0b'), null)
assert.deepEqual(api.localplayer.getSteamIdDetails(id), { universe: 1, accountType: 1, isInvalid: false })
assert.deepEqual(api.localplayer.getSteamIdDetails(0n), { universe: 0, accountType: 0, isInvalid: true })
for (const universe of [1, 2, 3, 4, 5]) {
    const steamId = (BigInt(universe) << 56n) | (1n << 52n) | 1n
    assert.deepEqual(api.localplayer.getSteamIdDetails(steamId), {
        universe: universe <= 4 ? universe : 0,
        accountType: 1,
        isInvalid: false,
    })
}
for (const [accountType, expected, isInvalid] of [[0, 0, true], [10, 10, false], [11, 0, false]]) {
    const steamId = (1n << 56n) | (BigInt(accountType) << 52n) | 1n
    assert.deepEqual(api.localplayer.getSteamIdDetails(steamId), { universe: 1, accountType: expected, isInvalid })
}
for (const value of [-1n, 1n << 64n]) {
    assert.throws(() => api.localplayer.getSteamIdDetails(value), /unsigned 64-bit/)
    assert.throws(() => api.localplayer.userHasLicenseForApp(value, 1), /unsigned 64-bit/)
    assert.throws(() => api.stats.requestUserStats(value), /unsigned 64-bit/)
    assert.throws(() => api.stats.getUserStatFloat(value, 'value'), /unsigned 64-bit/)
}
assert.throws(() => api.stats.requestUserStats(0n), /nonzero/)
assert.throws(() => api.stats.requestUserStats(id + 1n), /rejected/)
api.stats.requestUserStats(id)
assert.equal(api.stats.getUserStatInt(id, 'value'), -42)
assert.equal(api.stats.getUserStatFloat(id, 'value'), 1.25)
assert.equal(api.stats.getUserAchievement(id, 'unlocked'), true)
assert.equal(api.stats.getUserAchievement(id, 'locked'), false)
assert.equal(api.stats.getUserStatInt(id, 'missing'), null)
assert.equal(api.stats.getUserStatFloat(id, 'missing'), null)
assert.equal(api.stats.getUserAchievement(id, 'missing'), null)
assert.throws(() => api.stats.getUserStatInt(0n, 'value'), /nonzero/)
assert.throws(() => api.stats.getUserAchievement(id, 'a\0b'), /NUL/)
assert.equal(api.localplayer.userHasLicenseForApp(id, 1), 0)
assert.equal(api.localplayer.userHasLicenseForApp(id, 2), 1)
assert.equal(api.localplayer.userHasLicenseForApp(id, 3), 2)
assert.equal(api.apps.launchQueryParam('token'), 'fixture value')
assert.throws(() => api.apps.launchQueryParam('a\0b'), /NUL/)
api.overlay.activateInviteDialogConnectString('connect')
assert.throws(() => api.overlay.activateInviteDialogConnectString('a\0b'), /NUL/)
api.timeline.setTimelineGameMode(1)
api.timeline.setTimelineStateDescription('state', 2.5)
api.timeline.clearTimelineStateDescription(3)
api.timeline.addTimelineEvent('icon', 'title', 'description', 7, -2, 4, 2)
assert.throws(() => api.timeline.setTimelineStateDescription('a\0b', 1), /NUL/)
assert.throws(() => api.timeline.clearTimelineStateDescription(-1), /Duration/)
assert.throws(() => api.timeline.addTimelineEvent('i', 't', 'd', 0, Infinity, 1, 0), /offset/)
for (const priority of [-1, 1.5, 1001]) assert.throws(() => api.timeline.addTimelineEvent('i', 't', 'd', priority, 0, 1, 0), /priority/)
assert.throws(() => api.timeline.addTimelineEvent('i', 't', 'd', 1, 0, 601, 0), /duration/)
await assert.rejects(api.stats.requestGlobalStats(5), /Timed out/)
console.log('Stats and timeline fixture passed')
}
main().then(() => process.exit(0), error => { console.error(error); process.exit(1) })
