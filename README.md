# windebug_logger

A logger that redirects all messages to the [`OutputDebugMessageW`]
Win32 API function in a format similar to [`simple_logger`].

[`OutputDebugMessageW`]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-outputdebugstringw
[`simple_logger`]: https://crates.io/crates/simple_logger

This crate is useful if you are writing a Windows GUI application, where
stdout and stderr do not work. The messages outputted by
`OutputDebugMessageW` can be monitored using a program such as [DebugView]
and Visual Studio's "Output" window.

[DebugView]: https://docs.microsoft.com/en-us/sysinternals/downloads/debugview

License: MIT/Apache-2.0