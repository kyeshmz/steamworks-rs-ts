## Sub-features
Native loading and the public callback identifiers.

## How to get to it (user POV)
Load the built package with `require('.')`.

## Driving it with Node
Run `node -e "const s=require('.');require('node:assert/strict').equal(s.SteamCallback.PersonaStateChange,0);console.log('PersonaStateChange = 0')"` and retain stdout.

## Gotchas
Do not call `init` for the loader check. Operations that need a Steam client require separate integration evidence.
