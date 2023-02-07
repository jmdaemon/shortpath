use serde::{Serialize, Deserialize};
use crate::{shortpaths::{SP, Shortpath, expand_shortpath}, config::{Config, read_config, write_config}, helpers::{expand_tilde, find_longest_keyname, tab_align}};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ShortpathsBuilder {
    pub shortpaths: Option<SP>,
    #[serde(skip)]
    pub cfg: Option<Config>,
}

pub trait ShortpathOperationsExt {
    /// Expand special chracters for shortpaths
    /// These include mapping '~' to the user's home directory.
    fn expand_special_characters(&self) -> SP;

    /// Expand shortpaths to full_paths at runtime
    fn populate_expanded_paths(&self) -> SP;

    /// Horizontally align shortpaths
    fn tab_align_paths(&self) -> SP;
}

impl ShortpathOperationsExt for SP {
    fn expand_special_characters(&self) -> SP {
        //let mut shortpaths: SP = self.shortpaths.iter().map(|(name, sp)| {
        let shortpaths: SP = self.iter().map(|(name, sp)| {
            let path = expand_tilde(&sp.path).unwrap();
            let shortpath = Shortpath { full_path: Some(path), ..sp.to_owned() };
            (name.to_owned(), shortpath)
        }).collect();
        shortpaths
    }

    fn populate_expanded_paths(&self) -> SP {
        //shortpaths.iter().map(|(k, sp)| {
        self.iter().map(|(k, sp)| {
            let full_path = expand_shortpath(sp, self);
            let shortpath = Shortpath{ full_path: Some(full_path), ..sp.to_owned()};
            (k.to_owned(), shortpath)
        }).collect()
    }

    fn tab_align_paths(&self) -> SP {
        let width = find_longest_keyname(self).len();
        //self.shortpaths = shortpaths;

        //let conts = toml::to_string_pretty(&self).expect("Could not serialize shortpaths");
        //let delim = " = ";
        
        self.into_iter().map(|(name, sp)| {
            //let aligned = tab_align(name, width, delim);
            let aligned = tab_align(name, width, None);
            (aligned, sp.to_owned())
            //let output = format!("{}{}\n", aligned);
            //let (key, path) = value;
            //let aligned = tab_align(key, width, delim);
            //trace!("{}", &aligned);
            //let output = format!("{}{}\n", aligned, path);
            //trace!("{}", &output);
            //return output
        }).collect()
        
        /*
        let fileconts: Vec<String> = conts.split('\n').map(|line| {
            if let Some(value) = line.split_once(delim) {
                let (key, path) = value;
                let aligned = tab_align(key, width, delim);
                trace!("{}", &aligned);
                let output = format!("{}{}\n", aligned, path);
                trace!("{}", &output);
                return output
            }
            format!("{}\n", line)
        }).collect();
        */
        //self.to_owned()
    }

}

impl ShortpathsBuilder {
    // TODO: Use FromIterator trait extension
    pub fn new() -> ShortpathsBuilder  {
        Default::default()
    }

    pub fn build(&mut self) -> Option<SP> {
        if let Some(shortpaths) = &mut self.shortpaths {
            let shortpaths = shortpaths
                .expand_special_characters()
                .populate_expanded_paths();
            return Some(shortpaths);
        }
        None
    }

    pub fn with_config(mut self, file: &str) -> Self {
        let mut config = Config::new();
        config.add_config(file.to_owned(), file);
        self.cfg = Some(config);
        self
    }

    pub fn read_shortpaths_from(self, file: &str) -> Self {
        assert!(self.cfg.is_some());
        let cfg = self.cfg.unwrap();
        let toml_conts = read_config(&cfg, file);

        let sp = toml::from_str(&toml_conts);
        assert!(sp.is_ok());
        let sp: ShortpathsBuilder = sp.unwrap();
        ShortpathsBuilder { cfg: Some(cfg), ..sp }
    }
}

pub fn to_disk(shortpaths: SP, cfg: &Config, file: &str) {
    //let result = write_config(&self.cfg, CONFIG_FILE_PATH, &conts);
    let conts = toml::to_string_pretty(&shortpaths).expect("Could not serialize shortpaths");
    let result = write_config(cfg, file, &conts);
    if let Err(e) = result {
        eprintln!("Failed to write shortpaths config to disk: {}", e);
    }
    println!("Wrote shortpaths config to {}", cfg.files.get(file).unwrap().display());
}
