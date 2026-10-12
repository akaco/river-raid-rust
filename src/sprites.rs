//! Pictures. Every picture is in `assets/sprites.txt`, as letters (one per
//! pixel) with the colour of each letter; see the top of that file. The
//! file is built into the game, so changing a picture needs no code.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::palette::{BLACK, Colour};

pub struct Sprite {
    /// First line, counted from the top of its river band (or of the plane).
    pub first_line: u8,
    /// What each letter in the picture stands for.
    pub colours: Vec<(u8, Colour)>,
    pub lines: Vec<String>,
}

impl Sprite {
    /// Width in pixels.
    pub fn width(&self) -> u8 {
        self.lines[0].len() as u8
    }

    /// The colour of a letter in the picture.
    pub fn colour(&self, letter: u8) -> Colour {
        self.colours.iter().find(|(c, _)| *c == letter).map_or(BLACK, |&(_, colour)| colour)
    }
}

/// The player's plane is drawn from this line of the screen down.
pub const PLANE_TOP: u8 = 182;

static PICTURES: LazyLock<HashMap<String, Sprite>> = LazyLock::new(|| parse(include_str!("../assets/sprites.txt")));

/// The picture called `name` in `assets/sprites.txt`.
pub fn get(name: &str) -> &'static Sprite {
    PICTURES.get(name).unwrap_or_else(|| panic!("no picture called [{name}] in assets/sprites.txt"))
}

/// Reads the pictures file. Mistakes in it stop the game with a message
/// saying where.
pub fn parse(text: &str) -> HashMap<String, Sprite> {
    let mut pictures = HashMap::new();
    let mut current: Option<(String, Sprite)> = None;
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        let fail = |what: &str| -> ! { panic!("assets/sprites.txt line {}: {what}: {line}", n + 1) };
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            pictures.extend(current.take());
            current = Some((name.to_string(), Sprite { first_line: 0, colours: vec![], lines: vec![] }));
            continue;
        }
        let Some((_, sprite)) = current.as_mut() else { fail("expected a [name] first") };
        if let Some((key, value)) = line.split_once('=') {
            let (key, value) = (key.trim(), value.trim());
            if key == "first_line" {
                sprite.first_line = value.parse().unwrap_or_else(|_| fail("not a number"));
            } else if key.len() == 1 && key != "." {
                let colour = Colour::from_str_radix(value, 16).unwrap_or_else(|_| fail("colour should be RRGGBB"));
                sprite.colours.push((key.as_bytes()[0], colour));
            } else {
                fail("expected first_line = N or X = RRGGBB");
            }
        } else {
            if sprite.lines.first().is_some_and(|first| first.len() != line.len()) {
                fail("this row is not as wide as the first");
            }
            if let Some(b) = line.bytes().find(|&b| b != b'.' && !sprite.colours.iter().any(|(c, _)| *c == b)) {
                fail(&format!("letter '{}' has no colour", b as char));
            }
            sprite.lines.push(line.to_string());
        }
    }
    pictures.extend(current);
    pictures
}

/// A 5 x 7 pixel font for the score and the status bar.
pub fn glyph(c: char) -> &'static [&'static str; 7] {
    match c {
        '0' => &[".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
        '1' => &["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
        '2' => &[".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
        '3' => &["#####", "...#.", "..#..", "...#.", "....#", "#...#", ".###."],
        '4' => &["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
        '5' => &["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
        '6' => &["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."],
        '7' => &["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
        '8' => &[".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
        '9' => &[".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
        '!' => &["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."],
        'A' => &[".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'B' => &["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
        'C' => &[".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."],
        'D' => &["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
        'E' => &["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
        'F' => &["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
        'G' => &[".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####"],
        'H' => &["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'I' => &[".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."],
        'J' => &["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
        'K' => &["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
        'L' => &["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
        'M' => &["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"],
        'N' => &["#...#", "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#"],
        'O' => &[".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'P' => &["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
        'Q' => &[".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
        'R' => &["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
        'S' => &[".####", "#....", "#....", ".###.", "....#", "....#", "####."],
        'T' => &["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
        'U' => &["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'V' => &["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
        'W' => &["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#."],
        'X' => &["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
        'Y' => &["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
        'Z' => &["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
        ' ' => &[".....", ".....", ".....", ".....", ".....", ".....", "....."],
        _ => &[".....", ".....", ".....", ".....", ".....", ".....", "....."],
    }
}
