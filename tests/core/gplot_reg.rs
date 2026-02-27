//! Regression tests for GPlot — graph plotting module.
//!
//! Corresponds to C Leptonica `gplot1_reg.c` / `gplot2_reg.c`.

use leptonica::{
    GPlot, GPlotOutput, GPlotScaling, Numa, Numaa, PlotStyle, gplot_simple_1, gplot_simple_2,
    gplot_simple_n, gplot_simple_pix_1, gplot_simple_pix_2, gplot_simple_pix_n,
};

// ---------------------------------------------------------------------------
// GPlot creation & configuration
// ---------------------------------------------------------------------------

#[test]
fn gplot_create_basic() {
    let gp = GPlot::new(None, GPlotOutput::Svg, "Test Plot", "X axis", "Y axis");
    assert!(gp.make_output_svg().unwrap().contains("Test Plot"));
}

#[test]
fn gplot_add_single_series() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Single", "x", "y");
    let mut na = Numa::new();
    for i in 0..30 {
        na.push((i as f32 * 0.2).sin());
    }
    gp.add_plot(None, &na, PlotStyle::Lines, Some("sin(x)"));
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("<path"));
    assert!(svg.contains("sin(x)"));
}

#[test]
fn gplot_add_multiple_series() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Multi", "x", "y");
    let mut na1 = Numa::new();
    let mut na2 = Numa::new();
    for i in 0..20 {
        na1.push(i as f32);
        na2.push((i as f32).sqrt());
    }
    gp.add_plot(None, &na1, PlotStyle::Lines, Some("linear"));
    gp.add_plot(None, &na2, PlotStyle::Points, Some("sqrt"));
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("linear"));
    assert!(svg.contains("sqrt"));
    // Two different color series should exist
    assert!(svg.contains("rgb(31,119,180)")); // first series color
    assert!(svg.contains("rgb(255,127,14)")); // second series color
}

#[test]
fn gplot_with_custom_x_data() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Custom X", "t", "v");
    let mut nax = Numa::new();
    let mut nay = Numa::new();
    for i in 0..15 {
        nax.push(i as f32 * 0.5);
        nay.push((i as f32 * 0.5).cos());
    }
    gp.add_plot(Some(&nax), &nay, PlotStyle::LinesPoints, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("<path"));
    assert!(svg.contains("<circle"));
}

#[test]
#[allow(deprecated)]
fn gplot_set_scaling() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Scaled", "", "");
    gp.set_scaling(GPlotScaling::LogX);
    let mut na = Numa::new();
    for i in 1..=10 {
        na.push(i as f32);
    }
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    // Should not panic
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("</svg>"));
}

// ---------------------------------------------------------------------------
// Plot styles
// ---------------------------------------------------------------------------

#[test]
fn gplot_style_impulses() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Impulses", "", "");
    let mut na = Numa::new();
    for i in 0..10 {
        na.push(if i % 2 == 0 { 5.0 } else { -3.0 });
    }
    gp.add_plot(None, &na, PlotStyle::Impulses, None);
    let svg = gp.make_output_svg().unwrap();
    // Impulses render as individual lines
    assert!(svg.contains("<line"));
}

#[test]
fn gplot_style_dots() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Dots", "", "");
    let mut na = Numa::new();
    for i in 0..10 {
        na.push(i as f32);
    }
    gp.add_plot(None, &na, PlotStyle::Dots, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains(r#"r="1""#)); // small dots
}

// ---------------------------------------------------------------------------
// SVG output
// ---------------------------------------------------------------------------

#[test]
fn gplot_svg_structure() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Structure", "X", "Y");
    let mut na = Numa::new();
    for i in 0..5 {
        na.push(i as f32 * 10.0);
    }
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    let svg = gp.make_output_svg().unwrap();

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("xmlns"));
    assert!(svg.contains("width=\"640\""));
    assert!(svg.contains("height=\"480\""));
    assert!(svg.contains("</svg>"));
    // Has title
    assert!(svg.contains("Structure"));
    // Has axis labels
    assert!(svg.contains(">X<"));
    assert!(svg.contains(">Y<"));
    // Has plot area
    assert!(svg.contains("clipPath"));
}

#[test]
fn gplot_svg_empty_plot() {
    let gp = GPlot::new(None, GPlotOutput::Svg, "", "", "");
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn gplot_make_output_svg_file() {
    let dir = std::env::temp_dir().join("leptonica_test_gplot");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_output.svg");

    let mut gp = GPlot::new(path.to_str(), GPlotOutput::Svg, "File Test", "x", "y");
    let mut na = Numa::new();
    for i in 0..10 {
        na.push(i as f32);
    }
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    gp.make_output().unwrap();

    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("File Test"));

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Pix output
// ---------------------------------------------------------------------------

#[test]
fn gplot_make_output_pix_basic() {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, "Pix Test", "x", "y");
    let mut na = Numa::new();
    for i in 0..30 {
        na.push((i as f32 * 0.3).sin());
    }
    gp.add_plot(None, &na, PlotStyle::Lines, Some("sin"));
    let pix = gp.make_output_pix().unwrap();
    assert_eq!(pix.width(), 640);
    assert_eq!(pix.height(), 480);
    assert_eq!(pix.depth(), leptonica::PixelDepth::Bit32);
}

#[test]
fn gplot_pix_multiple_series() {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, "Multi Pix", "", "");
    let mut na1 = Numa::new();
    let mut na2 = Numa::new();
    for i in 0..20 {
        na1.push(i as f32);
        na2.push(20.0 - i as f32);
    }
    gp.add_plot(None, &na1, PlotStyle::Lines, Some("up"));
    gp.add_plot(None, &na2, PlotStyle::Points, Some("down"));
    let pix = gp.make_output_pix().unwrap();
    assert_eq!(pix.width(), 640);
}

