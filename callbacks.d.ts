import client = require('./client')

export const enum ChatMemberStateChange {
    Entered,
    Left,
    Disconnected,
    Kicked,
    Banned,
}

export type ChatEntryType =
    | 'Invalid'
    | 'ChatMsg'
    | 'Typing'
    | 'InviteGame'
    | 'Emote'
    | 'LeftConversation'
    | 'Entered'
    | 'WasKicked'
    | 'WasBanned'
    | 'Disconnected'
    | 'HistoricalChat'
    | 'LinkBlocked'

export interface CallbackReturns {
    [client.callback.SteamCallback.RelayNetworkStatus]: {
        availability: number
        ping_measurement_in_progress: boolean
        network_config: number
        any_relay: number
        debug_message: string
    }
    [client.callback.SteamCallback.GlobalStatsReceived]: { game_id: bigint; error: string | null }
    [client.callback.SteamCallback.DeleteItemResult]: { published_file_id: bigint; error: string | null }
    [client.callback.SteamCallback.LobbyCreated]: { result: number; lobby: bigint }
    [client.callback.SteamCallback.LobbyEnter]: {
        lobby: bigint
        chat_permissions: number
        blocked: boolean
        chat_room_enter_response: 'Success' | 'DoesntExist' | 'NotAllowed' | 'Full' | 'Error' | 'Banned' | 'Limited' | 'ClanDisabled' | 'CommunityBan' | 'MemberBlockedYou' | 'YouBlockedMember' | 'RatelimitExceeded'
    }
    [client.callback.SteamCallback.NewUrlLaunchParameters]: {}
    [client.callback.SteamCallback.UserStatsReceived]: { steam_id: bigint; game_id: bigint; error: string | null }
    [client.callback.SteamCallback.UserStatsStored]: { game_id: bigint; error: string | null }
    [client.callback.SteamCallback.UserAchievementStored]: { game_id: bigint; achievement_name: string; current_progress: number; max_progress: number }
    [client.callback.SteamCallback.UserAchievementIconFetched]: { game_id: bigint; achievement_name: string; achieved: boolean; icon_handle: number }
    [client.callback.SteamCallback.DownloadItemResult]: { app_id: number; published_file_id: bigint; error: string | null }
    [client.callback.SteamCallback.RemotePlayConnected]: { session: number }
    [client.callback.SteamCallback.RemotePlayDisconnected]: { session: number }
    [client.callback.SteamCallback.SteamInputDeviceConnected]: { device_handle: number | bigint }
    [client.callback.SteamCallback.SteamInputDeviceDisconnected]: { device_handle: number | bigint }
    [client.callback.SteamCallback.SteamInventoryDefinitionUpdate]: null
    [client.callback.SteamCallback.SteamInventoryFullUpdate]: { handle: number }
    [client.callback.SteamCallback.ValidateAuthTicketResponse]: {
        steam_id: bigint
        owner_steam_id: bigint
        response: number
    }
    [client.callback.SteamCallback.ScreenshotRequested]: {}
    [client.callback.SteamCallback.ScreenshotReady]: { handle: number; error: null } | { handle: null; error: string }
    [client.callback.SteamCallback.GameRichPresenceJoinRequested]: {
        friend_steam_id: bigint
        connect: string
    }
    [client.callback.SteamCallback.FriendRichPresenceUpdate]: {
        friend_steam_id: bigint
        app_id: number
    }
    [client.callback.SteamCallback.PersonaStateChange]: {
        steam_id: bigint
        flags: { bits: number }
    }
    [client.callback.SteamCallback.SteamServersConnected]: {}
    [client.callback.SteamCallback.SteamServersDisconnected]: {
        reason: number
    }
    [client.callback.SteamCallback.SteamServerConnectFailure]: {
        reason: number
        still_retrying: boolean
    }
    [client.callback.SteamCallback.LobbyDataUpdate]: {
        lobby: bigint
        member: bigint
        success: boolean
    }
    [client.callback.SteamCallback.LobbyChatUpdate]: {
        lobby: bigint
        user_changed: bigint
        making_change: bigint
        member_state_change: ChatMemberStateChange
    }
    [client.callback.SteamCallback.LobbyChatMsg]: {
        lobby: bigint
        user: bigint
        chat_entry_type: ChatEntryType
        chat_id: number
    }
    [client.callback.SteamCallback.P2PSessionRequest]: {
        remote: bigint
    }
    [client.callback.SteamCallback.P2PSessionConnectFail]: {
        remote: bigint
        error: number
    }
    [client.callback.SteamCallback.GameLobbyJoinRequested]: {
        lobby_steam_id: bigint
        friend_steam_id: bigint
    }
    [client.callback.SteamCallback.MicroTxnAuthorizationResponse]: {
        app_id: number
        order_id: number | bigint
        authorized: boolean
    }
    [client.callback.SteamCallback.GameOverlayActivated]: {
        active: boolean
        userInitiated: boolean
        appId: number
    }
}

export type RuntimeCallbackReturns = Omit<CallbackReturns,
    | client.callback.SteamCallback.SteamServersConnected
    | client.callback.SteamCallback.SteamServersDisconnected
    | client.callback.SteamCallback.SteamServerConnectFailure
    | client.callback.SteamCallback.LobbyChatUpdate
> & {
    [client.callback.SteamCallback.SteamServersConnected]: null
    [client.callback.SteamCallback.SteamServersDisconnected]: { reason: string }
    [client.callback.SteamCallback.SteamServerConnectFailure]: { reason: string; still_retrying: boolean }
    [client.callback.SteamCallback.LobbyChatUpdate]: {
        lobby: bigint
        user_changed: bigint
        making_change: bigint
        member_state_change: 'Entered' | 'Left' | 'Disconnected' | 'Kicked' | 'Banned'
    }
}
