//! # mathjax-svg-rs
//!
//! Very thin wrapper around MathJax to render TeX to SVG, using QuickJS-ng as the JavaScript
//! engine.
//!
//! # Notes
//!
//! - The SVG rendered use `ex` as width and height, so the actual size will depend on the
//!   font size used when rendering.
//! - The `render_tex` renders through a shared worker thread. Create a [`MathJax`] instance if you
//!   want to own a separate worker thread.

use std::str::FromStr;
use std::sync::{OnceLock, mpsc};
use std::thread;

struct Runtime {
    runtime: boa_engine::Context,
}

impl Runtime {
    fn new() -> Self {
        let mut context = boa_engine::Context::builder()
            .build()
            .expect("Failed to create JavaScript context");
        let current = std::time::Instant::now();
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
        log::debug!(
            "MathJax JavaScript context initialized in {} ms",
            current.elapsed().as_millis()
        );
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

enum WorkerMessage {
    Render {
        tex: String,
        font_size: f64,
        response: mpsc::SyncSender<Result<String, String>>,
    },
    Shutdown,
}

/// MathJax renderer backed by an internal worker thread.
pub struct MathJax {
    sender: mpsc::Sender<WorkerMessage>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MathJax {
    /// Creates a new MathJax renderer.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("mathjax-svg-rs".into())
            .stack_size(4 * 1024 * 1024) // 4 MiB stack size to avoid stack overflow when parsing MathJax JS code
            .spawn(move || {
                let mut runtime = Runtime::new();
                while let Ok(message) = receiver.recv() {
                    match message {
                        WorkerMessage::Render {
                            tex,
                            font_size,
                            response,
                        } => {
                            let _ = response.send(runtime.render_tex(&tex, font_size));
                        }
                        WorkerMessage::Shutdown => break,
                    }
                }
            })
            .expect("Failed to spawn MathJax worker thread");

        Self {
            sender,
            handle: Some(handle),
        }
    }

    /// Renders TeX to SVG.
    pub fn render_tex(&self, tex: &str) -> Result<String, String> {
        self.render_tex_with_font_size(tex, DEFAULT_FONT_SIZE)
    }

    /// Renders TeX to SVG with a font size in pixels.
    pub fn render_tex_with_font_size(&self, tex: &str, font_size: f64) -> Result<String, String> {
        validate_font_size(font_size)?;

        let (response, result) = mpsc::sync_channel(1);
        self.sender
            .send(WorkerMessage::Render {
                tex: tex.to_owned(),
                font_size,
                response,
            })
            .map_err(|_| "MathJax worker thread is unavailable".to_string())?;
        result
            .recv()
            .map_err(|_| "MathJax worker thread stopped before rendering finished".to_string())?
    }
}

impl Default for MathJax {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MathJax {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Default font size in pixels.
pub const DEFAULT_FONT_SIZE: f64 = 16.0;

fn validate_font_size(font_size: f64) -> Result<(), String> {
    if !font_size.is_finite() || font_size <= 0.0 {
        return Err(format!(
            "Font size must be positive and finite: {}",
            font_size
        ));
    }

    Ok(())
}

fn shared_mathjax() -> &'static MathJax {
    static MATHJAX: OnceLock<MathJax> = OnceLock::new();
    MATHJAX.get_or_init(MathJax::new)
}

/// Renders TeX to SVG.
pub fn render_tex(tex: &str) -> Result<String, String> {
    shared_mathjax().render_tex(tex)
}

/// Renders TeX to SVG with a font size in pixels.
pub fn render_tex_with_font_size(tex: &str, font_size: f64) -> Result<String, String> {
    shared_mathjax().render_tex_with_font_size(tex, font_size)
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

    #[test]
    fn test_mathjax_render_tex() {
        let mathjax = MathJax::new();
        let svg = mathjax
            .render_tex(r"\sqrt{x}")
            .expect("Failed to render TeX");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_mathjax_can_render_from_multiple_threads() {
        let mathjax = std::sync::Arc::new(MathJax::new());
        let handles = (0..4)
            .map(|index| {
                let mathjax = mathjax.clone();
                std::thread::spawn(move || {
                    mathjax
                        .render_tex(&format!("x_{}", index))
                        .expect("Failed to render TeX")
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            let svg = handle.join().expect("Thread panicked");
            assert!(svg.contains("<svg"));
            assert!(svg.contains("</svg>"));
        }
    }
}
