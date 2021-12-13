use pegasus::cell::Cell;
use pegasus::utility::{keep_alive, select_cell, setup_fds, talk_to_cell};
#[test]
// Make sure all clients have sent their results before trying to read from them
fn chaos_monkey_all_fds_ready() {
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--", "--nocapture"];
    let mut cell1 = Cell::new(cell_id1, &program_plus_args, None);
    let mut cell2 = Cell::new(cell_id2, &program_plus_args, None);
    // Needed to keep chaos monkey running to give cell a chance to finish
    talk_to_cell(&mut cell1, "Hello 1\n");
    talk_to_cell(&mut cell2, "Hello 2\n");
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let msgs = select_cell(&master_fds, &mut from_cell_fds).expect("Select error");
    for (cell_pid, msg) in &msgs {
        println!("Chaos monkey got From {} '{}'", cell_pid, msg);
    }
    std::thread::sleep(std::time::Duration::from_secs(3));
    let test_results = select_cell(&master_fds, &mut from_cell_fds).expect("Select error");
    for (cell_pid, test_result) in &test_results {
        assert_eq!(test_result, "test tests::chaos_monkey ... ok");
        println!("{}: {}", cell_pid, test_result);
    }
    assert_eq!(msgs.len(), 2);
    keep_alive("Enter anything to have chaos_monkey exit");
    println!("Chaos monkey exiting");
}
#[test]
// Make sure only one client message is ready at a time
fn chaos_monkey_some_fds_ready() {
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--", "--nocapture"];
    let mut cell1 = Cell::new(cell_id1, &program_plus_args, None);
    let mut cell2 = Cell::new(cell_id2, &program_plus_args, None);
    // Needed to keep chaos monkey running to give cell a chance to finish
    talk_to_cell(&mut cell1, Some("Hello 1\n"));
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let mut msgs = select_cell(&master_fds, &mut from_cell_fds).expect("Select error 1");
    talk_to_cell(&mut cell2, Some("Hello 2\n"));
    std::thread::sleep(std::time::Duration::from_secs(3));
    let mut cells = vec![&mut cell1, &mut cell2];
    let (master_fds, mut from_cell_fds) = setup_fds(&mut cells);
    let mut msgs2 = select_cell(&master_fds, &mut from_cell_fds).expect("Select error 2");
    msgs.append(&mut msgs2);
    for (cell_pid, msg) in &msgs {
        println!("Chaos monkey got From {} '{}'", cell_pid, msg);
    }
    std::thread::sleep(std::time::Duration::from_secs(3));
    let test_results = select_cell(&master_fds, &mut from_cell_fds).expect("Select error");
    for (cell_pid, test_result) in &test_results {
        assert_eq!(test_result, "test tests::chaos_monkey ... ok");
        println!("{}: {}", cell_pid, test_result);
    }
    assert_eq!(msgs.len(), 2);
    keep_alive("Enter anything to have chaos_monkey exit");
    println!("Chaos monkey exiting");
}

