import { init, SteamCallback } from '../../index'
import { CallbackReturns, ChatMemberStateChange, RuntimeCallbackReturns } from '../../callbacks'
import clientTypes = require('../../client')

type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false
type Assert<T extends true> = T
type LegacyAndRuntimeContracts = [
    Assert<Equal<CallbackReturns[clientTypes.callback.SteamCallback.SteamServersConnected], {}>>,
    Assert<Equal<CallbackReturns[clientTypes.callback.SteamCallback.SteamServersDisconnected]['reason'], number>>,
    Assert<Equal<CallbackReturns[clientTypes.callback.SteamCallback.SteamServerConnectFailure]['reason'], number>>,
    Assert<Equal<CallbackReturns[clientTypes.callback.SteamCallback.LobbyChatUpdate]['member_state_change'], ChatMemberStateChange>>,
    Assert<Equal<RuntimeCallbackReturns[clientTypes.callback.SteamCallback.SteamServersConnected], null>>,
    Assert<Equal<RuntimeCallbackReturns[clientTypes.callback.SteamCallback.SteamServersDisconnected]['reason'], string>>,
    Assert<Equal<RuntimeCallbackReturns[clientTypes.callback.SteamCallback.SteamServerConnectFailure]['reason'], string>>,
    Assert<Equal<RuntimeCallbackReturns[clientTypes.callback.SteamCallback.LobbyChatUpdate]['member_state_change'], 'Entered' | 'Left' | 'Disconnected' | 'Kicked' | 'Banned'>>
]

const client = init(480)
client.callback.register(SteamCallback.LobbyCreated, event => {
    const lobby: bigint = event.lobby
    const result: number = event.result
    return [lobby, result]
})
client.callback.register(SteamCallback.LobbyEnter, event => {
    const lobby: bigint = event.lobby
    const response: string = event.chat_room_enter_response
    return [lobby, response]
})
client.callback.register(SteamCallback.UserStatsReceived, event => {
    const user: bigint = event.steam_id
    const game: bigint = event.game_id
    const error: string | null = event.error
    return [user, game, error]
})
client.callback.register(SteamCallback.UserStatsStored, event => event.game_id satisfies bigint)
client.callback.register(SteamCallback.UserAchievementStored, event => event.game_id satisfies bigint)
client.callback.register(SteamCallback.UserAchievementIconFetched, event => event.achievement_name satisfies string)
client.callback.register(SteamCallback.DownloadItemResult, event => event.published_file_id satisfies bigint)
client.callback.register(SteamCallback.RemotePlayConnected, event => event.session satisfies number)
client.callback.register(SteamCallback.RemotePlayDisconnected, event => event.session satisfies number)
client.callback.register(SteamCallback.NewUrlLaunchParameters, event => event satisfies object)
client.callback.register(SteamCallback.RelayNetworkStatus, event => {
    const availability: number = event.availability
    const ping: boolean = event.ping_measurement_in_progress
    const message: string = event.debug_message
    return [availability, ping, message]
})
client.callback.register(SteamCallback.GlobalStatsReceived, event => {
    const game: bigint = event.game_id
    const error: string | null = event.error
    return [game, error]
})
client.callback.register(SteamCallback.DeleteItemResult, event => {
    const file: bigint = event.published_file_id
    const error: string | null = event.error
    return [file, error]
})
client.callback.register(SteamCallback.SteamServersConnected, event => event satisfies {})
client.callback.register(SteamCallback.SteamServersDisconnected, event => event.reason satisfies number)
client.callback.register(SteamCallback.SteamServerConnectFailure, event => event.reason satisfies number)
client.callback.register(SteamCallback.LobbyChatUpdate, event => event.member_state_change satisfies ChatMemberStateChange)
client.callback.registerTyped(SteamCallback.SteamServersConnected, event => event satisfies null)
client.callback.registerTyped(SteamCallback.SteamServersDisconnected, event => event.reason satisfies string)
client.callback.registerTyped(SteamCallback.SteamServerConnectFailure, event => event.reason satisfies string)
client.callback.registerTyped(SteamCallback.LobbyChatUpdate, event => event.member_state_change satisfies 'Entered' | 'Left' | 'Disconnected' | 'Kicked' | 'Banned')
client.callback.registerTyped(SteamCallback.LobbyCreated, event => event.lobby satisfies bigint)
