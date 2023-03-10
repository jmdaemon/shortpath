use crate::consts::{
    QUALIFIER,
    ORGANIZATION,
    APPLICATION,
};

use std::{
    path::PathBuf,
    fs::{create_dir_all, read_to_string, write},
};

use derivative::Derivative;
use directories::ProjectDirs;

#[derive(Derivative)]
#[derivative(Debug, Default, Clone)]
pub struct Config {
    #[derivative(Default(value = "ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).expect(\"Could not initialize config\")"))]
    pub project_dirs: ProjectDirs,
    pub path: PathBuf
}

impl Config {
    pub fn new(file: impl Into<String>) -> Config {
        let cfg = Config::default();
        Config {path: cfg.format_path(file.into()), ..cfg}
    }

    fn format_path(&self, config: impl Into<String>) -> PathBuf {
        self.project_dirs.config_dir().to_path_buf().join(config.into())
    }

    pub fn make_dirs(&self) {
        create_dir_all(self.project_dirs.config_dir()).expect("Could not create config directory")
    }

    pub fn read(&self) -> String {
        read_to_string(&self.path).expect("Could not read config file.")
    }

    pub fn write(&self, conts: &str) -> std::io::Result<()> {
        write(&self.path, conts)
    }
}
