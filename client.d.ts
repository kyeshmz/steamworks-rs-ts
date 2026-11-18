/// <reference types="node" />

export declare function init(appId?: number | undefined | null): void

export interface PlayerSteamId {
  steamId64: bigint
  steamId32: string
  accountId: number
}

export declare function restartAppIfNecessary(appId: number): boolean

export declare function runCallbacks(): void

export interface TicketValidation {
  steam_id: bigint
  owner_steam_id: bigint
  response: number
}

export declare namespace achievement {
  export function activate(achievement: string): boolean
  export function clear(achievement: string): boolean
  export function count(): number
  /** Returns the cached unlock percentage (0-100), or null if Steam has no data. */
  export function getAchievedPercent(achievement: string): number | null
  /** Use "name", "desc", or "hidden". The hidden flag is "0" or "1". */
  export function getDisplayAttribute(achievement: string, key: string): string | null
  /** Returns a 64x64 RGBA buffer, or null if the icon is unavailable. */
  export function getIcon(achievement: string): Buffer | null
  export function isActivated(achievement: string): boolean
  export function names(): Array<string>
  /** Wait for Steam's global percentage request before reading individual achievements. */
  export function requestGlobalPercentages(): Promise<void>
}

export declare namespace apps {
  export function appBuildId(): number
  export function appInstallDir(appId: number): string
  export function appOwner(): PlayerSteamId
  export function availableGameLanguages(): Array<string>
  export function currentBetaName(): string | null
  export function currentGameLanguage(): string
  export interface DlcDownloadProgress {
    bytesDownloaded: bigint
    bytesTotal: bigint
  }
  export interface DlcInfo {
    appId: number
    available: boolean
    name: string
  }
  export function getDlcDownloadProgress(appId: number): apps.DlcDownloadProgress | null
  /** Starts a background download for an owned DLC. */
  export function installDlc(appId: number): void
  export function isAppInstalled(appId: number): boolean
  export function isCybercafe(): boolean
  export function isDlcInstalled(appId: number): boolean
  export function isLowViolence(): boolean
  export function isSubscribed(): boolean
  export function isSubscribedApp(appId: number): boolean
  export function isSubscribedFromFreeWeekend(): boolean
  export function isVacBanned(): boolean
  export function launchCommandLine(): string
  export function launchQueryParam(key: string): string
  export function listDlc(): Array<apps.DlcInfo>
  export function uninstallDlc(appId: number): void
}

