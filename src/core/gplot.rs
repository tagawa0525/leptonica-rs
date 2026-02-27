//! GPlot — Graph plotting with SVG and Pix output
//!
//! Rust port of C Leptonica's `gplot.c`. Instead of depending on gnuplot,
//! this implementation generates SVG directly and renders to [`Pix`] using
//! the existing graphics primitives.
//!
//! # Example
//!
//! ```
//! use leptonica::{GPlot, GPlotOutput, GPlotScaling, PlotStyle, Numa};
//!
//! let mut na = Numa::new();
//! for i in 0..50 {
//!     na.push((i as f32).sin());
//! }
//!
//! let mut gp = GPlot::new(None, GPlotOutput::Svg, "Sine", "x", "y");
//! gp.add_plot(None, &na, PlotStyle::Lines, Some("sin"));
//! let svg = gp.make_output_svg().unwrap();
//! ```

use std::fmt;
use std::path::PathBuf;

use super::error::{Error, Result};
use super::numa::{Numa, Numaa};
use super::pix::graphics::Color;
use super::pix::{Pix, PixMut, PixelDepth};
// ---------------------------------------------------------------------------
// Public enums
// ---------------------------------------------------------------------------

/// Output format for plots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GPlotOutput {
    /// Scalable Vector Graphics (written as text)
    Svg,
    /// PNG — currently unsupported; returns an error
    Png,
    /// Raster image returned as [`Pix`]
    Pix,
}

/// Axis scaling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GPlotScaling {
    #[default]
    Linear,
    LogX,
    LogY,
    LogXY,
}

/// Visual style of a data series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlotStyle {
    #[default]
    Lines,
    Points,
    LinesPoints,
    Impulses,
    Dots,
}

impl fmt::Display for PlotStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lines => write!(f, "lines"),
            Self::Points => write!(f, "points"),
            Self::LinesPoints => write!(f, "linespoints"),
            Self::Impulses => write!(f, "impulses"),
            Self::Dots => write!(f, "dots"),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal data
// ---------------------------------------------------------------------------

/// One data series to be plotted.
#[derive(Debug, Clone)]
struct PlotData {
    x_data: Vec<f32>,
    y_data: Vec<f32>,
    style: PlotStyle,
    label: Option<String>,
}

/// Default canvas dimensions (pixels).
const CANVAS_W: u32 = 640;
const CANVAS_H: u32 = 480;

/// Plot area margins.
const MARGIN_LEFT: f32 = 80.0;
const MARGIN_RIGHT: f32 = 30.0;
const MARGIN_TOP: f32 = 50.0;
const MARGIN_BOTTOM: f32 = 60.0;

/// Pix-rendering margins (integer).
const PIX_MARGIN_LEFT: i32 = 80;
const PIX_MARGIN_RIGHT: i32 = 30;
const PIX_MARGIN_TOP: i32 = 50;
const PIX_MARGIN_BOTTOM: i32 = 60;

/// Palette for multiple series.
const SERIES_COLORS: &[(u8, u8, u8)] = &[
    (31, 119, 180),  // blue
    (255, 127, 14),  // orange
    (44, 160, 44),   // green
    (214, 39, 40),   // red
    (148, 103, 189), // purple
    (140, 86, 75),   // brown
    (227, 119, 194), // pink
    (127, 127, 127), // gray
];

fn series_color(index: usize) -> (u8, u8, u8) {
    SERIES_COLORS[index % SERIES_COLORS.len()]
}

fn series_color_obj(index: usize) -> Color {
    let (r, g, b) = series_color(index);
    Color::new(r, g, b)
}

// ---------------------------------------------------------------------------
// GPlot struct
// ---------------------------------------------------------------------------

/// A graph plotter that generates SVG text or rasterised [`Pix`] images.
///
/// Rust replacement for C Leptonica's `GPLOT` / gnuplot pipeline.
#[derive(Debug, Clone)]
pub struct GPlot {
    title: String,
    xlabel: String,
    ylabel: String,
    output_path: Option<PathBuf>,
    output_type: GPlotOutput,
    scaling: GPlotScaling,
    plots: Vec<PlotData>,
}

