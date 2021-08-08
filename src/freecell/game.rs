use crate::deck;
use crate::freecell::basis::*;
use crate::freecell::invariant::Key64;
use crate::util::Consumer;
use crate::util::SingleConsumer;
use crate::util::TotalConsumer;

/// Represents a step in the game, where a card is moved from a giver's position to a taker's position.
#[derive(Clone)]
pub struct Move(u8, u8);

pub type MoveConsumer = TotalConsumer<Move>;
pub type SingleMoveConsumer = SingleConsumer<Move>;
pub type Path = Vec<Move>;

type Pile = Vec<u8>;
type Desk = Vec<Pile>;

/// A freecell game object.
pub struct Game {
    desk: Desk,
    path: Path,
}

impl Move {
    pub fn new(giver: usize, taker: usize) -> Self {
        // Note: Casting down to u8 to save memory!
        Self(giver as u8, taker as u8)
    }

    pub fn giver(&self) -> usize {
        self.0 as usize
    }

    pub fn taker(&self) -> usize {
        self.1 as usize
    }
}

pub struct BaseRanks(usize, usize);

impl BaseRanks {
    pub fn new(blacks: usize, reds: usize) -> Self {
        Self(blacks, reds)
    }

    /// Tests if next rank greater than or equal to card rank.
    pub fn ge(&self, card: u8) -> bool {
        self.next_rank(card) >= deck::card_rank(card)
    }

    /// Returns opposite color rank + 1
    pub fn next_rank(&self, card: u8) -> usize {
        1 + match deck::is_card_black(card) {
            true => self.1,
            false => self.0,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            desk: desk_range().map(|_| Vec::new()).collect(),
            path: Path::new(),
        }
    }