export declare namespace auth {
  export class Ticket {
    cancel(): void
    getBytes(): Buffer
  }
  /** Submit a remote peer's ticket. Wait for ValidateAuthTicketResponse before granting access. */
  export function beginAuthSession(steamId64: bigint, ticket: Buffer): void
  /** End an accepted submission when the peer disconnects or validation fails. */
  export function endAuthSession(steamId64: bigint): void
  export function getAuthTicketForWebApi(identity: string, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  /**
   * @param ip - The string of IPv4 or IPv6 address. Use as NetworkIdentity of the remote system that will authenticate the ticket.
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithIp(ip: string, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  /**
   * @param steamId64 - The user steam id or game server steam id. Use as NetworkIdentity of the remote system that will authenticate the ticket. If it is peer-to-peer then the user steam ID. If it is a game server, then the game server steam ID may be used if it was obtained from a trusted 3rd party
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithSteamId(steamId64: bigint, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  /**
   * Request an encrypted app ticket and resolve with its bytes. Unlike a session
   * ticket this is issued for your backend to decrypt with the app's encrypted
   * ticket key, never for beginAuthSession.
   *
   * @param dataToInclude - Optional application data encrypted into the ticket, at most 128 bytes.
   * @param timeoutSeconds - The number of seconds to wait for Steam to complete the request. Default value is 10 seconds.
   */
  export function requestEncryptedAppTicket(dataToInclude?: Buffer | undefined | null, timeoutSeconds?: number | undefined | null): Promise<Buffer>
}

export declare namespace callback {
  export class Handle {
    disconnect(): void
  }
  export function register<C extends keyof import('./callbacks').CallbackReturns>(steamCallback: C, handler: (value: import('./callbacks').CallbackReturns[C]) => void): callback.Handle
  export const enum SteamCallback {
    PersonaStateChange = 0,
    SteamServersConnected = 1,
    SteamServersDisconnected = 2,
    SteamServerConnectFailure = 3,
    LobbyDataUpdate = 4,
    LobbyChatUpdate = 5,
    P2PSessionRequest = 6,
    P2PSessionConnectFail = 7,
    GameLobbyJoinRequested = 8,
    MicroTxnAuthorizationResponse = 9,
    GameOverlayActivated = 10,
    LobbyChatMsg = 11,
    GameRichPresenceJoinRequested = 12,
    FriendRichPresenceUpdate = 13,
    ScreenshotRequested = 14,
    ScreenshotReady = 15,
    ValidateAuthTicketResponse = 16,
    SteamInputDeviceConnected = 17,
    SteamInputDeviceDisconnected = 18,
    SteamInventoryDefinitionUpdate = 19,
    SteamInventoryFullUpdate = 20,
  }
}

export declare namespace cloud {
  export class FileInfo {
    name: string
    get size(): bigint
    set size(value: bigint)
  }
  /** Hints to Steam that several writes belong together. This is not a transaction. */
  export function beginFileWriteBatch(): boolean
  export interface CloudQuota {
    totalBytes: bigint
    availableBytes: bigint
  }
  export function deleteFile(name: string): boolean
  /** Call after all writes have settled, even if one failed. */
  export function endFileWriteBatch(): boolean
  export function fileExists(name: string): boolean
  export function fileTimestamp(name: string): number
  export function getQuota(): cloud.CloudQuota | null
  export function getSyncPlatforms(name: string): number
  export function isEnabledForAccount(): boolean
  export function isEnabledForApp(): boolean
  export function isFilePersisted(name: string): boolean
  export function listFiles(): Array<cloud.FileInfo>
  export function readFile(name: string): string
  /** Read binary save data without blocking the Node event loop. */
  export function readFileBytes(name: string): Promise<Buffer>
  export function setEnabledForApp(enabled: boolean): void
  /**
   * Combine Windows (1), macOS (2), PS3 (4), Linux (8), Switch (16),
   * Android (32) and iOS (64). Use 0 for none or 0xffffffff for all.
   */
  export function setSyncPlatforms(name: string, mask: number): boolean
  export function writeFile(name: string, content: string): boolean
  export function writeFileBytes(name: string, data: Buffer): Promise<void>
}

export declare namespace friends {
  export const enum AvatarSize {
    Small = 0,
    Medium = 1,
    Large = 2,
  }
  export interface CoplayFriend {
    steamId64: bigint
    appId: number
    lastPlayed: number
  }
  export const enum FriendFilter {
    Immediate = 0,
    Blocked = 1,
    FriendshipRequested = 2,
    RequestingFriendship = 3,
    RequestingInfo = 4,
    Ignored = 5,
    IgnoredFriend = 6,
    ClanMember = 7,
    OnGameServer = 8,
    ChatMember = 9,
    All = 10,
  }
  export interface FriendGameInfo {
    appId: number
    lobbyId?: bigint
  }
  export interface FriendsGroup {
    /** Signed 16-bit group ID as Steam reports it. Steam's -1 invalid marker never appears here. */
    id: number
    name: string
  }
  /** RGBA pixels at 32x32, 64x64, or 184x184. Returns null when unavailable. */
  export function getAvatar(steamId64: bigint, size: friends.AvatarSize): Buffer | null
  export function getCoplayFriends(): Array<friends.CoplayFriend>
  /** Defaults to immediate friends, excluding pending invitations. */
  export function getFriends(filter?: friends.FriendFilter | undefined | null): Array<bigint>
  /** Member SteamID64s of one named group. IDs outside the user's groups throw; an empty group returns an empty list. */
  export function getFriendsGroupMembersList(groupId: number): Array<bigint>
  /** Named friends groups of the signed-in user, in Steam's order. Empty when the account has none. */
  export function getFriendsGroups(): Array<friends.FriendsGroup>
  export function getGamePlayed(steamId64: bigint): friends.FriendGameInfo | null
  export function getPersona(steamId64: bigint): friends.Persona
  export function getPersonaName(steamId64: bigint): string
  /** Read a cached presence value. Missing keys return the empty string. */
  export function getRichPresence(steamId64: bigint, key: string): string
  export function getRichPresenceKeys(steamId64: bigint): Array<string>
  export function inviteUserToGame(steamId64: bigint, connect: string): boolean
  export interface Persona {
    name: string
    state: string
    /** 64 x 64 RGBA bytes when Steam has an avatar available. */
    mediumAvatar?: Buffer
  }
  /** Requests fresh presence. Steam delivers FriendRichPresenceUpdate when available. */
  export function requestRichPresence(steamId64: bigint): void
  /** Defaults to name only. True when Steam started a fetch; false when cached. */
  export function requestUserInformation(steamId64: bigint, nameOnly?: boolean | undefined | null): boolean
  export function setPlayedWith(steamId64: bigint): void
}

export declare namespace game_server {
  export class AuthListener {
    disconnect(): void
  }
  /** Submit a remote peer's ticket. Wait for the validation event before granting access. */
  export function beginAuthSession(steamId64: bigint, ticket: Buffer): void
  export const enum ConnectionState {
    Ready = 'Ready',
    Connecting = 'Connecting',
    Connected = 'Connected',
    Failed = 'Failed',
    Disconnected = 'Disconnected',
  }
  /** End a session when its peer disconnects, validation fails, or you abandon it. */
  export function endAuthSession(steamId64: bigint): void
  export function init(options: game_server.ServerOptions): void
  export function logOn(token: string): void
  export function logOnAnonymous(): void
  export const enum Mode {
    NoAuthentication = 'NoAuthentication',
    Authentication = 'Authentication',
    AuthenticationAndSecure = 'AuthenticationAndSecure',
  }
  /** The game server's ticket validation event. One listener; a later call replaces it. */
  export function onValidateAuthTicketResponse(handler: (event: { steam_id: bigint; owner_steam_id: bigint; response: number }) => void): game_server.AuthListener
  export function runCallbacks(): void
  export interface ServerListing {
    serverName: string
    mapName: string
    maxPlayers: number
  }
  export interface ServerOptions {
    bindIp: string
    gamePort: number
    queryPort: number
    mode: game_server.Mode
    version: string
    product: string
    description: string
    listing: game_server.ServerListing
  }
  export interface ServerStatus {
    state: game_server.ConnectionState
    steamId?: bigint
  }
  export function setAdvertising(active: boolean): void
  export function shutdown(): boolean
  export function status(): game_server.ServerStatus
  export function updateListing(listing: game_server.ServerListing): void
}

export declare namespace input {
  export class Controller {
    activateActionSet(actionSetHandle: bigint): void
    isDigitalActionPressed(actionHandle: bigint): boolean
    getAnalogActionVector(actionHandle: bigint): AnalogActionVector
    getType(): input.InputType
    getHandle(): bigint
    /** Read orientation, acceleration and angular velocity from a motion-capable controller. */
    getMotionData(): MotionData
    /** Set RGB on a supported controller, or restore the user's preferred color. */
    setLightbarColor(red: number, green: number, blue: number, restoreDefaultLightbarColor?: boolean | undefined | null): void
    getDigitalActionOrigins(actionSet: bigint, action: bigint): Array<number>
    getAnalogActionOrigins(actionSet: bigint, action: bigint): Array<number>
    showBindingPanel(): boolean
    getCurrentActionSet(): bigint
    activateActionSetLayer(layer: bigint): void
    deactivateActionSetLayer(layer: bigint): void
    deactivateAllActionSetLayers(): void
    getActiveActionSetLayers(): Array<bigint>
    /** Motor speeds range from 0 (stop) to 65535. */
    triggerVibration(leftSpeed: number, rightSpeed: number): void
    /** Send a haptic pulse on Steam Deck or Steam Controller. */
    triggerSimpleHapticEvent(location: input.HapticLocation, intensity: number, gainDb: number, otherIntensity: number, otherGainDb: number): void
  }
  export interface AnalogActionVector {
    x: number
    y: number
  }
  /** Enable hotplug notifications after registering the device callbacks. */
  export function enableDeviceCallbacks(): void
  export function getActionSet(actionSetName: string): bigint
  export function getAnalogAction(actionName: string): bigint
  export function getControllers(): Array<input.Controller>
  export function getDigitalAction(actionName: string): bigint
  export function getGlyphForActionOrigin(origin: number): string
  export function getGlyphPngForActionOrigin(origin: number, size: input.GlyphSize, flags?: number | undefined | null): string
  export function getGlyphSvgForActionOrigin(origin: number, flags?: number | undefined | null): string
  export function getStringForActionOrigin(origin: number): string
  export const enum GlyphSize {
    Small = 0,
    Medium = 1,
    Large = 2,
  }
  export const enum HapticLocation {
    Left = 0,
    Right = 1,
    Both = 2,
  }
  export function init(): void
  export const enum InputType {
    Unknown = 'Unknown',
    SteamController = 'SteamController',
    XBox360Controller = 'XBox360Controller',
    XBoxOneController = 'XBoxOneController',
    GenericGamepad = 'GenericGamepad',
    PS4Controller = 'PS4Controller',
    AppleMFiController = 'AppleMFiController',
    AndroidController = 'AndroidController',
    SwitchJoyConPair = 'SwitchJoyConPair',
    SwitchJoyConSingle = 'SwitchJoyConSingle',
    SwitchProController = 'SwitchProController',
    MobileTouch = 'MobileTouch',
    PS3Controller = 'PS3Controller',
    PS5Controller = 'PS5Controller',
    SteamDeckController = 'SteamDeckController',
  }
  export interface MotionData {
    rotQuatX: number
    rotQuatY: number
    rotQuatZ: number
    rotQuatW: number
    posAccelX: number
    posAccelY: number
    posAccelZ: number
    rotVelX: number
    rotVelY: number
    rotVelZ: number
  }
  export function runFrame(): void
  export function setInputActionManifestFilePath(path: string): boolean
  export function shutdown(): void
}

export declare namespace inventory {
  /** Consumes item quantity permanently and returns the Steam-signed receipt bytes. */
  export function consumeItemSerialized(itemId: bigint, quantity: number): Promise<inventory.SerializedResult>
  export interface ExchangeDestroy {
    itemId: bigint
    quantity: number
  }
  export interface ExchangeGenerate {
    definition: number
    quantity: number
  }
  /** Exchanges item quantities in one atomic Steam transaction and returns the resulting snapshot. */
  export function exchangeItems(generate: Array<inventory.ExchangeGenerate>, destroy: Array<inventory.ExchangeDestroy>): Promise<inventory.InventoryResult>
  /** Exchanges items in one atomic Steam transaction and returns the Steam-signed receipt bytes. */
  export function exchangeItemsSerialized(generate: Array<inventory.ExchangeGenerate>, destroy: Array<inventory.ExchangeDestroy>): Promise<inventory.SerializedResult>
  export function getAllItemProperties(): Promise<Array<inventory.ItemProperties>>
  export function getAllItems(): Promise<inventory.InventoryResult>
  /** Returns the cached definition IDs, or null if Steam could not read them. */
  export function getItemDefinitionIds(): Array<number> | null
  /** Omit name to read the comma-separated property names. Missing properties return null. */
  export function getItemDefinitionProperty(definition: number, name?: string | undefined | null): string | null
  export function getItemProperties(itemIds: Array<bigint>): Promise<Array<inventory.ItemProperties>>
  export function getItemsById(itemIds: Array<bigint>): Promise<inventory.InventoryResult>
  export function getItemsByID(itemIds: Array<bigint>): Promise<inventory.InventoryResult>
  export interface InventoryItem {
    itemId: bigint
    definition: number
    quantity: number
    flags: number
  }
  export interface InventoryResult {
    timestamp: number
    items: Array<inventory.InventoryItem>
  }
  export interface ItemProperties {
    itemId: bigint
    definition: number
    quantity: number
    properties: Record<string, string>
  }
  /** Requests a refresh. A successful request does not mean definitions are ready. */
  export function loadItemDefinitions(): boolean
  /** Records play time for playtime-based item drops. Does not grant an item. */
  export function sendItemDropHeartbeat(): void
  export interface SerializedResult {
    timestamp: number
    items: Array<inventory.InventoryItem>
    buffer: Buffer
  }
}

export declare namespace leaderboards {
  /** Global ranks start at 1. Around-user offsets may be negative. Friends ignores the range. */
  export function downloadLeaderboardEntries(name: string, request: leaderboards.LeaderboardDataRequest, rangeStart: number, rangeEnd: number, maxDetails?: number | undefined | null): Promise<Array<leaderboards.LeaderboardEntry>>
  /** Resolves to false for a missing board; Steam transport errors reject. */
  export function findLeaderboard(name: string): Promise<boolean>
  /** Sort and display are only applied when Steam creates the board. */
  export function findOrCreateLeaderboard(name: string, sortMethod: leaderboards.LeaderboardSortMethod, displayType: leaderboards.LeaderboardDisplayType): Promise<boolean>
  export function getLeaderboardEntryCount(name: string): Promise<number>
  export function getLeaderboardInfo(name: string): Promise<leaderboards.LeaderboardInfo>
  export const enum LeaderboardDataRequest {
    Global = 0,
    GlobalAroundUser = 1,
    Friends = 2,
  }
  export const enum LeaderboardDisplayType {
    Numeric = 0,
    TimeSeconds = 1,
    TimeMilliSeconds = 2,
  }
  export interface LeaderboardEntry {
    /** SteamId64 of the entry's owner. */
    steamId: bigint
    /** 1-indexed global rank. */
    globalRank: number
    score: number
    /** Game-defined metadata attached at upload (up to 64 i32s). */
    details: Array<number>
  }
  export interface LeaderboardInfo {
    name: string
    entryCount: number
    sortMethod?: leaderboards.LeaderboardSortMethod
    displayType?: leaderboards.LeaderboardDisplayType
  }
  export interface LeaderboardScoreUploaded {
    score: number
    scoreChanged: boolean
    globalRankNew: number
    globalRankPrevious: number
  }
  export const enum LeaderboardSortMethod {
    Ascending = 0,
    Descending = 1,
  }
  /** Creates the board if missing. Existing boards keep their configuration. */
  export function uploadLeaderboardScore(name: string, sortMethod: leaderboards.LeaderboardSortMethod, displayType: leaderboards.LeaderboardDisplayType, method: leaderboards.UploadScoreMethod, score: number, details?: Array<number> | undefined | null): Promise<leaderboards.LeaderboardScoreUploaded>
  export const enum UploadScoreMethod {
    KeepBest = 0,
    ForceUpdate = 1,
  }
}

export declare namespace localplayer {
  export function clearRichPresence(): void
  /** @returns the 2 digit ISO 3166-1-alpha-2 format country code which client is running in, e.g. "US" or "UK". */
  export function getIpCountry(): string
  export function getLevel(): number
  export function getName(): string
  export function getSteamId(): PlayerSteamId
  export function isLoggedOn(): boolean
  export function setRichPresence(key: string, value?: string | undefined | null): void
}

export declare namespace matchmaking {
  export class Lobby {
    get id(): bigint
    set id(value: bigint)
    join(): Promise<matchmaking.Lobby>
    leave(): void
    openInviteDialog(): void
    /** Broadcast up to 4096 bytes to the lobby, including the sender. */
    sendChatMessage(message: Buffer): void
    /** Read the bytes identified by a LobbyChatMsg callback. Missing entries return an empty buffer. */
    getChatEntry(chatId: number): Buffer
    getMemberCount(): bigint
    getMemberLimit(): bigint | null
    /** Owner only. Returns false if Steam refuses the new limit. */
    setMemberLimit(limit: number): boolean
    /** Transfer ownership to a current lobby member. Only the current owner can do this. */
    setOwner(steamId64: bigint): boolean
    /** Change the lobby's visibility as its owner. */
    setType(lobbyType: matchmaking.LobbyType): boolean
    /** Send a lobby invite without opening the overlay. Success does not mean the user joined. */
    inviteUser(steamId64: bigint): boolean
    getMembers(): Array<PlayerSteamId>
    getOwner(): PlayerSteamId
    /** Set metadata for the local member, not for the lobby itself. */
    setMemberData(key: string, value: string): void
    /** Steam does not distinguish a missing member value from an empty one. */
    getMemberData(steamId64: bigint, key: string): string | null
    setJoinable(joinable: boolean): boolean
    getData(key: string): string | null
    setData(key: string, value: string): boolean
    deleteData(key: string): boolean
    /** Get an object containing all the lobby data */
    getFullData(): Record<string, string>
    /**
     * Merge current lobby data with provided data in a single batch
     * @returns true if all data was set successfully
     */
    mergeFullData(data: Record<string, string>): boolean
  }
  export function createLobby(lobbyType: matchmaking.LobbyType, maxMembers: number): Promise<matchmaking.Lobby>
  export function getLobbies(filter?: matchmaking.LobbyListFilter | undefined | null): Promise<Array<matchmaking.Lobby>>
  export function joinLobby(lobbyId: bigint): Promise<matchmaking.Lobby>
  export const enum LobbyComparison {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    GreaterThanOrEqual = 3,
    LessThan = 4,
    LessThanOrEqual = 5,
  }
  export const enum LobbyDistance {
    Close = 0,
    Default = 1,
    Far = 2,
    Worldwide = 3,
  }
  export interface LobbyListFilter {
    stringFilters?: Array<matchmaking.LobbyStringFilter>
    numberFilters?: Array<matchmaking.LobbyNumberFilter>
    /** Sort by proximity to these values, in priority order. */
    nearValueFilters?: Array<matchmaking.LobbyNearFilter>
    slotsAvailable?: number
    distance?: matchmaking.LobbyDistance
    count?: number
  }
  export interface LobbyNearFilter {
    key: string
    value: number
  }
  export interface LobbyNumberFilter {
    key: string
    value: number
    comparison: matchmaking.LobbyComparison
  }
  export interface LobbyStringFilter {
    key: string
    value: string
    /** Defaults to Equal. */
    comparison?: matchmaking.LobbyComparison
  }
  export const enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3,
  }
}

