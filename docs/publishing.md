# Putting Delta Run in the App Store and Google Play

What is ready in this repository:

| | Where |
| --- | --- |
| App icons (iPhone, iPad, Mac, Android) | `ios/Assets.xcassets`, `android/res`; remade by `store/make-art.sh` |
| App ID | `io.github.akaco.deltarun` on every store |
| Version | 1.0.0, build 1 (`Cargo.toml`, `ios/project.yml`) |
| Mac app | the `DeltaRunMac` target in the Xcode project |
| Google Play bundle (.aab) | `android/build-aab.sh` |
| Privacy policy | [privacy.md](privacy.md) |
| Store text and answers | [store/listing.md](../store/listing.md) |
| Play icon and feature graphic | `store/play-icon-512.png`, `store/feature-graphic.png` |

The App ID can never change once the app is in a store. If you want a
different one, change it in `Cargo.toml` and `ios/project.yml` before the
first upload.

## Read this first: the look-alike risk

Delta Run is clean: its own name, pictures, sounds, font and levels, and no
other game's name anywhere. But it plays like a famous 1982 game whose
owner is still in business. Game rules are not protected by copyright;
names, pictures, sounds and, in some courts, a game's overall look are.

So:

- A reviewer may reject it as a copy (Apple guideline 4.1, "Copycats";
  Google Play's intellectual property and impersonation policies). If that
  happens, reply that all art, sound and code are original, and point to
  this repository.
- The owner of the original could ask the stores to remove it, even after
  it is approved.
- Lower the risk: never name the original game or its maker in the
  listing, keywords, screenshots or repository; rename this repository
  (its name contains the original's name); the more Delta Run looks and
  plays like its own game, the safer it is.
- Never upload the private, exact rewrite that uses the original pictures.

## 1. Run it on your own iPhone or iPad (free)

Follow "iPhone and iPad" in [mobile.md](mobile.md). One change: put your
Team ID in `ios/project.yml` (`DEVELOPMENT_TEAM: ""`) instead of picking it
in Xcode, so it stays when you run `xcodegen` again. With a free Apple ID,
the team is shown in Xcode under Settings, Accounts; with a paid account,
the Team ID is on [developer.apple.com/account](https://developer.apple.com/account)
under Membership details.

To try the Mac version: pick the `DeltaRunMac` scheme at the top of Xcode,
then "My Mac", then Run.

Play it for a while on a real phone and a real iPad before going further.

## 2. Make the accounts

- **Apple Developer Program**: $99 a year.
  [developer.apple.com/programs/enroll](https://developer.apple.com/programs/enroll/).
  Enroll as an individual with your Apple ID. Approval can take a day or two.
- **Google Play Console**: $25 once.
  [play.google.com/console/signup](https://play.google.com/console/signup).
  Choose a personal account. Google checks your ID. A new personal account
  must run a closed test with at least 12 testers for at least 14 days
  before the game can be published to everyone, so start that early (step
  4).

## 3. App Store (iPhone, iPad and Mac)

Use the newest Xcode: App Store Connect only takes apps built with the
current one.

1. In [App Store Connect](https://appstoreconnect.apple.com), Apps, "+",
   New App. Platforms: iOS and macOS. Name: Delta Run. Bundle ID:
   `io.github.akaco.deltarun` (if it is not in the list, build once in
   Xcode with your paid team, or add it under Certificates, Identifiers &
   Profiles, Identifiers). SKU: `deltarun`.
2. Upload the iPhone/iPad version: in Xcode pick the `DeltaRun` scheme and
   "Any iOS Device", then Product, Archive. In the window that opens,
   Distribute App, App Store Connect, Upload.
3. Upload the Mac version the same way with the `DeltaRunMac` scheme and
   "Any Mac".
4. **TestFlight**: after a few minutes the builds show in App Store
   Connect under TestFlight. Add yourself as a tester, install the
   TestFlight app on your iPhone, and play the real store build.
5. Fill in the app page from [store/listing.md](../store/listing.md):
   description, keywords, subtitle, support and privacy links,
   screenshots (below), App Privacy ("Data Not Collected"), age rating,
   price (free), category (Games, Arcade).
6. Pick the uploaded build for iOS and for macOS, and press "Add for
   Review", then "Submit". Review usually takes a day or two.

For each new version: raise `MARKETING_VERSION` (what players see) and
`CURRENT_PROJECT_VERSION` (by one, every upload) in `ios/project.yml`, run
`xcodegen` again, and archive again.

## 4. Google Play (Android phones and tablets)

1. **Make your upload key, once**, on your Mac:

   ```sh
   keytool -genkeypair -v -keystore ~/delta-run-upload.jks -alias upload \
       -keyalg RSA -keysize 2048 -validity 10000
   ```

   Pick a password and remember it. Back the file up somewhere safe (not in
   the repository, not public). Every future update must be signed with it.
   (`keytool` comes with Java: `brew install openjdk`.)
2. **Make the bundle (.aab).** Build the APK as in [mobile.md](mobile.md),
   then download `bundletool-all-*.jar` from
   [bundletool's releases](https://github.com/google/bundletool/releases)
   and run:

   ```sh
   BUNDLETOOL=~/Downloads/bundletool-all-1.18.1.jar sh android/build-aab.sh
   ```

   It asks for the key's password and makes
   `target/android-artifacts/release/delta-run.aab`. Without the key it
   makes an unsigned bundle and prints the command to sign it later.
3. In the [Play Console](https://play.google.com/console), Create app:
   name Delta Run, Game, Free.
4. Do the "Set up your app" list on the dashboard, with the answers in
   [store/listing.md](../store/listing.md): privacy policy, app access (no
   login), ads (none), content rating, target audience (13 and over keeps
   it simple; younger ages bring extra rules for children's apps), data
   safety (no data collected), and the store listing (text, icon, feature
   graphic, screenshots).
5. Testing, Closed testing: create a track, upload the .aab, and accept
   "Play App Signing" (Google keeps the final key; yours only uploads).
   Add at least 12 testers by email (friends, family) and send them the
   link. They must stay in the test for 14 days.
6. After the 14 days: Dashboard, "Apply for production". Once approved,
   Production, create a release with the same .aab, and roll it out.
   Review takes from a few hours to a few days.

For each new version: raise `version` and `version_code` (by one, every
upload) in `Cargo.toml`, then build the APK and the bundle again.

## Screenshots

The stores need pictures of the game running at exact sizes:

- **iPhone**: in Xcode run the game in the simulator for the biggest
  iPhone (Pro Max), and press Cmd-S in the simulator to save a screenshot.
  At least one, up to ten.
- **iPad**: the same with the biggest iPad Pro (13-inch).
- **Mac**: play full screen, take a screenshot (Cmd-Shift-3), and in
  Preview crop it to 16:10 and resize it to 2880 x 1800.
- **Google Play**: at least 2 phone screenshots. The iPhone ones are too
  long for Play (no more than twice as long as wide), so crop them to
  2640 x 1320. For tablets, the iPad ones work as they are.

Landscape shows the game best.
