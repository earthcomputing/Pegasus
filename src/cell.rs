use std::fs::File;
use std::os::unix::prelude::RawFd;
use std::{
    os::unix::{net::UnixStream, prelude::FromRawFd},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use passfd::FdPassingExt;

pub struct Cell {
    pub pid: u32,
    pub process: Child,
    pub cell_stdin: ChildStdin,
    pub cell_stdout: ChildStdout,
    pub chaos_to_cell: File,
    pub chaos_from_cell: File
}
impl Cell {
    pub fn new<'a>(
        cell_id: &'static str,
        chaos_to_cell_raw: RawFd,
        chaos_from_cell_raw: RawFd,
        program_plus_args: &[&str],
        stream_name: &'a PathBuf,
    ) -> Cell {
        let program = program_plus_args[0];
         let mut child = Command::new(program)
            .args(&program_plus_args[1..])
            .arg(cell_id)
            .arg(stream_name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not spawn cell");
        let child_id = child.id();
        println!("{} has PID {}", cell_id, child.id());
        let cell_stdin = child
            .stdin
            .take()
            .expect(&format!("Can't get stdout for {}", cell_id));
        let cell_stdout = child
            .stdout
            .take()
            .expect(&format!("Can't get stdin for {}", cell_id));
        let chaos_to_cell = unsafe { std::fs::File::from_raw_fd(chaos_to_cell_raw) };
        let chaos_from_cell = unsafe { std::fs::File::from_raw_fd(chaos_from_cell_raw) };
        Cell {
            pid: child_id,
            process: child,
            cell_stdin,
            cell_stdout,
            chaos_to_cell,
            chaos_from_cell
        }
    }
}
fn _get_fds(stream_name: &PathBuf) -> Result<(File, File), Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(stream_name.clone())
        .expect(&format!("Can't connect to {:?}", stream_name));
    let fd_raw = stream.recv_fd().expect("Can't receive tx");
    let tx = unsafe { std::fs::File::from_raw_fd(fd_raw) };
    let fd_raw = stream.recv_fd().expect("Can't receive rx");
    let rx = unsafe { std::fs::File::from_raw_fd(fd_raw) };
    Ok((tx, rx))
}
