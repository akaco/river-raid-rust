#!/bin/sh
# Makes the Android App Bundle (.aab) that Google Play asks for, signed with
# your upload key. Run from the top folder after building the APK:
#
#   cargo quad-apk build --release
#   sh android/build-aab.sh
#
# Needs ANDROID_HOME (as for the APK), Java, and bundletool
# (https://github.com/google/bundletool/releases): put the jar's path in
# BUNDLETOOL. The upload key is KEYSTORE (default ~/delta-run-upload.jks)
# with the name KEY_ALIAS (default upload); docs/publishing.md says how to
# make one. Keep that file and its password safe and out of the repository.
# Without the key, the bundle is left unsigned, to sign later.
#
# cargo-quad-apk makes only APKs. It leaves the pieces of the app behind,
# though (the program, the Java part, the manifest, the pictures), and this
# puts them together again in the bundle's layout.
set -eu

: "${ANDROID_HOME:?set ANDROID_HOME to the Android SDK}"
: "${BUNDLETOOL:?set BUNDLETOOL to bundletool-all-*.jar}"
keystore=${KEYSTORE:-$HOME/delta-run-upload.jks}
alias=${KEY_ALIAS:-upload}

parts=target/android-artifacts/release/bin/delta-run
out=target/android-artifacts/release/delta-run.aab
[ -f "$parts/classes.dex" ] || { echo "run cargo quad-apk build --release first" >&2; exit 1; }

tools=$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -1)
platform=$(sed -n 's/^android_version *= *\([0-9]*\).*/\1/p' Cargo.toml)
jar="$ANDROID_HOME/platforms/android-$platform/android.jar"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The same resources, linked the same way as for the APK, so they get the
# same numbers, but in the bundle's format.
"$tools/aapt2" link --proto-format -o "$work/linked.zip" -I "$jar" \
    --manifest "$parts/AndroidManifest.xml" \
    -R "$parts/build/res.zip" -R "$parts/build/res2.zip" --auto-add-overlay

mkdir -p "$work/base/manifest" "$work/base/dex"
(cd "$work/base" && unzip -q ../linked.zip)
mv "$work/base/AndroidManifest.xml" "$work/base/manifest/"
cp "$parts/classes.dex" "$work/base/dex/"
cp -R "$parts/lib" "$work/base/"
(cd "$work/base" && zip -q -r ../base.zip .)

rm -f "$out"
java -jar "$BUNDLETOOL" build-bundle --modules="$work/base.zip" \
    --config=android/BundleConfig.json --output="$out"

if [ -f "$keystore" ]; then
    echo "Signing with $keystore ($alias)"
    jarsigner -sigalg SHA256withRSA -digestalg SHA-256 -keystore "$keystore" "$out" "$alias"
    echo "Done: $out"
else
    echo "Made $out, not signed (no upload key at $keystore). To sign it:"
    echo "  jarsigner -sigalg SHA256withRSA -digestalg SHA-256 -keystore \"$keystore\" $out $alias"
fi
