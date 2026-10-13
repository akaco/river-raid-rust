# Delta Run

A small river shooter written in Rust. Fly up a river, shoot boats,
helicopters, jets and bridges, and fly over fuel pumps before you run dry.

![Delta Run](docs/screenshot.png)

It is a project about rebuilding a classic kind of game from scratch in
Rust: plain structs and functions, no emulator, no engine beyond
[macroquad](https://github.com/not-fl3/macroquad) for the window and
[tinyaudio](https://github.com/mrDIMAS/tinyaudio) for sound. Everything in
it (the pictures, colours, sounds, font and the river levels) was made for
this project and is MIT licensed.

## Play it

You need Rust ([rustup.rs](https://rustup.rs)). Then:

```sh
git clone https://github.com/akaco/river-raid-rust.git
cd river-raid-rust
cargo run --release
```

Tested on macOS (Apple Silicon) and Linux. On Linux you may need the usual
window and sound libraries (`libx11-dev libxi-dev libgl1-mesa-dev
libasound2-dev` on Debian/Ubuntu). To build without sound:
`cargo run --release --no-default-features`.

| Key | Does |
| --- | --- |
| Arrows or WASD | Steer left/right, speed up (up) or slow down (down) |
| Space | Fire |
| Enter | Start a game |
| Tab | One or two players |
| 1 | Straight missiles on/off (off: missiles follow the plane) |
| P | Pause |
| M | Mute |
| T | Show the touch controls (the mouse works as a finger) |

## Phones and tablets

The same game runs on iPhone, iPad and Android, in landscape (full screen)
or portrait. A pad on the left steers and changes speed, FIRE is on the
right, and pause and sound are in the top corner. Game controllers work
too.

<img src="docs/phone-landscape.png" height="240" alt="Phone, landscape"> <img src="docs/phone-portrait.png" height="240" alt="Phone, portrait">

- **Android**: `cargo quad-apk build --release` makes an APK you can
  install on your phone.
- **iPhone, iPad and Mac**: `cd ios && xcodegen`, open the project in
  Xcode, pick your team and press Run.

[docs/mobile.md](docs/mobile.md) has the full steps, the controls, and
why the screen is laid out the way it is.
[docs/publishing.md](docs/publishing.md) is the way into the App Store and
Google Play.

## How to play

- Fuel goes down all the time. Fly over a fuel pump to fill up; shooting
  it gives points but no fuel.
- Hitting a bank, a boat, a helicopter, a jet or a bridge costs a plane.
- Each bridge is a checkpoint: after a crash you start again from the last
  one you blew up.
- The river gets narrower and busier further up. Islands appear from the
  second section, jets from the third.
- You get an extra plane every 10,000 points.

## How it is built

```
src/
  lib.rs       the game as a library (no window needed)
  game.rs      rules: phases, fuel, score, lives, steering, missiles
  river.rs     the river: six bands on screen, the level generator
  objects.rs   boats, helicopters, jets, fuel pumps, houses, bridges
  collide.rs   what touches what, by overlapping pixels
  screen.rs    where things are on screen, and drawing to any Canvas
  sprites.rs   reads the pictures file; the font
  palette.rs   colours
  sound.rs     sound effects  ┐ our own little synthesizer:
  synth.rs     square + noise ┘ two voices
  touch.rs     screen layout for phones and tablets; the steering pad
  main.rs      the app: window, keys, touch, controllers, audio thread
assets/
  sprites.txt  every picture, as letters you can edit
examples/
  sprite_sheet.rs  draws every sprite into one picture
  store_art.rs     draws the app icon and store pictures
tests/         river and autopilot tests
docs/          how it works, phones and tablets, publishing, privacy
ios/           Xcode project (XcodeGen) for iPhone, iPad and Mac
android/       app icons and the Google Play bundle script
store/         store text, store pictures
vendor/        miniquad with fixes for iOS touch and Android controllers
```

The game runs at a fixed 60 frames a second and draws a 160 x 199 pixel
picture, which the app cuts up and scales to fit the screen. Because the
game itself never touches the window, the same code is tested headless
and runs on computers and phones alike.

**Read next:** [docs/how-it-works.md](docs/how-it-works.md) is a tour of
the code: one frame step by step, the river generator, collisions, sound,
how the app draws on any screen, and where to change what.

## Change the pictures

Every picture (plane, enemies, fuel pump, house, bridge, explosions) is in
[assets/sprites.txt](assets/sprites.txt): one letter per pixel, with the
colour of each letter written above the picture. Edit it and
`cargo run --release` again; no code to change. If something in the file
is wrong, the game stops and says which line. To see every picture at
once: `cargo run --example sprite_sheet > sprites.ppm`.

![All sprites](docs/sprites.png)

## Tests

```sh
cargo test --release
```

- `tests/river.rs` checks every river section the generator makes can be
  flown through, objects start on open water, and every game gets the same
  river.
- `tests/play.rs` lets a simple autopilot play 60,000 frames (about 17
  minutes) and checks it gets past the second bridge.

## Licence

MIT, see [LICENSE](LICENSE).
