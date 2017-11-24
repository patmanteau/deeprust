use std::io;
use std::io::Write;

pub fn run() {
    println!("id name deeprust v{}", env!("CARGO_PKG_VERSION"));
    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
    println!("uciok");
    io::stdout().flush().unwrap();
    loop {
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        let command = command.trim();

        match command {
            "isready" => println!("readyok"),
            "quit" => break,
            unknown => println!("Unknown command: {}", unknown),
        }
        io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn isready() {
        
    }
}
