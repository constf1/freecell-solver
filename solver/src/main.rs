// #[macro_use]
extern crate clap;
use clap::App;
use clap::Arg;
use std::str::FromStr;

// use freecell_solver::deck;
// use freecell_solver::freecell::{spot_name, spot_to_hex, Game, Path, Solver};
use freecell_solver::freecell::{spot_to_hex, Path, Solver};

fn print_link(deal: u64, path: &Path) {
    let mut buf = String::with_capacity(path.len() * 2);
    for mv in path {
        buf.push_str(&spot_to_hex(mv.giver()));
        buf.push_str(&spot_to_hex(mv.taker()));
    }

    println!(
        "https://constf1.github.io/angular/freecell-demo?deal={}&path={}",
        deal, buf
    );
}

// fn print_game(game: &mut Game, path: &Path) {
//     game.rewind();
//     println!("\n{}", game);

//     for (i, mv) in path.iter().enumerate() {
//         let giver = mv.giver();
//         let taker = mv.taker();

//         println!(
//             "\n{}. {}: {} -> {}",
//             i + 1,
//             deck::card_to_string(*game.card_at(giver).expect("Giver should exist")),
//             spot_name(giver),
//             spot_name(taker)
//         );

//         game.move_card(giver, taker);
//         println!("{}", game);
//     }
// }
// struct Config {
//     pub deal: u64,
// }

fn is_unsigned<T: FromStr>(v: String) -> Result<(), String> {
    match v.parse::<T>() {
        Err(_) => Err(format!(
            "should be a non-negative integer value, but got '{}'.",
            v
        )),
        Ok(_) => Ok(()),
    }
}

const DEFAULT_DEAL: u64 = 0;
const DEFAULT_PATH_MAX: usize = 256;
const DEFAULT_GRAB_MAX: usize = 1000;

fn main() {
    let deal = "deal";
    let path_max = "path-max";
    let grab_max = "grab-max";

    let matches = App::new("FreeCell Solver")
        .version("v1.0-beta")
        .about("Solves FreeCell solitaries for [https://constf1.github.io/angular/freecell-demo]")
        // Regular App configuration goes here...
        .arg(
            Arg::with_name(deal)
                .help("The deal number to use") // Displayed when showing help info.
                .index(1) // Set the order in which the user must specify this argument.
                .required(true) // By default this argument MUST be present.
                .value_name("NUMBER")
                .validator(is_unsigned::<u64>), // It should be a non-negative integer value.
        )
        .arg(
            Arg::with_name(path_max)
                .help("The upper bound of the search range (inclusive)")
                .short("P")
                .long("path-max")
                .required(false)
                .takes_value(true)
                .value_name("NUMBER")
                .validator(is_unsigned::<usize>),
        )
        .arg(
            Arg::with_name(grab_max)
                .help("The maximum number of variants to be processed in one iteration")
                .short("G")
                .long("grab-max")
                .required(false)
                .takes_value(true)
                .value_name("NUMBER")
                .validator(is_unsigned::<usize>),
        )
        .get_matches();

    let deal = matches
        .value_of(deal)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_DEAL);
    let path_max = 1 + matches
        .value_of(path_max)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_PATH_MAX);
    let grab_max = matches
        .value_of(grab_max)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_GRAB_MAX)
        .max(1); // At least one path should be processed.

    println!("Deal #{}", deal);

    let mut sol = Solver::new();
    sol.deal(deal);

    // let (mut game, path) = loop {
    let (_, path) = loop {
        if sol.next(path_max, grab_max).is_none() || {
            if sol.done().len() > 10_000_000 {
                println!(
                    "Done: {}, {} still in process, but we're over the limit!",
                    sol.done().len(),
                    sol.bank().len()
                );
                true
            } else {
                false
            }
        } {
            break sol.into_solution();
        }
    };

    if let Some(path) = path {
        print_link(deal, &path);
        // print_game(&mut game, &path);
    }
}
