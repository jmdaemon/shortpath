use crate::{
    consts::PROGRAM_NAME,
    export::Export,
    shortpaths::{Shortpaths, fold_shortpath, expand_shortpath},
};

use std::{
    path::PathBuf,
    cmp::Ordering,
};

use bimap::BiHashMap;

/* NOTE
 * It may be a good idea in the future to generate a proper bash completions
 * file instead of just generating the aliases script.
 */
pub struct BashExporter {
    spaths: Shortpaths,
}

pub fn fmt_bash_alias(name: &str, path: &PathBuf) -> String {
    format!("alias {}=\"{}\"\n", name, path.display())
}

impl BashExporter {
    pub fn new(spaths: Shortpaths) -> BashExporter {
        BashExporter { spaths }
    }

    pub fn default() -> BashExporter {
        BashExporter::new(BiHashMap::new())
    }

    pub fn serialize_bash(&self) -> String {
        let mut output: String = String::new();
        output += "#!/bin/bash\n\n";

        let sp = &self.spaths;

        let mut m = Vec::from_iter(sp.into_iter().map(|(_,v)| expand_shortpath(v, sp)));

        m.sort_by(|a, b| {
            let (sa, sb) = (a.to_str().unwrap(), b.to_str().unwrap());
            let (la, lb) = (sa.len(), sb.len());
            if la < lb {
                Ordering::Less
            } else if la == lb {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });

        for p in m {
            let path = fold_shortpath(&p, &sp);
            let name = sp.get_by_right(&path).unwrap();
            output += &fmt_bash_alias(name, &path);
        }
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
}

// Unit Tests
#[test]
fn test_serialize_bash() {
    use std:: path::Path;

    let mut bexp = BashExporter::new(BiHashMap::new());
    bexp.spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());

    //let mut spaths: Shortpaths = BiHashMap::new();
    //spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let actual = serialize_bash(&spaths);
    let actual = bexp.gen_completions();
    let expect = "#!/bin/bash\n\naaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