#[test]
fn gplot_pix_all_styles() {
    for style in [
        PlotStyle::Lines,
        PlotStyle::Points,
        PlotStyle::LinesPoints,
        PlotStyle::Impulses,
        PlotStyle::Dots,
    ] {
        let mut gp = GPlot::new(None, GPlotOutput::Pix, &format!("{:?}", style), "", "");
        let mut na = Numa::new();
        for i in 0..15 {
            na.push(i as f32);
        }
        gp.add_plot(None, &na, style, None);
        let pix = gp.make_output_pix().unwrap();
        assert_eq!(pix.width(), 640, "style {:?} failed", style);
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

#[test]
fn gplot_simple_1_svg() {
    let mut na = Numa::new();
    for i in 0..25 {
        na.push(i as f32 * i as f32);
    }
    gplot_simple_1(&na, GPlotOutput::Svg, None, "Simple 1").unwrap();
}

#[test]
fn gplot_simple_2_svg() {
    let mut na1 = Numa::new();
    let mut na2 = Numa::new();
    for i in 0..25 {
        na1.push(i as f32);
        na2.push((i as f32).sqrt() * 5.0);
    }
    gplot_simple_2(&na1, &na2, GPlotOutput::Svg, None, "Simple 2").unwrap();
}

#[test]
fn gplot_simple_n_svg() {
    let mut naa = Numaa::new();
    for k in 0..3 {
        let mut na = Numa::new();
        for i in 0..20 {
            na.push(i as f32 + k as f32 * 5.0);
        }
        naa.push(na);
    }
    gplot_simple_n(&naa, GPlotOutput::Svg, None, "Simple N").unwrap();
}

#[test]
fn gplot_simple_pix_1_test() {
    let mut na = Numa::new();
    for i in 0..30 {
        na.push((i as f32 * 0.2).cos());
    }
    let pix = gplot_simple_pix_1(&na, "Pix 1").unwrap();
    assert_eq!(pix.width(), 640);
    assert_eq!(pix.height(), 480);
}

#[test]
fn gplot_simple_pix_2_test() {
    let mut na1 = Numa::new();
    let mut na2 = Numa::new();
    for i in 0..20 {
        na1.push(i as f32);
        na2.push(100.0 - i as f32 * 5.0);
    }
    let pix = gplot_simple_pix_2(&na1, &na2, "Pix 2").unwrap();
    assert_eq!(pix.width(), 640);
}

#[test]
fn gplot_simple_pix_n_test() {
    let mut naa = Numaa::new();
    for k in 0..4 {
        let mut na = Numa::new();
        for i in 0..15 {
            na.push(i as f32 * (k + 1) as f32);
        }
        naa.push(na);
    }
    let pix = gplot_simple_pix_n(&naa, "Pix N").unwrap();
    assert_eq!(pix.width(), 640);
}

// ---------------------------------------------------------------------------
// PNG unsupported
// ---------------------------------------------------------------------------

#[test]
fn gplot_png_output_unsupported() {
    let mut gp = GPlot::new(None, GPlotOutput::Png, "PNG", "", "");
    let mut na = Numa::new();
    na.push(1.0);
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    assert!(gp.make_output().is_err());
}

// ---------------------------------------------------------------------------
// No-op helpers
// ---------------------------------------------------------------------------

#[test]
fn gplot_gen_command_file_noop() {
    let gp = GPlot::new(None, GPlotOutput::Svg, "", "", "");
    assert!(gp.gen_command_file().is_ok());
}

#[test]
fn gplot_gen_data_files_noop() {
    let gp = GPlot::new(None, GPlotOutput::Svg, "", "", "");
    assert!(gp.gen_data_files().is_ok());
}

// ---------------------------------------------------------------------------
// Numa parameters (startx, delx)
// ---------------------------------------------------------------------------

#[test]
fn gplot_numa_with_parameters() {
    let mut na = Numa::new();
    na.set_parameters(10.0, 0.5);
    for i in 0..20 {
        na.push(i as f32 * 2.0);
    }
    // x values should be 10.0, 10.5, 11.0, ... 19.5
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Params", "t", "val");
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("</svg>"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn gplot_single_point() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Single Point", "", "");
    let mut na = Numa::new();
    na.push(42.0);
    gp.add_plot(None, &na, PlotStyle::Points, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("</svg>"));
}

#[test]
fn gplot_constant_data() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Constant", "", "");
    let mut na = Numa::new();
    for _ in 0..10 {
        na.push(5.0);
    }
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("</svg>"));
}

#[test]
fn gplot_negative_data() {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, "Negative", "", "");
    let mut na = Numa::new();
    for i in 0..20 {
        na.push(i as f32 - 10.0);
    }
    gp.add_plot(None, &na, PlotStyle::Impulses, None);
    let pix = gp.make_output_pix().unwrap();
    assert_eq!(pix.width(), 640);
}

#[test]
fn gplot_large_dataset() {
    let mut gp = GPlot::new(None, GPlotOutput::Svg, "Large", "", "");
    let mut na = Numa::new();
    for i in 0..1000 {
        na.push((i as f32 * 0.01).sin());
    }
    gp.add_plot(None, &na, PlotStyle::Lines, None);
    let svg = gp.make_output_svg().unwrap();
    assert!(svg.contains("</svg>"));
}
