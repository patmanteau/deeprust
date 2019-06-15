#[macro_use]
extern crate indoc;
extern crate clap;
extern crate unicode_segmentation;
extern crate bitwise;
#[macro_use]
extern crate log;
extern crate simple_logging;
#[macro_use]
extern crate lazy_static;

use clap::{Arg, App};
use log::LevelFilter;

mod uci;
mod cli;
mod engine;
mod bits;

fn main() {
    simple_logging::log_to_file("test.log", LevelFilter::Info);

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
        let mut c = uci::UCIInterface::new();
        c.run();
    }
}

