# Store listing text

Copy these into App Store Connect and the Google Play Console. Do not add
the name of any other game or company anywhere in the listing, keywords or
screenshots: both stores reject apps that use other people's names to be
found.

## Name

Delta Run

(Check first that the name is free in both stores: search for it on an
iPhone and in Google Play. If it is taken, pick another and change
`GAME_NAME` in `src/lib.rs`, `label` in `Cargo.toml` and
`CFBundleDisplayName` in `ios/project.yml`.)

## App Store subtitle (30 characters at most)

Retro river shooter

## Google Play short description (80 characters at most)

Fly up a river, shoot what is in your way, and never run out of fuel.

## Description (both stores)

Fly a jet up a winding river. Boats, helicopters and enemy jets are in
your way, and every bridge has to come down before you can fly on. Your
fuel runs out fast, so fly over the fuel pumps to fill up, or shoot
them for points if you are brave.

The river never ends, and it gets narrower and busier the further you go.
Each bridge you destroy is a checkpoint.

- Simple controls: steer, speed up, slow down, fire
- Play on a phone, tablet or computer
- Touch controls made for thumbs, in landscape or portrait
- Works with game controllers
- One or two players taking turns
- No ads, no in-app purchases, no tracking, no internet needed

Delta Run is open source and written in Rust.

## App Store promotional text (170 characters at most)

A fast, simple arcade game about one jet, one river and never enough fuel.

## App Store keywords (100 characters at most, commas, no spaces)

retro,arcade,shooter,jet,plane,river,pixel,8-bit,fuel,classic,scrolling,shmup,flight

## Category

Games, then Arcade (both stores).

## Support and privacy links

- Support URL: https://github.com/akaco/river-raid-rust/issues
- Privacy policy URL: https://github.com/akaco/river-raid-rust/blob/main/docs/privacy.md

Rename the repository before you submit, so these links do not carry
another game's name (GitHub keeps the old links working after a rename),
and update the two links here and in `docs/privacy.md`.

## Answers to the stores' questions

- Price: free. No in-app purchases. No ads.
- Data collected: none (App Store: "Data Not Collected"; Google Play Data
  safety: no data collected or shared).
- Age rating: answer the questionnaires honestly; the only thing to report
  is mild cartoon or fantasy violence (shooting boats and planes, no
  blood). Expect the lowest or second-lowest age band.
- Export compliance (App Store): the app uses no encryption. The app
  already says so, so App Store Connect will not ask.
- Login needed to review the app: no.

## Pictures

- `store/play-icon-512.png`: Google Play app icon (512 x 512).
- `store/feature-graphic.png`: Google Play feature graphic (1024 x 500).
- Screenshots: see "Screenshots" in [docs/publishing.md](../docs/publishing.md).
