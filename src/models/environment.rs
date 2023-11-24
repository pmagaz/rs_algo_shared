#[derive(PartialEq, Debug)]
pub enum Environment {
    Production,
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
