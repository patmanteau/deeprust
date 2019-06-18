#[rustfmt::skip::macros(indoc)]
extern crate indoc;

use std::io;
use std::io::Write;

use crate::uci;

fn print_version() {
    println!("deeprust v{}", env!("CARGO_PKG_VERSION"));
    println!("\t(c) 2017 {}", env!("CARGO_PKG_AUTHORS"));
}

fn print_help() {
    let help = indoc::indoc!(
        "
        Known commands:
            exit, quit  Exit CLI mode
            help        This page
            uci         Switch the engine to UCI mode
            version     Print version information"
    );
    println!("{}", help);
}

pub fn run() {
    println!("deeprust v{} CLI", env!("CARGO_PKG_VERSION"));
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        let command = command.trim();

        match command {
            "exit" | "quit" => {
                println!("Byebye");
                break;
            }
            "help" => print_help(),
            "version" => print_version(),
            "uci" => {
                uci::UCIInterface::new().run();
                break;
            }
            unknown => println!("Unknown command: {}", unknown),
        }
    }
}
