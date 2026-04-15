# mathjax-svg-rs

Very thin wrapper around MathJax to render TeX to SVG, using Boajs as the JavaScript engine.

## Usage

```rust
let runtime = mathjax_svg_rs::MathJax::new();
let options = Options {
    font_size: args.font_size,
    horizontal_align: args.align.into(),
};
let svg = runtime
    .render_tex(&args.tex, &options)
    .expect("Failed to render TeX");
log::info!("Rendered SVG: {}", svg);
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
