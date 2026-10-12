//! Pictures: everything is drawn from these small pixel-art images.
//!
//! Each letter in a picture is one pixel, in the colour the sprite's
//! `colours` list gives it; `.` is see-through. Pixels are twice as wide as
//! they are tall, so the pictures look squashed in the source. River
//! objects face right and say on which line of their 32-line band they
//! start.

use crate::palette::*;

pub struct Sprite {
    /// First line, counted from the top of its river band (or of the plane).
    pub first_line: u8,
    /// What each letter in the picture stands for.
    pub colours: &'static [(char, Colour)],
    pub lines: &'static [&'static str],
}

impl Sprite {
    /// Width in pixels.
    pub fn width(&self) -> u8 {
        self.lines[0].len() as u8
    }

    /// The colour of a letter in the picture.
    pub fn colour(&self, letter: u8) -> Colour {
        self.colours.iter().find(|(c, _)| *c as u8 == letter).map_or(BLACK, |&(_, colour)| colour)
    }
}

const BLAST: &[(char, Colour)] = &[('F', FIRE), ('S', SPARK), ('W', FLARE), ('K', SMOKE)];

pub const EXPLOSION_1: Sprite = Sprite {
    first_line: 12,
    colours: BLAST,
    lines: &[".......SS.......", ".....SFFFFS.....", "....SFFWWFFS....", ".....SFFFFS.....", ".......SS......."],
};

pub const EXPLOSION_2: Sprite = Sprite {
    first_line: 10,
    colours: BLAST,
    lines: &[
        "..S.....S.....S.",
        "....F..FF..F....",
        "..F..FFSSFF..F..",
        "S...FSSWWSSF...S",
        "...FFSWWWWSFF...",
        "S...FSSWWSSF...S",
        "..F..FFSSFF..F..",
        "....F..FF..F....",
        ".S.....S.....S..",
    ],
};

pub const EXPLOSION_3: Sprite = Sprite {
    first_line: 10,
    colours: BLAST,
    lines: &[
        "...K.......K....",
        "......K..K......",
        ".K....KKKK....K.",
        "....KKK..KKK....",
        ".K....KKKK....K.",
        "......K..K......",
        "...K.......K....",
    ],
};

/// Enemy jet, side on: tail fin, canopy, pointed nose, swept wing, exhaust.
pub const JET: Sprite = Sprite {
    first_line: 12,
    colours: &[('B', STEEL), ('D', STEEL_DARK), ('G', CANOPY), ('F', FIRE)],
    lines: &[
        "DD..............",
        "DDD.............",
        "DDDD.....GGG....",
        ".DBBBBBBBBGGBB..",
        "FBBBBBBBBBBBBBBB",
        ".DBBBBDDDDBBBB..",
        ".....DDDDD......",
        "....DDDD........",
    ],
};

const HELICOPTER_COLOURS: &[(char, Colour)] = &[('R', ROTOR), ('M', SKID), ('B', OLIVE), ('D', OLIVE_DARK), ('G', CANOPY), ('S', SKID)];

/// Helicopter, side on: main rotor, tail boom and rotor, glass nose,
/// skids. Two frames so the rotors spin.
pub const HELICOPTER_1: Sprite = Sprite {
    first_line: 11,
    colours: HELICOPTER_COLOURS,
    lines: &[
        "RRRRRRRRRRRRRRR.",
        "........M.......",
        "......BBBBBBB...",
        "RD...BBBBBBGGG..",
        "RDDBBBBBBBBGGGG.",
        "R....DBBBBBBBBB.",
        "......DDDDDDDD..",
        ".......S....S...",
        ".....SSSSSSSSSS.",
    ],
};

pub const HELICOPTER_2: Sprite = Sprite {
    first_line: 11,
    colours: HELICOPTER_COLOURS,
    lines: &[
        "....RRRRRRRRR...",
        "........M.......",
        "......BBBBBBB...",
        ".D...BBBBBBGGG..",
        "RDDBBBBBBBBGGGG.",
        ".....DBBBBBBBBB.",
        "......DDDDDDDD..",
        ".......S....S...",
        ".....SSSSSSSSSS.",
    ],
};

