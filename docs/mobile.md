# Phones and tablets

The same code runs on iPhone, iPad and Android. Only the build differs.

## Screen and controls

**Landscape is the full-screen way to play.** The river is as tall as the
screen and as wide as the safe area allows; the land at its edges runs on
under the notch. The status (planes, score, fuel) is a small panel in the
top-left corner, and pause and sound are in the top-right corner. The pad
and FIRE sit over the bottom corners, which are mostly bank.

**Portrait** works like a handheld console: the river fills the width at
the top, and the panel below it holds the status, pause, sound and the
thumb controls. The river cannot be bigger in portrait without hiding
water, because it is wider than it is tall.

**Tablets** get the same layouts with smaller controls for the size of the
screen, and in landscape the status panel goes under the river.

- **Pad** (left thumb): left and right steer, up speeds up, down slows
  down. Press where you want to go; you do not have to drag. A knob
  follows your thumb. A finger that lands away from the pad, anywhere on
  the left half, steers from where it landed.
- **FIRE** (right thumb): hold to keep firing. Anywhere on the right half
  works.
- **Menu**: START and 1/2 PLAYERS appear over the river when a game is
  over (FIRE starts one too); RESUME and NEW GAME when paused.
- **Game controllers**: D-pad or left stick to fly, A/B (or the right
  trigger) to fire, Menu/Start to pause or start, X/Select for players.
  Using a controller or keyboard hides the touch controls; touching the
  screen brings them back.
- **iPhone vibration**: a short bump when the plane crashes.

On a computer, press T to see the touch controls; the mouse then works as
a finger.

### Why it is laid out this way

What the layout follows, and where it comes from:

- **Use the safe area.** Keep controls clear of the notch or Dynamic
  Island, rounded corners and the home bar; put frequent actions near the
  thumbs and rare ones (pause, menus) at the top; make the touch area
  bigger than the drawn control, because you cannot feel where your thumb
  is; show buttons only when they are useful. Apple, WWDC26 session
  ["Make your game great with touch"](https://developer.apple.com/videos/play/wwdc2026/358/)
  and the [Touch Controller framework](https://developer.apple.com/documentation/touchcontroller).
  The window library does not report the safe area, so the game
  estimates it from the screen's shape (phones longer than 19.5:9 have a
  notch or island).
- **Thumbs in the lower corners, a mark where the thumb is.** Each touch
  sets its own starting point, and a mark at that point teaches the
  control; rare actions go to a top corner. ["Virtual gamepad",
  Pin Eight](https://pineight.com/mw/page/Virtual_gamepad.xhtml).
- **A pad that works on press, with a dead zone.** Steering acts as soon
  as the thumb is on one side of the pad, with no drag needed, and a small
  middle zone does nothing. Sideways wins near the diagonals, so steering
  does not change speed by accident.
- **Game on top, controls below in portrait.** The same split that
  console-style apps such as the [Delta](https://apps.apple.com/app/id1048524688)
  emulator use for wide games on a tall phone.
- **Controllers.** Android sends controller buttons as key events
  (`KEYCODE_BUTTON_A`, `KEYCODE_BUTTON_START` and so on) and turns the
  D-pad and stick into arrow keys
  ([Android: handle controller actions](https://developer.android.com/training/game-controllers/controller-input)).
  iPhone and iPad read controllers through Apple's GameController
  framework.

Not done yet: vibration on Android, reading the exact safe area from the
system, and letting players move or resize the controls.

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
(lifting one finger also let go of every other one) and game controller
buttons on Android. See
[vendor/miniquad/PATCHED.md](../vendor/miniquad/PATCHED.md).
