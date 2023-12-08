use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[derive(Default)]
pub enum Environment {
    Production,
    #[default]
    Development,
    Backtesting,
}

impl Environment {
    pub fn is_prod(&self) -> bool {
        match *self {
            Environment::Production => true,
            _ => false,
        }
    }

    pub fn is_dev(&self) -> bool {
        match *self {
            Environment::Development => true,
            _ => false,
        }
    }

    pub fn is_backtest(&self) -> bool {
        match *self {
            Environment::Backtesting => true,
            _ => false,
        }
    }

    pub fn value(&self) -> String {
        match self {
            Environment::Production => "Production".to_owned(),
            Environment::Development => "Development".to_owned(),
            Environment::Backtesting => "Backtesting".to_owned(),
        }
    }
}



pub fn from_str(env: &str) -> Environment {
    match env.to_lowercase().as_str() {
        "production" => Environment::Production,
        "development" => Environment::Development,
        "backtesting" => Environment::Backtesting,
        _ => {
            log::error!("No {} env found!", env);
            panic!();
        }
    }
}
