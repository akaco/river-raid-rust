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

The picture is 160 pixels wide and 238 lines tall. Each pixel is drawn
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
  rather than a fuel tank or a house. Jets appear from section 3.

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

## Phones and tablets

The game logic has no window or sound code in it, and the window code
already has touch controls and scales the picture to any screen. macroquad
builds for iOS and Android; that is the next step.
