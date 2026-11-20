import type { matchmaking_servers } from '../../client'

declare const browser: typeof matchmaking_servers
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false
type Assert<T extends true> = T
type Kinds = 'internet' | 'lan' | 'friends' | 'favorites' | 'history' | 'spectator'
type CheckKinds = Assert<Equal<Parameters<typeof browser.requestList>[1], Kinds>>
const typeCheck: CheckKinds = true
const list = browser.requestList(3104020, 'internet', [{ key: 'map', value: 'island' }])
const server = list.getServer(0)
if (server) {
  const id: bigint = server.steamId
  const address: string = server.address
  void [id, address]
}
const refreshing: boolean = list.isRefreshing()
const count: number = list.count()
const cancelled: boolean = list.cancel()
void [refreshing, count, cancelled, typeCheck]
