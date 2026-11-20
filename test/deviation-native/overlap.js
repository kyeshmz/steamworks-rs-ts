const assert = require('node:assert/strict')
const steam = require(process.argv[2])
const scenario = process.argv[3]
const id = 76561198012345678n

async function run() {
    const client = steam.init(3104020)
    assert.equal(client.stats.store(), true)
    const isWeb = scenario.includes('web') || scenario === 'overlap-failure' || scenario === 'overlap-length'
    const issue = (marker, seconds = 4) => isWeb
        ? client.auth.getAuthTicketForWebApi(`peer-${marker}`, seconds)
        : client.auth.getSessionTicketWithSteamId(id + BigInt(marker), seconds)
    const expected = marker => isWeb
        ? Buffer.from([0xca, 0xfe, 0, marker])
        : Buffer.from([0xf1, 0, marker])
    const check = (ticket, marker) => {
        const bytes = ticket.getBytes()
        ticket.cancel()
        assert.deepEqual(bytes, expected(marker))
    }
    if (scenario === 'overlap-session' || scenario === 'overlap-web') {
        const results = await Promise.allSettled([issue(1), issue(2)])
        for (let i = 0; i < results.length; i++) {
            if (results[i].status === 'fulfilled') check(results[i].value, i + 1)
        }
        const statuses = isWeb ? ['fulfilled', 'fulfilled'] : ['fulfilled', 'rejected']
        assert.deepEqual(results.map(result => result.status), statuses, results.map(result => String(result.reason)).join(', '))
        if (!isWeb) assert.match(results[1].reason.message, /generic failure/i)
        console.log(`${scenario}: reversed callbacks reached the correct requests`)
    } else if (scenario === 'overlap-late-session' || scenario === 'overlap-late-web') {
        await assert.rejects(issue(3, 1), /tim(?:e|ed|ing)|deadline/i)
        console.log(`${scenario}: first request timed out`)
        check(await issue(4), 4)
        console.log(`${scenario}: later request succeeded after late callback`)
        check(await issue(5), 5)
        console.log(`${scenario}: callback pump still handles a subsequent request`)
    } else if (scenario === 'overlap-failure') {
        await assert.rejects(issue(1), error => {
            assert.doesNotMatch(error.message, /tim(?:e|ed|ing)|deadline/i)
            console.log(`failed web ticket error: ${error.message}`)
            return true
        })
        console.log('failed web ticket rejected and SDK handle canceled')
    } else if (scenario === 'overlap-length') {
        for (const marker of [1, 2, 3]) await assert.rejects(issue(marker), /length|size/i)
        check(await issue(4), 4)
        console.log('malformed ticket lengths rejected; subsequent valid ticket delivered')
    } else if (scenario === 'overlap-cancel') {
        const ticket = await issue(1)
        assert.deepEqual(ticket.getBytes(), expected(1))
        ticket.cancel()
        ticket.cancel()
        console.log('manual cancellation is idempotent')
    } else {
        throw new Error(`Unknown scenario ${scenario}`)
    }
}

run().then(() => process.exit(0), error => { console.error(error); process.exit(1) })
