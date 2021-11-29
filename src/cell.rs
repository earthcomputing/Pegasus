use std::fs::File;
use std::{
    os::unix::{net::UnixStream, prelude::FromRawFd},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use passfd::FdPassingExt;

pub struct Cell {
    pub pid: u32,
    pub process: Child,
    pub chaos_to_cell: ChildStdin,
    pub chaos_from_cell: ChildStdout,
}
impl Cell {
    pub fn new(
        cell_id: &'static str,
        program_opt: Option<&str>,
        stream_name_opt: Option<&PathBuf>,
    ) -> Cell {
        let program = program_opt.unwrap_or("target/debug/test_cell");
        let mut child = Command::new(program)
            .arg(cell_id)
            .arg(stream_name_opt.unwrap_or(&PathBuf::from("")))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not spawn cell");
        let child_id = child.id();
        println!("{} has PID {}", cell_id, child.id());
        let from_chaos_monkey = child
            .stdin
            .take()
            .expect(&format!("Can't get stdout for {}", cell_id));
        let to_chaos_monkey = child
            .stdout
            .take()
            .expect(&format!("Can't get stdin for {}", cell_id));
        
        Cell {
            pid: child_id,
            process: child,
            chaos_to_cell: from_chaos_monkey,
            chaos_from_cell: to_chaos_monkey,
        }
    }
}
fn get_fds(stream_name: &PathBuf) -> Result<(File, File), Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(stream_name.clone())
        .expect(&format!("Can't connect to {:?}", stream_name));
    let fd_raw = stream.recv_fd().expect("Can't receive tx");
    let tx = unsafe { std::fs::File::from_raw_fd(fd_raw) };
    let fd_raw = stream.recv_fd().expect("Can't receive rx");
    let rx = unsafe { std::fs::File::from_raw_fd(fd_raw) };
    Ok((tx, rx))
}
