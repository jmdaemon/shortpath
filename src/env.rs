use std::env::vars;
use indexmap::IndexMap;

use crate::shortpaths::SP;

// We have a set of environment variables
pub type EP = IndexMap<String, String>;

#[derive(Clone, Debug)]
pub struct EnvVars {
    pub vars: EP,
}

/// Get the hashmap of every environment variable available
pub fn env_vars() -> EP {
    let mut ep: EP = IndexMap::new();
    vars().into_iter().for_each(|(var_name, var_path)| {
        ep.insert(var_name, var_path);
    });
    ep
}

/// Return the set of environment variables not in the set of shortpaths
pub fn unique_vars(shortpaths: &SP, envpaths: &EP) -> EP {
    let envpaths: EP = envpaths.into_iter().filter_map(|(var_name, var_path)| {
        if shortpaths.contains_key(&var_name.to_owned()) {
            None
        } else {
            Some((var_name.to_owned(), var_path.to_owned()))
        }
    }).collect();
    envpaths
}

pub trait EnvPathOperationsExt{
    fn unique(&self, shortpaths: &SP) -> EP;
    fn non_null(self) -> EP;
}

impl EnvPathOperationsExt for EP {
    fn unique(&self, shortpaths: &SP) -> EP {
        unique_vars(shortpaths, self)
    }

    fn non_null(self) -> EP {
        let envpaths: EP = self.into_iter().filter(|(_, envpath)| {
            !envpath.is_empty()
        }).collect();
        envpaths
    }
}

impl Default for EnvVars {
    fn default() -> Self {
        EnvVars { vars: env_vars() }
    }
}

impl EnvVars {
    pub fn new() -> EnvVars {
        Default::default()
    }
}