export declare namespace networking {
  export function acceptP2PSession(steamId64: bigint): void
  export function closeP2PSession(steamId64: bigint): void
  export function isP2PPacketAvailable(): number
  export function isP2PPacketAvailableOnChannel(channel: number): number
  export interface P2PPacket {
    data: Buffer
    size: number
    steamId: PlayerSteamId
  }
  export function readP2PPacket(size: number): networking.P2PPacket
  export function readP2PPacketFromChannel(size: number, channel: number): networking.P2PPacket
  export function sendP2PPacket(steamId64: bigint, sendType: networking.SendType, data: Buffer): boolean
  export function sendP2PPacketOnChannel(steamId64: bigint, sendType: networking.SendType, data: Buffer, channel: number): boolean
  /** The method used to send a packet */
  export const enum SendType {
    /**
     * Send the packet directly over udp.
     *
     * Can't be larger than 1200 bytes
     */
    Unreliable = 0,
    /**
     * Like `Unreliable` but doesn't buffer packets
     * sent before the connection has started.
     */
    UnreliableNoDelay = 1,
    /**
     * Reliable packet sending.
     *
     * Can't be larger than 1 megabyte.
     */
    Reliable = 2,
    /**
     * Like `Reliable` but applies the nagle
     * algorithm to packets being sent
     */
    ReliableWithBuffering = 3,
  }
}

