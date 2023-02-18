use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use crate::{
    shortpaths::{SP, Shortpath, expand_shortpath, parse_env_alias, substitute_env_path},
    config::Config,
    helpers::{expand_tilde, find_longest_keyname, tab_align, sort_shortpaths, getenv}, env::{EnvVars, env_vars}
};
use log::{trace, info, debug};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Shortpaths {
    pub shortpaths: SP,
    #[serde(skip)]
    pub env_vars: Option<EnvVars>,
    #[serde(skip)]
    pub cfg: Option<Config>,
}

#[derive(Default, Debug)]
pub struct ShortpathsBuilder {
    pub paths: Option<Shortpaths>,
    pub cfg: Option<Config>,
}

pub trait ShortpathsAlignExt {
    /// Horizontally align shortpaths for the shortpaths config file
    fn tab_align_paths(&self) -> String;
    fn fold_env_paths(self) -> Shortpaths;
}

pub trait ShortpathOperationsExt {
    /// Expand special chracters for shortpaths
    /// These include mapping '~' to the user's home directory.
    fn expand_special_characters(&self) -> SP;

    /// Expand shortpaths to full_paths at runtime
    fn populate_expanded_paths(&self) -> SP;

    /// Sort shortpaths in lexicographical order of the expanded paths
    fn sort_paths(&self) -> SP;

    /// Same as sort_paths but without creating a copy
    fn sort_paths_inplace(&mut self) -> SP;
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

    fn fold_env_paths(self) -> Shortpaths {
        let evars = self.env_vars.unwrap();
        let shortpaths: SP = self.shortpaths.into_iter().map(|(name, mut sp)| {
            let mut path = sp.path.to_str().unwrap().to_owned();
            evars.vars.iter().for_each(|(envname, envpath)| {
                debug!("envname: {}", envname);
                debug!("envpath: {}", envpath);
                if !envpath.is_empty() && path.contains(envpath) {
                    //let this = format!("${{env:{}}}", envpath);
                    //let with = envname;
                    let this = envpath;
                    let with = format!("${{env:{}}}", envname);
                    path = path.replace(this, &with);
                }
            });
            sp.path = PathBuf::from(path);
            (name, sp)
        }).collect();

        Shortpaths { shortpaths, env_vars: Some(evars), ..self}
    }
}

impl ShortpathOperationsExt for SP {
    fn expand_special_characters(&self) -> SP {
        info!("expand_special_characters()");
        let shortpaths: SP = self.iter().map(|(name, sp)| {
            let path = if let Some(full_path) = &sp.full_path {
                full_path
            } else {
                &sp.path
            };
            let expanded = expand_tilde(path).unwrap();
            debug!("{}: {} -> {}", &name, &path.display(), &expanded.display());
            let shortpath = Shortpath { full_path: Some(expanded), ..sp.to_owned() };
            (name.to_owned(), shortpath)
            
        }).collect();
        debug!("");
        shortpaths
    }

    fn populate_expanded_paths(&self) -> SP {
        info!("populate_expanded_paths()");
        self.iter().map(|(k, sp)| {
            let full_path = expand_shortpath(sp, self);
            let shortpath = Shortpath{ full_path: Some(full_path), ..sp.to_owned()};
            info!("Final Shortpath {:?}", shortpath);
            (k.to_owned(), shortpath)
        }).collect()
    }

    fn sort_paths(&self) -> SP { sort_shortpaths(self.to_owned()) }

    fn sort_paths_inplace(&mut self) -> SP {
        self.sort_by(|_, v1, _, v2| {
            v1.cmp(v2)
        });
        self.to_owned()
    }
}

impl From<SP> for ShortpathsBuilder {
    fn from(item: SP) -> Self {
        let shortpaths = Shortpaths { shortpaths: item, cfg: None, env_vars: None};
        ShortpathsBuilder { paths: Some(shortpaths), cfg: None }
    }
}

impl ShortpathsBuilder {
    pub fn new() -> ShortpathsBuilder  { Default::default() }

    pub fn build(mut self) -> Option<Shortpaths> {
        if let Some(paths) = &mut self.paths {
            let shortpaths = paths.shortpaths
                .populate_expanded_paths()
                .expand_special_characters()
                .sort_paths_inplace();
            let env_vars = Default::default();
            let paths = Shortpaths { shortpaths, cfg: self.cfg, env_vars: Some(env_vars)};
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

    pub fn shortpath(mut self, key: impl Into<String>, path: impl Into<String>) -> Self {
        let mut shortpaths = self.paths.unwrap().shortpaths;
        let path = PathBuf::from(path.into());
        let sp = Shortpath::new(path, None);
        shortpaths.insert(key.into(), sp);
        self.paths = Some(Shortpaths { shortpaths, cfg: None, env_vars: None});
        self
    }
}

/// Saves the current shortpath.toml configuration to disk
pub fn to_disk(paths: Shortpaths) {
    let paths = paths.fold_env_paths();
    let conts = paths.tab_align_paths();
    let cfg = paths.cfg.expect("Config was empty");
    let result = cfg.write_config(&conts);
    if let Err(e) = result {
        eprintln!("Failed to write shortpaths config to disk: {}", e);
    }
    info!("Wrote shortpaths config to {}", cfg.file.display());
}