impl GPlot {
    // -- Core operations ---------------------------------------------------

    /// Create a new plot (corresponds to C `gplotCreate`).
    ///
    /// `output_path` is optional; when `None` the plot is generated in-memory
    /// only (useful for [`GPlotOutput::Pix`] or when you call
    /// [`make_output_svg`] directly).
    pub fn new(
        output_path: Option<&str>,
        output_type: GPlotOutput,
        title: &str,
        xlabel: &str,
        ylabel: &str,
    ) -> Self {
        Self {
            title: title.to_string(),
            xlabel: xlabel.to_string(),
            ylabel: ylabel.to_string(),
            output_path: output_path.map(PathBuf::from),
            output_type,
            scaling: GPlotScaling::default(),
            plots: Vec::new(),
        }
    }

    /// Add a data series (corresponds to C `gplotAddPlot`).
    ///
    /// If `numa_x` is `None`, the x-values are derived from the Numa
    /// parameters (`startx`, `delx`) of `numa_y`.
    pub fn add_plot(
        &mut self,
        numa_x: Option<&Numa>,
        numa_y: &Numa,
        style: PlotStyle,
        label: Option<&str>,
    ) {
        let n = numa_y.len();
        let y_data: Vec<f32> = numa_y.as_slice().to_vec();

        let x_data: Vec<f32> = if let Some(nx) = numa_x {
            nx.as_slice()[..n.min(nx.len())].to_vec()
        } else {
            (0..n).map(|i| numa_y.x_value(i)).collect()
        };

        self.plots.push(PlotData {
            x_data,
            y_data,
            style,
            label: label.map(String::from),
        });
    }

    /// Set axis scaling (corresponds to C `gplotSetScaling`).
    pub fn set_scaling(&mut self, scaling: GPlotScaling) {
        self.scaling = scaling;
    }

    // -- Output generation -------------------------------------------------

    /// Generate output according to `output_type` (corresponds to C
    /// `gplotMakeOutput`).
    ///
    /// For [`GPlotOutput::Svg`] the SVG is written to `output_path`.
    /// For [`GPlotOutput::Pix`] a raster image is written as BMP to
    /// `output_path` (if any). Returns `Ok(())`.
    pub fn make_output(&self) -> Result<()> {
        match self.output_type {
            GPlotOutput::Svg => {
                let svg = self.make_output_svg()?;
                if let Some(path) = &self.output_path {
                    std::fs::write(path, &svg)?;
                }
                Ok(())
            }
            GPlotOutput::Pix => {
                // Just generate the pix; writing is caller's responsibility
                let _pix = self.make_output_pix()?;
                Ok(())
            }
            GPlotOutput::Png => Err(Error::NotSupported(
                "PNG plot output is not supported; use Svg or Pix".into(),
            )),
        }
    }

    /// Generate the plot as a [`Pix`] image (corresponds to C
    /// `gplotMakeOutputPix`).
    pub fn make_output_pix(&self) -> Result<Pix> {
        render_pix(self)
    }

    /// Generate command file — no-op in the SVG/Pix pipeline
    /// (corresponds to C `gplotGenCommandFile`).
    pub fn gen_command_file(&self) -> Result<()> {
        Ok(())
    }

    /// Write data as CSV files — no-op in the SVG/Pix pipeline
    /// (corresponds to C `gplotGenDataFiles`).
    pub fn gen_data_files(&self) -> Result<()> {
        Ok(())
    }