export declare namespace networking_messages {
  /** Accept a known peer's pending request without sending a message. */
  export function acceptSessionWithUser(steamId64: bigint): boolean
  export function closeChannelWithUser(steamId64: bigint, channel: number): boolean
  /** Close the peer's session on all channels. This does not block future requests. */
  export function closeSessionWithUser(steamId64: bigint): boolean
  export interface Message {
    data: Buffer
    steamId?: PlayerSteamId
  }
  export const enum MessageSendType {
    ReliableNoNagle = 0,
    Reliable = 1,
    Unreliable = 2,
    UnreliableNoDelay = 3,
  }
  /** Polls one channel (default 0) for up to 1–32 messages (default 10). */
  export function receiveMessagesOnChannel(channel?: number | undefined | null, maxMessages?: number | undefined | null): Array<networking_messages.Message>
  /** Queues a message and implicitly accepts a session from this peer. Success is not delivery. */
  export function sendMessageToUser(steamId64: bigint, data: Buffer, channel?: number | undefined | null, sendType?: networking_messages.MessageSendType | undefined | null): void
  export interface SessionFailure {
    steamId?: PlayerSteamId
  }
  export interface SessionRequest {
    steamId?: PlayerSteamId
  }
  /** Replaces the failure handler. Omit the handler to stop listening. */
  export function setSessionFailedHandler(handler?: (failure: SessionFailure) => void): void
  /** Notifies without accepting. Omit the handler to stop listening. */
  export function setSessionRequestHandler(handler?: (request: SessionRequest) => void): void
}

