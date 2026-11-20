# Remote Play

After `init`, `client.remote_play.sessions()` lists active, nonzero Remote Play session IDs as 32-bit numbers. The list is a snapshot and a session may expire before lookup. `client.remote_play.session(id)` returns `null` when Steam no longer provides a client name, otherwise it returns the ID, the user's 64-bit Steam ID as a `bigint`, client name, form factor (`phone`, `tablet`, `computer`, or `tv`), and resolution. Unknown form factors and unavailable resolutions are optional properties.

Session IDs must be nonzero integers within the unsigned 32-bit range. `client.remote_play.invite(friendSteamId)` sends a Remote Play Together invite to a nonzero unsigned 64-bit `bigint` and returns Steam's boolean result. It does not require an existing session. Steam controls whether an invite can be sent or accepted.

The session list is bounded to 4096 entries to avoid unbounded allocation if the SDK reports a corrupt count. Details can change or expire during lookup. The returned values are copies, not live session handles.
