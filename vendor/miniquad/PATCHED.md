# miniquad 0.4.11, patched

This is [miniquad](https://github.com/not-fl3/miniquad) 0.4.11 from
crates.io (MIT or Apache-2.0, see the licence files here) with one change,
in `src/native/ios.rs`:

On iOS, every touch event reported *all* fingers on the screen with the
event's phase. So lifting one finger (say, letting go of FIRE) also ended
every other finger, and a thumb held still on the steering pad stopped
steering until it moved. The fix reports only the touches that changed,
which is what UIKit passes to `touchesBegan`/`Moved`/`Ended`/`Cancelled`.

The root `Cargo.toml` uses this copy through `[patch.crates-io]`. Drop it
once a miniquad release has the fix.

In `src/native/android/keycodes.rs`, game controller buttons now map to
the game's keys (A/B/X/Y and the shoulder buttons to Space, Start to
Pause, Select to Tab); upstream reported them as unknown keys.

Also `#![allow(warnings)]` at the top of `src/lib.rs`, so building the game
does not print miniquad's own warnings.
