fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if std::env::args().len() < 4 {
        eprintln!("Usage: render_arg <TeX string> <png output file> <font size>");
        std::process::exit(1);
    }
    log::info!("Rendering TeX: {}", std::env::args().nth(1).unwrap());
    let tex = std::env::args().nth(1).unwrap();
    let runtime = mathjax_svg_rs::MathJax::new();
    let font_size = std::env::args()
        .nth(3)
        .expect("Font size is required")
        .parse::<f64>()
        .expect("Failed to parse font size as a number");
    let svg = runtime
        .render_tex_with_font_size(&tex, font_size)
        .expect("Failed to render TeX");
    log::info!("Rendered SVG: {}", svg);

    let tree = resvg::usvg::Tree::from_str(
        &svg,
        &resvg::usvg::Options {
            font_size: font_size as f32,
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

    let output_file = std::env::args().nth(2).unwrap();
    log::info!("Saving PNG to: {}", output_file);
    canvas.save_png(output_file).expect("Failed to save PNG");
}
