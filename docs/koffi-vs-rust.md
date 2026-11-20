# Koffi and Rust bindings

This repository is a Rust N-API addon. The Outdraw projector trial uses a
TypeScript binding that calls the same Steamworks flat C API through Koffi.
Both approaches expose Steam to JavaScript. They differ in where the binding
lives, how it is built, and what can be checked before the game runs.

## Benefits of Koffi

- **No Rust toolchain in the game build.** Koffi publishes prebuilt N-API
  binaries for common platforms, so a game project can install it without
  compiling a native addon.
- **Bindings stay in TypeScript.** C signatures, struct layouts, callbacks and
  ticket handling are ordinary TypeScript code. There is no Rust crate, N-API
  glue or generated binding layer to maintain.
- **Faster iteration.** A binding change is a TypeScript edit plus a Node test.
  The feedback loop does not include a Rust rebuild.
- **Node-only tests.** The binding can run under Node without Electron, which
  makes callback and error-path tests cheaper to write and run.
- **Electron main-process support.** Electron can load Koffi in its main
  process. The renderer only sees the IPC surface the app exposes.
- **Stable N-API ABI.** Koffi targets Node-API, so the same binary can be used
  across supported Node and Electron versions without a per-version rebuild.
- **Small repository surface.** The project ships Koffi and Valve's Steam SDK
  redistributable instead of a Rust workspace, Cargo lockfile and N-API build
  pipeline.
- **Easy inspection.** The exact C declaration used for a call is visible next
  to the call site, which helps when comparing behaviour with the Steamworks
  headers.

## Benefits of Rust

- **Compile-time checking.** The Rust wrapper checks types, ownership and much
  of the SDK surface before the game runs. Hand-written FFI declarations cannot
  provide the same guarantee.
- **Memory safety in binding code.** Rust reduces the risk of use-after-free,
  buffer overruns and data races in the glue code. The Steam SDK boundary still
  needs `unsafe` blocks and careful callback handling.
- **No JavaScript FFI marshalling layer.** Rust calls the SDK directly. Koffi's
  overhead is small, but it is still a JavaScript-to-C transition per call.
- **Native hosts.** A Rust addon can run in a Tauri host or another native
  process without bundling Node. A Node-based Koffi binding needs a sidecar in
  those environments.
- **Mature Steamworks crate.** `steamworks-rs` provides typed wrappers for
  users, friends, matchmaking, networking, inventory, workshop and callbacks.
- **Generated declarations.** N-API generates `client.d.ts` from the Rust
  signatures, so the published TypeScript surface stays close to the native
  implementation.

## Tradeoffs

| Area | Koffi binding | Rust addon |
| --- | --- | --- |
| Build | Ship Koffi's prebuilt binary and Valve's SDK binary. | Compile an N-API addon for each target. |
| Toolchain | Node; a C toolchain only when no prebuilt binary exists. | Rust, Cargo and a native toolchain. |
| ABI checks | Declare C signatures and struct layouts by hand. | The Rust wrapper and compiler check more of the interface. |
| API changes | Edit TypeScript and run the Node tests. | Edit Rust, rebuild, and regenerate declarations. |
| Packaging | Koffi binary plus the SDK; unpack both from Electron's archive. | `.node` addon plus the SDK. |
| Hosts | Node and Electron main process; Tauri needs a Node sidecar. | Node, Electron and native hosts, including Tauri. |
| Debugging | Signatures and layouts are visible in TypeScript. | Rust types and stack traces; slower rebuild cycle. |
| Callback safety | Lifetime and thread affinity are managed manually. | Ownership helps, but SDK callbacks still require care. |

## When Koffi is the better fit

- The game runs only in Electron.
- The Steam surface is small and changes often.
- The team wants to iterate and test in TypeScript.
- Avoiding a Rust build in CI is more valuable than compile-time ABI checks.
- The team accepts manual verification of C signatures and struct layouts.

## When Rust is the better fit

- The game runs in Tauri or another native host.
- The Steam surface is broad and long-lived.
- Compile-time checks and memory safety in the binding code matter more than
  build simplicity.
- The team already maintains Rust and wants generated TypeScript declarations.
- The binding must ship as one native addon without a Node runtime.

## Verification

Neither approach removes the need to test against a real Steam client. The SDK
redistributable, a signed-in Steam client and the target platform are required
for live authentication, overlay and callback checks. Offline fixtures can
prove call sequences and error handling, but they cannot prove that Steam
delivers a ticket, renders an overlay or validates a session.

Test the packaged artifact, not only the source tree. On Linux, Electron tests
need a display server; overlay rendering needs a real compositor. On macOS and
Windows, test the signed release build because code signing and archive layout
can change which native library is loaded.
