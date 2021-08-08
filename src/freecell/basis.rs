//! # FreeCell layout

use crate::deck;
use core::ops::Range;

/// There are 4 open *foundations*.
pub const BASE_NUM: usize = 4; // foundation piles
/// There are 4 open *freecells*.
pub const CELL_NUM: usize = 4; // open cells
/// Cards are dealt face-up into 8 *cascades*.
pub const PILE_NUM: usize = 8; // cascades
/// Desk size.
pub const DESK_SIZE: usize = PILE_NUM + CELL_NUM + BASE_NUM;

pub const BASE_START: usize = 0;
pub const BASE_END: usize = BASE_START + BASE_NUM;

pub const CELL_START: usize = BASE_END;
pub const CELL_END: usize = CELL_START + CELL_NUM;

pub const PILE_START: usize = CELL_END;
pub const PILE_END: usize = PILE_START + PILE_NUM;

pub fn desk_range() -> Range<usize> {
    0..DESK_SIZE
}

pub fn play_range() -> Range<usize> {
    BASE_END..DESK_SIZE
}

pub fn pile_range() -> Range<usize> {
    PILE_START..PILE_END
}

pub fn base_range() -> Range<usize> {
    BASE_START..BASE_END
}

pub fn cell_range() -> Range<usize> {
    CELL_START..CELL_END
}

pub fn is_play(index: usize) -> bool {
    play_range().contains(&index)
}

pub fn is_pile(index: usize) -> bool {
    pile_range().contains(&index)
}

pub fn is_base(index: usize) -> bool {
    base_range().contains(&index)
}

pub fn is_cell(index: usize) -> bool {
    cell_range().contains(&index)
}

pub fn spot_name(index: usize) -> String {
    if is_base(index) {
        return format!("base {}", 1 + index - BASE_START);
    }
    if is_pile(index) {
        return format!("pile {}", 1 + index - PILE_START);
    }
    if is_cell(index) {
        return format!("cell {}", 1 + index - CELL_START);
    }
    format!("unknown {}", index)
}

pub fn spot_to_hex(mut index: usize) -> String {
    if is_pile(index) {
        index = index - PILE_START;
    } else if is_base(index) {
        index = index - BASE_START + PILE_NUM;
    } else if is_cell(index) {
        index = index - CELL_START + PILE_NUM + BASE_NUM;
    }

    format!("{:x}", index)
}

/// Returns [`true`] if cards can form a tableau.
/// Tableaux must be built down by alternating colors.
pub fn is_tableau(card_a: u8, card_b: u8) -> bool {
    deck::card_rank(card_a) == deck::card_rank(card_b) + 1
        && deck::card_color(card_a) != deck::card_color(card_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        for spot in play_range() {
            assert!(is_cell(spot) || is_pile(spot));
            assert!(!is_base(spot));
        }
        for spot in base_range() {
            assert!(!is_play(spot));
        }
    }

    #[test]
    fn spots() {
        for spot in 0..DESK_SIZE {
            if is_base(spot) {
                assert!(!is_cell(spot));
                assert!(!is_pile(spot));
            }
            if is_cell(spot) {
                assert!(!is_base(spot));
                assert!(!is_pile(spot));
            }
            if is_pile(spot) {
                assert!(!is_base(spot));
                assert!(!is_cell(spot));
            }
        }
    }

    #[test]
    fn tableaux() {
        let a = deck::to_card(1, 2);
        let b = deck::to_card(2, 2);
        let c = deck::to_card(2, 3);

        assert!(!is_tableau(a, b));
        assert!(!is_tableau(b, a));

        assert!(!is_tableau(b, c));
        assert!(!is_tableau(c, b));

        assert!(!is_tableau(a, c));
        assert!(is_tableau(c, a));
    }
}
