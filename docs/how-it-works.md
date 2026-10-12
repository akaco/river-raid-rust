# How Delta Run works

A short tour of the code, for anyone who wants to change it or rebuild a
game like it.

## One frame

`Game::frame(input)` runs 60 times a second and does everything in order:

1. **Start a frame** (`start_frame`): read the start and players keys,
   scroll the river by the plane's speed, and make new river bands as
   needed. After this the game is in the state the picture shows.
2. **Look for collisions** (`collide::check`): build the picture's pixels
   line by line and note what overlaps.
3. **Finish the frame** (`finish_frame`): act on the collisions (crash,
   refuel, score), burn fuel, steer, change speed, move the missile and
   the enemies, and set the sound.

`screen::draw` then paints the same picture to the window through the
`Canvas` trait, so drawing and collisions always agree.

## The picture

The picture is 160 pixels wide and 199 lines tall (lines 39 to 237; the
numbers follow the old TV-game screens this style comes from). Each pixel is drawn
twice as wide as it is tall, like old TV games.

- Lines 39 to 199 are the river.
- The plane sits at the bottom, from line 182 down.
- Lines 200 to 237 are the status bar: score, fuel gauge, spare planes and
  the title.

## The river

The river is six horizontal **bands**, each 32 lines tall. Band 0 is at the
bottom. Each band holds:

- a land shape (`Land`): how many 4-pixel blocks of bank come in from each
  edge, sliding from one width at the bottom of the band to another at the
  top, and whether there is an island in the middle;
- whether the bank is a road leading to a bridge;
- at most one object.

As the plane flies, the bands scroll down. When the top band has fully
appeared, the bottom one is dropped and `add_band` makes a new one above.

### The level generator

New bands come from a small random number generator (xorshift) that always
starts from the same seed, so everyone gets the same river and can learn
it. One "width" lasts two bands; sixteen widths make a **section**, and
every section ends with a bridge.

- The first stretch after a bridge is wide and calm.
- Later widths are random, and the narrowest allowed width grows with the
  section number, so the river tightens as you go.
- From section 2, one width in five opens or closes an island.
- Just before a bridge any island closes, so the bridge spans one channel.
- Objects: the further you get, the more likely a band holds an enemy
  rather than a fuel pump or a house. Jets appear from section 3.

When you blow up a bridge, the generator's state is saved as a checkpoint.
After a crash the river is rebuilt from that checkpoint, so you replay the
same section.

`tests/river.rs` flies through the first 60 sections and checks there is
always a channel at least 16 pixels wide that connects from one row to the
next, so every section can be flown.

## Collisions

Things collide when their drawn pixels overlap, so a near miss really is a
miss. For each line of the river, `collide.rs` works out the bank, the
band's object, the plane and the missile, and records:

- plane on bank, plane on object;
- missile on bank, missile on object;
- object on bank (moving boats and helicopters turn round).

If the plane touches objects in two bands at once, the lowest band counts.

## Steering and speed

The plane moves in whole pixels with a fraction kept in `sub_x`, so a tap
moves it one pixel straight away and holding the stick speeds the turn up.
Up and down change the scroll speed.

## Sound

`synth.rs` is a tiny two-voice synthesizer: each voice is a square wave or
noise with a pitch in hertz and a volume. Every frame `sound.rs` decides
what the two voices play:

- voice 0: the engine (pitch rises with speed), the low-fuel beep,
  refuelling, the tank-full ping and crashes;
- voice 1: the missile whistle and explosions.

`main.rs` hands the two voices to an audio thread (tinyaudio), which calls
`Synth::sample()` for every sample.

## The app: from game to screen

`main.rs` is the only part that knows about the window. Each time the
screen refreshes it:

1. works out the layout for the screen's size (`touch::Layout`);
2. reads the fingers, keys and game controllers into one `Input`;
3. runs as many 1/60 s game frames as are due (none, one or more, so the
   game runs at the same speed on a 60 Hz or 120 Hz screen);
4. draws the whole picture into an off-screen texture with `screen::draw`;
5. copies the river part and the status part of that texture to where the
   layout puts them, then draws the touch controls on top.

Cutting the picture in two is what lets the river fill a phone: in
landscape it is as tall as the screen with a small status panel in the
corner; in portrait it fills the width with the status and controls in a
panel below. On a computer the whole picture is shown as it is.
[mobile.md](mobile.md) explains the layout choices.

`touch.rs` has no window code, so the layout is unit tested: for common
phone and tablet sizes, both ways round, it checks the water stays inside
the safe area, nothing covers the river in portrait, and the controls do
not overlap.

## Where to change what

| To change | Look in |
| --- | --- |
| A picture (plane, enemy, bridge...) | `assets/sprites.txt` |
| Colours of water, bank, road, status bar | `src/palette.rs` |
| Rules: fuel use, speeds, scoring, extra planes | `src/game.rs` |
| How the river is made, how hard it gets | `src/river.rs` (`add_band`) |
| Enemy movement, what each object is worth | `src/objects.rs` |
| Sound effects | `src/sound.rs` |
| What is drawn where in the picture | `src/screen.rs` |
| Layout on phones and tablets, control sizes | `src/touch.rs` |
| Keys, touch, controllers, drawing the controls | `src/main.rs` |
| Android app name, package, orientation | `Cargo.toml` (`[package.metadata.android]`) |
| iPhone/iPad app settings | `ios/project.yml` |

After a change, `cargo test --release` checks the river can still be
flown, the autopilot still gets past the second bridge, and the phone
layouts still fit.
