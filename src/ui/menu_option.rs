
use super::MenuResult;

pub enum MenuOption {
    WatchTv {
        name: String,
        url: String,
    },
}

impl MenuOption {
    pub fn name(&self) -> &str {
        match self {
            &MenuOption::WatchTv {ref name, ..} => { // TODO remove url
               name
            },
        }
    }

    pub fn execute(&self) -> MenuResult {
        match self {
            &MenuOption::WatchTv {ref name, ref url} => {
                MenuResult::AddGameView { name: name.clone(), url: url.clone() }
            },
            //_ => { MenuResult::None }
        }
    }
}
