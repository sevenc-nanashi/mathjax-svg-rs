use clap::{Parser, ValueEnum};
use mathjax_svg_rs::{HorizontalAlign, Options};

#[derive(Debug, Parser)]
struct Args {
    /// TeX string to render.
    tex: String,

    /// PNG or SVG output file.
    output_file: String,

    /// Font size used for rendering.
    #[arg(long, default_value_t = 16.0, value_parser = validate_font_size)]
    font_size: f64,

    /// Alignment of the rendered TeX.
    #[arg(long, value_enum, default_value_t = CliHorizontalAlign::Center)]
    align: CliHorizontalAlign,
}

fn validate_font_size(font_size: &str) -> Result<f64, String> {
    let font_size: f64 = font_size
        .parse()
        .map_err(|_| format!("Font size must be a valid number: {}", font_size))?;
    if !font_size.is_finite() || font_size <= 0.0 {
        return Err(format!(
            "Font size must be positive and finite: {}",
            font_size
        ));
    }
    Ok(font_size)
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliHorizontalAlign {
    Left,
    Center,
    Right,
}

impl From<CliHorizontalAlign> for HorizontalAlign {
    fn from(value: CliHorizontalAlign) -> Self {
        match value {
            CliHorizontalAlign::Left => Self::Left,
            CliHorizontalAlign::Center => Self::Center,
            CliHorizontalAlign::Right => Self::Right,
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    log::info!("Rendering TeX: {}", args.tex);
    let runtime = mathjax_svg_rs::MathJax::new();
    let options = Options {
        font_size: args.font_size,
        horizontal_align: args.align.into(),
    };
    let svg = runtime
        .render_tex(&args.tex, &options)
        .expect("Failed to render TeX");
    log::info!("Rendered SVG: {}", svg);

    if args.output_file.ends_with(".svg") {
        log::info!("Saving SVG to: {}", args.output_file);
        std::fs::write(args.output_file, svg).expect("Failed to save SVG");
        return;
    }
    let tree = resvg::usvg::Tree::from_str(
        &svg,
        &resvg::usvg::Options {
            font_size: options.font_size as f32,
            ..Default::default()
        },
    )
    .expect("Failed to parse SVG");
    log::info!(
        "SVG size: {}x{}",
        tree.size().width().ceil(),
        tree.size().height().ceil()
    );
    let mut canvas = resvg::tiny_skia::Pixmap::new(
        tree.size().width().ceil() as u32,
        tree.size().height().ceil() as u32,
    )
    .expect("Failed to create canvas");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut canvas.as_mut(),
    );

    log::info!("Saving PNG to: {}", args.output_file);
    canvas
        .save_png(args.output_file)
        .expect("Failed to save PNG");
}
