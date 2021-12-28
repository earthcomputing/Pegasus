use nix::unistd::{pipe, unlink};
use users::get_current_username;

use pegasus::cell::Cell;
use pegasus::utility::{keep_alive, talk_to_cell};

#[test]
fn cell2cell() {
    println!(
        "Running as user {:?}",
        get_current_username().expect("No username")
    );
    let (from_cell1, to_cell2) = pipe().expect("Can't create pipe 1");
    let (from_cell2, to_cell1) = pipe().expect("Can't create pipe 2");
    let socket_name1 = share_stream("cell1", from_cell2, to_cell2).expect("Can't share stream1");
    let socket_name2 = share_stream("cell2", from_cell1, to_cell1).expect("Can't share stream2");
    let program_plus_args = ["cargo", "test", "chaos_monkey", "--bin", "cell", "--"];
    let mut cell1 = Cell::new("cell1", &program_plus_args, &socket_name1);
    let mut cell2 = Cell::new("cell2", &program_plus_args, &socket_name2);
    talk_to_cell(cell1.pid, &mut cell1.cell_stdin, None);
    talk_to_cell(cell2.pid, &mut cell2.cell_stdin, None);
    keep_alive(None, "Enter anything to have cell2cell exit");
    println!("cell2cell exiting");
}
