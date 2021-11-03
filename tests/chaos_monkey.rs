use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::ChildStdout,
};

use nix::sys::select::{select, FdSet};
use pegasus::cell::Cell;
use pegasus::utility::{random_sleep, setup_fds, talk_to_cell};
#[test]
// Make sure all clients have sent their results before trying to read from them
fn chaos_monkey_all_fds_ready() {
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let mut cell1 = Cell::new(cell_id1, None);
    let mut cell2 = Cell::new(cell_id2, None);
    // Needed to keep chaos monkey running to give cell a chance to finish
    talk_to_cell(&mut cell1, "Hello 1\n");
    talk_to_cell(&mut cell2, "Hello 2\n");
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let msgs = select_cell(&master_fds, &mut from_cell_fds);
    for (cell_pid, msg) in &msgs {
        println!("Chaos monkey got From {} '{}'", cell_pid, msg);
    }
    assert_eq!(msgs.len(), 2);
    println!("Chaos monkey sleeping to let cells finish");
    std::thread::sleep(std::time::Duration::from_secs(2)); // Let cells finish their work
    println!("Chaos monkey exiting");
}
#[test]
// Make sure only one client message is ready at a time
fn chaos_monkey_some_fds_ready() {
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let mut cell1 = Cell::new(cell_id1, None);
    let mut cell2 = Cell::new(cell_id2, None);
    // Needed to keep chaos monkey running to give cell a chance to finish
    talk_to_cell(&mut cell1, "Hello 1\n");
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let mut msgs = select_cell(&master_fds, &mut from_cell_fds);
    talk_to_cell(&mut cell2, "Hello 2\n");
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let mut msgs2 = select_cell(&master_fds, &mut from_cell_fds);
    msgs.append(&mut msgs2);
    for (cell_pid, msg) in &msgs {
        println!("Chaos monkey got From {} '{}'", cell_pid, msg);
    }
    println!("Chaos monkey sleeping to let cells finish");
    std::thread::sleep(std::time::Duration::from_secs(2)); // Let cells finish their work
    println!("Chaos monkey exiting");
    assert_eq!(msgs.len(), 2);
}
fn select_cell<'a>(
    master_fds: &FdSet,
    from_cell_from_fd_raw: &mut HashMap<i32, (u32, &'a mut ChildStdout)>,
) -> Vec<(u32, String)> {
    random_sleep("Chaos monkey", std::process::id());
    let mut msgs = Vec::new();
    let mut count = 0;
    let mut fdset_rd = master_fds.clone();
    let mut fdset_err = FdSet::new();
    println!("1: count {} \nfdset {:?}", count, fdset_rd);
    match select(None, &mut fdset_rd, None, &mut fdset_err, None) {
        Ok(r) => println!("Success: {} fd ready", r),
        Err(e) => println!("Failure: {}\nfdset_rd {:?}", e, fdset_rd),
    }
    println!("2: count {} \nfdset_rd {:?}", count, fdset_rd);
    for fd_raw in fdset_rd.fds(None) {
        count = count + 1;
        println!("3: count {}, fd {}", count, fd_raw);
        let cell_info = from_cell_from_fd_raw
            .get_mut(&fd_raw)
            .expect("from_cell_fds error");
        let cell_pid = cell_info.0;
        let fd = &mut *cell_info.1;
        let mut reader = BufReader::new(fd).lines();
        let msg = match reader.next() {
            Some(m) => match m {
                Ok(msg) => {
                    msgs.push((cell_pid, msg.clone()));
                    msg
                }
                Err(e) => format!("Read error {}", e),
            },
            None => format!("Empty message"),
        };
        println!("Message: {}", msg);
        println!("4: count {}", count);
    }
    msgs
}
