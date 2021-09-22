#[macro_use]
extern crate clap;
use clap::{App, Arg};
use std::str::FromStr;

use freecell_solver::deck;
use freecell_solver::freecell::{spot_name, spot_to_hex, Game, Path, Solver};

fn print_link(deal: u64, path: &Path) {
    let mut buf = String::with_capacity(path.len() * 2);
    for mv in path {
        buf.push_str(&spot_to_hex(mv.giver()));
        buf.push_str(&spot_to_hex(mv.taker()));
    }

    println!(
        "https://constf1.github.io/angular/freecell-demo?deal={}&path={}\n",
        deal, buf
    );
}

fn print_path(game: &mut Game, path: &Path) {
    game.rewind();
    // println!("\n{}", game);

    for (i, mv) in path.iter().enumerate() {
        let giver = mv.giver();
        let taker = mv.taker();

        println!(
            "{}. {}: {} -> {}",
            i + 1,
            deck::card_to_string(*game.card_at(giver).expect("Giver should exist")),
            spot_name(giver),
            spot_name(taker)
        );

        game.move_card(giver, taker);
        // println!("{}", game);
    }
}

fn is_unsigned<T: FromStr>(v: String) -> Result<(), String> {
    match v.parse::<T>() {
        Err(_) => Err(format!(
            "should be a non-negative integer value, but got '{}'.",
            v
        )),
        Ok(_) => Ok(()),
    }
}

pub struct DefaultParam<T> {
    value: T,
    name: &'static str,
}

macro_rules! define_param {
    ( $name:ident : $t:ty = $val:expr ) => {
        pub const $name: DefaultParam<$t> = DefaultParam {
            value: $val,
            name: stringify!($val),
        };
    };
}

define_param!(DEAL: u64 = 0);
define_param!(PATH_MAX: usize = 256);
define_param!(GRAB_MAX: usize = 1000);
define_param!(DONE_MAX: usize = 10000000);

fn main() {
    let deal = "deal";
    let path_max = "path-max";
    let grab_max = "grab-max";
    let done_max = "done-max";
    let verbose = "verbose";
    let any = "any";

    let matches = App::new("FreeCell Solver")
        .version(crate_version!())
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
                .long("path")
                .required(false)
                .takes_value(true)
                .default_value(PATH_MAX.name)
                .value_name("NUMBER")
                .validator(is_unsigned::<usize>),
        )
        .arg(
            Arg::with_name(grab_max)
                .help("The maximum number of variants to be processed in one iteration")
                .short("S")
                .long("scoop")
                .required(false)
                .takes_value(true)
                .default_value(GRAB_MAX.name)
                .value_name("NUMBER")
                .validator(is_unsigned::<usize>),
        )
        .arg(
            Arg::with_name(done_max)
                .help("The maximum number of variants to be processed in total")
                .short("L")
                .long("limit")
                .required(false)
                .takes_value(true)
                .default_value(DONE_MAX.name)
                .value_name("NUMBER")
                .validator(is_unsigned::<usize>),
        )
        .arg(
            Arg::with_name(verbose)
                .help("Use debug output")
                .short("D")
                .long("debug")
                .alias("verbose")
                .required(false),
        )
        .arg(
            Arg::with_name(any)
                .help("Stop on the first result")
                .short("A")
                .long("any")
                .required(false),
        )
        .get_matches();

    let deal = matches
        .value_of(deal)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEAL.value);
    let path_max = 1 + matches
        .value_of(path_max)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(PATH_MAX.value);
    let grab_max = matches
        .value_of(grab_max)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(GRAB_MAX.value)
        .max(1); // At least one path should be processed.
    let done_max = matches
        .value_of(done_max)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DONE_MAX.value)
        .max(1000); // At least one thousand paths should be processed.
    let verbose = matches.is_present(verbose);
    let any = matches.is_present(any);

    let mut sol = Solver::new();
    sol.deal(deal);
    let (mut game, path) = loop {
        let mut stop = true;

        if let Some(found) = sol.next(path_max, grab_max, verbose) {
            if found {
                if let Some(path) = &sol.path() {
                    println!("Path ({}):", path.len());
                    print_link(deal, path);
                }
            }

            if !(any && found) {
                stop = sol.done().len() > done_max;
                if stop && verbose {
                    println!(
                        "Done: {}, {} still in process, but we're over the limit!\n",
                        sol.done().len(),
                        sol.bank().len()
                    );
                }
            }
        };

        if stop {
            break sol.into_solution();
        }
    };

    if verbose {
        game.rewind();
        println!("Deal #{}", deal);
        println!("{}\n", game);

        if let Some(path) = path {
            println!("Solution:");
            print_path(&mut game, &path);
        } else {
            println!("Solution not found!");
        }
    }
}
