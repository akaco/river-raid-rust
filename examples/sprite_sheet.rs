//! Draws every sprite side by side, to look at them:
//! `cargo run --example sprite_sheet > sprites.ppm`

use delta_run::palette::{self, Colour};
use delta_run::sprites::{self, Sprite};

fn main() {
    let all: [(&Sprite, Colour); 13] = [
        (sprites::get("helicopter_1"), palette::WATER),
        (sprites::get("helicopter_2"), palette::WATER),
        (sprites::get("ship"), palette::WATER),
        (sprites::get("jet"), palette::WATER),
        (sprites::get("fuel"), palette::WATER),
        (sprites::get("house"), palette::BANK),
        (sprites::get("bridge"), palette::WATER),
        (sprites::get("plane"), palette::WATER),
        (sprites::get("plane_left"), palette::WATER),
        (sprites::get("plane_right"), palette::WATER),
        (sprites::get("explosion_1"), palette::WATER),
        (sprites::get("explosion_2"), palette::WATER),
        (sprites::get("explosion_3"), palette::WATER),
    ];
    // each pixel 2 wide and 1 tall, then everything 4 times bigger; two rows
    let (sx, sy, gap, cell_h) = (8, 4, 4, 22);
    let rows = [&all[..6], &all[6..]];
    let row_w = |row: &[(&Sprite, Colour)]| row.iter().map(|(s, _)| s.width() as usize + 2 * gap).sum::<usize>();
    let w = rows.iter().map(|r| row_w(r)).max().unwrap() * sx;
    let h = rows.len() * cell_h * sy;
    let mut img = vec![palette::STATUS; w * h];
    for (r, row) in rows.iter().enumerate() {
        let mut left = 0;
        for (sprite, background) in row.iter() {
            let cw = sprite.width() as usize + 2 * gap;
            for y in 0..cell_h {
                for x in 0..cw {
                    let colour = sprite
                        .lines
                        .get(y.wrapping_sub(2))
                        .and_then(|line| line.as_bytes().get(x.wrapping_sub(gap)))
                        .filter(|&&b| b != b'.')
                        .map_or(*background, |&b| sprite.colour(b));
                    for dy in 0..sy {
                        for dx in 0..sx {
                            img[((r * cell_h + y) * sy + dy) * w + (left + x) * sx + dx] = colour;
                        }
                    }
                }
            }
            left += cw;
        }
    }
    print!("P3\n{w} {h}\n255\n");
    for c in img {
        println!("{} {} {}", c >> 16, (c >> 8) & 0xFF, c & 0xFF);
    }
}
