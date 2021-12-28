use std::os::unix::prelude::FromRawFd;

use nix::unistd::pipe;

use pegasus::cell::Cell;
use pegasus::utility::{keep_alive, select_cell, share_stream, setup_fds, setup_fds_test, select_cell_test, talk_to_cell};
#[test]
// Make sure all clients have sent their results before trying to read from them
fn chaos_monkey_all_fds_ready() {
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let (from_cell1_raw, to_cell1_raw) = pipe().expect("Can't create pipe 1");
    let (from_cell2_raw, to_cell2_raw) = pipe().expect("Can't create pipe 2");
    let socket_name1 = share_stream("cell1", from_cell2_raw, to_cell2_raw).expect("Can't share stream1");
    let socket_name2 = share_stream("cell2", from_cell1_raw, to_cell1_raw).expect("Can't share stream2");
    let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--", "--nocapture"];
    let mut cell1 = Cell::new(cell_id1, to_cell1_raw, from_cell1_raw, &program_plus_args, &socket_name1);
    let mut cell2 = Cell::new(cell_id2, to_cell2_raw, from_cell2_raw, &program_plus_args, &socket_name2);
    let mut expected = vec!(
        "Hello 1",
        "Hello 2",
        "test tests::chaos_monkey ... ok",
        "test tests::chaos_monkey ... ok"
    );
    let from_cell1 = unsafe { std::fs::File::from_raw_fd(from_cell1_raw) };
    let from_cell2 = unsafe { std::fs::File::from_raw_fd(from_cell2_raw) };
    let mut cell_rxs = vec![(cell1.pid, &from_cell1), (cell2.pid, &from_cell2)];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cell_rxs);
    // Needed to keep chaos monkey running to give cell a chance to finish
    talk_to_cell(cell1.pid, to_cell1_raw, "Hello 1\n");
    talk_to_cell(cell2.pid, to_cell2_raw, "Hello 2\n");
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut found = Vec::new();
    let msgs = select_cell(&master_fds, &mut from_cell_fds).expect("Read error");
    for (cell_pid, msg) in &msgs {
        found.push(msg);
        println!("Chaos monkey got From {} '{}'", cell_pid, msg);
    }
    std::thread::sleep(std::time::Duration::from_secs(10));
    let mut cell_rxs = vec![
        (cell1.pid, &mut cell1.cell_stdout), 
        (cell2.pid, &mut cell2.cell_stdout)
    ];
    // Get empty message from test harness
    let (master_fds, mut from_cell_fds) = setup_fds_test(&mut cell_rxs);
    let test_results = select_cell_test(&master_fds, &mut from_cell_fds).expect("Select error");
    for (cell_pid, test_result) in &test_results {
        found.push(test_result);
        assert_eq!(test_result, "test tests::chaos_monkey ... ok");
        println!("{}: {}", cell_pid, test_result);
    }
    expected.sort();
    found.sort();
    assert_eq!(expected, found);
    keep_alive(Some(std::time::Duration::from_secs(5)), "Waiting for cells");
    println!("Chaos monkey exiting");
}
// #[test]
// // Make sure only one client message is ready at a time
// fn chaos_monkey_some_fds_ready() {
//     let mut expected = vec!(
//         "Hello 1",
//         "Hello 2",
//         "test tests::chaos_monkey ... ok"
//     );
//     expected.sort();
//     let mut found = Vec::new();
//     let cell_id1 = "Cell:0";
//     let cell_id2 = "Cell:1";
//     let (from_cell1, to_cell2) = pipe().expect("Can't create pipe 1");
//     let (from_cell2, to_cell1) = pipe().expect("Can't create pipe 2");
//     let socket_name1 = share_stream("cell1", from_cell2, to_cell2).expect("Can't share stream1");
//     let socket_name2 = share_stream("cell2", from_cell1, to_cell1).expect("Can't share stream2");
//     let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--"];
//     let mut cell1 = Cell::new("cell1", &program_plus_args, &socket_name1);
//     let mut cell2 = Cell::new("cell2", &program_plus_args, &socket_name2);
//     talk_to_cell(cell1.pid, &mut cell1.chaos_to_cell, None);
//     talk_to_cell(cell2.pid, &mut cell2.chaos_to_cell, None);
//     let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--", "--nocapture"];
//     let mut cell1 = Cell::new(cell_id1, &program_plus_args, None);
//     let mut cell2 = Cell::new(cell_id2, &program_plus_args, None);
//     // Needed to keep chaos monkey running to give cell a chance to finish
//     let mut cell_rxs = vec![(cell1.pid, cell1.chaos_from_cell), (cell2.pid, cell2.chaos_from_cell)];
//     let (master_fds, mut from_cell_fds) = setup_fds(&mut cell_rxs);
//     talk_to_cell(cell1.pid, &mut cell1.chaos_to_cell, Some("Hello 1\n"));
//     let mut msgs = select_cell(&master_fds, &mut from_cell_fds).expect("Select error 1");
//     talk_to_cell(cell2.pid, &mut cell2.chaos_to_cell, Some("Hello 2\n"));
//     std::thread::sleep(std::time::Duration::from_secs(3));
//     let mut msgs2 = select_cell(&master_fds, &mut from_cell_fds).expect("Select error 2");
//     msgs.append(&mut msgs2);
//     for (cell_pid, msg) in &msgs {
//         found.push(msg);
//         println!("Chaos monkey got From {} '{}'", cell_pid, msg);
//     }
//     std::thread::sleep(std::time::Duration::from_secs(3));
//     let test_results = select_cell(&master_fds, &mut from_cell_fds).expect("Select error");
//     for (cell_pid, test_result) in &test_results {
//         found.push(test_result);
//         assert_eq!(test_result, "test tests::chaos_monkey ... ok");
//         println!("{}: {}", cell_pid, test_result);
//     }
//     found.sort();
//     assert_eq!(expected, found);
//     keep_alive(None, "Enter anything to have chaos_monkey exit");
//     println!("Chaos monkey exiting");
// }

