//! # mathjax-svg-rs
//!
//! Very thin wrapper around MathJax to render TeX to SVG, using QuickJS-ng as the JavaScript
//! engine.
//!
//! # Notes
//!
//! - The SVG rendered use `ex` as width and height, so the actual size will depend on the
//!   font size used when rendering.
//! - The `render_tex` initializes the JavaScript runtime on the first call for each thread, thus
//!   you might want to create a dedicated thread for rendering if you are developing a
//!   multi-threaded application.

use std::str::FromStr;

struct Runtime {
    runtime: boa_engine::Context,
}

impl Runtime {
    fn new() -> Self {
        let mut context = boa_engine::Context::builder()
            .build()
            .expect("Failed to create JavaScript context");
        context
            .eval(boa_engine::Source::from_bytes(include_zstd::file_str!(
                "../js/dist/index.js"
            )))
            .expect("Failed to evaluate MathJax script");
        let log = boa_engine::object::FunctionObjectBuilder::new(
            context.realm(),
            boa_engine::NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let message = args
                    .get(1)
                    .and_then(|v| v.to_string(ctx).ok())
                    .and_then(|s| s.to_std_string().ok())
                    .unwrap_or_else(|| "Unknown log message".into());
                let level = args.first().and_then(|v| v.to_u32(ctx).ok()).unwrap_or(2);
                match level {
                    0 => log::trace!("{}", message),
                    1 => log::debug!("{}", message),
                    2 => log::info!("{}", message),
                    3 => log::warn!("{}", message),
                    4 => log::error!("{}", message),
                    _ => {
                        log::warn!("Unknown log level {}: {}", level, message);
                    }
                }
                Ok(boa_engine::JsValue::undefined())
            }),
        )
        .build();
        context
            .global_object()
            .set(
                boa_engine::property::PropertyKey::String("__host_log".into()),
                log,
                false,
                &mut context,
            )
            .expect("Failed to set log function");
        Self { runtime: context }
    }

    fn render_tex(&mut self, tex: &str, font_size: f64) -> Result<String, String> {
        let result = self
            .runtime
            .global_object()
            .get(
                boa_engine::property::PropertyKey::String("__entry_renderTeX".into()),
                &mut self.runtime,
            )
            .expect("Failed to get render function")
            .as_object()
            .expect("Render function is not an object")
            .call(
                &boa_engine::JsValue::null(),
                &[
                    boa_engine::JsValue::new(boa_engine::JsString::from_str(tex).map_err(|e| {
                        format!("Failed to convert TeX to JavaScript string: {}", e)
                    })?),
                    boa_engine::JsValue::new(font_size),
                ],
                &mut self.runtime,
            )
            .map_err(|e| format!("Failed to call render function: {}", e))?;
        result
            .to_string(&mut self.runtime)
            .map_err(|e| format!("Failed to convert result to string: {}", e))?
            .to_std_string()
            .map_err(|e| format!("Failed to convert result to Rust string: {}", e))
    }
}

/// Default font size in pixels.
pub const DEFAULT_FONT_SIZE: f64 = 16.0;

/// Renders TeX to SVG.
pub fn render_tex(tex: &str) -> Result<String, String> {
    render_tex_with_font_size(tex, DEFAULT_FONT_SIZE)
}

/// Renders TeX to SVG with a font size in pixels.
pub fn render_tex_with_font_size(tex: &str, font_size: f64) -> Result<String, String> {
    if !font_size.is_finite() || font_size <= 0.0 {
        return Err(format!(
            "Font size must be positive and finite: {}",
            font_size
        ));
    }

    thread_local! {
        static RUNTIME: std::sync::Mutex<Runtime> = std::sync::Mutex::new(Runtime::new());
    }
    RUNTIME.with(|runtime| {
        let mut runtime = runtime.lock().unwrap();
        runtime.render_tex(tex, font_size)
    })
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

    #[test]
    fn test_render_tex_with_font_size() {
        let tex = r"\frac{a}{b}";
        let default_svg = render_tex(tex).expect("Failed to render TeX");
        let svg = render_tex_with_font_size(tex, 32.0).expect("Failed to render TeX");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert_ne!(default_svg, svg);
    }

    #[test]
    fn test_render_tex_with_invalid_font_size() {
        let error = render_tex_with_font_size(r"x", 0.0).expect_err("Expected an error");
        assert!(error.contains("Font size must be positive and finite"));
    }
}
