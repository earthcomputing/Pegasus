fn main() {
    println!("Hello from Pegasus");
    let cell_id1 = "Cell:0";
    let cell_id2 = "Cell:1";
    let mut cell1 = create_cell(cell_id1);
    let mut cell2 = create_cell(cell_id2);
    // Needed to keep simulator running to give cell a chance to finish
    std::thread::sleep(std::time::Duration::from_secs(2));
    let reply1 = talk_to_cell(&mut cell1);
    let reply2 = talk_to_cell(&mut cell2);
    println!("Simulator got {} from {}", reply1, cell_id1);
    println!("Simulator got {} from {}", reply2, cell_id2);
}
fn create_cell(cell_id: &'static str) -> std::process::Child {
    println!("Starting cell {}", cell_id);
    let cell = std::process::Command::new("target/debug/cell")
        .arg(cell_id)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Could not spawn cell");
    println!("{} has PID {}", cell_id, cell.id());
    cell
}
fn talk_to_cell(cell: &mut std::process::Child) -> String {
    use std::io::{BufRead, BufReader, Write};
    let pid = cell.id();
    println!("Simulator talking to cell with PID {}", pid);
    let to_cell = cell.stdin.as_mut().expect("Cannot get cell stdin");
    let from_cell = cell.stdout.as_mut().expect("Cannot get cell stdout");
    to_cell
        .write(b"Hello from simulator\n")
        .expect("Cannot write to cell");
    pegasus::utility::random_sleep("Simulator", std::process::id());
    println!("Simulator listening to cell with PID {}", pid);
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
        let mut cell = create_cell("Cell:Test");
        let msg = talk_to_cell(&mut cell);
        assert_eq!(cell.id().to_string(), msg);
    }
}
