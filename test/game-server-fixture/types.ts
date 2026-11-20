import { gameServer } from '../../index'

gameServer.onServerEvent(event => {
    const user: bigint = event.user
    if (event.kind === 'approve') {
        const owner: bigint = event.owner
        void owner
    } else if (event.kind === 'group') {
        const group: bigint = event.group
        const member: boolean = event.member
        void [group, member]
    } else {
        const reason: number = event.reason
        void reason
    }
    void user
})

const packet = gameServer.getNextOutgoingPacket()
if (packet) {
    const data: Buffer = packet.data
    gameServer.handleIncomingPacket(data, packet.address, packet.port)
}
gameServer.requestUserGroupStatus(1n, 2n)
gameServer.updateListing({ serverName: 'Room', mapName: 'map', maxPlayers: 8, botPlayers: 2, passwordProtected: true })

const diagnostics = gameServer.diagnostics()
const appId: number = diagnostics.appId
const time: number = diagnostics.serverRealTime
gameServer.onP2pSessionRequest(({ steamId }) => gameServer.acceptP2pSession(steamId))
gameServer.onMessageSessionRequest(({ steamId }) => {
    if (steamId !== undefined) gameServer.acceptMessageSession(steamId)
})
gameServer.sendMessageToUser(1n, Buffer.from('test'), 0, true)
for (const message of gameServer.receiveMessages(0, 10)) {
    const data: Buffer = message.data
    void data
}
gameServer.openHostedListenSocket(0)
for (const event of gameServer.takeHostedEvents(10).events) {
    const connection: bigint = event.connection
    gameServer.acceptHostedConnection(connection)
    const messageNumber: bigint = gameServer.sendHostedMessage(connection, Buffer.from('test'), true)
    gameServer.receiveHostedMessages(connection, 10)
    gameServer.closeHostedConnection(connection)
    void messageNumber
}
gameServer.closeHostedListenSocket()
void [appId, time]