/// Patrol boat, side on: mast, cabin with windows, funnel, grey hull with
/// a red waterline, bow to the right.
pub const SHIP: Sprite = Sprite {
    first_line: 12,
    colours: &[('M', SKID), ('W', CABIN), ('G', WINDOW), ('F', FUNNEL), ('H', HULL), ('K', WATERLINE)],
    lines: &[
        "........M...............",
        "........M.......FF......",
        "....WWWWWWWWW...FF......",
        "....WGGWGGWGGW..FF......",
        "...WWWWWWWWWWWWWWWWW....",
        "HHHHHHHHHHHHHHHHHHHHHHHH",
        "HHHHHHHHHHHHHHHHHHHHHH..",
        ".KKKKKKKKKKKKKKKKKKKK...",
    ],
};

/// Bridge, seen from above: the road carried across the river between two
/// red railings, with its shadow on the water.
pub const BRIDGE: Sprite = Sprite {
    first_line: 8,
    colours: &[('E', RAIL), ('P', RAIL_POST), ('A', ROAD), ('Y', ROAD_LINE), ('S', SHADOW)],
    lines: &[
        "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
        "PEEEPEEEPEEEPEEEPEEEPEEEPEEEPEEE",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY",
        "YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "PEEEPEEEPEEEPEEEPEEEPEEEPEEEPEEE",
        "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
        "SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS",
        "SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS",
    ],
};

/// A house on the bank with a tree beside it: roof, chimney, two windows
/// and a door; a round tree on a trunk.
pub const HOUSE: Sprite = Sprite {
    first_line: 10,
    colours: &[
        ('R', ROOF),
        ('C', CHIMNEY),
        ('W', WALL),
        ('G', WINDOW_LIT),
        ('D', DOOR),
        ('T', LEAVES),
        ('L', LEAVES_LIGHT),
        ('K', LEAVES_DARK),
        ('B', TRUNK),
    ],
    lines: &[
        "...................TTT..",
        "..................TLLTT.",
        ".......RR...CC...TTLTTTT",
        ".....RRRRRR.CC...TTTTTTT",
        "...RRRRRRRRRRCC..TTTTTKT",
        ".RRRRRRRRRRRRRR..TTTTKKT",
        "RRRRRRRRRRRRRRRR..TKKKT.",
        ".WWWWWWWWWWWWWW.....B...",
        ".WGGWWWDDWWWGGW.....B...",
        ".WGGWWWDDWWWGGW.....B...",
        ".WWWWWWDDWWWWWW....BBB..",
    ],
};

/// Fuel: a red jerrycan with a yellow cap. Fly over it to refuel, or shoot
/// it for points.
pub const FUEL_TANK: Sprite = Sprite {
    first_line: 10,
    colours: &[('R', CAN), ('L', CAN_LIGHT), ('H', CAN_DARK), ('C', CAN_CAP)],
    lines: &[
        ".HHHHH..CC",
        ".H...H..CC",
        "RRRRRRRRRR",
        "RLRRRRRRLR",
        "RRLRRRRLRR",
        "RRRLRRLRRR",
        "RRRRLLRRRR",
        "RRRLRRLRRR",
        "RRLRRRRLRR",
        "RLRRRRRRLR",
        "RRRRRRRRRR",
        ".RRRRRRRR.",
    ],
};

/// The player's plane is drawn from this line of the screen down.
pub const PLANE_TOP: u8 = 182;

/// (The plane is drawn in its player's colour.)
const PLANE_COLOURS: &[(char, Colour)] = &[('#', PLAYER_1)];

pub const PLANE: Sprite = Sprite {
    first_line: 0,
    colours: PLANE_COLOURS,
    lines: &[
        "...#....", "...#....", "..###...", "..###...", ".#####..", "#######.", "##.#.##.", "...#....", "...#....", "..###...", ".##.##..",
    ],
};

pub const PLANE_RIGHT: Sprite = Sprite {
    first_line: 0,
    colours: PLANE_COLOURS,
    lines: &[
        "...#....", "...#....", "..###...", "..###...", "..####..", ".#####..", "..#.##..", "...#....", "...#....", "..###...", "...##...",
    ],
};

pub const PLANE_LEFT: Sprite = Sprite {
    first_line: 0,
    colours: PLANE_COLOURS,
    lines: &[
        "...#....", "...#....", "..###...", "..###...", ".####...", ".#####..", ".##.#...", "...#....", "...#....", "..###...", "..##....",
    ],
};

pub const PLANE_CRASHED: Sprite = Sprite {
    first_line: 1,
    colours: PLANE_COLOURS,
    lines: &["#.....#.", "..#.#...", ".#...#..", "...#....", "#.#.#.#.", "...#....", ".#...#..", "..#.#...", "#.....#."],
};

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
