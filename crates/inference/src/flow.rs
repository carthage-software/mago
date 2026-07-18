#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlFlow {
    Fallthrough,
    Return,
    Break(u64),
    Continue(u64),
    Diverge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flow {
    pub reachable: bool,
    pub exit: ControlFlow,
}
