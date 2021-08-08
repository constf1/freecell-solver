//! # FreeCell - a solitaire card game
//! ## FreeCell Rules
//! Source: [Wikipedia](https://en.wikipedia.org/wiki/FreeCell#Rules)
//! ### Construction and layout
//! - One standard 52-card deck is used.
//! - There are four open *cells* and four open *foundations*.
//! - Cards are dealt face-up into eight *cascades*, four of which comprise seven cards each and four of which comprise six cards each.
//! ### Building during play
//! - The top card of each *cascade* begins a *tableau*.
//! - *Tableaux* must be built down by alternating colors.
//! - *Foundations* are built up by suit.
//! ### Moves
//! - Any top card may be moved to build on a tableau, or moved to an empty cell, an empty cascade, or its foundation.
//! ### Victory
//! - The game is won after all cards are moved to their foundation piles.
mod basis;
mod game;
mod invariant;

pub use basis::*;
pub use game::*;
pub use invariant::*;
