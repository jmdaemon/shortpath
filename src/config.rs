use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
};

use std::{
    path::PathBuf,
    collections::HashMap,
    fs::create_dir_all,
};

use derivative::Derivative;
use directories::ProjectDirs;

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Config {
    #[derivative(Default(value = "ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).expect(\"Could not initialize config\")"))]
    project_dirs: ProjectDirs,
    pub files: HashMap<String, PathBuf>
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config::default();
        config.files = HashMap::new();
        config
    }

    pub fn make_config_directory(&self) {
        create_dir_all(self.project_dirs.config_dir()).expect("Could not create config directory")
    }

    pub fn format_config_path(&self, config: impl Into<String>) -> PathBuf {
        self.project_dirs.config_dir().to_path_buf().join(config.into())
    }

    pub fn add_config(&mut self, key: String, file: &str) {
        self.files.insert(key, self.format_config_path(file));
    }
}
