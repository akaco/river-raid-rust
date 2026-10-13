#!/bin/sh
# Makes the app icons and store pictures from the game's own pictures
# (examples/store_art.rs). Needs ImageMagick (`brew install imagemagick`).
# The results are in the repository already; run this again only after
# changing the art. Run from the top folder: sh store/make-art.sh
set -eu
magick=$(command -v magick || command -v convert)
tmp=$(mktemp -d)
art() { cargo run -q --no-default-features --example store_art -- "$1" > "$tmp/$1.ppm"; }
art icon; art adaptive; art feature
# 24-bit PNG with no see-through parts (the App Store refuses icons with
# them); PNG32 where Google Play asks for 32-bit
png() { "$magick" "$tmp/$1.ppm" -resize "$2" -alpha off -strip "PNG24:$3"; }

# iPhone, iPad and Mac
set_dir=ios/Assets.xcassets/AppIcon.appiconset
png icon 1024x1024 "$set_dir/icon-1024.png"
for s in 16 32 64 128 256 512; do png icon "${s}x$s" "$set_dir/icon-$s.png"; done

# Android: plain icons, and the full-bleed layer for round and squircle icons
for d in mdpi:48:108 hdpi:72:162 xhdpi:96:216 xxhdpi:144:324 xxxhdpi:192:432; do
    IFS=: read -r dpi plain layer <<END
$d
END
    mkdir -p "android/res/mipmap-$dpi"
    png icon "${plain}x$plain" "android/res/mipmap-$dpi/ic_launcher.png"
    png adaptive "${layer}x$layer" "android/res/mipmap-$dpi/ic_launcher_background.png"
done

# Google Play listing
"$magick" "$tmp/icon.ppm" -resize 512x512 -strip PNG32:store/play-icon-512.png
png feature 1024x500 store/feature-graphic.png
rm -r "$tmp"
