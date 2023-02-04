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
    proj_dirs: ProjectDirs,
    pub config_files: HashMap<String, PathBuf>
}

impl Config {
    pub fn new() -> Config {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).expect("Could not initialize config");
        let config_files = HashMap::new();

        // Initialize config
        let config = Config { proj_dirs, config_files };
        config.init();
        config
    }

    fn init(&self) {
        let proj_cfg_dir = self.proj_dirs.config_dir();
        create_dir_all(proj_cfg_dir).expect("Could not create config directory")
    }

    pub fn format_config_path(&self, config: &str) -> PathBuf {
        self.proj_dirs.config_dir().to_path_buf().join(config)
    }

    pub fn add_config(&mut self, key: String, file: &str) {
        self.config_files.insert(key, self.format_config_path(file));
    }
}
