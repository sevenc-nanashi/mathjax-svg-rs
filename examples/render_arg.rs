fn main() {
    unsafe { std::env::set_var("RUST_LOG", "info") };
    env_logger::init();

    if std::env::args().len() < 2 {
        eprintln!("Usage: render_arg <TeX string> <png output file>");
        std::process::exit(1);
    }
    let tex = std::env::args().nth(1).unwrap();
    let svg = mathjax_svg_rs::render_tex(&tex).expect("Failed to render TeX");

    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .expect("Failed to parse SVG");
    let mut canvas = resvg::tiny_skia::Pixmap::new(
        tree.size().width().ceil() as u32,
        tree.size().height().ceil() as u32,
    )
    .expect("Failed to create canvas");
    resvg::render(
        &resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
            .expect("Failed to parse SVG"),
        resvg::tiny_skia::Transform::identity(),
        &mut canvas.as_mut(),
    );

    let output_file = std::env::args().nth(2).unwrap();
    canvas.save_png(output_file).expect("Failed to save PNG");
}
