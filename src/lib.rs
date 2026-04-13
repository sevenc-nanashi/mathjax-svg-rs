//! # mathjax-svg-rs
//!
//! Very thin wrapper around MathJax to render TeX to SVG, using QuickJS-ng as the JavaScript
//! engine.

struct Runtime {
    runtime: quickjs_rusty::Context,
}

struct LogConsoleHandler;
impl quickjs_rusty::console::ConsoleBackend for LogConsoleHandler {
    fn log(&self, level: quickjs_rusty::console::Level, values: Vec<quickjs_rusty::OwnedJsValue>) {
        match level {
            quickjs_rusty::console::Level::Log => log::info!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            quickjs_rusty::console::Level::Trace => log::trace!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            quickjs_rusty::console::Level::Debug => log::debug!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            quickjs_rusty::console::Level::Info => log::info!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            quickjs_rusty::console::Level::Warn => log::warn!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            quickjs_rusty::console::Level::Error => log::error!(
                "{}",
                values
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

fn format_value(value: &quickjs_rusty::OwnedJsValue) -> String {
    if let Ok(s) = value.to_string() {
        s
    } else {
        format!("{:?}", value)
    }
}

impl Runtime {
    fn new() -> Self {
        let context = quickjs_rusty::Context::builder()
            .console(LogConsoleHandler)
            .build()
            .expect("Failed to create QuickJS context");
        context
            .eval(include_zstd::file_str!("../js/dist/index.js"), false)
            .map_err(|e| match e {
                quickjs_rusty::ExecutionError::Exception(exception) => {
                    if exception.is_string() {
                        format!("JavaScript exception: {}", exception.to_string().unwrap())
                    } else {
                        format!("JavaScript exception: {:?}", exception)
                    }
                }
                other => format!("Failed to evaluate script: {:?}", other),
            })
            .expect("Failed to evaluate MathJax script");
        Self { runtime: context }
    }

    fn render_tex(&self, tex: &str) -> Result<String, String> {
        let result = self
            .runtime
            .call_function("__entry_renderTeX", [tex.to_string()])
            .map_err(|e| format!("Failed to call render function: {}", e))?;
        result
            .to_string()
            .map_err(|e| format!("Failed to convert result to string: {}", e))
    }
}

/// Renders TeX to SVG.
pub fn render_tex(tex: &str) -> Result<String, String> {
    thread_local! {
        static RUNTIME: std::cell::RefCell<Runtime> = std::cell::RefCell::new(Runtime::new());
    }
    RUNTIME.with(|runtime| runtime.borrow().render_tex(tex))
}

/// Information about the license used by this crate.
pub const NOTICE: &str = include_str!("../js/dist/NOTICE.txt");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_tex() {
        let tex = r"\frac{a}{b}";
        let svg = render_tex(tex).expect("Failed to render TeX");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }
}
