use nix::sys::select::{select, FdSet};
use nix::unistd::{pipe, unlink};
use passfd::FdPassingExt;
use rand::prelude::*;
use std::os::unix::prelude::RawFd;
use std::process::ChildStdout;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    io::{BufRead, BufReader},
    os::unix::{
        net::UnixListener,
        prelude::{AsRawFd, FromRawFd},
    },
};
use std::path::PathBuf;


pub fn random_sleep(who: &str, id: u32) {
    let ms: u8 = rand::thread_rng().gen();
    eprintln!("{} {} sleeping for {} ms", who, id, ms);
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    eprintln!("{} {} awake", who, id);
}
pub fn talk_to_cell<'a>(pid: u32, tx_raw: RawFd, msg: impl Into<Option<&'a str>>) {
    let mut tx = unsafe { std::fs::File::from_raw_fd(tx_raw) };
    if let Some(msg) = msg.into() {
        tx
            .write_all(msg.as_bytes())
            .expect("Cannot write to cell");
        println!("Sent '{}' to cell {}", msg.trim(), pid);
    }
}
pub fn keep_alive(duration: Option<std::time::Duration>, msg: &str) {
    match duration {
        Some(d) => {
            eprintln!("{} {:?}", msg, d);
            std::thread::sleep(d);
        }, 
        None => {
            eprintln!("{}", msg);
            let mut buf = String::new();
            let _ = std::io::stdin().read_line(&mut buf);
}
    }
}
pub fn setup_fds<'a>(
    cell_rxs: &'a mut Vec<(u32, &File)>,
) -> (FdSet, HashMap<i32, (u32, &'a File)>) {
    let mut master_fds = FdSet::new();
    let mut from_fds = HashMap::new();
    for (pid, from_cell) in cell_rxs.iter() {
        let from_cell_raw = from_cell.as_raw_fd();
        from_fds.insert(from_cell_raw, (*pid, *from_cell));
        master_fds.insert(from_cell_raw);
    }
    (master_fds, from_fds)
}
pub fn setup_fds_test<'a>(
    cells_rxs: &'a mut Vec<(u32, &'a mut ChildStdout)>,
) -> (FdSet, HashMap<i32, (u32, &'a mut ChildStdout)>) {
    let mut master_fds = FdSet::new();
    let mut from_cell_fds = HashMap::new();
    for cell_info in cells_rxs.iter_mut() {
        let pid = cell_info.0;
        let from_cell = &mut *cell_info.1;
        let from_cell_raw = from_cell.as_raw_fd();
        from_cell_fds.insert(from_cell_raw, (pid, from_cell));
        master_fds.insert(from_cell_raw);
    }
    (master_fds, from_cell_fds)
}
pub fn pipe_pair() -> Result<[i32; 4], Box<dyn std::error::Error>> {
    let (from_left, to_rite) = pipe().expect("--> Can't create pipe 1");
    let (from_rite, to_left) = pipe().expect("--> Can't create pipe 2");
    Ok([from_left, to_rite, from_rite, to_left])
}
pub fn send_fds(socket_name: &str, tx: i32, rx: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket_path = std::env::temp_dir();
    socket_path.push(socket_name);
    let listener = UnixListener::bind(socket_path).expect("Can't bind socket");
    let (stream, _) = listener.accept().expect("Can't accept socket");
    stream.send_fd(tx)?;
    stream.send_fd(rx)?;
    Ok(())
}
pub fn pipes(socket_name: &str) -> Result<(File, File), Box<dyn std::error::Error>> {
    let file = "/tmp/socket/".to_owned() + socket_name;
    let (from_left, to_rite) = pipe().expect("Can't create pipe 1");
    let (from_rite, to_left) = pipe().expect("Can't create pipe 2");
    // Need thread so I can wait for connect
    std::thread::spawn(move || {
        let _ = unlink(&file[..]);  // Unlink file if it already exists
        let listener = UnixListener::bind(file).expect("Can't bind socket");
        let (stream, _) = listener.accept().expect("Can't accept on socket");
        stream.send_fd(to_rite).expect("Can't send to_server");
        stream.send_fd(from_rite).expect("Can't send from_server");
    });
    let to_client = unsafe { File::from_raw_fd(to_left) };
    let from_client = unsafe { File::from_raw_fd(from_left) };
    Ok((to_client, from_client))
}
pub fn select_cell(
    master_fds: &FdSet,
    from_cell_fds: &mut HashMap<i32, (u32, &File)>,
) -> Result<Vec<(u32, String)>, String> {
    let mut msgs = Vec::new();
    let mut fdset_rd = master_fds.clone();
    let mut fdset_err = FdSet::new();
    match select(None, &mut fdset_rd, None, &mut fdset_err, None) {
        Ok(r) => println!("{} fds ready", r),
        Err(e) => println!("--> Failure: {}\nfdset_rd {:?}", e, fdset_rd),
    }
    select(None, &mut fdset_rd, None, &mut fdset_err, None).expect("Select problem");
    for fd_raw in fdset_rd.fds(None) {
        let cell_info = from_cell_fds
            .get_mut(&fd_raw)
            .expect("from_cell_fds error");
        let cell_pid = cell_info.0;
        let mut reader = BufReader::new(cell_info.1).lines();
        match reader.next() {
            Some(m) => match m {
                Ok(msg) => {
                    println!("Select got |{}|", msg);
                    msgs.push((cell_pid, msg))
                },
                Err(e) => return Err(format!("--> Read error {}", e)),
            },
            None => {} // return Err("Empty message".to_owned()),
        };
    }
    Ok(msgs)
}
// Test report is returned on ChildStdout
pub fn select_cell_test<'a>(
    master_fds: &FdSet,
    from_cell_from_fd: &mut HashMap<i32, (u32, &'a mut ChildStdout)>,
) -> Result<Vec<(u32, String)>, String> {
    let mut msgs = Vec::new();
    let mut fdset_rd = master_fds.clone();
    let mut fdset_err = FdSet::new();
    match select(None, &mut fdset_rd, None, &mut fdset_err, None) {
        Ok(r) => println!("{} fds ready", r),
        Err(e) => println!("--> Failure: {}\nfdset_rd {:?}", e, fdset_rd),
    }
    select(None, &mut fdset_rd, None, &mut fdset_err, None).expect("Select problem");
    for fd_raw in fdset_rd.fds(None) {
        let cell_info = from_cell_from_fd
            .get_mut(&fd_raw)
            .expect("from_cell_fds error");
        let cell_pid = cell_info.0;
        let from_cell = &mut *cell_info.1;
        let mut reader = BufReader::new(from_cell).lines();
        match reader.next() {
            Some(m) => match m {
                Ok(msg) => {
                    println!("Select got |{}|", msg);
                    msgs.push((cell_pid, msg))
                },
               Err(e) => return Err(format!("--> Read error {}", e)),
            },
            None => {} // return Err("Empty message".to_owned()),
        };
    }
    Ok(msgs)
}
pub fn share_stream(
    cell_name: &str,
    from_cell_raw: i32,
    to_cell_raw: i32,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut socket_name = PathBuf::from("/tmp/test");
    socket_name.push(cell_name);
    let socket_name_clone = socket_name.clone();
    let cell_name_clone = cell_name.to_owned();
    std::thread::spawn( move || {
        let _ = unlink(&socket_name); // Unlink file if it already exists
        println!("Listening on stream {}", &socket_name.to_str().unwrap());
        let listener = UnixListener::bind(&socket_name.clone())
            .expect(&format!("--> Can't bind socket: {:?}", socket_name));
        let (stream, _addr) = listener.accept().expect("--> Can't accept on socket");
        println!(
            "Sending pipe handles on stream {} from_cell {} to_cell {}",
            socket_name.to_str().unwrap(),
            from_cell_raw,
            to_cell_raw
        );
        // Do not change the order of the next two lines
        let _ = stream.send_fd(to_cell_raw).map_err(|e| println!("--> Can't send to_cell {} to {}: {}", to_cell_raw, cell_name_clone, e));
        let _ = stream.send_fd(from_cell_raw).map_err(|e| println!("--> Can't send from_cell {} to {}: {}", from_cell_raw, cell_name_clone, e));
        keep_alive(None, &format!(
            "Keep stream {} alive",
            socket_name.to_str().unwrap()
        ));
    });
    Ok(socket_name_clone)
}