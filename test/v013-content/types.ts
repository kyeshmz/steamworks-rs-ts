import * as steam from '../..'

export async function content(client: ReturnType<typeof steam.init>, lobby: Awaited<ReturnType<typeof client.matchmaking.createLobby>>) {
    const ids: bigint[] = client.workshop.getSubscribedItems()
    client.workshop.getSubscribedItems(true)
    await client.workshop.startPlaytimeTracking(ids)
    await client.workshop.stopPlaytimeTracking(ids)
    await client.workshop.stopPlaytimeTrackingForAllItems()
    client.workshop.suspendDownloads(false)
    await client.workshop.updateItem(42n, { language: 'german' })
    const shared: bigint = await client.cloud.shareFile('save')
    const forgotten: boolean = client.cloud.forgetFile('save')
    const mask: number = client.cloud.getSyncPlatforms('save')
    client.cloud.setSyncPlatforms('save', mask)
    lobby.setGameServer('127.0.0.1', 27015, shared)
    const server: { ip: string; port: number; steamId?: bigint } | null = lobby.getGameServer()
    const filtered: Promise<typeof lobby[]> = client.matchmaking.getLobbies({
        stringFilters: [{ key: 'map', value: 'arena', comparison: client.matchmaking.LobbyComparison.Equal }],
        numberFilters: [{ key: 'rank', value: 3, comparison: client.matchmaking.LobbyComparison.GreaterThan }],
        nearValueFilters: [{ key: 'rank', value: 3 }],
        count: 10,
    })
    const args = [1, client.workshop.UGCQueryType.RankedByVote, client.workshop.UGCType.Items, 480, 480] as const
    const found: bigint[] = await client.workshop.getAllItemIds(...args)
    const userIds: bigint[] = await client.workshop.getUserItemIds(
        1, 1, client.workshop.UserListType.Published, client.workshop.UGCType.Items,
        client.workshop.UserListOrder.CreationOrderAsc, {}, { cloudFileNameFilter: 'save.dat' }
    )
    const total: number = await client.workshop.getAllItemTotal(...args)
    return { forgotten, server, filtered, found, userIds, total }
}
