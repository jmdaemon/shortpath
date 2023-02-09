use std::{
    path::PathBuf,
    io::Write, sync::atomic::{AtomicUsize, Ordering}, fmt,
};

use crate::consts::PROGRAM_DESCRIPTION;

use clap::{Parser, arg, Subcommand, ValueEnum};
use log::{Level, LevelFilter};
use env_logger::fmt::{Color, Style, StyledValue};

// Custom Log Format

fn colored_level(style: &mut Style, level: Level) -> StyledValue<&str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub fn create_logger() {
    env_logger::builder()
        .format(|buf, record| {

            // Format with horizontally aligned src:line info
            let src = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            let target = format!("{}:{}", src, line);
            let max_width = max_target_width(&target);

            // Use color
            let mut style = buf.style();
            let level = colored_level(&mut style, record.level());

            // Set custom time format
            let time = chrono::Local::now().format("%H:%M:%S");
            //let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            
            //let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            //let time = buf.timestamp_millis();

            // Dim line numbers
            //let gray = (103, 106, 141);
            //let mut style = buf.style();
            //let (r,g,b) = gray;
            //style
                //.set_bold(true)
                //.set_dimmed(true)
                //.set_color(Color::Rgb(r,g,b));

            // ANSI Style
            let gray = 60;
            let mut style = buf.style();
            style
                .set_bold(true)
                .set_dimmed(true)
                .set_color(Color::Ansi256(gray));

            let target = style.set_bold(true).value(Padded {
                value: target,
                width: max_width,
            });

            writeln!(buf, "{} {} {} > {}", time, level, target, record.args())
        })
    .filter(Some("shortpaths"), LevelFilter::Trace).init();
}
 
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
