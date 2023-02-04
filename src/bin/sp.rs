use shortpaths::sp::{
    SPT,
    Shortpath,
    ShortpathsBuilder,
    FindKeyIndexMapExt,
    sort_shortpaths,
};

use std::path::PathBuf;

fn main() {
    // TODO Create more ergonomic api for this later
    // Wrap it together with the builder construct to reduce the noise
     let sp_paths = vec![
         Shortpath::new(SPT::new_path("d", PathBuf::from("$a/dddd")), None, None),
         Shortpath::new(SPT::new_path("c", PathBuf::from("$b/cccc")), None, None),
         Shortpath::new(SPT::new_path("b", PathBuf::from("$a/bbbb")), None, None),
         Shortpath::new(SPT::new_path("a", PathBuf::from("aaaa")), None, None),
     ];
     println!("{:?}", sp_paths);

     let mut sp_builder = ShortpathsBuilder::new(sp_paths);

     let sp_im = sp_builder.build().unwrap();
     sp_im.iter().for_each(|p| println!("{:?}", p));

     // Test find_key
     let key = sp_im.find_key_for_value("$a/bbbb");
     println!("{:?}", key);

     let key = sp_im.find_key_for_value("$a/bbbb".to_string());
     println!("{:?}", key);

     // Test sort_shortpaths
     println!("Sorted list of shortpaths");
     let sorted = sort_shortpaths(sp_im);
     sorted.iter().for_each(|p| println!("{:?}", p));

     // Test serialization
}
