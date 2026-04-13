//! # mathjax-svg-rs
//!
//! Very thin wrapper around MathJax to render TeX to SVG, using QuickJS-ng as the JavaScript
//! engine.

struct Runtime {
    runtime: quickjs_rusty::Context,
}

impl Runtime {
    fn new() -> Self {
        let context = quickjs_rusty::Context::new(None).expect("Failed to create QuickJS context");
        context
            .eval(include_str!("../js/dist/index.js"), false)
            .map_err(|e| match e {
                quickjs_rusty::ExecutionError::Exception(exception) => {
                    if exception.is_string() {
                        format!(
                            "JavaScript exception: {}",
                            exception.to_string().unwrap()
                        )
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
