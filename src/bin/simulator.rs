use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};
fn main() {
    println!("Hello from Pegasus");
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let mut cell1 = create_cell(cell_id1);
    let mut cell2 = create_cell(cell_id2);
    // Needed to keep simulator running to give cell a chance to finish
    let reply1 = talk_to_cell(&mut cell1, "Hello 1\n");
    let reply2 = talk_to_cell(&mut cell2, "Hello 2\n");
    println!("Simulator got '{}' from {}", reply1, cell_id1);
    println!("Simulator got '{}' from {}", reply2, cell_id2);
}
fn create_cell(cell_id: &'static str) -> Child {
    println!("Starting cell {}", cell_id);
    let cell = Command::new("target/debug/cell")
        .arg(cell_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Could not spawn cell");
    println!("{} has PID {}", cell_id, cell.id());
    cell
}
fn talk_to_cell(cell: &mut Child, msg: &str) -> String {
    let pid = cell.id();
    println!("Simulator talking to cell with PID {}", pid);
    let to_cell = cell.stdin.as_mut().expect("Cannot get cell stdin");
    to_cell.write(msg.as_bytes()).expect("Cannot write to cell");
    println!("Simulator listening to cell with PID {}", pid);
    let from_cell = cell.stdout.as_mut().expect("Cannot get cell stdout");
    let mut reader = BufReader::new(from_cell).lines();
    let msg = reader
        .next()
        .expect("No message from cell")
        .expect("Cannot read from cell");
    msg
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comm_test() {
        let test_msg = "test message\n";
        let mut cell = create_cell("Cell:Test");
        let msg = talk_to_cell(&mut cell, test_msg);
        assert_eq!(test_msg.trim(), msg);
    }
}
