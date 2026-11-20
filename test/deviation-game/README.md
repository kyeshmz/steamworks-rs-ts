# Verify a specific release tarball

`pack.sh <game-checkout> <game-commit> <vendor|fixture> [tarball]` installs the supplied tarball in a temporary Electron application under `target/deviation-game/`. If the last argument is omitted, it packs the current checkout first.

Run `vendor` to check the actual redistributable and `fixture` to exercise the game's IPC handlers against the offline SDK. Run both against the same tarball. Each process verifies the paths of the native addon and library in `/proc/self/maps`.

On Linux hosts that restrict unprivileged user namespaces, set `CHROME_DEVEL_SANDBOX` to an existing root-owned, mode-4755 Chromium sandbox helper, such as `/opt/google/chrome/chrome-sandbox`. The script links it in the disposable app bundle. It does not modify the host helper or disable sandboxing.
