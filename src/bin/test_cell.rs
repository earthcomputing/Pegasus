use pegasus::utility::random_sleep;
use std::{
    env::args,
    io::{stdin, stdout, BufRead, BufReader, Write},
    process,
};

fn main() {
    let pid = process::id();
    eprintln!("  Cell starting with PID {}", pid);
    let default_id = "Default".to_owned();
    let args: Vec<_> = args().collect();
    let cell_id = args.get(1).or(Some(&default_id)).unwrap();
    eprintln!("  Hello from {}", cell_id);
    let from_chaos_monkey = stdin();
    let mut reader = BufReader::new(from_chaos_monkey).lines();
    random_sleep("  Cell", process::id());
    eprintln!("  Cell {} listening to chaos monkey", pid);
    let msg = reader
        .next()
        .expect("No msg from chaos monkey")
        .expect("Cannot read from chaos monkey");
    eprintln!("  Cell {} got '{}'", pid, msg);
    let msg = msg + "\n";
    let buf = msg.as_bytes();
    eprintln!("  Cell {} sending to chaos monkey", pid);
    let mut to_chaos_monkey = stdout();
    to_chaos_monkey
        .write_all(buf)
        .expect("Cannot write to chaos monkey");
    match reader.next() {
        Some(m) => match m {
            Ok(m) => eprintln!("  Cell {} got msg from chaos monkey {}", pid, m),
            Err(e) => eprintln!("  Cell {} got error from chaos monkey {}", pid, e),
        },
        None => eprintln!("  Cell {} got no message from chaos monkey", pid),
    }
    eprintln!("  Cell {} exiting", pid);
}
