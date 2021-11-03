use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Cell {
    pub pid: u32,
    pub process: Child,
    pub to_cell: ChildStdin,
    pub from_cell: ChildStdout,
}
impl Cell {
    pub fn new(cell_id: &'static str, program_opt: Option<&'static str>) -> Cell {
        let program = program_opt.unwrap_or("target/debug/cell");
        let mut child = Command::new(program)
            .arg(cell_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not spawn cell");
        let child_id = child.id();
        println!("{} has PID {}", cell_id, child.id());
        let to_cell = child
            .stdin
            .take()
            .expect(&format!("Can't get stdout for {}", cell_id));
        let from_cell = child
            .stdout
            .take()
            .expect(&format!("Can't get stdin for {}", cell_id));
        Cell {
            pid: child_id,
            process: child,
            to_cell,
            from_cell,
        }
    }
}
