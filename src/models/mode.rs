#[derive(PartialEq, Debug)]
pub enum ExecutionMode {
    Scanner,
    ScannerBackTest,
    BackTest,
    Bot,
    BotBackTest,
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

pub fn from_str(execution_mode: &str) -> ExecutionMode {
    match execution_mode {
        "Scanner" => ExecutionMode::Scanner,
        "BackTest" => ExecutionMode::BackTest,
        "ScannerBackTest" => ExecutionMode::ScannerBackTest,
        "Bot" => ExecutionMode::Bot,
        "BotBackTest" => ExecutionMode::BotBackTest,
        _ => {
            log::error!("No {} EXECUTION_MODE found!", &execution_mode);
            panic!();
        }
    }
}
