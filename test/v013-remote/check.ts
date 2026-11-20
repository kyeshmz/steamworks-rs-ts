import { remote_play } from '../../client'

function check(id: number, friend: bigint): void {
    const ids: number[] = remote_play.sessions()
    const result: remote_play.RemotePlaySession | null = remote_play.session(id)
    const sent: boolean = remote_play.invite(friend)
    if (result) {
        const user: bigint = result.user
        const name: string = result.clientName
        const formFactor: string | undefined = result.clientFormFactor
        const width: number | undefined = result.clientResolution?.width
        void [user, name, formFactor, width]
    }
    void [ids, sent]
}

void check