    /// Generate SVG string from the current plot data.
    pub fn make_output_svg(&self) -> Result<String> {
        render_svg(self)
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Plot a single [`Numa`] (corresponds to C `gplotSimple1`).
pub fn gplot_simple_1(
    numa: &Numa,
    output_type: GPlotOutput,
    output_path: Option<&str>,
    title: &str,
) -> Result<()> {
    let mut gp = GPlot::new(output_path, output_type, title, "", "");
    gp.add_plot(None, numa, PlotStyle::Lines, None);
    gp.make_output()
}

/// Plot two [`Numa`]s (corresponds to C `gplotSimple2`).
pub fn gplot_simple_2(
    numa1: &Numa,
    numa2: &Numa,
    output_type: GPlotOutput,
    output_path: Option<&str>,
    title: &str,
) -> Result<()> {
    let mut gp = GPlot::new(output_path, output_type, title, "", "");
    gp.add_plot(None, numa1, PlotStyle::Lines, None);
    gp.add_plot(None, numa2, PlotStyle::Lines, None);
    gp.make_output()
}

/// Plot N [`Numa`]s from a [`Numaa`] (corresponds to C `gplotSimpleN`).
pub fn gplot_simple_n(
    numas: &Numaa,
    output_type: GPlotOutput,
    output_path: Option<&str>,
    title: &str,
) -> Result<()> {
    let mut gp = GPlot::new(output_path, output_type, title, "", "");
    for numa in numas.iter() {
        gp.add_plot(None, numa, PlotStyle::Lines, None);
    }
    gp.make_output()
}

/// Plot a single [`Numa`] and return a [`Pix`]
/// (corresponds to C `gplotSimplePix1`).
pub fn gplot_simple_pix_1(numa: &Numa, title: &str) -> Result<Pix> {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, title, "", "");
    gp.add_plot(None, numa, PlotStyle::Lines, None);
    gp.make_output_pix()
}

/// Plot two [`Numa`]s and return a [`Pix`]
/// (corresponds to C `gplotSimplePix2`).
pub fn gplot_simple_pix_2(numa1: &Numa, numa2: &Numa, title: &str) -> Result<Pix> {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, title, "", "");
    gp.add_plot(None, numa1, PlotStyle::Lines, None);
    gp.add_plot(None, numa2, PlotStyle::Lines, None);
    gp.make_output_pix()
}

/// Plot N [`Numa`]s from a [`Numaa`] and return a [`Pix`]
/// (corresponds to C `gplotSimplePixN`).
pub fn gplot_simple_pix_n(numas: &Numaa, title: &str) -> Result<Pix> {
    let mut gp = GPlot::new(None, GPlotOutput::Pix, title, "", "");
    for numa in numas.iter() {
        gp.add_plot(None, numa, PlotStyle::Lines, None);
    }
    gp.make_output_pix()
}

// ===========================================================================
// SVG Renderer
// ===========================================================================

/// Compute nice axis tick parameters: (tick_start, tick_step, tick_count).
fn nice_ticks(min_val: f32, max_val: f32, target_ticks: u32) -> (f32, f32, u32) {
    if (max_val - min_val).abs() < f32::EPSILON {
        let v = min_val;
        if v.abs() < f32::EPSILON {
            return (-1.0, 1.0, 3);
        }
        let mag = 10f32.powf(v.abs().log10().floor());
        let lo = (v / mag).floor() * mag - mag;
        return (lo, mag, 3);
    }

    let range = max_val - min_val;
    let rough_step = range / target_ticks as f32;
    let mag = 10f32.powf(rough_step.abs().log10().floor());
    let nice = if rough_step / mag < 1.5 {
        mag
    } else if rough_step / mag < 3.5 {
        2.0 * mag
    } else if rough_step / mag < 7.5 {
        5.0 * mag
    } else {
        10.0 * mag
    };

    let lo = (min_val / nice).floor() * nice;
    let hi = (max_val / nice).ceil() * nice;
    let count = ((hi - lo) / nice).round() as u32 + 1;
    (lo, nice, count.max(2))
}

