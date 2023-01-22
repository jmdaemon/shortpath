use crate::shortpaths::{Shortpaths, fold_shortpath};
use crate::{consts::PROGRAM_NAME, shortpaths::expand_shortpath};
use crate::export::Export;

// Idea: Think about creating a proper completions file for bash instead of just
// creating bash aliases

use std::{
    path::PathBuf,
};

use std::cmp::*;

use bimap::BiHashMap;

//fn compare_len_reverse_alpha(a: &String, b: &String) -> Ordering {
    //// Sort by length from short to long first.
    //let length_test = a.len().cmp(&b.len());
    //if length_test == Ordering::Equal {
        //// If same length, sort in reverse alphabetical order.
        //return b.cmp(&a);
    //}
    //return length_test;
//}

//fn sort_by_string_length(vector: Vec<String>) {
    //let mut s = String::new();
    //for elem in vector {
        //peek
    //}
    //String temp = s[i];

    //// Insert s[j] at its correct position
    //int j = i - 1;
    //while (j >= 0 && temp.length() < s[j].length())
    //{
        //s[j+1] = s[j];
        //j--;
    //}
    //s[j+1] = temp;
//}
//}

//pub fn insertion_sort<T: Ord>(arr: &mut [T]) {
    //for i in 1..arr.len() {
        //let mut j = i;
        //while j > 0 && arr[j] < arr[j - 1] {
            //arr.swap(j, j - 1);
            //j = j - 1;
        //}
    //}
//}

pub struct BashExporter {
    spaths: Shortpaths,
}

pub fn fmt_bash_alias(name: String, path: &PathBuf) -> String {
    format!("{}=\"{}\"\n", name, path.display())
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
        //let m = Vec::from_iter(sp.into_iter().map(|(k,v)| expand_shortpath(v, sp)));
        //let m: HashMap<&String, PathBuf> = HashMap::from_iter(sp.into_iter().map(|(k,v)| (k, expand_shortpath(v, sp))));

        //for (k,v) in m {
        //}
        //let mut m = Vec::from_iter(sp.into_iter().map(|(k,v)| expand_shortpath(v, sp)));
        let mut m = Vec::from_iter(sp.into_iter().map(|(_,v)| expand_shortpath(v, sp)));
        //m.sort_by(|p:PathBuf| if p.capacity())
        //let mut max = 0;
        //m.sort_by(|k,v| compare_len_reverse_alpha(k, v.to_str().unwrap().to_owned()));
        //m.sort_by(|k,v| compare_len_reverse_alpha(k, v.to_str().unwrap().to_owned()));
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

        //let iter = sp.into_iter().chain(m);
        for p in m {
            let path = fold_shortpath(&p, &sp);
            let name = sp.get_by_right(&path).unwrap();
            //output += &fmt_bash_alias(name.to_string(), &path);
            output += &fmt_bash_alias(name.to_string(), &path);
        }


        //for (name, path) in &mut self.spaths.into_iter() {
        //for (name, path) in sp.into_iter() {
            ////output += &fmt_bash_alias(name.to_string(), &path);
            //output += &fmt_bash_alias(name.to_string(), &path);
        //}
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
    let mut bexp = BashExporter::new(BiHashMap::new());
    bexp.spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let mut spaths: Shortpaths = BiHashMap::new();
    //spaths.insert(String::from("aaaa"), Path::new("/test").to_path_buf());
    //let actual = serialize_bash(&spaths);
    let actual = bexp.gen_completions();
    let expect = "#!/bin/bash\n\naaaa=\"/test\"\n";
    assert_eq!(actual, expect);
}