export declare namespace networking_sockets {
  export function acceptConnection(connection: number): void
  export function closeConnection(connection: number, linger?: boolean | undefined | null): boolean
  export function closeListenSocket(socket: number): boolean
  export function connectByIpAddress(address: string, port: number): number
  export function connectionInfo(connection: number): networking_sockets.ConnectionInfo | null
  export interface ConnectionInfo {
    connection: number
    listenSocket: number
    steamId64?: bigint
    state: number
    endReason: number
  }
  export function connectP2P(steamId64: bigint, virtualPort: number): number
  export function createListenSocketIp(port: number, address?: string | undefined | null): number
  export function createListenSocketP2P(virtualPort: number): number
  export function createPollGroup(): number
  export function destroyPollGroup(group: number): boolean
  export function initRelayNetworkAccess(): void
  export interface PollGroupMessage {
    connection: number
    data: Buffer
  }
  export function receiveMessages(connection: number, maxMessages?: number | undefined | null): Array<Buffer>
  export function receiveMessagesOnPollGroup(group: number, maxMessages?: number | undefined | null): Array<networking_sockets.PollGroupMessage>
  export function relayStatus(): string
  export function sendMessage(connection: number, data: Buffer, reliable?: boolean | undefined | null): bigint
  export function setConnectionPollGroup(connection: number, group: number): boolean
  export function setConnectionStatusHandler(handler?: (info: ConnectionInfo) => void): void
}

