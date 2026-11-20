import steamworks from 'steamworks.js'

export function relaunch(): boolean {
    return steamworks.restartAppIfNecessary(3104020)
}

export async function projector() {
    const client = steamworks.init(3104020)
    steamworks.electronEnableSteamOverlay()
    const name: string = client.localplayer.getName()
    const id: bigint = client.localplayer.getSteamId().steamId64
    const ticket = await client.auth.getSessionTicketWithSteamId(id)
    const bytes: Buffer = ticket.getBytes()
    const score: number | null = client.stats.getInt('GAMES_WON')
    const stored: boolean[] = [client.stats.setInt('GAMES_WON', 1), client.stats.store(), client.stats.resetAll(true)]
    const achievement: boolean[] = [client.achievement.activate('a'), client.achievement.isActivated('a'), client.achievement.clear('a')]
    const installed: boolean = client.apps.isAppInstalled(3104020)
    return { name, id, hex: bytes.toString('hex'), score, stored, achievement, installed }
}
