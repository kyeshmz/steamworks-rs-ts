# Request an encrypted app ticket

Call `steam.auth.requestEncryptedAppTicket(data?, timeoutSeconds?)` to get a ticket encrypted with your app's secret "encrypted app ticket key". Send the resolved `Buffer` to your secure backend and decrypt it there with the SteamEncryptedAppTicket functions. This is a different credential from a session or Web API ticket. Never pass it to `steam.auth.beginAuthSession`, which expects a peer's session ticket.

The optional `data` argument is application data, at most 128 bytes, that Steam encrypts into the ticket. Read it back server-side with `SteamEncryptedAppTicket_GetUserVariableData`. The optional `timeoutSeconds` bounds the whole promise, queue wait included, and defaults to 10. The promise resolves with an owned copy of the ticket bytes.

Steam allows one pending response and rate-limits this request to once per 60 seconds. Concurrent local callers are queued so each promise receives the ticket Steam produced for its own request instead of racing the shared retrieval buffer. A caller that reaches Steam inside the rate window gets Steam's rejection, such as a rate-limit `EResult`, as a thrown error. Requests are also rejected before touching Steam when the data exceeds 128 bytes, and a request that completes late or never is rejected with a timeout error after `timeoutSeconds`.

The request key is configured on the Encrypted App Ticket Key page of your app's Steamworks administration. Keep it off the client. A returned ticket is only proof after your backend decrypts and checks it; treat an unresolved promise as no proof at all.

The offline fixture covers concurrent calls, input copying, resizing and rejection. It does not verify real Steam ticket issuance, decryption, or how a live callback pump interacts with polling the result. A timeout releases the local queue slot; it does not cancel Steam’s pending request.
