# Delta Run

A small river shooter written in Rust. Fly up a river, shoot boats,
helicopters, jets and bridges, and fly over fuel tanks before you run dry.

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

On a touch screen: drag on the left half to steer, touch the right half to
fire, and use the START and PLAYERS buttons along the top.

## How to play

- Fuel goes down all the time. Fly over a fuel tank to fill up; shooting
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
  lib.rs      the game as a library (no window needed)
  game.rs     rules: phases, fuel, score, lives, steering, missiles
  river.rs    the river: six bands on screen, the level generator
  objects.rs  boats, helicopters, jets, fuel tanks, houses, bridges
  collide.rs  what touches what, by overlapping pixels
  screen.rs   where things are on screen, and drawing to any Canvas
  sprites.rs  all pictures and the font, as text pixel art
  palette.rs  colours
  sound.rs    sound effects  ┐ our own little synthesizer:
  synth.rs    square + noise ┘ two voices
  main.rs     the macroquad window, keys, touch, audio thread
```

The game runs at a fixed 60 frames a second and draws a 160 x 238 pixel
picture, which the window scales to fit. Because `game.rs` never touches
the window, the same code can be tested headless and run on phones.

[docs/how-it-works.md](docs/how-it-works.md) explains the river, the
collisions and the sound in more detail.

## Tests

```sh
cargo test --release
```

- `tests/river.rs` checks every river section the generator makes can be
  flown through, objects start on open water, and every game gets the same
  river.
- `tests/play.rs` lets a simple autopilot play 60,000 frames (about 17
  minutes) and checks it gets past the second bridge.

## Plans

- iPhone, iPad and Android builds (macroquad supports both).
- Better touch controls.

## Licence

MIT, see [LICENSE](LICENSE).
