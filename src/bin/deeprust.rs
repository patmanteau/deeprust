// #![warn(clippy::all)]
// #![feature(test)]
// #![feature(simd_x86_bittest)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate simple_logging;

extern crate deeprust;

// pub mod bitboard;
// pub mod board;
// pub mod cli;
// pub mod color;
// pub mod common;
// pub mod interfaces;
// pub mod move_generator;
// pub mod moves;
// pub mod piece;
// pub mod position;
// pub mod search;
// pub mod square;
// pub mod uci;

// use clap::{App, Arg};
use deeprust::board::Board;
use deeprust::interfaces::FenInterface;
use deeprust::search::Search;

use ansi_term::Colour::{Cyan, Green, Red};
use log::LevelFilter;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
// use std::str::FromStr;

fn batchperft(f: &str) {
    let path = PathBuf::from(f);
    let file = match File::open(&path) {
        Err(why) => panic!("Could not open {}: {}", path.display(), why.description()),
        Ok(file) => file,
    };

    println!("Running batch perft from {}", path.to_str().unwrap());
    debug!("Running batch perft from {}", path.to_str().unwrap());

    let (mut num_ok, mut num_error) = (0, 0);

    for (num, line) in BufReader::new(file).lines().map(|l| l.unwrap()).enumerate() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        } else {
            let parts: Vec<&str> = line.split(';').map(|l| l.trim()).collect();

            let fen_str = parts[0];
            let mut b = Board::from_fen_str(&fen_str).unwrap();

            let mut ok = true;

            print!("perft({}) ", fen_str);
            for depth_def in &parts[1..] {
                let depth_def_parts: Vec<u64> = depth_def
                    .split_whitespace()
                    .map(|d| u64::from_str_radix(d, 10).unwrap())
                    .collect();
                let (depth, nodes) = (depth_def_parts[0], depth_def_parts[1]);
                // print!("Running perft {} on '{}' expecting {} nodes...", depth, fen_str, nodes);
                // print!("{}..", depth);
                // io::stdout().flush().unwrap();
                let res = b.perft(depth as u32);
                if res.nodes == nodes {
                    // println!("OK");
                } else {
                    ok = false;
                    // println!("Line {} failed at depth {}: got {} instead of {} nodes", num + 1, depth, res.nodes, nodes);
                    debug!(
                        "Line {} failed at depth {}: got {} instead of {} nodes",
                        num + 1,
                        depth,
                        res.nodes,
                        nodes
                    );
                }
            }

            if ok {
                println!("{}", Green.bold().paint("[OK]"));
                num_ok += 1;
            } else {
                println!("{}", Red.bold().paint("[FAIL]"));
                num_error += 1;
            }
            io::stdout().flush().unwrap();
        }
    }
    println!();
    println!(
        "Finished batch perft: {} passed, {} failed.",
        num_ok, num_error
    );
    debug!(
        "Finished batch perft: {} passed, {} failed.",
        num_ok, num_error
    );
}

fn main() {
    simple_logging::log_to_file("deeprust.log", LevelFilter::Debug).unwrap();

    // let matches = App::new("deeprust")
    //     .version(env!("CARGO_PKG_VERSION"))
    //     .author(env!("CARGO_PKG_AUTHORS"))
    //     .about("A Rust chess engine")
    //     .arg(
    //         Arg::with_name("cli")
    //             .short("c")
    //             .long("cli")
    //             .help("Start in CLI mode instead of UCI"),
    //     )
    //     .arg(
    //         Arg::with_name("selftest")
    //             .short("s")
    //             .long("selftest")
    //             .value_name("FILE")
    //             .help("Runs perfts from tests/perftsuite.epd")
    //             .takes_value(true),
    //     )
    //     .get_matches();
    if cfg!(windows) {
        let _enabled = ansi_term::enable_ansi_support();
    }

    let matches = clap_app!(deeprust =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: "A chess playing program.")
        (@arg cli: -c --cli "Starts in CLI mode")
        (@arg FENFILE: -b --batch +takes_value "Runs perfts from a FEN file")
        (@arg debug: -d ... "Sets the level of debugging information")
    )
    .get_matches();

    if matches.is_present("cli") {
        deeprust::cli::run();
    } else if matches.is_present("FENFILE") {
        let config = matches
            .value_of("FENFILE")
            .unwrap_or("tests/perftsuite.epd");
        batchperft(config);
    } else {
        let mut c = deeprust::uci::UCIInterface::new();
        c.run();
    }
}
