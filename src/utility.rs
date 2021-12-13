use nix::sys::select::{select, FdSet};
use nix::unistd::{pipe, unlink};
use passfd::FdPassingExt;
use rand::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    io::{BufRead, BufReader},
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
pub fn talk_to_cell<'a>(cell: &mut Cell, msg: impl Into<Option<&'a str>>) {
    if let Some(msg) = msg.into() {
        cell.chaos_to_cell
            .write_all(msg.as_bytes())
            .expect("Cannot write to cell");
        println!("Sent '{}' to cell {}", msg.trim(), cell.pid);
    }
}
pub fn keep_alive(msg: &str) {
    println!("{}", msg);
    //let mut buf = String::new();
    //let _ = std::io::stdin().read_line(&mut buf);
    std::thread::sleep(std::time::Duration::from_secs(5));
}
pub fn setup_fds<'a>(
    cells: &'a mut Vec<&'a mut Cell>,
) -> (FdSet, HashMap<i32, (u32, &'a mut ChildStdout)>) {
    let mut master_fds = FdSet::new();
    let mut from_cell_fds = HashMap::new();
    for cell in cells.iter_mut() {
        let from_cell = &mut cell.chaos_from_cell;
        let from_cell_raw = from_cell.as_raw_fd();
        println!("Insert fd {}", from_cell_raw);
        from_cell_fds.insert(from_cell_raw, (cell.pid, from_cell));
        master_fds.insert(from_cell_raw);
    }
    (master_fds, from_cell_fds)
}
pub fn pipe_pair() -> Result<[i32; 4], Box<dyn std::error::Error>> {
    let (from_left, to_rite) = pipe().expect("Can't create pipe 1");
    let (from_rite, to_left) = pipe().expect("Can't create pipe 2");
    Ok([from_left, to_rite, from_rite, to_left])
}
pub fn send_fds(socket_name: &str, tx: i32, rx: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket_path = std::env::temp_dir();
    socket_path.push(socket_name);
    let listener = UnixListener::bind(socket_path).expect("Can't bind socket");
    let (stream, _) = listener.accept().expect("Can't accept socket");
    stream.send_fd(tx).expect("Can't send tx");
    stream.send_fd(rx).expect("Can't send rx");
    Ok(())
}
pub fn pipes(socket_name: &str) -> Result<(File, File), Box<dyn std::error::Error>> {
    let file = "/tmp/socket/".to_owned() + socket_name;
    let (from_left, to_rite) = pipe().expect("Can't create pipe 1");
    let (from_rite, to_left) = pipe().expect("Can't create pipe 2");
    // Need thread so I can wait for connect
    std::thread::spawn(move || {
        unlink(&file[..]).expect("Can't unlink file");
        let listener = UnixListener::bind(file).expect("Can't bind socket");
        let (stream, _) = listener.accept().expect("Can't accept on socket");
        stream.send_fd(to_rite).expect("Can't send to_server");
        stream.send_fd(from_rite).expect("Can't send from_server");
    });
    let to_client = unsafe { File::from_raw_fd(to_left) };
    let from_client = unsafe { File::from_raw_fd(from_left) };
    Ok((to_client, from_client))
}
pub fn select_cell<'a>(
    master_fds: &FdSet,
    from_cell_from_fd_raw: &mut HashMap<i32, (u32, &'a mut ChildStdout)>,
) -> Result<Vec<(u32, String)>, String> {
    let mut msgs = Vec::new();
    let mut fdset_rd = master_fds.clone();
    let mut fdset_err = FdSet::new();
    match select(None, &mut fdset_rd, None, &mut fdset_err, None) {
        Ok(r) => println!("Success: {} fds ready", r),
        Err(e) => println!("Failure: {}\nfdset_rd {:?}", e, fdset_rd),
    }
    select(None, &mut fdset_rd, None, &mut fdset_err, None).expect("Select problem");
    for fd_raw in fdset_rd.fds(None) {
        let cell_info = from_cell_from_fd_raw
            .get_mut(&fd_raw)
            .expect("from_cell_fds error");
        let cell_pid = cell_info.0;
        let fd = &mut *cell_info.1;
        let mut reader = BufReader::new(fd).lines();
        match reader.next() {
            Some(m) => match m {
                Ok(msg) => {
                    msgs.push((cell_pid, msg.clone()));
                    msg
                }
                Err(e) => return Err(format!("Read error {}", e)),
            },
            None => return Err("Empty message".to_owned()),
        };
    }
    Ok(msgs)
}