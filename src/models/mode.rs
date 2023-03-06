#[derive(PartialEq, Debug)]
pub enum ExecutionMode {
    Scanner,
    ScannerBackTest,
    BackTest,
    Bot,
}

impl ExecutionMode {
    pub fn is_bot(&self) -> bool {
        match *self {
            ExecutionMode::Bot => true,
            _ => false,
        }
    }

    pub fn is_back_test(&self) -> bool {
        match *self {
            ExecutionMode::BackTest => true,
            _ => false,
        }
    }
}

pub fn from_str(strategy: &str) -> ExecutionMode {
    match strategy {
        "Scanner" => ExecutionMode::Scanner,
        "BackTest" => ExecutionMode::BackTest,
        "ScannerBackTest" => ExecutionMode::ScannerBackTest,
        "Bot" => ExecutionMode::Bot,
        _ => ExecutionMode::Bot,
    }
}
