const assert = require('node:assert/strict')
const steam = require(process.argv[2])
const remote = steam.init(3104020).remote_play

if (process.argv[3] === 'too-many') {
    assert.throws(() => remote.sessions(), /count exceeds limit/)
} else {
    assert.deepEqual(remote.sessions(), [4294967295, 42])
    const session = remote.session(4294967295)
    assert.equal(session.id, 4294967295)
    assert.equal(session.user, 76561198012345678n)
    assert.equal(session.clientName, 'Living Room')
    assert.equal(session.clientFormFactor, 'tv')
    assert.deepEqual(session.clientResolution, { width: 1920, height: 1080 })
    const unknown = remote.session(7)
    assert.equal(unknown.user, 1n)
    assert.equal(unknown.clientName, 'Unknown Device')
    assert.equal(unknown.clientFormFactor, undefined)
    assert.equal(unknown.clientResolution, undefined)
    assert.equal(remote.session(8).clientFormFactor, 'phone')
    assert.equal(remote.session(9).clientFormFactor, 'tablet')
    assert.equal(remote.session(10).clientFormFactor, 'computer')
    assert.equal(remote.session(10).clientResolution, undefined)
    assert.equal(remote.session(42), null)
    for (const id of [0, -1, 1.5, 4294967296, NaN, Infinity]) {
        assert.throws(() => remote.session(id), /session ID/)
    }
    assert.equal(remote.invite(76561198012345678n), true)
    assert.equal(remote.invite(18446744073709551615n), false)
    for (const id of [0n, -1n, 18446744073709551616n]) {
        assert.throws(() => remote.invite(id), /Steam ID/)
    }
}
console.log(`Remote Play fixture passed: ${process.argv[3]}`)
process.exit(0)
