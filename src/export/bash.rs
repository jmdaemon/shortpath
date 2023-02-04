use crate::{
    consts::PROGRAM_NAME,
    export::Export,
    shortpaths::{Shortpaths, fold_shortpath, expand_shortpath},
};

use crate::sp::{expand_tilde, SP};

use std::path::PathBuf;
//use std::collections::HashMap;

use log::trace;
use bimap::BiHashMap;

/* NOTE
 * It may be a good idea in the future to generate a proper bash completions
 * file instead of just generating the aliases script.
 */
pub struct BashExporter {
    spaths: Shortpaths,
    shortpaths: Option<SP>,
}

pub fn fmt_bash_alias(name: &str, path: &PathBuf) -> String {
    format!("export {}=\"{}\"\n", name, path.display())
}

impl BashExporter {
    pub fn new(spaths: Shortpaths) -> BashExporter {
        BashExporter { spaths, shortpaths: None}
    }

    pub fn default() -> BashExporter {
        BashExporter::new(BiHashMap::new())
    }

    pub fn serialize_bash(&self) -> String {
        let mut output = String::from("#!/bin/bash\n\n");
        let sp = &self.spaths;

        // Sort length of String
        trace!("Expanding shortpaths");

        let m =
            sp.into_iter()
            .map(|(k,v)| {
                (k.to_owned(), expand_shortpath(v, sp).unwrap_or(v.to_owned()))
            })
            .into_iter()
            .collect::<BiHashMap<String, PathBuf>>();

        let mut v = 
            //sp.into_iter()
            //m.into_iter()
            m.iter()
            .map(|(_,v)| v.to_owned())
            .collect::<Vec<PathBuf>>();

        let get_length = |p: &PathBuf| { p.to_str().unwrap().len() };
        //m.sort_by(|a, b| {
        v.sort_by(|a, b| {
            let (la, lb) = (get_length(a), get_length(b));
            la.cmp(&lb)
       });

        trace!("Folding shortpaths");
        //for p in m {
        for p in v {
            dbg!(p.display());
            let name = m.get_by_right(&p).unwrap();
            let path = fold_shortpath(name, p.as_path(), sp);

            // Expand all environment variables
            let path = expand_tilde(path).unwrap();
            //let name = sp.get_by_right(&path).unwrap();

            trace!("name: {}", name);
            trace!("path: {}", path.display());
            let serialized = fmt_bash_alias(&name, &path);
            trace!("serialized: {}", serialized);
            output += &serialized;
        }
        trace!("output: {}", output);
        output
    }
}

impl Export for BashExporter {
    /// Get the default local platform independent shell completions path 
    fn get_completions_path(&self) -> String {
        format!("completions/{}.bash", PROGRAM_NAME)
    }

    /// Get the system shell completions file path
    fn get_completions_sys_path(&self) -> String {
        format!("/usr/share/bash-completion/completions/{}", PROGRAM_NAME)
    }

    /** Let only users with equal permissions edit
      * the shell completions file */
    fn set_completions_fileperms(&self) -> String {
        todo!("Set user completion file perms");
        //String::from("")
    }

    fn gen_completions(&self) -> String {
        self.serialize_bash()
    }

    fn set_shortpaths(&mut self, spaths: &Shortpaths) {
        self.spaths = spaths.clone();
    }

    fn set_shortpaths_imap(&mut self, shortpaths: &SP) {
        self.shortpaths = Some(shortpaths.to_owned());
    }

    fn gen_completions_imap(&self) -> String {
        let mut output = String::from("#!/bin/bash\n\n");
        if let Some(shortpaths) = &self.shortpaths {
            let serialized: Vec<String> = shortpaths.iter().map(|(name, sp)| {
                let path = expand_tilde(sp.path()).unwrap();
                let shortpath = fmt_bash_alias(&name, &path);
                shortpath
            }).collect();

            //let mut output = String::from("#!/bin/bash\n\n");
            serialized.iter().for_each(|line| {
                output += line;
            });
            println!("output: {}", output);
        }
        output
    }
}

// Unit Tests
#[test]
fn test_serialize_bash() {
    use std:: path::Path;
    use log::LevelFilter;
    use pretty_env_logger::formatted_timed_builder;

    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    let mut bexp = BashExporter::new(BiHashMap::new());
    bexp.spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());

    //let mut spaths: Shortpaths = BiHashMap::new();
    //spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let actual = serialize_bash(&spaths);
    let actual = bexp.gen_completions();
    let expect = "#!/bin/bash\n\nexport aaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