/// Format a number for axis labels: drop trailing zeros.
fn fmt_num(v: f32) -> String {
    if v.abs() >= 1e6 || (v != 0.0 && v.abs() < 1e-3) {
        format!("{:.2e}", v)
    } else if (v - v.round()).abs() < 1e-9 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Escape XML special characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Data range across all series.
struct DataRange {
    xmin: f32,
    xmax: f32,
    ymin: f32,
    ymax: f32,
}

fn compute_range(gplot: &GPlot) -> Option<DataRange> {
    if gplot.plots.is_empty() {
        return None;
    }
    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;
    for p in &gplot.plots {
        for &x in &p.x_data {
            if x < xmin {
                xmin = x;
            }
            if x > xmax {
                xmax = x;
            }
        }
        for &y in &p.y_data {
            if y < ymin {
                ymin = y;
            }
            if y > ymax {
                ymax = y;
            }
        }
    }
    if xmin > xmax || ymin > ymax {
        return None;
    }
    // Add small padding when range is zero
    if (xmax - xmin).abs() < f32::EPSILON {
        xmin -= 1.0;
        xmax += 1.0;
    }
    if (ymax - ymin).abs() < f32::EPSILON {
        ymin -= 1.0;
        ymax += 1.0;
    }
    Some(DataRange {
        xmin,
        xmax,
        ymin,
        ymax,
    })
}

/// Map data coordinate → SVG pixel coordinate.
fn map_x(val: f32, dmin: f32, dmax: f32, pmin: f32, pmax: f32) -> f32 {
    if (dmax - dmin).abs() < f32::EPSILON {
        return (pmin + pmax) / 2.0;
    }
    pmin + (val - dmin) / (dmax - dmin) * (pmax - pmin)
}

fn map_y(val: f32, dmin: f32, dmax: f32, pmin: f32, pmax: f32) -> f32 {
    if (dmax - dmin).abs() < f32::EPSILON {
        return (pmin + pmax) / 2.0;
    }
    // Y is flipped (SVG origin at top-left)
    pmax - (val - dmin) / (dmax - dmin) * (pmax - pmin)
}

fn render_svg(gplot: &GPlot) -> Result<String> {
    let range = compute_range(gplot).unwrap_or(DataRange {
        xmin: 0.0,
        xmax: 1.0,
        ymin: 0.0,
        ymax: 1.0,
    });

    let w = CANVAS_W as f32;
    let h = CANVAS_H as f32;
    let plot_left = MARGIN_LEFT;
    let plot_right = w - MARGIN_RIGHT;
    let plot_top = MARGIN_TOP;
    let plot_bottom = h - MARGIN_BOTTOM;

    let (xtick_start, xtick_step, xtick_count) = nice_ticks(range.xmin, range.xmax, 6);
    let (ytick_start, ytick_step, ytick_count) = nice_ticks(range.ymin, range.ymax, 5);

    let xdata_min = xtick_start;
    let xdata_max = xtick_start + xtick_step * (xtick_count - 1) as f32;
    let ydata_min = ytick_start;
    let ydata_max = ytick_start + ytick_step * (ytick_count - 1) as f32;

    let mut svg = String::with_capacity(4096);

    // Header
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        CANVAS_W, CANVAS_H, CANVAS_W, CANVAS_H
    ));
    svg.push('\n');

    // Background
    svg.push_str(&format!(
        r#"<rect width="{}" height="{}" fill="white"/>"#,
        CANVAS_W, CANVAS_H
    ));
    svg.push('\n');

    // Title
    if !gplot.title.is_empty() {
        svg.push_str(&format!(
            r#"<text x="{}" y="30" text-anchor="middle" font-size="16" font-family="sans-serif" font-weight="bold">{}</text>"#,
            w / 2.0,
            xml_escape(&gplot.title)
        ));
        svg.push('\n');
    }

    // X label
    if !gplot.xlabel.is_empty() {
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" text-anchor="middle" font-size="12" font-family="sans-serif">{}</text>"#,
            (plot_left + plot_right) / 2.0,
            h - 5.0,
            xml_escape(&gplot.xlabel)
        ));
        svg.push('\n');
    }

    // Y label (rotated)
    if !gplot.ylabel.is_empty() {
        let ymid = (plot_top + plot_bottom) / 2.0;
        svg.push_str(&format!(
            r#"<text x="15" y="{}" text-anchor="middle" font-size="12" font-family="sans-serif" transform="rotate(-90,15,{})">{}</text>"#,
            ymid,
            ymid,
            xml_escape(&gplot.ylabel)
        ));
        svg.push('\n');
    }

    // Plot area border
    svg.push_str(&format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#ccc\" stroke-width=\"1\"/>",
        plot_left,
        plot_top,
        plot_right - plot_left,
        plot_bottom - plot_top
    ));
    svg.push('\n');

    // X axis ticks & labels
    for i in 0..xtick_count {
        let val = xtick_start + i as f32 * xtick_step;
        let px = map_x(val, xdata_min, xdata_max, plot_left, plot_right);
        // Grid line
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eee\" stroke-width=\"0.5\"/>",
            px, plot_top, px, plot_bottom
        ));
        // Tick
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#333\" stroke-width=\"1\"/>",
            px,
            plot_bottom,
            px,
            plot_bottom + 5.0
        ));
        // Label
        svg.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-size="10" font-family="sans-serif">{}</text>"#,
            px,
            plot_bottom + 18.0,
            fmt_num(val)
        ));
        svg.push('\n');
    }

    // Y axis ticks & labels
    for i in 0..ytick_count {
        let val = ytick_start + i as f32 * ytick_step;
        let py = map_y(val, ydata_min, ydata_max, plot_top, plot_bottom);
        // Grid line
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#eee\" stroke-width=\"0.5\"/>",
            plot_left, py, plot_right, py
        ));
        // Tick
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#333\" stroke-width=\"1\"/>",
            plot_left - 5.0,
            py,
            plot_left,
            py
        ));
        // Label
        svg.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" text-anchor="end" font-size="10" font-family="sans-serif" dominant-baseline="middle">{}</text>"#,
            plot_left - 8.0,
            py,
            fmt_num(val)
        ));
        svg.push('\n');
    }

    // Clip path for plot area
    svg.push_str(&format!(
        r#"<defs><clipPath id="plotarea"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath></defs>"#,
        plot_left,
        plot_top,
        plot_right - plot_left,
        plot_bottom - plot_top
    ));
    svg.push('\n');

    // Data series
    for (si, p) in gplot.plots.iter().enumerate() {
        let (cr, cg, cb) = series_color(si);
        let color_str = format!("rgb({},{},{})", cr, cg, cb);
        let n = p.x_data.len().min(p.y_data.len());
        if n == 0 {
            continue;
        }

        match p.style {
            PlotStyle::Lines | PlotStyle::LinesPoints => {
                let mut path = String::with_capacity(n * 20);
                for i in 0..n {
                    let px = map_x(p.x_data[i], xdata_min, xdata_max, plot_left, plot_right);
                    let py = map_y(p.y_data[i], ydata_min, ydata_max, plot_top, plot_bottom);
                    if i == 0 {
                        path.push_str(&format!("M{:.2},{:.2}", px, py));
                    } else {
                        path.push_str(&format!(" L{:.2},{:.2}", px, py));
                    }
                }
                svg.push_str(&format!(
                    r#"<path d="{}" fill="none" stroke="{}" stroke-width="1.5" clip-path="url(#plotarea)"/>"#,
                    path, color_str
                ));
                svg.push('\n');

                if p.style == PlotStyle::LinesPoints {
                    for i in 0..n {
                        let px = map_x(p.x_data[i], xdata_min, xdata_max, plot_left, plot_right);
                        let py = map_y(p.y_data[i], ydata_min, ydata_max, plot_top, plot_bottom);
                        svg.push_str(&format!(
                            r#"<circle cx="{:.2}" cy="{:.2}" r="2.5" fill="{}" clip-path="url(#plotarea)"/>"#,
                            px, py, color_str
                        ));
                    }
                    svg.push('\n');
                }
            }
            PlotStyle::Points => {
                for i in 0..n {
                    let px = map_x(p.x_data[i], xdata_min, xdata_max, plot_left, plot_right);
                    let py = map_y(p.y_data[i], ydata_min, ydata_max, plot_top, plot_bottom);
                    svg.push_str(&format!(
                        r#"<circle cx="{:.2}" cy="{:.2}" r="2.5" fill="{}" clip-path="url(#plotarea)"/>"#,
                        px, py, color_str
                    ));
                }
                svg.push('\n');
            }
            PlotStyle::Dots => {
                for i in 0..n {
                    let px = map_x(p.x_data[i], xdata_min, xdata_max, plot_left, plot_right);
                    let py = map_y(p.y_data[i], ydata_min, ydata_max, plot_top, plot_bottom);
                    svg.push_str(&format!(
                        r#"<circle cx="{:.2}" cy="{:.2}" r="1" fill="{}" clip-path="url(#plotarea)"/>"#,
                        px, py, color_str
                    ));
                }
                svg.push('\n');
            }
            PlotStyle::Impulses => {
                for i in 0..n {
                    let px = map_x(p.x_data[i], xdata_min, xdata_max, plot_left, plot_right);
                    let py = map_y(p.y_data[i], ydata_min, ydata_max, plot_top, plot_bottom);
                    let baseline = map_y(
                        0f32.clamp(ydata_min, ydata_max),
                        ydata_min,
                        ydata_max,
                        plot_top,
                        plot_bottom,
                    );
                    svg.push_str(&format!(
                        r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{}" stroke-width="1" clip-path="url(#plotarea)"/>"#,
                        px, baseline, px, py, color_str
                    ));
                }
                svg.push('\n');
            }
        }
    }

    // Legend
    let labeled: Vec<(usize, &str)> = gplot
        .plots
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.label.as_deref().map(|l| (i, l)))
        .collect();
    if !labeled.is_empty() {
        let lx = plot_right - 130.0;
        let mut ly = plot_top + 15.0;
        for &(si, label) in &labeled {
            let (cr, cg, cb) = series_color(si);
            svg.push_str(&format!(
                r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="rgb({},{},{})" stroke-width="2"/>"#,
                lx,
                ly,
                lx + 20.0,
                ly,
                cr,
                cg,
                cb
            ));
            svg.push_str(&format!(
                r#"<text x="{:.0}" y="{:.0}" font-size="10" font-family="sans-serif" dominant-baseline="middle">{}</text>"#,
                lx + 25.0,
                ly,
                xml_escape(label)
            ));
            svg.push('\n');
            ly += 16.0;
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

// ===========================================================================
// Pix Renderer
// ===========================================================================

fn render_pix(gplot: &GPlot) -> Result<Pix> {
    let cw = CANVAS_W;
    let ch = CANVAS_H;

    let mut pm = PixMut::new(cw, ch, PixelDepth::Bit32)?;

    // Fill white background
    let white_pixel = crate::core::pixel::compose_rgb(255, 255, 255);
    for row in 0..ch {
        let row_data = pm.row_data_mut(row);
        for px in row_data.iter_mut() {
            *px = white_pixel;
        }
    }

    let range = compute_range(gplot).unwrap_or(DataRange {
        xmin: 0.0,
        xmax: 1.0,
        ymin: 0.0,
        ymax: 1.0,
    });

    let plot_left = PIX_MARGIN_LEFT;
    let plot_right = cw as i32 - PIX_MARGIN_RIGHT;
    let plot_top = PIX_MARGIN_TOP;
    let plot_bottom = ch as i32 - PIX_MARGIN_BOTTOM;

    let (xtick_start, xtick_step, xtick_count) = nice_ticks(range.xmin, range.xmax, 6);
    let (ytick_start, ytick_step, ytick_count) = nice_ticks(range.ymin, range.ymax, 5);
    let xdata_min = xtick_start;
    let xdata_max = xtick_start + xtick_step * (xtick_count - 1) as f32;
    let ydata_min = ytick_start;
    let ydata_max = ytick_start + ytick_step * (ytick_count - 1) as f32;

    let gray = Color::new(200, 200, 200);

    // Plot area border
    pm.render_line_color(plot_left, plot_top, plot_right, plot_top, 1, gray)?;
    pm.render_line_color(plot_left, plot_bottom, plot_right, plot_bottom, 1, gray)?;
    pm.render_line_color(plot_left, plot_top, plot_left, plot_bottom, 1, gray)?;
    pm.render_line_color(plot_right, plot_top, plot_right, plot_bottom, 1, gray)?;

    // Grid lines
    let grid_color = Color::new(230, 230, 230);
    for i in 0..xtick_count {
        let val = xtick_start + i as f32 * xtick_step;
        let px = map_x(
            val,
            xdata_min,
            xdata_max,
            plot_left as f32,
            plot_right as f32,
        ) as i32;
        pm.render_line_color(px, plot_top, px, plot_bottom, 1, grid_color)?;
    }
    for i in 0..ytick_count {
        let val = ytick_start + i as f32 * ytick_step;
        let py = map_y(
            val,
            ydata_min,
            ydata_max,
            plot_top as f32,
            plot_bottom as f32,
        ) as i32;
        pm.render_line_color(plot_left, py, plot_right, py, 1, grid_color)?;
    }

    // Data series
    for (si, p) in gplot.plots.iter().enumerate() {
        let color = series_color_obj(si);
        let n = p.x_data.len().min(p.y_data.len());
        if n == 0 {
            continue;
        }

        match p.style {
            PlotStyle::Lines | PlotStyle::LinesPoints => {
                // Draw connected line segments
                for i in 1..n {
                    let x1 = map_x(
                        p.x_data[i - 1],
                        xdata_min,
                        xdata_max,
                        plot_left as f32,
                        plot_right as f32,
                    ) as i32;
                    let y1 = map_y(
                        p.y_data[i - 1],
                        ydata_min,
                        ydata_max,
                        plot_top as f32,
                        plot_bottom as f32,
                    ) as i32;
                    let x2 = map_x(
                        p.x_data[i],
                        xdata_min,
                        xdata_max,
                        plot_left as f32,
                        plot_right as f32,
                    ) as i32;
                    let y2 = map_y(
                        p.y_data[i],
                        ydata_min,
                        ydata_max,
                        plot_top as f32,
                        plot_bottom as f32,
                    ) as i32;
                    pm.render_line_color(x1, y1, x2, y2, 2, color)?;
                }
                if p.style == PlotStyle::LinesPoints {
                    for i in 0..n {
                        let px = map_x(
                            p.x_data[i],
                            xdata_min,
                            xdata_max,
                            plot_left as f32,
                            plot_right as f32,
                        ) as i32;
                        let py = map_y(
                            p.y_data[i],
                            ydata_min,
                            ydata_max,
                            plot_top as f32,
                            plot_bottom as f32,
                        ) as i32;
                        pm.render_filled_circle_color(px, py, 3, color)?;
                    }
                }
            }
            PlotStyle::Points => {
                for i in 0..n {
                    let px = map_x(
                        p.x_data[i],
                        xdata_min,
                        xdata_max,
                        plot_left as f32,
                        plot_right as f32,
                    ) as i32;
                    let py = map_y(
                        p.y_data[i],
                        ydata_min,
                        ydata_max,
                        plot_top as f32,
                        plot_bottom as f32,
                    ) as i32;
                    pm.render_filled_circle_color(px, py, 3, color)?;
                }
            }
            PlotStyle::Dots => {
                for i in 0..n {
                    let px = map_x(
                        p.x_data[i],
                        xdata_min,
                        xdata_max,
                        plot_left as f32,
                        plot_right as f32,
                    ) as i32;
                    let py = map_y(
                        p.y_data[i],
                        ydata_min,
                        ydata_max,
                        plot_top as f32,
                        plot_bottom as f32,
                    ) as i32;
                    pm.render_filled_circle_color(px, py, 1, color)?;
                }
            }
            PlotStyle::Impulses => {
                let zero_y = map_y(
                    0f32.clamp(ydata_min, ydata_max),
                    ydata_min,
                    ydata_max,
                    plot_top as f32,
                    plot_bottom as f32,
                ) as i32;
                for i in 0..n {
                    let px = map_x(
                        p.x_data[i],
                        xdata_min,
                        xdata_max,
                        plot_left as f32,
                        plot_right as f32,
                    ) as i32;
                    let py = map_y(
                        p.y_data[i],
                        ydata_min,
                        ydata_max,
                        plot_top as f32,
                        plot_bottom as f32,
                    ) as i32;
                    pm.render_line_color(px, zero_y, px, py, 1, color)?;
                }
            }
        }
    }

    // Legend: small colored lines with text rendered via Bmf
    let labeled: Vec<(usize, &str)> = gplot
        .plots
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.label.as_deref().map(|l| (i, l)))
        .collect();

    // Convert to Pix first, then add text if needed
    let mut pix: Pix = pm.into();

    if !labeled.is_empty()
        && let Ok(font) = super::bmf::Bmf::new(10)
    {
        let lx = plot_right - 120;
        let mut ly = plot_top + 8;
        for &(si, label) in &labeled {
            let color = series_color_obj(si);
            let mut pm2 = pix.to_mut();
            pm2.render_line_color(lx, ly + 5, lx + 18, ly + 5, 2, color)?;
            pix = pm2.into();
            let (new_pix, _) = font.set_textline(&pix, label, lx + 22, ly, 0x00000000)?;
            pix = new_pix;
            ly += 14;
        }
    }

    // Render title via Bmf
    if !gplot.title.is_empty()
        && let Ok(font) = super::bmf::Bmf::new(14)
    {
        let tw = font.get_string_width(&gplot.title) as i32;
        let tx = ((cw as i32 - tw) / 2).max(5);
        let (new_pix, _) = font.set_textline(&pix, &gplot.title, tx, 10, 0x00000000)?;
        pix = new_pix;
    }

    // Render axis labels
    if !gplot.xlabel.is_empty()
        && let Ok(font) = super::bmf::Bmf::new(10)
    {
        let tw = font.get_string_width(&gplot.xlabel) as i32;
        let tx = (((plot_left + plot_right) - tw) / 2).max(5);
        let (new_pix, _) =
            font.set_textline(&pix, &gplot.xlabel, tx, ch as i32 - 20, 0x00000000)?;
        pix = new_pix;
    }

    Ok(pix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_ticks_basic() {
        let (start, step, count) = nice_ticks(0.0, 100.0, 5);
        assert!(start <= 0.0);
        assert!(start + step * (count - 1) as f32 >= 100.0);
        assert!(step > 0.0);
    }

    #[test]
    fn test_fmt_num() {
        assert_eq!(fmt_num(0.0), "0");
        assert_eq!(fmt_num(100.0), "100");
        assert_eq!(fmt_num(1.5), "1.5");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("<a>&b"), "&lt;a&gt;&amp;b");
    }

    #[test]
    fn test_gplot_new() {
        let gp = GPlot::new(None, GPlotOutput::Svg, "Title", "X", "Y");
        assert_eq!(gp.title, "Title");
        assert_eq!(gp.plots.len(), 0);
    }

    #[test]
    fn test_gplot_add_plot() {
        let mut gp = GPlot::new(None, GPlotOutput::Svg, "T", "X", "Y");
        let mut na = Numa::new();
        for i in 0..10 {
            na.push(i as f32 * 2.0);
        }
        gp.add_plot(None, &na, PlotStyle::Lines, Some("series1"));
        assert_eq!(gp.plots.len(), 1);
        assert_eq!(gp.plots[0].y_data.len(), 10);
    }

    #[test]
    fn test_svg_output() {
        let mut gp = GPlot::new(None, GPlotOutput::Svg, "Test", "x", "y");
        let mut na = Numa::new();
        for i in 0..20 {
            na.push((i as f32 * 0.3).sin());
        }
        gp.add_plot(None, &na, PlotStyle::Lines, Some("sin"));
        let svg = gp.make_output_svg().unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Test"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_pix_output() {
        let mut gp = GPlot::new(None, GPlotOutput::Pix, "Test", "x", "y");
        let mut na = Numa::new();
        for i in 0..20 {
            na.push(i as f32);
        }
        gp.add_plot(None, &na, PlotStyle::Lines, None);
        let pix = gp.make_output_pix().unwrap();
        assert_eq!(pix.width(), CANVAS_W);
        assert_eq!(pix.height(), CANVAS_H);
    }
}
