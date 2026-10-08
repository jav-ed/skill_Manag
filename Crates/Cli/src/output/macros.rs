//! Small helpers for building output text.

/// Appends formatted text to a `String`. Writing to a string cannot fail, so there is no result to handle.
macro_rules! put {
    ($buf:expr, $($arg:tt)*) => {
        $buf.push_str(&format!($($arg)*))
    };
}

/// Like `put!`, plus a newline.
macro_rules! putln {
    ($buf:expr, $($arg:tt)*) => {{
        $buf.push_str(&format!($($arg)*));
        $buf.push('\n');
    }};
}
