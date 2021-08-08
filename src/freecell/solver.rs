use crate::deck;
use crate::freecell::game::{Game, Path};
use crate::freecell::invariant::Key64;
use crate::util::Grader;
use std::collections::HashMap;

pub fn game_priority(game: &Game) -> usize {
    10 * game.count_unsolved() + 9 * game.count_locks()
}

type Bank = Grader<usize, Path>;
type Done = HashMap<Key64, usize>;

pub struct Solver {
    bank: Bank,
    done: Done,
    game: Game,
    path: Option<Path>,
}

const INPUT_LIMIT: usize = 1000;

impl Solver {
    pub fn new(seed: u64) -> Self {
        let mut game = Game::new();
        game.deal(&deck::deal(seed));
        game.move_cards_auto();

        let mut bank: Grader<usize, Path> = Grader::new();
        let grade = game_priority(&game);
        bank.add(grade, game.path().clone());

        let mut done: HashMap<Key64, usize> = HashMap::new();
        done.insert(game.get_invariant(), game.path().len());

        Self {
            bank,
            done,
            game,
            path: None,
        }
    }

    pub fn bank(&self) -> &Bank {
        &self.bank
    }

    pub fn done(&self) -> &Done {
        &self.done
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }

    pub fn into_solution(self) -> (Game, Option<Path>) {
        (self.game, self.path)
    }

    pub fn next(&mut self) -> Option<()> {
        let grade = *self.bank.grades().next()?;
        let mut input = self.bank.split_off(grade, INPUT_LIMIT)?;

        while let Some(path) = input.pop() {
            self.game.set_path(path.iter());

            let mark = path.len();
            debug_assert_eq!(mark, self.game.path().len());

            #[cfg(debug)]
            let mold = self.game.get_invariant();

            let moves = self.game.get_all_moves();
            debug_assert!(moves.len() > 0);

            for mv in moves {
                self.game.backward(mark);
                debug_assert_eq!(mark, self.game.path().len());

                #[cfg(debug)]
                debug_assert_eq!(mold, self.game.get_invariant());

                self.game.move_card(mv.giver(), mv.taker());
                self.game.move_cards_auto();

                let estm_len = self.game.estimate_path_len();

                // Skip over long solutons.
                if let Some(sol) = &self.path {
                    if estm_len >= sol.len() {
                        continue;
                    }
                }

                // State Analysis.
                if self.game.is_done() {
                    // Solved!
                    self.path = Some(self.game.path().clone());

                    let sol_len = self.game.path().len();
                    println!("Solved! Path of {}", sol_len);

                    // Drain out our input.
                    while let Some(path) = input.pop() {
                        self.bank.add(grade, path);
                    }

                    // Cleaning. Get rid of long paths.
                    println!("Cleaning:");
                    let old_len = self.bank.len();
                    self.bank.retain(|_, row| {
                        row.retain(|path| path.len() + 1 < sol_len);
                        row.len() > 0
                    });
                    println!(
                        "    bank: {}; {} -> {}",
                        old_len - self.bank.len(),
                        old_len,
                        self.bank.len()
                    );

                    let old_len = self.done.len();
                    self.done.retain(|_, len| *len < sol_len);
                    println!(
                        "    done: {}; {} -> {}",
                        old_len - self.done.len(),
                        old_len,
                        self.done.len()
                    );

                    // Not intrested in other moves anymore.
                    break;
                } else if self.game.has_next_move() {
                    // Not solved yet.
                    let key = self.game.get_invariant();
                    if match self.done.get(&key) {
                        None => true,
                        Some(&min_len) => estm_len < min_len,
                    } {
                        // Keep this path.
                        self.done.insert(key, estm_len);
                        let grade = game_priority(&self.game);
                        self.bank.add(grade, self.game.path().clone());
                    }
                }
            }
        }

        Some(())
    }
}
