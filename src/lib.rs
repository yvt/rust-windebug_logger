//! A logger that redirects all messages to the [`OutputDebugMessageW`]
//! Win32 API function in a format similar to [`simple_logger`].
//!
//! [`OutputDebugMessageW`]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-outputdebugstringw
//! [`simple_logger`]: https://crates.io/crates/simple_logger
//!
//! This crate is useful if you are writing a Windows GUI application, where
//! stdout and stderr do not work. The messages outputted by
//! `OutputDebugMessageW` can be monitored using a program such as [DebugView]
//! and Visual Studio's "Output" window.
//!
//! [DebugView]: https://docs.microsoft.com/en-us/sysinternals/downloads/debugview

// Used by `init_with_level_static!`
#[doc(hidden)]
pub extern crate log;

use log::{Level, SetLoggerError};
use std::{
    convert::TryInto,
    mem::{transmute, MaybeUninit},
    ptr::null,
};
use winapi::um::{datetimeapi, debugapi, sysinfoapi, winbase, winnt};

mod codecvt;

#[doc(hidden)]
#[derive(Debug)]
pub struct WinDebugLogger {
    pub level: Level,
}

impl log::Log for WinDebugLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Silently ignore errors
        let _ = log(record);
    }

    fn flush(&self) {}
}

fn log(record: &log::Record) -> Option<()> {
    let target = if record.target().len() > 0 {
        record.target()
    } else {
        record.module_path().unwrap_or_default()
    };

    // Everything except the timestamp
    let body = format!("{:<5} [{}] {}", record.level(), target, record.args());
    let body = codecvt::str_to_c_wstr(&body)?;

    // The timestamp is rendered using `GetTimeFormatW`
    let system_time = unsafe {
        let mut out = MaybeUninit::uninit();
        sysinfoapi::GetSystemTime(out.as_mut_ptr());
        out.assume_init()
    };

    const MAX_LEN: usize = 40;

    let (date_str, date_str_len) = unsafe {
        // This is safe because `[MaybeUninit<u16>; MAX_LEN]` has no portion
        // that requires initialization
        let mut date_str_buf: [MaybeUninit<u16>; MAX_LEN] =
            transmute(MaybeUninit::<[u16; MAX_LEN]>::uninit());

        let result = datetimeapi::GetDateFormatW(
            winnt::LOCALE_INVARIANT,
            0, // no flags
            &system_time,
            null(),
            date_str_buf[0].as_mut_ptr(),
            MAX_LEN as _,
        );
        if result == 0 {
            return None;
        }
        (date_str_buf, (result - 1).try_into().ok()?)
    };

    let (time_str, time_str_len) = unsafe {
        // This is safe because `[MaybeUninit<u16>; MAX_LEN]` has no portion
        // that requires initialization
        let mut time_str_buf: [MaybeUninit<u16>; MAX_LEN] =
            transmute(MaybeUninit::<[u16; MAX_LEN]>::uninit());

        let result = datetimeapi::GetTimeFormatW(
            winnt::LOCALE_INVARIANT,
            0, // no flags
            &system_time,
            null(),
            time_str_buf[0].as_mut_ptr(),
            MAX_LEN as _,
        );
        if result == 0 {
            return None;
        }
        (time_str_buf, (result - 1).try_into().ok()?)
    };

    let _: usize = date_str_len;
    let _: usize = time_str_len;

    // Build the final output
    let final_str = unsafe {
        let mut out = MaybeUninit::<*mut u16>::uninit();

        let parts = [date_str[0].as_ptr(), time_str[0].as_ptr(), body.as_ptr()];

        let result = winbase::FormatMessageW(
            winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER		// allocate buffer using `LocalAlloc`
                | winbase::FORMAT_MESSAGE_FROM_STRING	// use a given format string
                | winbase::FORMAT_MESSAGE_ARGUMENT_ARRAY, // arguments are in an array, not `va_list`
            wchar::wch_c!("%1 %2 %3\n").as_ptr() as _,
            0, // message id - ignored
            0, // language id - ignored
            out.as_mut_ptr() as _,
            1, // minmum number of output `WCHAR` elements
            parts.as_ptr() as _,
        );

        if result == 0 {
            return None;
        }

        out.assume_init()
    };

    // Write the output
    unsafe {
        debugapi::OutputDebugStringW(final_str);
    }

    unsafe {
        winbase::LocalFree(final_str as _);
    }
    Some(())
}

/// Initialize the global logger with a specific log level that is
/// determined at compile time.
///
/// ```
/// # use log::{warn, info};
/// # fn main() {
/// windebug_logger::init_with_level_static!(log::Level::Warn).unwrap();
///
/// warn!("This is an example message.");
/// info!("This message will not be logged.");
/// # }
/// ```
#[macro_export]
macro_rules! init_with_level_static {
    ($level:expr) => {{
        let logger = &$crate::WinDebugLogger { level: $level };
        match $crate::log::set_logger(logger) {
            ::std::result::Result::Ok(()) => {
                $crate::log::set_max_level(logger.level.to_level_filter());
                Ok(())
            }
            ::std::result::Result::Err(e) => ::std::result::Result::Err(e),
        }
    }};
}

/// Initialize the global logger with a specific log level.
///
/// ```
/// # use log::{warn, info};
/// # fn main() {
/// windebug_logger::init_with_level(log::Level::Warn).unwrap();
///
/// warn!("This is an example message.");
/// info!("This message will not be logged.");
/// # }
/// ```
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    let logger = WinDebugLogger { level };
    match log::set_boxed_logger(Box::new(logger)) {
        Ok(()) => {
            log::set_max_level(level.to_level_filter());
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Initializes the global logger with a log level set to `LogLevel::Trace`.
///
/// ```
/// # use log::warn;
/// # fn main() {
/// windebug_logger::init().unwrap();
/// warn!("This is an example message.");
/// # }
/// ```
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level_static!(Level::Trace)
}
