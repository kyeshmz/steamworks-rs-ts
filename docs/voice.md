# Push-to-talk voice

Get the `voice` namespace from `require('@kyeshmz/steamworks.ts').init(appId)`. Initialization does not turn on the microphone. The addon never sends, plays, or saves audio.

1. When the player presses the talk key, call `voice.startVoiceRecording()`.
2. Once per frame, call `voice.getVoice()`. A nonempty `Buffer` contains one compressed Steam Voice chunk. An empty buffer means no data is ready. Poll on a later frame rather than looping.
3. When the player releases the key or the game loses focus, call `voice.stopVoiceRecording()`. Steam briefly continues recording after this call. Continue polling once per frame to collect the tail.
4. Stop polling when the read throws `GetAvailableVoice: k_EVoiceResultNotRecording` or `GetVoice: k_EVoiceResultNotRecording`. Handle other errors separately. A failed read does not return partial audio.

If your app sends voice, preserve each chunk's boundaries. On receipt, call `voice.decompressVoice(compressed, sampleRate)`. The result contains raw mono 16-bit PCM samples, not a WAV file. Supply an integer output rate from 11,025 to 48,000 Hz. `voice.getVoiceOptimalSampleRate()` returns the decoder's native rate if you prefer it to the playback device's rate. Your app owns transport, playback, and microphone permissions.

Each read and each decode input is limited to 64 KiB of compressed data. Decode rejects empty input. PCM output is limited to 1 MiB per call. If Steam requests a larger PCM buffer within that limit, the addon retries once. If captured audio grows between the size check and the read, `getVoice()` can throw `GetVoice: k_EVoiceResultBufferTooSmall`. Poll again on the next frame. Decode failures such as `k_EVoiceResultDataCorrupted` throw rather than returning silence.
