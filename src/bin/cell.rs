use users::get_current_username;
fn main() {
    let (cell_id, pid, stream_name) = process_args(0).expect("Can't process args");
    eprintln!("cell_id {}, pid {}, stream_name {:?}", cell_id, pid, stream_name);
}
fn process_args(skip: usize) -> Result<(String, u32, Option<String>), Box<dyn std::error::Error>> {
    let pid = std::process::id();
    let user = get_current_username().expect("No usernamd for process");
    let default_id = "Default".to_owned();
    let args: Vec<_> = std::env::args().collect();
     let cell_id = args.get(skip+1).or(Some(&default_id)).unwrap();
    let stream_name = args.get(skip+2).cloned();
    eprintln!("  Cell {} starting with PID {} as user {:?}", cell_id, pid, user);
    Ok((cell_id.clone(), pid, stream_name))
}
#[cfg(test)]
mod tests {
    use super::process_args;
    use passfd::FdPassingExt;
    use pegasus::utility::{keep_alive, random_sleep};
    // use std::io::Read;
    use std::{
        io::{stdin, stdout, BufRead, BufReader, Write},
        os::unix::{net::UnixStream, prelude::FromRawFd},
        process,
    };
    #[test]
fn chaos_monkey() {
    eprintln!("  Test chaos_monkey");
    let (cell_id, pid, _) = process_args(2).expect("Can't process args");
    eprintln!("  Hello from {}", cell_id);
    let mut to_chaos_monkey = stdout();
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
    let msg = msg;
    eprintln!("  Cell {} sending to chaos monkey {}", pid, msg.trim());
    let buf = msg.as_bytes(); 
    to_chaos_monkey
        .write_all(buf)
        .expect("Cannot write to chaos monkey");
    to_chaos_monkey.flush().expect("Can't flush");
    std::thread::sleep(std::time::Duration::from_secs(3));
    // assert!(false); // Test for failed test
    eprintln!("  Cell {} exiting", pid);
 }
#[test]
fn cell2cell() {
    eprintln!("  Test cell2cell");
    let (cell_id, pid, stream_name_opt) = process_args(2).expect("Can't process args");
    if let Some(stream_name) = stream_name_opt {
        eprintln!("  {}: Connecting to stream {}", cell_id, stream_name);
        let stream = UnixStream::connect(stream_name.clone())
            .expect(&format!("Can't connect to {}", stream_name));
        eprintln!("  {} Connected: Reading fds", cell_id);
        let tx_raw = stream.recv_fd().expect("Can't receive tx");
        let rx_raw = stream.recv_fd().expect("Can't receive rx");
        eprintln!(
            "  {}: Got tx/rx {}/{} for other cell {:?}",
            cell_id, tx_raw, rx_raw, stream_name
        );
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
        assert!(false);
    }
    keep_alive(&format!("  {} alive", cell_id));
    eprintln!("  Cell {} exiting", pid);
}
}
