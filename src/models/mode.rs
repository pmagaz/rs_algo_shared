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
            ExecutionMode::Bot | ExecutionMode::BotBackTest => true,
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
    match execution_mode.to_lowercase().as_str() {
        "scanner" => ExecutionMode::Scanner,
        "backtest" => ExecutionMode::BackTest,
        "scannerbacktest" => ExecutionMode::ScannerBackTest,
        "bot" => ExecutionMode::Bot,
        "botbacktest" => ExecutionMode::BotBackTest,
        _ => {
            log::error!("No {} EXECUTION_MODE found!", &execution_mode);
            panic!();
        }
    }
}
