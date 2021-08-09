// #[macro_use]
extern crate clap;
use clap::App;
use clap::Arg;

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

fn is_u64(v: String) -> Result<(), String> {
    match v.parse::<u64>() {
        Err(_) => Err(format!(
            "should be a positive integer value, but got '{}'.",
            v
        )),
        Ok(_) => Ok(()),
    }
}

fn main() {
    let deal = "deal-number";
    let matches = App::new("FreeCell Solver")
        .version("v1.0-beta")
        .about("Solves FreeCell solitaries for https://constf1.github.io/angular/freecell-demo")
        // Regular App configuration goes here...
        .arg(
            Arg::with_name(deal)
                .help("The deal number to use") // Displayed when showing help info.
                .index(1) // Set the order in which the user must specify this argument.
                .required(true) // By default this argument MUST be present.
                .validator(is_u64), // It should be a non-negative integer value.
        )
        .get_matches();

    // let deal = value_t!(matches, deal, u64).unwrap();
    let deal = matches.value_of(deal).unwrap_or("").parse().unwrap_or(0);

    println!("Deal #{}", deal);

    let mut sol = Solver::new(256);
    sol.deal(deal);

    // let (mut game, path) = loop {
    let (_, path) = loop {
        if sol.next().is_none() || {
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
