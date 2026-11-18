export declare var init: (appId?: number) => Omit<Client, 'init' | 'runCallbacks' | 'restartAppIfNecessary' | 'game_server'>;
export declare var restartAppIfNecessary: (appId: number) => boolean;
export declare var electronEnableSteamOverlay: (disableEachFrameInvalidation?: boolean) => void;
export { SteamCallback };
export { gameServer };
export type Client = typeof import('./client.d');
declare const SteamCallback: typeof import("./client.d").callback.SteamCallback;
/** @type {Client['game_server']} */
declare const gameServer: Client['game_server'];
