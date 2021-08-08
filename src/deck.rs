//! # Standard 52-card deck
//! A standard 52-card deck comprises 13 ranks in each of the 4 French suits:
//! spades (♠), diamonds (♦), clubs (♣) and hearts (♥).

/// 13 ranks: ace, 2, 3, 4, 5, 6, 7, 8, 9, 10, jack, queen, king.
pub const RANK_NUM: usize = 13;
/// 4 suits: spades (♠), diamonds (♦), clubs (♣) and hearts (♥).
pub const SUIT_NUM: usize = 4;
/// A standard 52-card deck.
pub const CARD_NUM: usize = RANK_NUM * SUIT_NUM;
/// An array of ranks: ['A', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K'].
/// 'A' for Ace, 'T' for 10, 'J' for Jack, 'Q' for Queen and 'K' for King.
pub const RANKS: [char; RANK_NUM] = [
    'A', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K',
];
// pub const SUITS: [char; SUIT_NUM] = ['S', 'D', 'C', 'H'];
/// An array of suits: ['♠', '♦', '♣', '♥'].
pub const SUITS: [char; SUIT_NUM] = ['♠', '♦', '♣', '♥'];

/// Returns the card index.
pub fn to_card(rank: usize, suit: usize) -> u8 {
    (rank * SUIT_NUM + suit) as u8
}

/// Returns the rank of a card.
/// Cards are ranked, from 0 to 12: A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q and K.
pub fn card_rank(card: u8) -> usize {
    (card as usize / SUIT_NUM) % RANK_NUM
}

/// Returns the suit of a card, from 0 to 3.
/// The order of suits: Spades ('♠'), Diamonds ('♦'), Clubs ('♣') and Hearts ('♥').
pub fn card_suit(card: u8) -> usize {
    card as usize % SUIT_NUM
}

/// Returns 0 for blacks (spades and clubs) and 1 for reds (diamonds and hearts).
pub fn card_color(card: u8) -> usize {
    card as usize & 1
}

/// Returns [`true`] for Spades ('♠') or Clubs ('♣'), [`false`] otherwise.
pub fn is_card_black(card: u8) -> bool {
    card_color(card) == 0
}

/// Returns [`true`] for Diamonds ('♦') or Hearts ('♥'), [`false`] otherwise.
pub fn is_card_red(card: u8) -> bool {
    !is_card_black(card)
}

/// Formats a card into a [`String`], e.g. "A♠", "2♦", "3♣", etc.
pub fn card_to_string(card: u8) -> String {
    format!("{}{}", RANKS[card_rank(card)], SUITS[card_suit(card)])
}

/// Creates a standard 52-card deck.
/// ## Examples
/// ```rust
/// # use freecell_solver::deck;
/// let cards = deck::new();
/// assert!(deck::to_string(&cards).starts_with("A♠A♦A♣A♥2♠2♦2♣2♥"));
/// ```
pub fn new() -> [u8; CARD_NUM] {
    let mut deck = [0u8; CARD_NUM];
    for i in 0..CARD_NUM {
        deck[i] = i as u8;
    }
    deck
}

/// Shuffles cards.
/// It uses [LCG algorithm](http://en.wikipedia.org/wiki/Linear_congruential_generator)
/// to pick up cards from the deck.
/// ## Examples
/// ```rust
/// # use freecell_solver::deck;
/// let mut cards = deck::new();
/// deck::shuffle(&mut cards, 1377011176);
///
/// assert!(deck::to_string(&cards).starts_with("K♦3♠4♠J♠T♥7♠K♠A♠"));
/// ```
pub fn shuffle(cards: &mut [u8], mut seed: u64) {
    let m = 0x80000000u64 as f64;
    let a = 1103515245u64 as f64;
    let c = 12345u64 as f64;
    let len = cards.len();

    for i in 0..len {
        seed = ((a * seed as f64 + c) % m).floor() as u64;

        // swap cards
        let j = (seed % len as u64) as usize;
        if i != j {
            let card = cards[i];
            cards[i] = cards[j];
            cards[j] = card;
        }
    }
}

/// Creates a standard 52-card deck and shuffles it.
pub fn deal(seed: u64) -> [u8; CARD_NUM] {
    let mut cards = new();
    shuffle(&mut cards, seed);
    cards
}

/// Formats a card array into a [`String`], e.g. "A♠2♦3♣4♥".
pub fn to_string(cards: &[u8]) -> String {
    cards.iter().map(|&c| card_to_string(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_and_suits() {
        for rank in 0..RANK_NUM {
            for suit in 0..SUIT_NUM {
                let card = to_card(rank, suit);
                assert_eq!(rank, card_rank(card));
                assert_eq!(suit, card_suit(card));
            }
        }
    }

    #[test]
    fn colors() {
        for card_a in 0..CARD_NUM as u8 {
            for card_b in 0..CARD_NUM as u8 {
                if is_card_black(card_a) {
                    assert!(!is_card_red(card_a));
                    if is_card_black(card_b) {
                        assert_eq!(card_color(card_a), card_color(card_b));
                    } else {
                        assert_ne!(card_color(card_a), card_color(card_b));
                    }
                }

                if is_card_red(card_a) {
                    assert!(!is_card_black(card_a));
                    if is_card_red(card_b) {
                        assert_eq!(card_color(card_a), card_color(card_b));
                    } else {
                        assert_ne!(card_color(card_a), card_color(card_b));
                    }
                }
            }
        }
    }

    #[test]
    fn creation() {
        let mut cards = new();
        shuffle(&mut cards, 1377011176);
        assert_eq!(
            "K♦3♠4♠J♠T♥7♠K♠A♠\
             K♥2♦2♠A♣K♣6♠2♥4♣\
             9♥Q♠2♣A♥T♦4♦A♦5♣\
             9♣4♥8♠5♦7♦3♥5♥5♠\
             Q♦3♦9♠9♦Q♣T♠3♣8♣\
             J♦7♥6♦8♥8♦T♣J♣J♥\
             Q♥6♣6♥7♣",
            to_string(&cards)
        )
    }
}
