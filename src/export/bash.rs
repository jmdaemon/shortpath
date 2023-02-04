use crate::{
    consts::PROGRAM_NAME,
    export::Export,
};

use crate::sp::{expand_tilde, SP, sort_shortpaths};

use std::path::PathBuf;

/* NOTE: Consider generating an actual bash completions
   file instead of just generating the aliases script. */
pub struct BashExporter {
    shortpaths: Option<SP>,
}

pub fn fmt_bash_alias(name: &str, path: &PathBuf) -> String {
    format!("export {}=\"{}\"\n", name, path.display())
}

impl BashExporter {
    pub fn new(shortpaths: Option<SP>) -> BashExporter {
        BashExporter { shortpaths }
    }

    pub fn default() -> BashExporter {
        BashExporter::new(None)
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

    fn set_shortpaths(&mut self, shortpaths: &SP) {
        self.shortpaths = Some(sort_shortpaths(shortpaths.to_owned()));
    }
}

// Unit Tests
#[test]
fn test_serialize_bash() {
    use crate::sp::{Shortpath, SPT, ShortpathsBuilder};

    use log::LevelFilter;
    use pretty_env_logger::formatted_timed_builder;

    // Enable debug statements
    formatted_timed_builder().filter_level(LevelFilter::Trace).init();

    // Init
    let sp_paths = vec![
        Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
        Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
        Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
        Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
    ];
    let mut sp_builder = ShortpathsBuilder::new(sp_paths);

    let shortpaths = sp_builder.build().unwrap();

    let mut exp = BashExporter::default();
    exp.set_shortpaths(&shortpaths);

    // Test
    let actual = exp.gen_completions();
    let expect = "#!/bin/bash\n\nexport a=\"aaaa\"\nexport b=\"$a/bbbb\"\nexport d=\"$a/dddd\"\nexport c=\"$b/cccc\"\n";
    assert_eq!(actual, expect);
}
