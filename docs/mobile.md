# Phones and tablets

The same code runs on iPhone, iPad and Android. Only the build differs.

## Touch controls

- **Pad** (left thumb): left and right steer, up speeds up, down slows
  down. Press where you want to go; you do not have to drag. A finger
  that lands away from the pad, anywhere on the left half, steers from
  where it landed.
- **FIRE** (right thumb): hold to keep firing. Anywhere on the right half
  works.
- **START, 1 PLAYER, PAUSE, SOUND**: small buttons under the picture
  (portrait) or beside it (landscape).
- After a game ends, FIRE starts the next one.

Both ways round work. In portrait the picture sits at the top with the
controls below, so nothing covers it. In landscape the controls go either
side. On a computer, press T to see the controls; the mouse then works as
a finger.

## Android

You need Rust, the Android SDK (platform 31, build-tools 31), the Android
NDK (r25) and Java 17. Then:

```sh
cargo install --git https://github.com/not-fl3/cargo-quad-apk
rustup target add aarch64-linux-android armv7-linux-androideabi
export ANDROID_HOME=/path/to/android-sdk NDK_HOME=/path/to/android-ndk-r25
cargo quad-apk build --release
```

The app is `target/android-artifacts/release/apk/delta-run.apk`, signed
with a debug key, which is fine for installing it yourself: copy it to the
phone and open it (allow installing from that app when asked), or
`adb install target/android-artifacts/release/apk/delta-run.apk`.

The version of cargo-quad-apk on crates.io (0.1.4) cannot read this
project's `Cargo.toml`, hence `--git`.

## iPhone and iPad

You need a Mac with Xcode, Rust and [XcodeGen](https://github.com/yonaskolb/XcodeGen).
A free Apple ID is enough to put it on your own phone (the app then works
for 7 days before you build it again).

1. `brew install xcodegen`
2. `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
3. `cd ios && xcodegen` (makes `DeltaRun.xcodeproj`)
4. `open DeltaRun.xcodeproj`
5. In Xcode, click DeltaRun in the left column, then Signing &
   Capabilities, and pick your team (Add Account... if none is listed).
6. Plug in the iPhone or iPad, pick it at the top of the window, and
   press Run (the play button).

The first time, the phone may ask you to turn on Developer Mode (Settings,
Privacy & Security) and to trust your Apple ID as a developer (Settings,
General, VPN & Device Management). To try it without a phone, pick a
simulator instead.

Xcode does not compile the game itself: `ios/build-rust.sh` runs cargo and
puts the result into the app before Xcode signs it.

## The patched miniquad

`vendor/miniquad` is miniquad 0.4.11 with a fix for multi-touch on iOS
(lifting one finger also let go of every other one). See
[vendor/miniquad/PATCHED.md](../vendor/miniquad/PATCHED.md).