    pub fn desk(&self) -> &Desk {
        &self.desk
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn estimate_path_len(&self) -> usize {
        self.path.len() + self.count_unsolved() + self.count_locks()
    }

    pub fn clear(&mut self) {
        self.path.clear();
        for pile in &mut self.desk {
            pile.clear();
        }
    }

    pub fn deal(&mut self, cards: &[u8]) {
        self.clear();
        for (index, card) in cards.iter().enumerate() {
            self.desk[PILE_START + index % PILE_NUM].push(*card);
        }
    }

    pub fn move_card(&mut self, giver: usize, taker: usize) {
        let c = self.desk[giver].pop().expect("empty giver");
        self.desk[taker].push(c);
        self.path.push(Move::new(giver, taker));
    }

    pub fn backward(&mut self, mark: usize) {
        while self.path.len() > mark {
            // move destination => source
            if let Some(mv) = self.path.pop() {
                let card = self.desk[mv.taker()].pop().expect("empty taker");
                self.desk[mv.giver()].push(card);
            }
        }
    }

    pub fn rewind(&mut self) {
        self.backward(0);
    }

    pub fn forward<'a, T: Iterator<Item = &'a Move>>(&mut self, iter: T) {
        for mv in iter {
            self.move_card(mv.giver(), mv.taker());
        }
    }

    pub fn set_path<'a, T: Iterator<Item = &'a Move>>(&mut self, iter: T) {
        self.rewind();
        self.forward(iter);
    }

    pub fn is_move_forward(&self, giver: usize, taker: usize) -> bool {
        match self.path.last() {
            Some(mv) => mv.giver() != taker || mv.taker() != giver,
            None => true,
        }
    }

    pub fn base_min_ranks(&self) -> BaseRanks {
        let mut black = deck::RANK_NUM;
        let mut red = deck::RANK_NUM;
        for i in base_range() {
            let rank = self.desk[i].len();
            if deck::is_card_black(i as u8) {
                black = std::cmp::min(black, rank);
            } else {
                red = std::cmp::min(red, rank);
            }
        }
        return BaseRanks::new(black, red);
    }

    pub fn move_cards_auto(&mut self) -> usize {
        let mut count = 0;
        let mut done = false;

        while !done {
            done = true;
            let ranks = self.base_min_ranks();
            for giver in play_range() {
                if let Some(&card) = self.card_at(giver) {
                    if ranks.ge(card) {
                        if let Some(taker) = self.get_base(card) {
                            count += 1;
                            self.move_card(giver, taker);
                            done = false;
                            break;
                        }
                    }
                }
            }
        }
        count
    }

    pub fn count_empty_cells(&self) -> usize {
        cell_range().filter(|&i| self.desk[i].is_empty()).count()
    }

    pub fn count_empty_piles(&self) -> usize {
        pile_range().filter(|&i| self.desk[i].is_empty()).count()
    }

    pub fn count_solved(&self) -> usize {
        base_range().map(|i| self.desk[i].len()).sum()
    }

    pub fn count_unsolved(&self) -> usize {
        play_range().map(|i| self.desk[i].len()).sum()
    }

    pub fn is_done(&self) -> bool {
        for i in play_range() {
            if self.desk[i].len() > 0 {
                return false;
            }
        }
        true
    }

    pub fn is_lock(pile: &Pile, card_index: usize) -> bool {
        let card_a = pile[card_index];
        let card_a_rank = deck::card_rank(card_a);
        let card_a_suit = deck::card_suit(card_a);

        for prev_index in 0..card_index {
            let card_b = pile[prev_index];
            if card_a_rank > deck::card_rank(card_b) && card_a_suit == deck::card_suit(card_b) {
                // Card A should be moved away to unlock card B.
                return true;
            }
        }
        false
    }

    pub fn count_locks_at(&self, index: usize) -> usize {
        let pile = &self.desk[index];
        (1..pile.len())
            .filter(|&card_index| Self::is_lock(pile, card_index))
            .count()
    }

    pub fn count_locks(&self) -> usize {
        pile_range().map(|i| self.count_locks_at(i)).sum()
    }

    pub fn count_empty(&self) -> usize {
        self.count_empty_cells() + self.count_empty_piles()
    }

    pub fn fill_base_invariant(&self, key: &mut Key64) {
        for i in base_range() {
            key.put(i, self.desk[i].len() as u8);
        }
    }

    pub fn fill_pile_invariant(&self, key: &mut Key64) {
        let mut buffer: Vec<&Pile> = Vec::new();
        for i in pile_range() {
            if !self.desk[i].is_empty() {
                buffer.push(&self.desk[i]);
            }
        }
        if !buffer.is_empty() {
            buffer.sort_unstable();

            let mut pos = BASE_NUM;
            for pile in buffer {
                key.put(pos, pile.len() as u8);
                pos += 1;
                for card in pile {
                    key.put(pos, *card);
                    pos += 1;
                }
            }
        }
    }

    pub fn get_invariant(&self) -> Key64 {
        let mut key = Key64::new();
        self.fill_base_invariant(&mut key);
        self.fill_pile_invariant(&mut key);
        key
    }

    pub fn get_base(&self, card: u8) -> Option<usize> {
        let s = deck::card_suit(card);
        let r = deck::card_rank(card);

        ((BASE_START + s)..BASE_END)
            .step_by(deck::SUIT_NUM)
            .find(|&i| self.desk[i].len() == r)
    }

    pub fn get_empty_cell(&self) -> Option<usize> {
        cell_range().find(|&i| self.desk[i].is_empty())
    }

    pub fn get_empty_pile(&self) -> Option<usize> {
        pile_range().find(|&i| self.desk[i].is_empty())
    }

    pub fn has_move_to_cell(&self) -> bool {
        SingleMoveConsumer::once(|c| self.get_moves_to_cell(c))
    }

    pub fn has_move_to_pile(&self) -> bool {
        SingleMoveConsumer::once(|c| self.get_moves_to_pile(c))
    }

    pub fn has_move_to_base(&self) -> bool {
        SingleMoveConsumer::once(|c| self.get_moves_to_base(c))
    }

    pub fn has_move_to_tableau(&self) -> bool {
        SingleMoveConsumer::once(|c| self.get_moves_to_tableau(c))
    }

    pub fn card_at(&self, spot: usize) -> Option<&u8> {
        self.desk[spot].last()
    }

    fn try_move(&self, giver: usize, taker: usize, consumer: &mut impl Consumer<Move>) -> bool {
        !self.is_move_forward(giver, taker) || consumer.accept(Move::new(giver, taker))
    }

    fn try_move_to_base(&self, giver: usize, consumer: &mut impl Consumer<Move>) -> bool {
        if let Some(&card) = self.card_at(giver) {
            if let Some(taker) = self.get_base(card) {
                return self.try_move(giver, taker, consumer);
            }
        }
        true
    }

    pub fn get_moves_to_base(&self, consumer: &mut impl Consumer<Move>) {
        // Test cells and piles:
        for giver in play_range() {
            if !self.try_move_to_base(giver, consumer) {
                return;
            }
        }
    }

    pub fn get_moves_to_cell(&self, consumer: &mut impl Consumer<Move>) {
        if let Some(taker) = self.get_empty_cell() {
            for giver in pile_range() {
                if self.desk[giver].len() > 0 && !self.try_move(giver, taker, consumer) {
                    break;
                }
            }
        }
    }

    pub fn get_moves_to_pile(&self, consumer: &mut impl Consumer<Move>) {
        if let Some(taker) = self.get_empty_pile() {
            // 1. Test piles:
            for giver in pile_range() {
                // We don't want to move the last card from one pile to another.
                if self.desk[giver].len() > 1 && !self.try_move(giver, taker, consumer) {
                    return;
                }
            }

            // 2. Test cells:
            for giver in cell_range() {
                if self.desk[giver].len() > 0 && !self.try_move(giver, taker, consumer) {
                    return;
                }
            }
        }
    }

    pub fn get_moves_to_tableau(&self, consumer: &mut impl Consumer<Move>) {
        // 1. Test cells and piles:
        for giver in play_range() {
            if let Some(&free_card) = self.card_at(giver) {
                for taker in pile_range() {
                    if let Some(&pile_card) = self.card_at(taker) {
                        if giver != taker
                            && is_tableau(pile_card, free_card)
                            && !self.try_move(giver, taker, consumer)
                        {
                            return;
                        }
                    }
                }
            }
        }

        // 2. Test bases:
        // We can take cards from bases to form a tableau only if their ranks
        // are greater than opposite color bases minimal ranks.
        let ranks = self.base_min_ranks();
        for giver in base_range() {
            if let Some(&free_card) = self.card_at(giver) {
                if !ranks.ge(free_card) {
                    for taker in pile_range() {
                        if let Some(&pile_card) = self.card_at(taker) {
                            if is_tableau(pile_card, free_card)
                                && !self.try_move(giver, taker, consumer)
                            {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_moves(&self, consumer: &mut impl Consumer<Move>) {
        self.get_moves_to_base(consumer);
        self.get_moves_to_tableau(consumer);
        self.get_moves_to_cell(consumer);
        self.get_moves_to_pile(consumer);
    }

    pub fn get_all_moves(&self) -> Vec<Move> {
        let mut consumer = MoveConsumer::new();
        self.get_moves(&mut consumer);
        consumer.into_vec()
    }

    pub fn has_next_move(&self) -> bool {
        self.has_move_to_cell()
            || self.has_move_to_pile()
            || self.has_move_to_base()
            || self.has_move_to_tableau()
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::with_capacity(100);
        s.push('|');

        for i in cell_range() {
            match self.desk[i].len() {
                0 => s.push_str("  "),
                1 => s.push_str(&deck::card_to_string(self.desk[i][0])),
                _ => s.push_str("XX"),
            }
            s.push('|');
        }

        for i in base_range() {
            match self.desk[i].len() {
                0 => s.push_str("  "),
                n => s.push_str(&deck::card_to_string(self.desk[i][n - 1])),
            }
            s.push('|');
        }

        s.push('\n');
        for _ in 0..=3 * (CELL_NUM + BASE_NUM) {
            s.push('-');
        }

        let n = pile_range().map(|i| self.desk[i].len()).max().unwrap_or(0);

        for row in 0..n {
            s.push('\n');
            s.push('|');
            for i in pile_range() {
                if self.desk[i].len() > row {
                    s.push_str(&deck::card_to_string(self.desk[i][row]))
                } else {
                    s.push_str("  ")
                }
                s.push('|');
            }
        }

        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn full_test() {
        // https://constf1.github.io/angular/freecell-demo?deal=173205951&path=4871317c7b737478653d35d53d3e39c8606c656a60e04e46e6461e1f16f6e6213ed35d535f575171f1272f262b2aead35d5e590939c94a083a395c56060a4204020205050beb6b1b1a1e12e21e1b17152b186869d9f9e96a6b2a2b6a6b2a2b4a38c818787958595a49686b28
        let mut game = Game::new();
        game.deal(&deck::deal(173205951));

        let mut key_map = HashMap::new();
        key_map.insert(game.get_invariant(), game.path().len());
        assert_eq!(1, key_map.len());
        assert!(key_map.get(&game.get_invariant()).is_some());

        assert_eq!(0, game.count_empty_piles());
        assert_eq!(0, game.count_solved());
        assert_eq!(CELL_NUM, game.count_empty_cells());
        assert_eq!(deck::CARD_NUM, game.count_unsolved());

        assert_eq!(0, game.path().len());
        let mut estimate = deck::CARD_NUM + 19;
        assert_eq!(estimate, game.estimate_path_len());

        game.move_cards_auto();
        assert_eq!(1, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert_eq!(None, key_map.get(&game.get_invariant()));
        key_map.insert(game.get_invariant(), game.path().len());
        assert_eq!(2, key_map.len());
        assert!(key_map.get(&game.get_invariant()).is_some());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 7, PILE_START + 1);
        game.move_card(PILE_START + 3, PILE_START + 1);
        game.move_card(PILE_START + 7, CELL_START + 0);
        assert_eq!(4, game.path().len());
        assert!(estimate <= game.estimate_path_len());
        assert!(key_map.get(&game.get_invariant()).is_none());

        estimate = game.estimate_path_len();
        game.move_cards_auto();
        assert_eq!(5, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 7, PILE_START + 3);
        game.move_card(PILE_START + 7, PILE_START + 4);
        assert_eq!(7, game.path().len());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        game.move_cards_auto();
        assert_eq!(8, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 6, PILE_START + 5);
        game.move_card(PILE_START + 3, CELL_START + 1);
        game.move_card(PILE_START + 3, PILE_START + 5);
        game.move_card(CELL_START + 1, PILE_START + 5);
        game.move_card(PILE_START + 3, CELL_START + 1);
        game.move_card(PILE_START + 3, CELL_START + 2);
        assert_eq!(14, game.path().len());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        game.move_cards_auto();
        assert_eq!(16, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 6, PILE_START + 0);
        game.move_card(PILE_START + 6, CELL_START + 0);
        game.move_card(PILE_START + 6, PILE_START + 5);
        assert_eq!(19, game.path().len());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        game.move_cards_auto();
        assert_eq!(20, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 6, PILE_START + 0);
        game.move_card(CELL_START + 2, PILE_START + 0);
        assert_eq!(0, game.move_cards_auto());
        assert_eq!(22, game.path().len());
        assert_eq!(4, game.count_empty());
        assert!(game.has_next_move());
        assert!(!game.has_move_to_base());
        assert!(game.has_move_to_cell());
        assert!(game.has_move_to_pile());
        assert!(game.has_move_to_tableau());
        assert!(estimate <= game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 4, CELL_START + 2);
        game.move_card(PILE_START + 4, PILE_START + 6);
        game.move_card(CELL_START + 2, PILE_START + 6);
        game.move_card(PILE_START + 4, PILE_START + 6);
        game.move_card(PILE_START + 1, CELL_START + 2);
        game.move_card(PILE_START + 1, CELL_START + 3);
        game.move_card(PILE_START + 1, PILE_START + 6);
        game.move_card(CELL_START + 3, PILE_START + 6);
        game.move_card(CELL_START + 2, PILE_START + 6);
        game.move_card(PILE_START + 2, PILE_START + 1);
        game.move_card(PILE_START + 3, CELL_START + 2);
        game.move_card(CELL_START + 1, PILE_START + 3);
        game.move_card(PILE_START + 5, CELL_START + 1);
        game.move_card(PILE_START + 5, PILE_START + 3);
        game.move_card(PILE_START + 5, CELL_START + 3);
        game.move_card(PILE_START + 5, PILE_START + 7);
        game.move_card(PILE_START + 5, PILE_START + 1);
        assert_eq!(39, game.path().len());
        assert_eq!(0, game.count_empty());
        assert!(game.has_next_move());
        assert!(!game.has_move_to_base());
        assert!(!game.has_move_to_cell());
        assert!(!game.has_move_to_pile());
        assert!(game.has_move_to_tableau());
        assert!(estimate <= game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 7, PILE_START + 1);
        assert!(game.has_move_to_pile());
        game.move_card(CELL_START + 3, PILE_START + 1);
        assert!(game.has_move_to_cell());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 2, PILE_START + 7);
        game.move_card(PILE_START + 2, CELL_START + 3);
        game.move_card(PILE_START + 2, PILE_START + 6);
        assert_eq!(44, game.path().len());
        assert!(!game.has_move_to_pile());
        assert!(!game.has_move_to_cell());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        assert!(game.has_move_to_base());
        game.move_cards_auto();
        assert_eq!(47, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(CELL_START + 1, PILE_START + 3);
        game.move_card(PILE_START + 5, CELL_START + 1);
        game.move_card(PILE_START + 5, CELL_START + 2);
        assert_eq!(50, game.path().len());
        assert!(!game.has_move_to_pile());
        assert!(!game.has_move_to_cell());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        assert!(game.has_move_to_base());
        game.move_cards_auto();
        assert_eq!(56, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        game.move_card(PILE_START + 3, BASE_START + 2);
        assert_eq!(57, game.path().len());
        assert!(estimate <= game.estimate_path_len());

        estimate = game.estimate_path_len();
        assert!(game.has_move_to_base());
        game.move_cards_auto();
        assert_eq!(58, game.path().len());
        assert_eq!(estimate, game.estimate_path_len());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 5, CELL_START + 0);
        game.move_card(PILE_START + 5, PILE_START + 6);
        game.move_card(PILE_START + 0, PILE_START + 6);
        game.move_card(PILE_START + 0, BASE_START + 2);
        game.move_card(PILE_START + 4, PILE_START + 2);
        game.move_card(PILE_START + 0, PILE_START + 4);
        game.move_card(PILE_START + 0, PILE_START + 2);
        game.move_card(PILE_START + 0, PILE_START + 2);
        game.move_card(PILE_START + 0, PILE_START + 5);
        game.move_card(PILE_START + 0, PILE_START + 5);
        assert_eq!(68, game.path().len());
        game.move_cards_auto();
        assert_eq!(73, game.path().len());
        assert_eq!(23, game.count_solved());
        assert_eq!(2, game.count_empty());
        assert_eq!(1, game.count_empty_cells());
        assert_eq!(1, game.count_empty_piles());

        assert!(game.has_next_move());
        game.move_card(PILE_START + 1, PILE_START + 0);
        game.move_card(PILE_START + 1, PILE_START + 2);
        game.move_card(PILE_START + 0, PILE_START + 2);
        game.move_card(PILE_START + 1, CELL_START + 2);
        game.move_card(PILE_START + 1, BASE_START + 3);
        game.move_card(PILE_START + 1, PILE_START + 7);
        game.move_card(PILE_START + 1, PILE_START + 5);
        assert_eq!(80, game.path().len());
        assert_eq!(108, game.estimate_path_len());
        assert!(!game.is_done());
        assert!(game.count_unsolved() > 0);
        assert_eq!(0, game.count_locks());

        game.move_cards_auto();
        assert_eq!(108, game.path().len());
        assert_eq!(108, game.estimate_path_len());
        assert!(game.is_done());
        assert_eq!(PILE_NUM, game.count_empty_piles());
        assert_eq!(CELL_NUM, game.count_empty_cells());
        assert_eq!(0, game.count_unsolved());
        assert_eq!(0, game.count_locks());

        game.rewind();
        assert_eq!(0, game.count_empty_piles());
        assert_eq!(0, game.count_solved());
        assert_eq!(CELL_NUM, game.count_empty_cells());
        assert_eq!(deck::CARD_NUM, game.count_unsolved());
        assert_eq!(0, game.path().len());
        assert!(!game.is_done());
        assert!(key_map.get(&game.get_invariant()).is_some());
    }

    #[test]
    fn invariants() {
        let mut game_a = Game::new();
        game_a.deal(&deck::deal(90));

        let mut game_b = Game::new();
        game_b.deal(&deck::deal(80));
        game_b.deal(&deck::deal(90));

        let key_a_0 = game_a.get_invariant();
        let key_b_0 = game_b.get_invariant();
        assert_eq!(key_a_0, key_b_0);
        assert_eq!(key_b_0, key_a_0);

        game_a.move_cards_auto();
        let key_a_1 = game_a.get_invariant();
        assert_ne!(key_a_1, key_b_0);
        assert_ne!(key_a_1, key_a_0);

        game_b.move_cards_auto();
        let key_b_1 = game_b.get_invariant();
        assert_eq!(key_a_1, key_b_1);
        assert_ne!(key_b_0, key_b_1);

        game_a.rewind();
        let key_a_2 = game_a.get_invariant();
        assert_eq!(key_a_0, key_a_2);
    }
}
