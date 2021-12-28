use std::{
    io::{BufRead, BufReader, Write},
    process, os::unix::prelude::FromRawFd,
};
use passfd::FdPassingExt;
use pegasus::utility::{keep_alive, random_sleep};
use users::get_current_username;
fn main() {
    let (cell_id, pid, stream_name) = process_args(0).expect("Can't process args");
    eprintln!("cell_id {}, pid {}, stream_name {:?}", cell_id, pid, stream_name);
    talk_to_chaos_monkey(&cell_id, pid, stream_name.expect("No stream name"));
}
fn process_args(skip: usize) -> Result<(String, u32, Option<String>), Box<dyn std::error::Error>> {
    let pid = std::process::id();
    let user = get_current_username().expect("No username for process");
    let default_id = "Default".to_owned();
    let args: Vec<_> = std::env::args().collect();
    let cell_id = args.get(skip+1).or(Some(&default_id)).unwrap();
    let stream_name = args.get(skip+2).cloned();
    eprintln!("  {} starting with PID {} as user {:?}", cell_id, pid, user);
    Ok((cell_id.clone(), pid, stream_name))
}
fn talk_to_chaos_monkey(cell_id: &str, pid: u32, stream_name: String) {
    eprintln!("  Hello from {}", cell_id);
    eprintln!("  {}: Connecting to stream {}", cell_id, stream_name);
    let stream = std::os::unix::net::UnixStream::connect(stream_name.clone())
        .expect(&format!("Can't connect to {}", stream_name));
    eprintln!("  {} Connected: Reading fds", cell_id);
    let tx_raw = stream.recv_fd().expect("Can't receive tx");
    let rx_raw = stream.recv_fd().expect("Can't receive rx");
    eprintln!(
        "  {}: Got tx/rx {}/{} for other cell {:?}",
        cell_id, tx_raw, rx_raw, stream_name
    );
    let from_chaos_monkey = unsafe { std::fs::File::from_raw_fd(rx_raw) };
    let mut reader = BufReader::new(from_chaos_monkey).lines();
    random_sleep("  Cell", process::id());
    eprintln!("  Cell {} listening to chaos monkey", pid);
    let msg = reader
        .next()
        .or(Some(Ok("No msg from chaos monkey".to_owned())))
        .unwrap()
        .expect("Cannot read from chaos monkey");
    eprintln!("  Cell {} sending to chaos monkey {}", pid, msg);
    // let msg = msg + "\n";
    let buf = msg.as_bytes(); 
    let mut to_chaos_monkey = unsafe { std::fs::File::from_raw_fd(tx_raw) };
    to_chaos_monkey
        .write_all(buf)
        .expect("Cannot write to chaos monkey");
    keep_alive(Some(std::time::Duration::from_secs(2)), "Sleeping 10 seconds");
    // assert!(false); // Test for failed test
    eprintln!("  Cell {} exiting", pid);
}
#[cfg(test)]
mod tests {
    use crate::talk_to_chaos_monkey;

    use super::process_args;
    use passfd::FdPassingExt;
    use pegasus::utility::keep_alive;
    // use std::io::Read;
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::{net::UnixStream, prelude::FromRawFd},
    };
    #[test]
fn chaos_monkey() {
    eprintln!("  Test chaos_monkey");
    let (cell_id, pid, stream_name) = process_args(2).expect("Can't process args");
    talk_to_chaos_monkey(&cell_id, pid, stream_name.expect("No stream name"));
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
    keep_alive(None, &format!("  {} alive", cell_id));
    eprintln!("  Cell {} exiting", pid);
}
}