export declare namespace overlay {
  export function activateDialog(dialog: overlay.Dialog): void
  export function activateDialogToUser(dialog: overlay.Dialog, steamId64: bigint): void
  export function activateInviteDialog(lobbyId: bigint): void
  export function activateToStore(appId: number, flag: overlay.StoreFlag): void
  export function activateToWebPage(url: string): void
  export const enum Dialog {
    Friends = 0,
    Community = 1,
    Players = 2,
    Settings = 3,
    OfficialGameGroup = 4,
    Stats = 5,
    Achievements = 6,
  }
  export const enum NotificationPosition {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
  }
  export function setNotificationPosition(position: overlay.NotificationPosition): void
  export const enum StoreFlag {
    None = 0,
    AddToCart = 1,
    AddToCartAndShow = 2,
  }
}

export declare namespace runtime {
  /** Timing of completed Steam callback pumps. A reset starts a fresh measurement window. */
  export function pumpStats(reset?: boolean | undefined | null): runtime.PumpStats
  export interface PumpStats {
    runs: number
    gaps: number
    meanGapUs: number
    maxGapUs: number
    lastGapUs: number
    maxRunUs: number
  }
}

export declare namespace screenshots {
  /** Imports a JPEG from disk. Pass null for thumbnailFilename to let Steam generate it. */
  export function addScreenshotToLibrary(filename: string, thumbnailFilename: string | null, width: number, height: number): number
  export function hookScreenshots(enabled: boolean): void
  export function isScreenshotsHooked(): boolean
  export function setLocation(screenshot: number, location: string): boolean
  export function tagPublishedFile(screenshot: number, fileId: bigint): boolean
  export function tagUser(screenshot: number, steamId64: bigint): boolean
  /** Asks the Steam overlay to capture the current frame. */
  export function triggerScreenshot(): void
  /** Supply tightly packed RGB24 rows, without alpha or stride padding. */
  export function writeScreenshot(data: Buffer, width: number, height: number): number
}

export declare namespace stats {
  export function getFloat(name: string): number | null
  export function getInt(name: string): number | null
  export function resetAll(achievementsToo: boolean): boolean
  /** Updates a FLOAT stat locally. Call store() to persist it on Steam. */
  export function setFloat(name: string, value: number): boolean
  export function setInt(name: string, value: number): boolean
  export function store(): boolean
}

export declare namespace utils {
  export const enum FloatingGamepadTextInputMode {
    SingleLine = 0,
    MultipleLines = 1,
    Email = 2,
    Numeric = 3,
  }
  export const enum GamepadTextInputLineMode {
    SingleLine = 0,
    MultipleLines = 1,
  }
  export const enum GamepadTextInputMode {
    Normal = 0,
    Password = 1,
  }
  export function getAppId(): number
  export function getServerRealTime(): number
  export function isOverlayEnabled(): boolean
  export function isSteamInBigPictureMode(): boolean
  export function isSteamRunningOnSteamDeck(): boolean
  /** @returns true if the floating keyboard was shown, otherwise, false */
  export function showFloatingGamepadTextInput(keyboardMode: utils.FloatingGamepadTextInputMode, x: number, y: number, width: number, height: number): Promise<boolean>
  /** @returns the entered text, or null if cancelled or could not show the input */
  export function showGamepadTextInput(inputMode: utils.GamepadTextInputMode, inputLineMode: utils.GamepadTextInputLineMode, description: string, maxCharacters: number, existingText?: string | undefined | null): Promise<string | null>
  export function uiLanguage(): string
}

