use pegasus::utility::random_sleep;
use std::{
    env::args,
    io::{stdin, stdout, BufRead, BufReader, Write},
    process,
};

fn main() {
    let pid = process::id();
    eprintln!("  Cell starting with PID {}", pid);
    let args: Vec<_> = args().collect();
    let cell_id = args.get(1).expect("No cell ID");
    eprintln!("  Hello from {}", cell_id);
    let from_simulator = stdin();
    let mut to_simulator = stdout();
    let mut reader = BufReader::new(from_simulator).lines();
    random_sleep("cell", process::id());
    eprintln!("  Cell {} listening to simulator", pid);
    let buf = reader
        .next()
        .expect("No msg from simulator")
        .expect("Cannot read from simulator");
    eprintln!("  Cell {} got '{}'", pid, buf);
    let msg = buf;
    let buf = msg.as_bytes();
    to_simulator
        .write_all(buf)
        .expect("Cannot write to simulator");
    to_simulator.flush().expect("Cannot flush");
}
