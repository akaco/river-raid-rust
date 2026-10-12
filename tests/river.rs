//! Checks on the river generator over many sections.

use delta_run::objects::Kind;
use delta_run::river::{BANDS, Band, River};
use delta_run::screen::row_of;

/// Water spans (start, end) on a land row.
fn water(band: &Band, row: u8) -> Vec<(u8, u8)> {
    let mut spans = vec![];
    let mut start = None;
    for x in 0..=160u16 {
        let wet = x < 160 && !band.land.is_land(row, x as u8);
        match (wet, start) {
            (true, None) => start = Some(x as u8),
            (false, Some(s)) => {
                spans.push((s, x as u8));
                start = None;
            }
            _ => {}
        }
    }
    spans
}

/// Plays the river forward band by band, calling `check` on each new top band.
fn each_new_band(sections: u32, mut check: impl FnMut(&River, &Band)) {
    let mut river = River::new();
    river.restart_at_checkpoint();
    let mut made = 0;
    while (river.section as u32) < sections {
        river.scroll(0xFE);
        if river.scroll_line < 2 {
            made += 1;
            check(&river, &river.bands[BANDS - 1]);
        }
        assert!(made < 100_000, "the river stopped making sections");
    }
}

#[test]
fn the_plane_can_always_get_through() {
    let mut last_row: Option<Vec<(u8, u8)>> = None;
    each_new_band(60, |_, band| {
        for row in 0..16 {
            let spans = water(band, row);
            assert!(spans.iter().any(|(a, b)| b - a >= 16), "no channel wide enough: {band:?} row {row}");
            if let Some(prev) = &last_row {
                // some channel continues from the row below
                let joined = spans.iter().any(|&(a, b)| prev.iter().any(|&(c, d)| a.max(c) + 10 <= b.min(d)));
                assert!(joined, "channel cut off: {band:?} row {row}");
            }
            last_row = Some(spans);
        }
    });
}

#[test]
fn objects_start_in_the_right_place() {
    let mut counts = [0u32; 8];
    each_new_band(60, |_, band| {
        let o = band.object;
        let Some(sprite) = o.kind.sprite() else { return };
        counts[match o.kind {
            Kind::Jet => 0,
            Kind::Helicopter(_) => 1,
            Kind::Ship => 2,
            Kind::FuelDepot => 3,
            Kind::House => 4,
            Kind::Bridge => 5,
            _ => 6,
        }] += 1;
        for (i, art) in sprite.lines.iter().enumerate() {
            let row = row_of(sprite.first_line + i as u8);
            for (x, _) in o.pixels(art) {
                let land = band.land.is_land(row, x);
                match o.kind {
                    Kind::House => assert!(land, "house in the water: {band:?}"),
                    Kind::Ship | Kind::Helicopter(_) | Kind::FuelDepot => assert!(!land, "{:?} on land: {band:?}", o.kind),
                    _ => {}
                }
            }
        }
    });
    // every kind turns up
    assert!(counts[..6].iter().all(|&n| n > 0), "{counts:?}");
}

#[test]
fn every_game_has_the_same_river() {
    let mut a = vec![];
    let mut b = vec![];
    each_new_band(10, |_, band| a.push(*band));
    each_new_band(10, |_, band| b.push(*band));
    assert_eq!(a, b);
}
