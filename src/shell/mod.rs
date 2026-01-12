use std::{
    fmt,
    io::{IsTerminal, Write},
};

use anstream::AutoStream;
use color_eyre::owo_colors::OwoColorize;
use miette::{Diagnostic, Severity};

use crate::AnnotatedResult;

pub mod ui;

pub struct Shell {
    verbosity: Verbosity,
    output: ShellOut,
    needs_clear: bool,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    pub fn new() -> Shell {
        let auto_clr = ColorChoice::Auto;
        let stdout_choice = auto_clr.into();
        let stderr_choice = auto_clr.into();

        Shell {
            output: ShellOut::Stream {
                stdout: AutoStream::new(std::io::stdout(), stdout_choice),
                stderr: AutoStream::new(std::io::stderr(), stderr_choice),
                color_choice: auto_clr,
                hyper_links: supports_hyperlinks(&std::io::stdout()),
                stderr_tty: std::io::stderr().is_terminal(),
                stdout_unicode: supports_unicode(&std::io::stdout()),
                stderr_unicode: supports_unicode(&std::io::stderr()),
                stderr_term_integration: supports_term_integration(&std::io::stderr()),
            },
            verbosity: Verbosity::Verbose,
            needs_clear: false,
        }
    }
    pub fn from_write(out: Box<dyn Write + Send + Sync>) -> Shell {
        Shell {
            output: ShellOut::Write(AutoStream::never(out)),
            verbosity: Verbosity::Verbose,
            needs_clear: false,
        }
    }
    pub fn set_verbosity(&mut self, verbosity: &clap_verbosity_flag::Verbosity) {
        self.verbosity = verbosity.into();
    }
    pub fn get_verbosity(&self) -> Verbosity {
        self.verbosity
    }
    pub fn out(&mut self) -> &mut (dyn Write + Send + Sync) {
        if self.needs_clear {
            self.err_clear_ln();
        }
        self.output.stdout()
    }
    pub fn err(&mut self) -> &mut (dyn Write + Send + Sync) {
        if self.needs_clear {
            self.err_clear_ln();
        }
        self.output.stderr()
    }
    pub fn err_fmt(&mut self) -> IoWriteAdapter<'_> {
        IoWriteAdapter { inner: self.err() }
    }
    fn err_clear_ln(&mut self) {
        if self.err_supports_color() {
            imp::err_erase_line(self);
            self.needs_clear = false;
        }
    }
    pub fn stdout_unicode(&self) -> bool {
        matches!(
            &self.output,
            ShellOut::Stream {
                stdout_unicode: true,
                ..
            }
        )
    }
    pub fn stderr_unicode(&self) -> bool {
        matches!(
            &self.output,
            ShellOut::Stream {
                stderr_unicode: true,
                ..
            }
        )
    }
    pub fn supports_hyperlinks(&self) -> bool {
        matches!(
            &self.output,
            ShellOut::Stream {
                hyper_links: true,
                ..
            }
        )
    }
    fn err_supports_color(&self) -> bool {
        match &self.output {
            ShellOut::Write(_) => false,
            ShellOut::Stream { stderr, .. } => support_color(stderr.current_choice()),
        }
    }
    pub fn print_miette(&mut self, diagnostic: &dyn Diagnostic) {
        let theme = if self.stderr_unicode() {
            miette::GraphicalTheme::unicode()
        } else {
            miette::GraphicalTheme::ascii()
        };
        let handler = miette::GraphicalReportHandler::new()
            .with_theme(theme)
            .with_links(self.supports_hyperlinks());

        self.write_miette_header(diagnostic);
        handler
            .render_report(&mut self.err_fmt(), diagnostic)
            .unwrap_annotated("Failed to render diagnostic");
    }
    fn write_miette_header(&mut self, diagnostic: &dyn Diagnostic) {
        let severity = match diagnostic.severity().unwrap_or_default() {
            Severity::Error => "Error".bold().red().to_string(),
            Severity::Advice => "Advice".bold().cyan().to_string(),
            Severity::Warning => "Warning".bold().yellow().to_string(),
        };
        let code = diagnostic
            .code()
            .map(|c| format!("{c}"))
            .unwrap_or("".into());

        write!(self.err(), "{}: {}", severity, code)
            .unwrap_annotated("Failed to format diagnostic header");
    }
    /// Returns the width of the terminal in spaces, if any.
    pub fn err_width(&self) -> TtyWidth {
        match self.output {
            ShellOut::Stream {
                stderr_tty: true, ..
            } => imp::stderr_width(),
            _ => TtyWidth::NoTty,
        }
    }
}

pub struct IoWriteAdapter<'a> {
    inner: &'a mut (dyn Write + Send + Sync),
}

impl<'a> fmt::Write for IoWriteAdapter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    Verbose,
    Normal,
    Quiet,
}

