#![cfg(feature = "std")]

use std::io::Write;

use crate::error::ErrMode;
use crate::stream::Stream;

pub struct Depth {
    depth: usize,
    inc: bool,
}

impl Depth {
    pub fn new() -> Self {
        let depth = DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let inc = true;
        Self { depth, inc }
    }

    pub fn existing() -> Self {
        let depth = DEPTH.load(std::sync::atomic::Ordering::SeqCst);
        let inc = false;
        Self { depth, inc }
    }
}

impl Drop for Depth {
    fn drop(&mut self) {
        if self.inc {
            let _ = DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

impl AsRef<usize> for Depth {
    #[inline(always)]
    fn as_ref(&self) -> &usize {
        &self.depth
    }
}

impl crate::lib::std::ops::Deref for Depth {
    type Target = usize;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.depth
    }
}

static DEPTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub enum Severity {
    Success,
    Backtrack,
    Cut,
    Incomplete,
}

impl Severity {
    pub fn with_result<T, E>(result: &Result<T, ErrMode<E>>) -> Self {
        match result {
            Ok(_) => Self::Success,
            Err(ErrMode::Backtrack(_)) => Self::Backtrack,
            Err(ErrMode::Cut(_)) => Self::Cut,
            Err(ErrMode::Incomplete(_)) => Self::Incomplete,
        }
    }
}

pub fn start<I: Stream>(
    depth: usize,
    name: &dyn crate::lib::std::fmt::Display,
    count: usize,
    input: &I,
) {
    let gutter_style = anstyle::Style::new().bold();
    let input_style = anstyle::Style::new().underline();
    let eof_style = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Cyan.into()));

    let (call_width, input_width) = column_widths();

    let count = if 0 < count {
        format!(":{count}")
    } else {
        "".to_owned()
    };
    let call_column = format!("{:depth$}> {name}{count}", "");

    // The debug version of `slice` might be wider, either due to rendering one byte as two nibbles or
    // escaping in strings.
    let mut debug_slice = format!("{:#?}", input.raw());
    let (debug_slice, eof) = if let Some(debug_offset) = debug_slice
        .char_indices()
        .enumerate()
        .find_map(|(pos, (offset, _))| (input_width <= pos).then_some(offset))
    {
        debug_slice.truncate(debug_offset);
        let eof = "";
        (debug_slice, eof)
    } else {
        let eof = if debug_slice.chars().count() < input_width {
            "∅"
        } else {
            ""
        };
        (debug_slice, eof)
    };

    let writer = anstream::stderr();
    let mut writer = writer.lock();
    let _ = writeln!(
        writer,
        "{call_column:call_width$} {gutter_style}|{gutter_reset} {input_style}{debug_slice}{input_reset}{eof_style}{eof}{eof_reset}",
        gutter_style=gutter_style.render(),
        gutter_reset=gutter_style.render_reset(),
        input_style=input_style.render(),
        input_reset=input_style.render_reset(),
        eof_style=eof_style.render(),
        eof_reset=eof_style.render_reset(),
    );
}

pub fn end(
    depth: usize,
    name: &dyn crate::lib::std::fmt::Display,
    count: usize,
    consumed: usize,
    severity: Severity,
) {
    let gutter_style = anstyle::Style::new().bold();

    let (call_width, _) = column_widths();

    let count = if 0 < count {
        format!(":{count}")
    } else {
        "".to_owned()
    };
    let call_column = format!("{:depth$}< {name}{count}", "");

    let (status_style, status) = match severity {
        Severity::Success => {
            let style = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Green.into()));
            let status = format!("+{}", consumed);
            (style, status)
        }
        Severity::Backtrack => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            "backtrack".to_owned(),
        ),
        Severity::Cut => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())),
            "cut".to_owned(),
        ),
        Severity::Incomplete => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())),
            "incomplete".to_owned(),
        ),
    };

    let writer = anstream::stderr();
    let mut writer = writer.lock();
    let _ = writeln!(
        writer,
        "{status_style}{call_column:call_width$}{status_reset} {gutter_style}|{gutter_reset} {status_style}{status}{status_reset}",
        gutter_style=gutter_style.render(),
        gutter_reset=gutter_style.render_reset(),
        status_style=status_style.render(),
        status_reset=status_style.render_reset(),
    );
}

pub fn result(depth: usize, name: &dyn crate::lib::std::fmt::Display, severity: Severity) {
    let gutter_style = anstyle::Style::new().bold();

    let (call_width, _) = column_widths();

    let call_column = format!("{:depth$}| {name}", "");

    let (status_style, status) = match severity {
        Severity::Success => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Green.into())),
            "",
        ),
        Severity::Backtrack => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into())),
            "backtrack",
        ),
        Severity::Cut => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())),
            "cut",
        ),
        Severity::Incomplete => (
            anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())),
            "incomplete",
        ),
    };

    let writer = anstream::stderr();
    let mut writer = writer.lock();
    let _ = writeln!(
        writer,
        "{status_style}{call_column:call_width$}{status_reset} {gutter_style}|{gutter_reset} {status_style}{status}{status_reset}",
        gutter_style=gutter_style.render(),
        gutter_reset=gutter_style.render_reset(),
        status_style=status_style.render(),
        status_reset=status_style.render_reset(),
    );
}

fn column_widths() -> (usize, usize) {
    let term_width = term_width();

    let min_call_width = 40;
    let min_input_width = 20;
    let decor_width = 3;
    let extra_width = term_width
        .checked_sub(min_call_width + min_input_width + decor_width)
        .unwrap_or_default();
    let call_width = min_call_width + 2 * extra_width / 3;
    let input_width = min_input_width + extra_width / 3;

    (call_width, input_width)
}

fn term_width() -> usize {
    columns_env().or_else(query_width).unwrap_or(80)
}

fn query_width() -> Option<usize> {
    use is_terminal::IsTerminal;
    if std::io::stderr().is_terminal() {
        terminal_size::terminal_size().map(|(w, _h)| w.0.into())
    } else {
        None
    }
}

fn columns_env() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<usize>().ok())
}
