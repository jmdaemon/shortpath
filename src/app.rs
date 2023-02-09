use std::{
    path::PathBuf,
    io::Write, sync::atomic::{AtomicUsize, Ordering}, fmt,
};

use crate::consts::PROGRAM_DESCRIPTION;

use clap::{Parser, arg, Subcommand, ValueEnum};
use log::LevelFilter;
//use pretty_env_logger::env_logger::fmt::termcolor::Color;
//use pretty_env_logger::env_logger::fmt::termcolor::Color;
//use pretty_env_logger::env_logger::Logger::

//use pretty_env_logger::env_logger::fmt::{Color, Level, StyledValue};

use env_logger::{
    fmt::{Color, Style, StyledValue},
};
use log::Level;

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
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
    //pretty_env_logger::env_logger::Builder::new()
    //pretty_env_logger::formatted_builder()
    env_logger::builder()
        .format(|buf, record| {
            // Style the level
            //let mut style = buf.style();
            //style.set_color(pretty_env_logger::env_logger::fmt::Color::Red);
            //style = buf.default_level_style(record.level());
            //let style = buf.default_styled_level(record.level());
            //let styled_level = buf.default_styled_level(record.level());
            //let style = buf.style().value(styled_level);
                //.set_color(styled_level);
            
            // Custom styling
            //let target = record.target();
            let src = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            let target = format!("{}:{}", src, line);

            //let max_width = max_target_width(target);
            let max_width = max_target_width(&target);

            // Styling
            let mut style = buf.style();
            let level = colored_level(&mut style, record.level());
            
            //let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let time = buf.timestamp_millis();

            // Dim line numbers
            //let gray = 90;
            //let gray = 0x1b;
            //let gray = 0x1b;
            //let gray = 0x90;
            //let gray = 0x90;
            //let gray = 0x1b;
            //let gray = 0x94;
            //let gray = 0x90;
            //let gray = 0x90;
            //let gray = 0x190;

            //let gray = 0x90;
            //let gray = 0x1b;
            //let gray = 90;
            //let gray = 60;

            //let gray = 59;
            //let gray = 144;

            //let style = buf.style().set_color(Color::Ansi256(gray));

            let gray = 60;
            let mut style = buf.style();
            //style.set_color(Color::Ansi256(gray)).set_bold(true);
            //style.set_bg(Color::Ansi256(gray)).set_bold(true);
            //env_logger::fmt::WriteColor::set_color(&mut self, spec)
            //ColorChoice::AlwaysAnsi
            style
                .set_bold(true)
                .set_dimmed(true)
                .set_color(Color::Ansi256(gray))
                ;
                //.set_intense(false)
                //.set_intense(true);
            //let style = .set_color(Color::Ansi256(gray));
            //style.set_dimmed(true);

            //let mut style = buf.style();
            let target = style.set_bold(true).value(Padded {
                value: target,
                width: max_width,
            });

            //let mut stdout = StandardStream::stdout(ColorChoice::Always);
            //stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            //stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            

            writeln!(buf, "{} {} {} > {}", time, level, target, record.args())
            /*
            writeln!(
                buf,
                //"{}:{} {} [{}] - {}",
                "{} {} {}:{}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level,
                //record.level(),
                //level_style(),
                //style.value(record.level()),
                //style.value(record.level()),
                //styled_level,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
            */
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
