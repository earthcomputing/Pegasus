use std::path::PathBuf;

use nix::unistd::{pipe, unlink};
use users::get_current_username;

use pegasus::cell::Cell;
use pegasus::utility::{talk_to_cell, keep_alive};

#[test]
fn cell2cell() {
    println!("Running as user {:?}", get_current_username().expect("No username"));
    let (from_cell1, to_cell2) = pipe().expect("Can't create pipe 1");
    let (from_cell2, to_cell1) = pipe().expect("Can't create pipe 2");
    let socket_name1 = share_stream("cell1", from_cell2, to_cell2).expect("Can't share stream1");
    let socket_name2 = share_stream("cell2", from_cell1, to_cell1).expect("Can't share stream2");
    let mut cell1 = Cell::new("cell1", "target/debug/cell", &socket_name1);
    let mut cell2 = Cell::new("cell2", "target/debug/cell", &socket_name2);
    talk_to_cell(&mut cell1, None);
    talk_to_cell(&mut cell2, None);
    keep_alive("Enter anything to have cell2cell exit");
    println!("cell2cell exiting");
}
fn share_stream(cell_name: &str, rx_raw: i32, tx_raw: i32) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use passfd::FdPassingExt;
    use std::os::unix::net::UnixListener;
    let mut socket_name = PathBuf::from("/tmp/test");
    socket_name.push(cell_name);
    let socket_name_clone = socket_name.clone();
    std::thread::spawn(move || {
        unlink(&socket_name).expect("Can't unlink file");
        println!("Listening on stream {}", &socket_name.to_str().unwrap());
        let listener = UnixListener::bind(&socket_name.clone())
            .expect(&format!("Can't bind socket: {:?}", socket_name));
        let (stream, addr) = listener.accept().expect("Can't accept on socket");
        println!("Sending pipe handles on stream {} at addr {:?} rx {} tx {}", socket_name.to_str().unwrap(), addr, rx_raw, tx_raw);
        stream.send_fd(tx_raw).expect("Can't send tx to_server");
        stream.send_fd(rx_raw).expect("Can't send rx from_server");
        keep_alive(&format!("Keep stream {} alive", socket_name.to_str().unwrap()));
     });
    Ok(socket_name_clone)
}
