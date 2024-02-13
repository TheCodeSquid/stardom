use std::{
    fmt,
    io::Write,
    sync::{Mutex, OnceLock},
};

use anstream::ColorChoice;
use anstyle::*;

const CLEAR_LINE: &str = "\x1b[2K";

const ERROR: Style = AnsiColor::Red.on_default().bold();
const WARN: Style = AnsiColor::Yellow.on_default().bold();
const STATUS: Style = AnsiColor::Green.on_default().bold();
const PROGRESS: Style = AnsiColor::Cyan.on_default().bold();

static SHELL: OnceLock<Shell> = OnceLock::new();

pub fn shell() -> &'static Shell {
    SHELL.get_or_init(Shell::new)
}

pub struct Shell {
    stderr: Mutex<(anstream::Stderr, bool)>,
    no_color: bool,
}

impl Shell {
    fn new() -> Self {
        let stderr = anstream::stderr();
        let no_color = stderr.current_choice() == ColorChoice::Never;
        Self {
            stderr: Mutex::new((anstream::stderr(), false)),
            no_color,
        }
    }

    pub fn no_color(&self) -> bool {
        self.no_color
    }

    pub fn clear_line(&self) {
        let (stderr, needs_clear) = &mut *self.stderr.lock().unwrap();
        *needs_clear = false;
        write!(stderr, "\r{}", CLEAR_LINE).expect("failed to write to stderr");
    }

    pub fn print(
        &self,
        style: Style,
        status: impl fmt::Display,
        message: impl fmt::Display,
        justified: bool,
        replaceable: bool,
    ) {
        let log = create_log(style, status, message, justified);
        let mut buf = vec![];
        let (stderr, needs_clear) = &mut *self.stderr.lock().unwrap();
        if *needs_clear {
            *needs_clear = false;
            write!(&mut buf, "{}", CLEAR_LINE).unwrap();
        }
        if replaceable {
            *needs_clear = true;
            write!(&mut buf, "{log}\r").unwrap();
        } else {
            writeln!(&mut buf, "{log}").unwrap();
        };
        stderr.write_all(&buf).expect("failed to write to stderr");
    }

    pub fn error(&self, message: impl fmt::Display) {
        self.print(ERROR, "error", message, false, false);
    }

    pub fn warn(&self, message: impl fmt::Display) {
        self.print(WARN, "warning", message, false, false);
    }

    pub fn status(&self, status: impl fmt::Display, message: impl fmt::Display) {
        self.print(STATUS, status, message, true, false);
    }

    pub fn progress(&self, status: impl fmt::Display, message: impl fmt::Display) {
        self.print(PROGRESS, status, message, true, true);
    }
}

fn create_log(
    style: Style,
    status: impl fmt::Display,
    message: impl fmt::Display,
    justified: bool,
) -> String {
    if justified {
        format!("{style}{status:>12}{style:#} {message}")
    } else {
        let bold = if style.get_effects().contains(Effects::BOLD) {
            Style::new().bold()
        } else {
            Style::new()
        };
        format!("{style}{status}{style:#}{bold}:{bold:#} {message}")
    }
}
