#!/bin/sh
# Run by Xcode: builds the game with cargo for the phone, the simulator or
# the Mac, and puts it in the app in place of the stand-in program.
set -eu

# The oldest system the app runs on, from the Xcode project, so the game
# and the app agree (App Store Connect checks).
case "$PLATFORM_NAME" in
    iphoneos) targets=aarch64-apple-ios; os="IPHONEOS_DEPLOYMENT_TARGET=$IPHONEOS_DEPLOYMENT_TARGET" ;;
    iphonesimulator) targets=aarch64-apple-ios-sim; os="IPHONEOS_DEPLOYMENT_TARGET=$IPHONEOS_DEPLOYMENT_TARGET" ;;
    # Macs with Apple chips and Intel Macs
    macosx) targets="aarch64-apple-darwin x86_64-apple-darwin"; os="MACOSX_DEPLOYMENT_TARGET=$MACOSX_DEPLOYMENT_TARGET" ;;
    *) echo "error: unknown platform $PLATFORM_NAME" >&2; exit 1 ;;
esac

# Xcode's settings (SDKROOT and friends) point at the iPhone SDK, which
# breaks the parts cargo builds to run on the Mac, so cargo gets a clean
# environment.
path="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
cd "$SRCROOT/.."
run() { env -i HOME="$HOME" PATH="$path" "$os" "$@"; }
built=""
for target in $targets; do
    if run sh -c "command -v rustup" >/dev/null; then
        run rustup target add "$target"
    fi
    run cargo build --release --target "$target"
    built="$built target/$target/release/delta-run"
done
# one program for both kinds of Mac (or just the one for a phone)
# shellcheck disable=SC2086
lipo -create $built -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
