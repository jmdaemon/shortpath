fn main() {
    // Our ideal user api

    // For binary
    let cfg = Config::new();
    let shortpaths: Shortpaths = ShortpathsBuilder()
        .with_paths_file(cfg.format_config_path("shortpaths.toml"))
        .build();

    // or
    let cfg = Config::new(CONFIG_FILE_PATH);
    let shortpaths: Shortpaths = ShortpathsBuilder()
        .with_paths_file(cfg.files.get(CONFIG_FILE_PATH).unwrap())
        // or even
        .with_config(cfg, CONFIG_FILE_PATH)
        .build();

    // For testing
    let sp_paths = vec![
        Shortpath::new("d", ShortpathType::AliasPath::from("$a/dddd"))),
        Shortpath::new("c", ShortpathType::AliasPath::from("$b/cccc"))),
        Shortpath::new("b", ShortpathType::AliasPath::from("$a/bbbb"))),
        Shortpath::new("a", ShortpathType::Path::from("aaaa"))),
    ];

    let shortpaths: Shortpaths = ShortpathsBuilder()
        .from_vec(sp_paths)
        .build();

    // Additionally
    let shortpaths: Shortpaths = ShortpathsBuilder()
        .shortpath(sp!("a", alias_path:""))
        .shortpath(sp!("b", alias_path:""))
        .build();

    println!("Hello, world!");
}

// Our ideal library api

// How to design our types to be less incorrect/invalid?
// Are there any invalid states?
// How to use immutability and , the borrow checker to reinforce this constraint?

// Data Structures
type SP = IndexMap<String, Shortpath>;

// Serializable Shortpaths
type SSP = IndexMap<String, PathBuf>;

enum ShortpathState {
    Folded(PathBuf),
    Expanded(PathBuf)
}

//struct ShortpathType {
//}
enum ShortpathType {
    NoPath,
    Path(PathBuf),
    Alias(PathBuf),
    Env(PathBuf),
}

struct Dependency<S: ShortpathType> {
    depends_on: S,
}

pub struct ShortpathsBuilder {
    paths: Option<SP>,
}

// Traits

// Serialize the key as a string
impl Serialize for ShortpathType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(get_shortpath_path(&self).to_str().unwrap())
    }
}

// Sort paths in lexicographical order according to their expanded full_paths
impl Ord for Shortpath {
    fn cmp(&self, other: &Self) -> Ordering {
        let get_paths = || {
            let (path_1, path_2) = (self.full_path.clone().unwrap(), other.full_path.clone().unwrap());
            (path_1, path_2)
        };
        let (path_1, path_2) = get_paths();
        path_1.cmp(&path_2).reverse()
    }
}

impl PartialOrd for Shortpath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ShortpathBuilder {
    fn build();
    fn shortpath(ShortpathType);
    fn shortpath();
}

// Behaviour

fn from_disk(file: &str) -> Shortpaths {
    let config = setup_config(file);
    let toml_conts = read_config(&cfg, file);

    // Load the serialized shortpaths into our serde struct
    let sp: Shortpaths = toml::from_str(&toml_conts).unwrap();
    sp
}

fn get_shortpath_type(path_component: &[char], path: PathBuf) -> ShortpathType {
    // Pattern match type, return path
}

fn get_shortpath_dependency(path_component: &[char]) -> Option<String> {
    // Pattern match type, return name
}

fn find_deps<S: ShortpathType>(path: &str) -> Dependency<S> { }

fn fold_path(src: &str, key_name: &str, key_path: &str) -> String;
fn expand_path(src: &str, key_name: &str, key_path: &str) -> String;

fn fold_shortpath(shortpath: &SP, shortpaths: &SP) -> Shortpath<Folded>;
fn expand_shortpath(shortpath: &SP,, shortpath: &SP) -> Shortpath<Expanded>;

pub fn find_paths(path: &PathBuf, options: FindPathOptions, find_by: impl Fn(PathBuf) -> Vec<DirEntry>) -> Option<Vec<DirEntry>> {
    // Consume options struct and produce a find_by function
    // Execute the find_by function
}

// Commands
fn add_shortpath(name: String, path: PathBuf, shortpaths: &mut SP) {
    match is_valid_shortpath(path) {
        Some(_) => shortpaths.insert(keyname, shortpath),
        None => eprintln!("Unable to add shortpath: ")
    }
}

fn remove_shortpath(shortpaths: &mut SP, key_name: &str, confirm_choice: bool) {
    //let overwrite_all = false;
    //let skip_all = false;
    if confirm_choice {
        match shortpaths.get(keyname)) {
            Some(path) => {
                let prompt = format!("Remove shortpath?: {}: {}", key_name, path);
                let choice = prompt_user(prompt);
                match choice {
                    Yes => shortpaths.remove(keyname),
                    No =>,
                }
            }
            None 
        }
    } else {
        // Forcibly remove path
        shortpaths.remove(keyname)
    }
}

fn remove_shortpaths(shortpaths: &mut SP, key_names: &[&str], confirm_choice: bool,
    overwrite_all: bool, skip_all: bool) {
}

fn list_shortpath(shortpath: &str, shortpaths: &SP) {
    let sp = shortpaths.get(shortpath);
    show_shortpath(sp);
}

fn list_shortpaths(shortpaths: &SP) {
    shortpaths.iter().for_each(|(_, sp)| {
        show_shortpath(shortpath);
    });
}

fn check_shortpaths(shortpaths: &SP) {
    let unreachable = find_unreachable(shortpaths);
    unreachable.iter().for_each(|(alias_name, alias_path)|
        println!("{} shortpath is unreachable: {}", alias_name, alias_path.path().display()));
    println!("Check Complete");

}

// TODO:
fn resolve_shortpaths(shortpaths: &mut SP);

fn export_shortpaths(shortpaths: &SP, export_type: &str, output_file: Option<&String>) -> String {
    let exp = get_exporter(export_type)
        .set_shortpaths(shortpaths);
    let dest = exp.gen_completions(dest);
    dest
}

fn update_shortpath(shortpaths: &mut SP, name: &str, new_name: Option<&String>, new_path: Option<&String>) {
    let update_name = || {};
    let update_path = || {};
    match (new_name, new_path) {
        (Some(new_name), _) => { update_name(); }
        (_, Some(new_path)) => { update_path(); }
        (_, _) => { println!("Nothing to do");}
    }
}

// Our ideal cli
