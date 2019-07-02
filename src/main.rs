#![warn(clippy::all)]
#![feature(test)]
#![feature(simd_x86_bittest)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nom;
extern crate quanta;
extern crate rand;
extern crate rayon;
extern crate simple_logging;
extern crate test;

pub mod bitboard;
pub mod board;
pub mod cli;
pub mod color;
pub mod common;
pub mod interfaces;
pub mod move_generator;
pub mod moves;
pub mod piece;
pub mod position;
pub mod search;
pub mod square;
pub mod uci;

// use clap::{App, Arg};
use board::Board;
use interfaces::FenInterface;
use search::{PerftContext, Search};

use log::LevelFilter;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

fn selftest(f: &str) {
    let mut path = PathBuf::from(f);
    let file = match File::open(&path) {
        Err(why) => panic!("Could not open {}: {}", path.display(), why.description()),
        Ok(file) => file,
    };

    debug!("Running selftest from {}", path.to_str().unwrap());
    for (num, line) in BufReader::new(file).lines().map(|l| l.unwrap()).enumerate() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        } else {
            let parts: Vec<&str> = line.split(';').map(|l| l.trim()).collect();
            
            let fen_str = parts[0];
            let mut b = Board::from_fen_str(&fen_str).unwrap();

            println!("Testing fen {}...", fen_str);
            for depth_def in &parts[1..] {
                let depth_def_parts: Vec<u64> = 
                    depth_def
                    .split_whitespace()
                    .map(|d| u64::from_str_radix(d, 10).unwrap())
                    .collect();
                let (depth, nodes) = (depth_def_parts[0], depth_def_parts[1]);
                // print!("Running perft {} on '{}' expecting {} nodes...", depth, fen_str, nodes);
                let res = b.perft(depth as u32);
                if res.nodes == nodes {
                    // println!("OK");
                } else {
                    println!("Line {} failed at depth {}: got {} instead of {} nodes", num + 1, depth, res.nodes, nodes);
                    debug!("Line {} failed at depth {}: got {} instead of {} nodes", num + 1, depth, res.nodes, nodes);
                }
            }
        }
    }
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

    let matches = clap_app!(deeprust =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: "A chess playing program.")
        (@arg cli: -c --cli "Starts in CLI mode")
        (@arg FENFILE: -s --selftest +takes_value "Runs perfts from a FEN file")
        (@arg debug: -d ... "Sets the level of debugging information")
    ).get_matches();

    if matches.is_present("cli") {
        cli::run();
    } else if matches.is_present("FENFILE") {
        let config = matches.value_of("FENFILE").unwrap_or("tests/perftsuite.epd");
        selftest(config);
    }else {
        let mut c = uci::UCIInterface::new();
        c.run();
    }
}
