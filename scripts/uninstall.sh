#!/usr/bin/env bash
# Remove what scripts/install.sh installed. Pass --with-schedule to also remove the LaunchAgent.
set -euo pipefail

if [ "$(uname -s)" != "Darwin" ]; then
    echo "Unsupported OS: $(uname -s). Disk Steward is macOS-only (APFS)." >&2
    exit 1
fi

WITH_SCHEDULE=0
for arg in "$@"; do
    case "$arg" in
    --with-schedule) WITH_SCHEDULE=1 ;;
    *) echo "Usage: $0 [--with-schedule]" >&2; exit 1 ;;
    esac
done

for dir in /Applications "$HOME/Applications"; do
    app="$dir/Disk Steward.app"
    [ -d "$app" ] || continue
    echo "Removing $app"
    rm -rf "$app"
done

if [ "$WITH_SCHEDULE" = 1 ]; then
    "$HOME/.cargo/bin/auto-compress-apfs" uninstall-launchd
else
    echo "The LaunchAgent schedule was kept (pass --with-schedule to remove it)."
fi
echo "Engine config, logs and state in ~/.config/auto-compress-apfs and ~/Library/Logs stay untouched."
