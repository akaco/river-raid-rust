#!/bin/sh
# Run by Xcode: builds the game with cargo for the phone (or the
# simulator) and puts it in the app in place of the stand-in program.
set -eu

case "$PLATFORM_NAME" in
    iphoneos) target=aarch64-apple-ios ;;
    iphonesimulator) target=aarch64-apple-ios-sim ;;
    *) echo "error: unknown platform $PLATFORM_NAME" >&2; exit 1 ;;
esac

# Xcode's settings (SDKROOT and friends) point at the iPhone SDK, which
# breaks the parts cargo builds to run on the Mac, so cargo gets a clean
# environment.
path="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
cd "$SRCROOT/.."
if env -i HOME="$HOME" PATH="$path" sh -c "command -v rustup" >/dev/null; then
    env -i HOME="$HOME" PATH="$path" rustup target add "$target"
fi
env -i HOME="$HOME" PATH="$path" cargo build --release --target "$target"
cp "target/$target/release/delta-run" "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
