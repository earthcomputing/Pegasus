use passfd::FdPassingExt;
use pegasus::utility::{keep_alive, random_sleep};
// use std::io::Read;
use std::{env::args, io::{BufRead, BufReader, Read, Write, stdin, stdout}, os::unix::{net::UnixStream, prelude::FromRawFd}, process};
use users::get_current_username;

fn main() {
    let pid = process::id();
    let user = get_current_username().expect("No usernamd for process");
    let default_id = "Default".to_owned();
    let args: Vec<_> = args().collect();
    let cell_id = args.get(1).or(Some(&default_id)).unwrap();
    eprintln!("  Cell starting with PID {} as user {:?}", cell_id, user);
    let stream_name = args.get(2);
    if let Some(stream_name) = stream_name {
        eprintln!("  {}: Connecting to stream {}", cell_id, stream_name);
        let stream = UnixStream::connect(stream_name.clone())
            .expect(&format!("Can't connect to {}", stream_name));
        eprintln!("  {} Connected: Reading fds", cell_id);
        let tx_raw = stream.recv_fd().expect("Can't receive tx");
        let rx_raw = stream.recv_fd().expect("Can't receive rx");
        eprintln!("  {}: Got tx/rx {}/{} for other cell {:?}", cell_id, tx_raw, rx_raw, stream_name);
        let mut to_other = unsafe { std::fs::File::from_raw_fd(tx_raw) };
        let from_other = unsafe { std::fs::File::from_raw_fd(rx_raw) };
        writeln!(to_other, "{}", cell_id).expect("Can't write to other cell");
        let mut reader = BufReader::new(from_other).lines();
        let msg = reader
            .next()
            .or(Some(Ok("No msg from other cell".to_owned())))
            .unwrap()
            .expect("Cannot read from other cell");
        eprintln!("  Cell {} got '{}'", cell_id, msg);
        
    } else {
        chaos_monkey(cell_id, pid).expect("Problem talking to chaos monkey");
    }
    keep_alive(&format!("  {} alive", cell_id));
    eprintln!("  Cell {} exiting", pid);
}
fn _read_stream(stream: &mut UnixStream) -> [u8;100] {
    let mut buf = [0;100];
    stream.read(&mut buf).expect("Can't read from stream");
    buf
}
fn chaos_monkey(cell_id: &str, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("  Hello from {}", cell_id);
    let from_chaos_monkey = stdin();
    let mut reader = BufReader::new(from_chaos_monkey).lines();
    random_sleep("  Cell", process::id());
    eprintln!("  Cell {} listening to chaos monkey", pid);
    let msg = reader
        .next()
        .or(Some(Ok("No msg from chaos monkey".to_owned())))
        .unwrap()
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
    Ok(())
}