export declare namespace voice {
  export function decompressVoice(compressed: Buffer, sampleRate: number): Buffer
  /** Returns the compressed byte count without consuming any recorded samples. */
  export function getAvailableVoice(): number
  export function getVoice(): Buffer
  export function getVoiceOptimalSampleRate(): number
  export function startVoiceRecording(): void
  export function stopVoiceRecording(): void
}

export declare namespace workshop {
  export interface AppIDs {
    creator?: number
    consumer?: number
  }
  export const enum ContentDescriptor {
    NudityOrSexualContent = 0,
    FrequentViolenceOrGore = 1,
    AdultOnlySexualContent = 2,
    GratuitousSexualContent = 3,
    AnyMatureContent = 4,
  }
  export function createItem(appId?: number | undefined | null, fileType?: workshop.FileType | undefined | null): Promise<workshop.UgcResult>
  export function deleteItem(itemId: bigint): Promise<void>
  /**
   * Download or update a workshop item.
   *
   * @param highPriority - If high priority is true, start the download in high priority mode, pausing any existing in-progress Steam downloads and immediately begin downloading this workshop item.
   * @returns true or false
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem}
   */
  export function download(itemId: bigint, highPriority: boolean): boolean
  /**
   * Get info about a pending download of a workshop item.
   *
   * @returns an object with the properties {current, total}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo}
   */
  export function downloadInfo(itemId: bigint): workshop.DownloadInfo | null
  export interface DownloadInfo {
    current: bigint
    total: bigint
  }
  export const enum FileType {
    Community = 0,
    Microtransaction = 1,
    Collection = 2,
    Art = 3,
    Video = 4,
    Screenshot = 5,
    Game = 6,
    Software = 7,
    Concept = 8,
    WebGuide = 9,
    IntegratedGuide = 10,
    Merch = 11,
    ControllerBinding = 12,
    SteamworksAccessInvite = 13,
    SteamVideo = 14,
    GameManagedItem = 15,
  }
  export function getAllItems(page: number, queryType: workshop.UGCQueryType, itemType: workshop.UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopPaginatedResult>
  export function getItem(item: bigint, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopItem | null>
  export function getItems(items: Array<bigint>, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopItemsResult>
  /**
   * Get all subscribed workshop items.
   * @returns an array of subscribed workshop item ids
   */
  export function getSubscribedItems(): Array<bigint>
  export function getUserItems(page: number, accountId: number, listType: workshop.UserListType, itemType: workshop.UGCType, sortOrder: workshop.UserListOrder, appIds: workshop.AppIDs, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopPaginatedResult>
  /**
   * Gets info about currently installed content on the disc for workshop item.
   *
   * @returns an object with the the properties {folder, size_on_disk, timestamp}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo}
   */
  export function installInfo(itemId: bigint): workshop.InstallInfo | null
  export interface InstallInfo {
    folder: string
    sizeOnDisk: bigint
    timestamp: number
  }
  export interface KeyValueTag {
    key: string
    value: string
  }
  /**
   * Gets the current state of a workshop item on this client. States can be combined.
   *
   * @returns a number with the current item state, e.g. 9
   * 9 = 1 (The current user is subscribed to this item) + 8 (The item needs an update)
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState}
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#EItemState}
   */
  export function state(itemId: bigint): number
  /**
   * Subscribe to a workshop item. It will be downloaded and installed as soon as possible.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem}
   */
  export function subscribe(itemId: bigint): Promise<void>
  export const enum UgcItemVisibility {
    Public = 0,
    FriendsOnly = 1,
    Private = 2,
    Unlisted = 3,
  }
  export const enum UGCQueryType {
    RankedByVote = 0,
    RankedByPublicationDate = 1,
    AcceptedForGameRankedByAcceptanceDate = 2,
    RankedByTrend = 3,
    FavoritedByFriendsRankedByPublicationDate = 4,
    CreatedByFriendsRankedByPublicationDate = 5,
    RankedByNumTimesReported = 6,
    CreatedByFollowedUsersRankedByPublicationDate = 7,
    NotYetRated = 8,
    RankedByTotalVotesAsc = 9,
    RankedByVotesUp = 10,
    RankedByTextSearch = 11,
    RankedByTotalUniqueSubscriptions = 12,
    RankedByPlaytimeTrend = 13,
    RankedByTotalPlaytime = 14,
    RankedByAveragePlaytimeTrend = 15,
    RankedByLifetimeAveragePlaytime = 16,
    RankedByPlaytimeSessionsTrend = 17,
    RankedByLifetimePlaytimeSessions = 18,
    RankedByLastUpdatedDate = 19,
  }
  export interface UgcResult {
    itemId: bigint
    needsToAcceptAgreement: boolean
  }
  export const enum UGCType {
    Items = 0,
    ItemsMtx = 1,
    ItemsReadyToUse = 2,
    Collections = 3,
    Artwork = 4,
    Videos = 5,
    Screenshots = 6,
    AllGuides = 7,
    WebGuides = 8,
    IntegratedGuides = 9,
    UsableInGame = 10,
    ControllerBindings = 11,
    GameManagedItems = 12,
    All = 13,
  }
  export interface UgcUpdate {
    title?: string
    description?: string
    changeNote?: string
    previewPath?: string
    contentPath?: string
    tags?: Array<string>
    visibility?: workshop.UgcItemVisibility
    metadata?: string
    keyValueTags?: Array<workshop.KeyValueTag>
    removeKeyValueTags?: Array<string>
    removeAllKeyValueTags?: boolean
    allowAdminTags?: boolean
    contentDescriptors?: Array<workshop.ContentDescriptor>
    removeContentDescriptors?: Array<workshop.ContentDescriptor>
  }
  /**
   * Unsubscribe from a workshop item. This will result in the item being removed after the game quits.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem}
   */
  export function unsubscribe(itemId: bigint): Promise<void>
  export function updateItem(itemId: bigint, updateDetails: workshop.UgcUpdate, appId?: number | undefined | null): Promise<workshop.UgcResult>
  export function updateItemWithCallback(itemId: bigint, updateDetails: workshop.UgcUpdate, appId: number | undefined | null, successCallback: (data: UgcResult) => void, errorCallback: (err: any) => void, progressCallback?: (data: UpdateProgress) => void, progressCallbackIntervalMs?: number | undefined | null): void
  export interface UpdateProgress {
    status: workshop.UpdateStatus
    progress: bigint
    total: bigint
  }
  export const enum UpdateStatus {
    Invalid = 0,
    PreparingConfig = 1,
    PreparingContent = 2,
    UploadingContent = 3,
    UploadingPreviewFile = 4,
    CommittingChanges = 5,
  }
  export const enum UserListOrder {
    CreationOrderAsc = 0,
    CreationOrderDesc = 1,
    TitleAsc = 2,
    LastUpdatedDesc = 3,
    SubscriptionDateDesc = 4,
    VoteScoreDesc = 5,
    ForModeration = 6,
  }
  export const enum UserListType {
    Published = 0,
    VotedOn = 1,
    VotedUp = 2,
    VotedDown = 3,
    Favorited = 4,
    Subscribed = 5,
    UsedOrPlayed = 6,
    Followed = 7,
  }
  export interface WorkshopItem {
    publishedFileId: bigint
    creatorAppId?: number
    consumerAppId?: number
    title: string
    description: string
    owner: PlayerSteamId
    /** Time created in unix epoch seconds format */
    timeCreated: number
    /** Time updated in unix epoch seconds format */
    timeUpdated: number
    /** Time when the user added the published item to their list (not always applicable), provided in Unix epoch format (time since Jan 1st, 1970). */
    timeAddedToUserList: number
    visibility: workshop.UgcItemVisibility
    banned: boolean
    acceptedForUse: boolean
    tags: Array<string>
    tagsTruncated: boolean
    url: string
    numUpvotes: number
    numDownvotes: number
    numChildren: number
    previewUrl?: string
    metadata?: string
    keyValueTags?: Array<workshop.KeyValueTag>
    children?: Array<bigint>
    statistics: workshop.WorkshopItemStatistic
  }
  export interface WorkshopItemQueryConfig {
    cachedResponseMaxAge?: number
    includeMetadata?: boolean
    returnChildren?: boolean
    includeKeyValueTags?: boolean
    requiredKeyValueTags?: Array<workshop.KeyValueTag>
    /** Only supported by getUserItems. */
    cloudFileNameFilter?: string
    includeLongDescription?: boolean
    includeAdditionalPreviews?: boolean
    onlyIds?: boolean
    onlyTotal?: boolean
    language?: string
    matchAnyTag?: boolean
    requiredTags?: Array<string>
    excludedTags?: Array<string>
    searchText?: string
    rankedByTrendDays?: number
  }
  export interface WorkshopItemsResult {
    items: Array<workshop.WorkshopItem | undefined | null>
    wasCached: boolean
  }
  export interface WorkshopItemStatistic {
    numSubscriptions?: bigint
    numFavorites?: bigint
    numFollowers?: bigint
    numUniqueSubscriptions?: bigint
    numUniqueFavorites?: bigint
    numUniqueFollowers?: bigint
    numUniqueWebsiteViews?: bigint
    reportScore?: bigint
    numSecondsPlayed?: bigint
    numPlaytimeSessions?: bigint
    numComments?: bigint
    numSecondsPlayedDuringTimePeriod?: bigint
    numPlaytimeSessionsDuringTimePeriod?: bigint
  }
  export interface WorkshopPaginatedResult {
    items: Array<workshop.WorkshopItem | undefined | null>
    returnedResults: number
    totalResults: number
    wasCached: boolean
  }
}
