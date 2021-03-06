fn main() {
    println!("Hello from Pegasus");
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        vec,
    };

    use pegasus::cell::Cell;
    use pegasus::utility::{keep_alive ,talk_to_cell};
    #[test]
    fn comm_test() {
        let test_msg = "test message\n";
        let program_plus_args = ["cargo", "test", "--bin", "cell", "--test", "chaos_monkey"];
        let mut cell = Cell::new("Cell", &program_plus_args, None);
        talk_to_cell(&mut cell, test_msg);
        let mut cells = vec![&mut cell];
        let msgs = listen_to_cells(&mut cells);
        println!("Simulator got {}", msgs[0]);
        assert_eq!(test_msg.trim(), &msgs[0]);
        keep_alive("Enter anything to tell the simulator to exit");
        println!("Simulator exiting");
    }
    fn listen_to_cells(cells: &mut [&mut Cell]) -> Vec<String> {
        cells
            .iter_mut()
            .map(|cell| {
                let pid = cell.pid;
                println!("Simulator listening to cell with PID {}", pid);
                let from_cell = &mut cell.cell_stdout;
                let mut reader = BufReader::new(from_cell).lines();
                reader
                    .next()
                    .expect("No message from cell")
                    .expect("Cannot read from cell")
            })
            .collect()
    }
}
