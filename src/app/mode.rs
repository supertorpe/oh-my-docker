#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Containers,
    ContainerDetails(String),
    Logs(String),
    Images,
    ImageRun(String),
    Shell(String),
    ShellConfig(String),
    Events,
    Statistics,
    Networks,
    Volumes,
    Help,
}

#[derive(Clone, Debug)]
pub struct ModeStack {
    stack: Vec<Mode>,
    max_depth: usize,
}

impl ModeStack {
    pub fn new() -> Self {
        Self {
            stack: vec![Mode::Containers],
            max_depth: 10,
        }
    }

    pub fn current(&self) -> &Mode {
        self.stack.last().unwrap_or(&Mode::Containers)
    }

    pub fn push(&mut self, mode: Mode) {
        if self.stack.len() >= self.max_depth {
            self.stack.remove(0);
        }
        self.stack.push(mode);
    }

    pub fn back(&mut self) -> Option<Mode> {
        if self.stack.len() > 1 {
            Some(self.stack.pop().unwrap())
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

impl Default for ModeStack {
    fn default() -> Self {
        Self::new()
    }
}