impl From<&clap_verbosity_flag::Verbosity> for Verbosity {
    fn from(value: &clap_verbosity_flag::Verbosity) -> Self {
        if value.is_silent() {
            Self::Quiet
        } else if value.is_present() {
            Self::Verbose
        } else {
            Self::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl From<ColorChoice> for anstream::ColorChoice {
    fn from(value: ColorChoice) -> Self {
        match value {
            ColorChoice::Auto => anstream::ColorChoice::Auto,
            ColorChoice::Always => anstream::ColorChoice::Always,
            ColorChoice::Never => anstream::ColorChoice::Never,
        }
    }
}

pub enum ShellOut {
    Write(AutoStream<Box<dyn Write + Send + Sync>>),
    Stream {
        stdout: AutoStream<std::io::Stdout>,
        stderr: AutoStream<std::io::Stderr>,
        stderr_tty: bool,
        stdout_unicode: bool,
        stderr_unicode: bool,
        stderr_term_integration: bool,
        color_choice: ColorChoice,
        hyper_links: bool,
    },
}

impl ShellOut {
    fn stdout(&mut self) -> &mut (dyn Write + Send + Sync) {
        match self {
            Self::Write(stream) => stream,
            Self::Stream { stdout, .. } => stdout,
        }
    }
    fn stderr(&mut self) -> &mut (dyn Write + Send + Sync) {
        match self {
            Self::Write(stream) => stream,
            Self::Stream { stderr, .. } => stderr,
        }
    }
}

fn supports_hyperlinks(stream: &dyn IsTerminal) -> bool {
    use std::env;
    if !stream.is_terminal() {
        return false;
    }

    if env::var_os("NO_COLOR").is_some() {
        return false;
    }

    if env::var_os("TERM_PROGRAM").is_some() {
        return true;
    }

    // Windows Terminal
    if env::var_os("WT_SESSION").is_some() {
        return true;
    }

    false
}

fn supports_unicode(stream: &dyn IsTerminal) -> bool {
    use std::env;
    if !stream.is_terminal() {
        return false;
    }

    // Windows: moderne Terminals können Unicode
    #[cfg(windows)]
    {
        return env::var_os("WT_SESSION").is_some() || env::var_os("TERM_PROGRAM").is_some();
    }

    #[cfg(not(windows))]
    {
        let locale = env::var("LC_ALL")
            .or_else(|_| env::var("LC_CTYPE"))
            .or_else(|_| env::var("LANG"))
            .unwrap_or_default();

        locale.to_uppercase().contains("UTF-8")
    }
}

fn support_color(choice: anstream::ColorChoice) -> bool {
    match choice {
        anstream::ColorChoice::Always
        | anstream::ColorChoice::AlwaysAnsi
        | anstream::ColorChoice::Auto => true,
        anstream::ColorChoice::Never => false,
    }
}

fn supports_term_integration(stream: &dyn IsTerminal) -> bool {
    let windows_terminal = std::env::var("WT_SESSION").is_ok();
    let conemu = std::env::var("ConEmuANSI").ok() == Some("ON".into());
    let wezterm = std::env::var("TERM_PROGRAM").ok() == Some("WezTerm".into());
    let ghostty = std::env::var("TERM_PROGRAM").ok() == Some("ghostty".into());

    (windows_terminal || conemu || wezterm || ghostty) && stream.is_terminal()
}

pub enum TtyWidth {
    NoTty,
    Known(usize),
    Guess(usize),
}

#[cfg(unix)]
mod imp {
    use crate::shell::{Shell, TtyWidth};
    use std::mem;
    pub fn stderr_width() -> TtyWidth {
        unsafe {
            let mut winsize: libc::winsize = mem::zeroed();
            // The .into() here is needed for FreeBSD which defines TIOCGWINSZ
            // as c_uint but ioctl wants c_ulong.
            #[allow(clippy::useless_conversion)]
            if libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ.into(), &mut winsize) < 0 {
                return TtyWidth::NoTty;
            }
            if winsize.ws_col > 0 {
                TtyWidth::Known(winsize.ws_col as usize)
            } else {
                TtyWidth::NoTty
            }
        }
    }
    pub fn err_erase_line(shell: &mut Shell) {
        let _ = shell.output.stderr().write_all(b"\r\x1B[2K");
    }
}

#[cfg(windows)]
mod imp {
    use std::{cmp, mem, ptr};

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileA, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Console::{
        CONSOLE_SCREEN_BUFFER_INFO, GetConsoleScreenBufferInfo, GetStdHandle, STD_ERROR_HANDLE,
    };
    use windows_sys::core::PCSTR;

    pub(super) use super::{TtyWidth, default_err_erase_line as err_erase_line};

    pub fn stderr_width() -> TtyWidth {
        unsafe {
            let stdout = GetStdHandle(STD_ERROR_HANDLE);
            let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
            if GetConsoleScreenBufferInfo(stdout, &mut csbi) != 0 {
                return TtyWidth::Known((csbi.srWindow.Right - csbi.srWindow.Left) as usize);
            }

            // On mintty/msys/cygwin based terminals, the above fails with
            // INVALID_HANDLE_VALUE. Use an alternate method which works
            // in that case as well.
            let h = CreateFileA(
                "CONOUT$\0".as_ptr() as PCSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            );
            if h == INVALID_HANDLE_VALUE {
                return TtyWidth::NoTty;
            }

            let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
            let rc = GetConsoleScreenBufferInfo(h, &mut csbi);
            CloseHandle(h);
            if rc != 0 {
                let width = (csbi.srWindow.Right - csbi.srWindow.Left) as usize;
                // Unfortunately cygwin/mintty does not set the size of the
                // backing console to match the actual window size. This
                // always reports a size of 80 or 120 (not sure what
                // determines that). Use a conservative max of 60 which should
                // work in most circumstances. ConEmu does some magic to
                // resize the console correctly, but there's no reasonable way
                // to detect which kind of terminal we are running in, or if
                // GetConsoleScreenBufferInfo returns accurate information.
                return TtyWidth::Guess(cmp::min(60, width));
            }

            TtyWidth::NoTty
        }
    }
}

#[cfg(windows)]
fn default_err_erase_line(shell: &mut Shell) {
    match imp::stderr_width() {
        TtyWidth::Known(max_width) | TtyWidth::Guess(max_width) => {
            let blank = " ".repeat(max_width);
            drop(write!(shell.output.stderr(), "{}\r", blank));
        }
        _ => (),
    }
}
