use nix::sys::select::FdSet;
use nix::unistd::{pipe, unlink};
use passfd::FdPassingExt;
use rand::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    os::unix::{
        net::UnixListener,
        prelude::{AsRawFd, FromRawFd},
    },
    process::ChildStdout,
};

use crate::cell::Cell;

pub fn random_sleep(who: &str, id: u32) {
    let ms: u8 = rand::thread_rng().gen();
    eprintln!("{} {} sleeping for {} ms", who, id, ms);
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    eprintln!("{} {} awake", who, id);
}
pub fn talk_to_cell(cell: &mut Cell, msg: &str) {
    cell.to_cell
        .write_all(msg.as_bytes())
        .expect("Cannot write to cell");
    println!("Sent '{}' to cell {}", msg.trim(), cell.pid);
}
pub fn setup_fds<'a>(
    cells: &'a mut Vec<&'a mut Cell>,
) -> (FdSet, HashMap<i32, (u32, &'a mut ChildStdout)>) {
    let mut master_fds = FdSet::new();
    let mut from_cell_fds = HashMap::new();
    for cell in cells.iter_mut() {
        let from_cell = &mut cell.from_cell;
        let from_cell_raw = from_cell.as_raw_fd();
        println!("Insert fd {}", from_cell_raw);
        from_cell_fds.insert(from_cell_raw, (cell.pid, from_cell));
        master_fds.insert(from_cell_raw);
    }
    (master_fds, from_cell_fds)
}
pub fn pipes(socket_name: &str) -> Result<(File, File), Box<dyn std::error::Error>> {
    let file = "/tmp/socket/".to_owned() + socket_name;
    let (from_cell_raw, to_simulator_raw) = pipe().expect("Can't create pipe 1");
    let (from_simulator_raw, to_cell_raw) = pipe().expect("Can't create pipe 2");
    // Need thread so I can wait for connect
    std::thread::spawn(move || {
        unlink(&file[..]).expect("Can't unlink file");
        let listener = UnixListener::bind(file).expect("Can't bind socket");
        let (stream, _) = listener.accept().expect("Can't accept on socket");
        stream
            .send_fd(to_simulator_raw)
            .expect("Can't send to_server");
        stream
            .send_fd(from_simulator_raw)
            .expect("Can't send from_server");
    });
    let to_client = unsafe { File::from_raw_fd(to_cell_raw) };
    let from_client = unsafe { File::from_raw_fd(from_cell_raw) };
    Ok((to_client, from_client))
}
