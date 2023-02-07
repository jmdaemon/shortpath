use serde::{Serialize, Deserialize};
use crate::{
    shortpaths::{SP, Shortpath, expand_shortpath},
    config::Config,
    helpers::{expand_tilde, find_longest_keyname, tab_align}
};
use log::trace;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Shortpaths {
    pub shortpaths: SP,
    #[serde(skip)]
    pub cfg: Config,
}

#[derive(Default, Debug)]
pub struct ShortpathsBuilder {
    pub paths: Option<Shortpaths>,
    pub cfg: Option<Config>,
}

pub trait ShortpathsAlignExt {
    /// Horizontally align shortpaths for the shortpaths config file
    fn tab_align_paths(&self) -> String;
}

pub trait ShortpathOperationsExt {
    /// Expand special chracters for shortpaths
    /// These include mapping '~' to the user's home directory.
    fn expand_special_characters(&self) -> SP;

    /// Expand shortpaths to full_paths at runtime
    fn populate_expanded_paths(&self) -> SP;
}

impl ShortpathsAlignExt for Shortpaths {
    fn tab_align_paths(&self) -> String {
        let width = find_longest_keyname(&self.shortpaths).len();
        let delim = " = ";

        let conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        let conts: Vec<String> = conts.split('\n').map(|line| {
            if let Some(value) = line.split_once(delim) {
                let (key, path) = value;
                let aligned = tab_align(key, width, Some(delim));
                trace!("{}", &aligned);
                let output = format!("{}{}\n", aligned, path);
                trace!("{}", &output);
                return output
            }
            format!("{}\n", line)
        }).collect();
        let conts = conts.join("").strip_suffix('\n').unwrap().to_owned();
        conts
    }
}

impl ShortpathOperationsExt for SP {
    fn expand_special_characters(&self) -> SP {
        let shortpaths: SP = self.iter().map(|(name, sp)| {
            let path = expand_tilde(&sp.path).unwrap();
            let shortpath = Shortpath { full_path: Some(path), ..sp.to_owned() };
            (name.to_owned(), shortpath)
        }).collect();
        shortpaths
    }

    fn populate_expanded_paths(&self) -> SP {
        self.iter().map(|(k, sp)| {
            let full_path = expand_shortpath(sp, self);
            let shortpath = Shortpath{ full_path: Some(full_path), ..sp.to_owned()};
            (k.to_owned(), shortpath)
        }).collect()
    }
}

impl ShortpathsBuilder {
    // TODO: Use FromIterator trait extension
    pub fn new() -> ShortpathsBuilder  {
        Default::default()
    }

    pub fn build(mut self) -> Option<Shortpaths> {
        if let Some(paths) = &mut self.paths {
            let shortpaths = paths.shortpaths
                .expand_special_characters()
                .populate_expanded_paths();
            let paths = Shortpaths { shortpaths, cfg: self.cfg.unwrap()};
            return Some(paths);
        }
        None
    }

    pub fn with_config(mut self, file: &str) -> Self {
        self.cfg = Some(Config::new(file));
        self
    }

    pub fn read_shortpaths(self) -> Self {
        assert!(self.cfg.is_some());
        let cfg = self.cfg.unwrap();
        let toml_conts = cfg.read_config();

        let sp = toml::from_str(&toml_conts);
        assert!(sp.is_ok());
        let sp: Shortpaths = sp.unwrap();
        ShortpathsBuilder { cfg: Some(cfg), paths: Some(sp)}
    }
}

pub fn to_disk(paths: Shortpaths) {
    let conts = paths.tab_align_paths();
    let result = paths.cfg.write_config(&conts);
    if let Err(e) = result {
        eprintln!("Failed to write shortpaths config to disk: {}", e);
    }
    println!("Wrote shortpaths config to {}", paths.cfg.file.display());
}
