#[macro_use]
extern crate indoc;
extern crate clap;
extern crate unicode_segmentation;

use clap::{Arg, App};

mod uci;
mod cli;
mod engine;

fn main() {
    let matches = App::new("deeprust")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A Rust chess engine")
        .arg(Arg::with_name("cli")
            .short("c")
            .long("cli")
            .help("Start in CLI mode instead of UCI"))
        .get_matches();
    
    if matches.is_present("cli") {
        cli::run();
    } else {
        uci::run();
    }
}

