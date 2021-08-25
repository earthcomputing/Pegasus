use std::io::{BufRead, BufReader, Write};

fn main() {
    let pid = std::process::id();
    eprintln!("  Cell starting with PID {}", pid);
    let args: Vec<_> = std::env::args().collect();
    let cell_id = args.get(1).expect("No cell ID");
    eprintln!("  Hello from {}", cell_id);
    let from_simulator = std::io::stdin();
    let mut to_simulator = std::io::stdout();
    let mut reader = BufReader::new(from_simulator).lines();
    pegasus::utility::random_sleep("cell", std::process::id());
    eprintln!("  Cell {} listening to simulator", pid);
    let buf = reader
        .next()
        .expect("No msg from simulator")
        .expect("Cannot read from simulator");
    eprintln!("  {} got {}", pid, buf);
    let msg = format!("{}\n", pid);
    let buf = msg.as_bytes();
    to_simulator
        .write_all(buf)
        .expect("Cannot write to simulator");
    to_simulator.flush().expect("Cannot flush");
}
