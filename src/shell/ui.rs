use std::fmt::Display;
use std::io::Write;

use anstyle::{AnsiColor, Effects, Style};

use crate::{
    CoreResult,
    shell::{Shell, ShellOut, Verbosity},
};

pub struct ProgressBar {
    size: usize,
    current: usize,
}

impl ProgressBar {
    pub fn new(size: usize) -> Self {
        Self { size, current: 1 }
    }
    pub fn next(&mut self) {
        if self.current < self.size {
            self.current += 1;
        }
    }
    pub fn is_finished(&self) -> bool {
        self.current == self.size + 1
    }
    pub fn render(&self) -> String {
        if self.is_finished() {
            return self.render_finished();
        }
        let mut out = String::with_capacity(self.size + 2);
        out.push('[');
        out.push_str(&"#".repeat(self.current.saturating_sub(1)));
        out.push('>');
        out.push_str(&"-".repeat(self.size - self.current));
        out.push(']');
        out
    }
    fn render_finished(&self) -> String {
        let mut out = String::with_capacity(self.size + 2);
        out.push('[');
        out.push_str(&"#".repeat(self.size));
        out.push(']');
        out
    }
}

impl Shell {
    pub fn print(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
        color: &Style,
    ) -> CoreResult<()> {
        if self.needs_clear {
            self.err_clear_ln();
        }
        let status = if self.stderr_unicode() {
            format!("▶ {status}")
        } else {
            format!("> {status}")
        };
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.output.message_stderr(&status, message, color),
        }
    }
    pub fn print_verbose(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
        color: &Style,
    ) -> CoreResult<()> {
        if self.needs_clear {
            self.err_clear_ln();
        }
        let status = if self.stderr_unicode() {
            format!("▶ {status}")
        } else {
            format!("> {status}")
        };
        match self.verbosity {
            Verbosity::Verbose => self.output.message_stderr(&status, message, color),
            _ => Ok(()),
        }
    }
    pub fn set_needs_clear(&mut self, needs_clear: bool) {
        self.needs_clear = needs_clear;
    }
}

impl ShellOut {
    fn message_stderr(
        &mut self,
        status: &dyn Display,
        message: Option<&dyn Display>,
        style: &Style,
    ) -> CoreResult<()> {
        let mut buffer = Vec::new();
        write!(buffer, "{style}{status}{style:#}")?;
        match message {
            Some(msg) => writeln!(buffer, " {msg}")?,
            None => writeln!(buffer, "")?,
        }
        self.stderr().write_all(&buffer)?;

        Ok(())
    }
}

#[macro_export]
macro_rules! info {
    ($status:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, None, &$crate::shell::ui::INFO)))
    };
    ($status:expr, $message:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, Some($message), &$crate::shell::ui::INFO)))
    };
}

#[macro_export]
macro_rules! warn {
    ($status:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, None, &$crate::shell::ui::WARN)))
    };
    ($status:expr, $message:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, Some($message), &$crate::shell::ui::WARN)))
    };
}

#[macro_export]
macro_rules! error {
    ($status:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, None, &$crate::shell::ui::ERROR)))
    };
    ($status:expr, $message:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, Some($message), &$crate::shell::ui::ERROR)))
    };
}

#[macro_export]
macro_rules! success {
    ($status:expr) => {
        $crate::with_shell(|sh| drop(sh.print($status, None, &$crate::shell::ui::SUCCESS)))
    };
    ($status:expr, $message:expr) => {
        $crate::with_shell(|sh| {
            drop(sh.print($status, Some($message), &$crate::shell::ui::SUCCESS))
        })
    };
}

#[macro_export]
macro_rules! status {
    ($status:expr) => {
        let style = &$crate::shell::ui::STATUS;
        let unicode = $crate::with_shell(|sh| sh.stderr_unicode());
        let msg = if unicode {
            format!("\r⏳{style}{}{style:#}", $status)
        } else {
            format!("\r{style}{}{style:#} {}", $status, $message)
        };
        $crate::drop_eprint!("{}", msg);
        drop($crate::with_shell(|sh| sh.set_needs_clear(true)));
    };
    ($status:expr, $message:expr) => {{
        let style = &$crate::shell::ui::STATUS;
        let unicode = $crate::with_shell(|sh| sh.stderr_unicode());
        let msg = if unicode {
            format!("\r⏳{style}{}{style:#} {}", $status, $message)
        } else {
            format!("\r{style}{}{style:#} {}", $status, $message)
        };
        $crate::drop_eprint!("{}", msg);
        #[allow(dropping_copy_types)]
        drop($crate::with_shell(|sh| sh.set_needs_clear(true)));
    }};
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        let status = "Verbose".to_string();
        let style = &$crate::shell::ui::VERBOSE;
        let msg = format_args!($($arg)*);
        drop($crate::with_shell(|sh| sh.print_verbose(&status, Some(&msg), style)));
    };
}

#[macro_export]
macro_rules! __shell_print {
    ($msg:expr, $newline:expr, $err:expr) => {
        $crate::with_shell(|sh| {
            let stream = if $err { sh.err() } else { sh.out() };
            drop(stream.write(format!("{}", $msg).as_bytes()));
            if $newline {
                drop(stream.write_all(b"\n"))
            }
        })
    };
}

#[macro_export]
macro_rules! drop_print {
    ($($arg:tt)*) => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!(format_args!($($arg)*), false, false));
    };
}

#[macro_export]
macro_rules! drop_println {
    () => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!('\n', true, false));
    };
    ($($arg:tt)*) => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!(format_args!($($arg)*), true, false));
    };
}

#[macro_export]
macro_rules! drop_eprint {
    ($($arg:tt)*) => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!(format_args!($($arg)*), false, true));
    };
}

#[macro_export]
macro_rules! drop_eprintln {
    () => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!('\n', true, true));
    };
    ($($arg:tt)*) => {
        #[allow(dropping_copy_types)]
        drop($crate::__shell_print!(format_args!($($arg)*), true, true));
    };
}

pub const EMPTY: Style = Style::new();
pub const WARN: Style = annotate_snippets::renderer::DEFAULT_WARNING_STYLE;
pub const ERROR: Style = annotate_snippets::renderer::DEFAULT_ERROR_STYLE;
pub const INFO: Style = annotate_snippets::renderer::DEFAULT_INFO_STYLE;
pub const SUCCESS: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub const STATUS: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const VERBOSE: Style = AnsiColor::Magenta.on_default().effects(Effects::BOLD);
