use crate::{
    consts::PROGRAM_NAME,
    export::Export,
    shortpaths::{Shortpaths, fold_shortpath, expand_shortpath},
};

use std::path::PathBuf;

use log::trace;
use bimap::BiHashMap;

/* NOTE
 * It may be a good idea in the future to generate a proper bash completions
 * file instead of just generating the aliases script.
 */
pub struct BashExporter {
    spaths: Shortpaths,
}

pub fn fmt_bash_alias(name: &str, path: &PathBuf) -> String {
    format!("export {}=\"{}\"\n", name, path.display())
}

impl BashExporter {
    pub fn new(spaths: Shortpaths) -> BashExporter {
        BashExporter { spaths }
    }

    pub fn default() -> BashExporter {
        BashExporter::new(BiHashMap::new())
    }

    pub fn serialize_bash(&self) -> String {
        //let mut output: String = String::new();
        //output += "#!/bin/bash\n\n";
        let mut output = String::from("#!/bin/bash\n\n");
        //let mut output: String = String::from("#!/bin/bash");
        //trace!("output: {}", output);

        let sp = &self.spaths;

        // Sort length of String
        trace!("Expanding shortpaths");
        //let mut m = Vec::from_iter(
        //let mut m = 
            //sp.into_iter()
            //.map(|(_,v)| expand_shortpath(v, sp))
            //.filter_map(Option::Some)
            //.collect::<Option<PathBuf>>()
            ////.filter_map(Option::Some)
            ////.collect::<Vec<&PathBuf>>());
            ////.collect::<Option<Vec<&PathBuf>>>());
            //.into_iter()
            //.collect::<Vec<PathBuf>>();

        let mut m = 
            sp.into_iter()
            .map(|(_,v)| expand_shortpath(v, sp).unwrap_or(v.to_owned()))
            //.filter_map(Option::Some)
            //.collect::<Vec<&PathBuf>>());
            //.collect::<Option<Vec<&PathBuf>>>());
            .into_iter()
            .collect::<Vec<PathBuf>>();
        //let mut m = 
            //sp.into_iter()
            //.map(|(_,v)| expand_shortpath(v, sp))
            ////.filter_map(Option::Some)
            ////.collect::<Vec<&PathBuf>>());
            ////.collect::<Option<Vec<&PathBuf>>>());
            //.into_iter()
            //.collect::<Vec<Option<PathBuf>>>();

        //m.iter().for_each(|p| trace!("p: {}", p.display()));

        let get_length = |p: &PathBuf| { p.to_str().unwrap().len() };
        m.sort_by(|a, b| {
            let (la, lb) = (get_length(a), get_length(b));
            la.cmp(&lb)

            //match (a,b) {
                //(Some(a), Some(b)) => {
                    //let (la, lb) = (get_length(a), get_length(b));
                    //la.cmp(&lb)
                //}
                //_ => {
                    //std::cmp::Ordering::Less
                //}
            //}
        });

        trace!("Folding shortpaths");
        // For expanding:
        // We have the expanded path, now we have to expand it
        // We want to default the path to the non expanded version if it does not exist
        // We have the folded path, now we have to expand it
        // We want to default the path to the non folded version if it does not exist

        // For folding:
        // We have the expanded path, now we have to fold it
        // We want to default the path to the non expanded version if it does not exist
        for p in m {
            // If the path is none 
            //if let Some(pb) = p {

            //let path = fold_shortpath(p.as_path(), sp).unwrap_or(p);
            //let path = fold_shortpath(pb.as_path(), sp);
            let path = fold_shortpath(p.as_path(), sp);
            //let mut name = sp.get_by_right(&path).unwrap();
            let name = sp.get_by_right(&path).unwrap();
            
            //let path = match path {
                //Some(folded) => fold_shortpath(&p, sp),
                //None => p
            //};

            //let path = if !p.exists() {
            ////let path = fold_shortpath(&p, sp);
                //fold_shortpath(&p, sp)
            //} else {
                //p
            //};
            //let name = sp.get_by_right(&path).unwrap();

            //let path = fold_shortpath(p.as_path(), sp);
            //let path = p;
            trace!("name: {}", name);
            trace!("path: {}", path.display());
            let serialized = fmt_bash_alias(&name, &path);
            trace!("serialized: {}", serialized);
            output += &serialized;
            }
        //}
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
}

// Unit Tests
#[test]
fn test_serialize_bash() {
    use std:: path::Path;
    use log::LevelFilter;
    use pretty_env_logger::formatted_timed_builder;

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
