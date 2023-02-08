use std::path::PathBuf;

use crate::consts::PROGRAM_DESCRIPTION;

use clap::{Parser, arg, Subcommand, ValueEnum};
 
#[derive(Parser)]
#[command(author, version, about, long_about = PROGRAM_DESCRIPTION)]
pub struct CLI {
    #[arg(short, long, default_value_t = false, help = "Toggle verbose information")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a shortpath")]
    Add     {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    #[command(about = "Remove a shortpath")]
    Remove  {
        #[arg(value_name = "NAME")]
        name: String,
    },

    #[command(about = "Lists shortpath configurations.")]
    List   {
        #[arg(help = "Show one or more specific shortpaths")]
        names: Option<Vec<String>>,
    },

    #[command(about = "Checks all shortpaths")]
    Check   {},

    #[command(about = "Fixes all shortpaths.")]
    Resolve {
        #[arg(value_enum, default_value_t = ResolveType::Matching, help = "Find and automatically fix shortpaths using the resolve_type algorithm")]
        resolve_type: ResolveType,

        #[arg(short, long, value_enum, default_value_t = Mode::Automatic, help = "Toggle Resolve Mode")]
        mode: Mode,

        #[arg(short, long, default_value_t = false, help = "Show shortpath config changes, but do not execute them")]
        dry_run: bool,
    },

    #[command(about = "Export shortpaths to other applications")]
    Export  {
        #[arg(value_enum)]
        export_type: ExportType,
        #[arg(short, long, help = "Output to file")]
        output_file: Option<PathBuf>,
    },

    #[command(about = "Update a shortpath")]
    Update  {
        #[arg(value_name = "CURRENT_NAME")]
        current_name: String,
        #[arg(value_name = "NAME")]
        #[arg(short, long, help = "New shortpath name")]
        name: Option<String>,

        #[arg(value_name = "PATH")]
        #[arg(short, long, help = "New shortpath path")]
        path: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ExportType {
    Bash,
    Powershell,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    Automatic,
    Manual
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ResolveType {
    Matching
}
