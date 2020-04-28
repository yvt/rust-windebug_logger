# windebug_logger

[![Latest version](https://img.shields.io/crates/v/windebug_logger.svg)](https://crates.io/crates/windebug_logger)
[![Documentation](https://docs.rs/windebug_logger/badge.svg)](https://docs.rs/windebug_logger)
![License](https://img.shields.io/crates/l/windebug_logger.svg)

A logger that redirects all messages to the [`OutputDebugStringW`]
Win32 API function in a format similar to [`simple_logger`].

[`OutputDebugStringW`]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-outputdebugstringw
[`simple_logger`]: https://crates.io/crates/simple_logger

This crate is useful if you are writing a Windows GUI application, where
stdout and stderr do not work. The messages outputted by
`OutputDebugStringW` can be monitored using a program such as [DebugView]
and Visual Studio's "Output" window.

[DebugView]: https://docs.microsoft.com/en-us/sysinternals/downloads/debugview

License: MIT/Apache-2.0
