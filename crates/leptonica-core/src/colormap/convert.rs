//! PixColormap conversion and effects functions
//!
//! Functions for creating derived colormaps (false color, monochrome tint,
//! grayscale conversion, depth conversion) and applying in-place color
//! transforms (gamma, contrast, intensity shift, component shift).
//!
//! # See also
//!
//! C Leptonica: `colormap.c` (pixcmapGrayToFalseColor, pixcmapGrayToColor,
//! pixcmapColorToGray, pixcmapConvertTo4, pixcmapConvertTo8,
//! pixcmapToArrays, pixcmapToRGBTable, pixcmapSerializeToMemory,
//! pixcmapDeserializeFromMemory, pixcmapConvertToHex,
//! pixcmapGammaTRC, pixcmapContrastTRC,
//! pixcmapShiftIntensity, pixcmapShiftByComponent)

use super::{PixColormap, RgbaQuad};
use crate::color;
use crate::error::{Error, Result};

/// Components per color for binary serialization.
///
/// # See also
///
/// C Leptonica: `pixcmapSerializeToMemory()` cpc parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentsPerColor {
    /// 3 bytes per color (RGB only, for PDF)
    Rgb,
    /// 4 bytes per color (RGBA)
    Rgba,
}

impl ComponentsPerColor {
    fn bytes_per_color(self) -> usize {
        match self {
            ComponentsPerColor::Rgb => 3,
            ComponentsPerColor::Rgba => 4,
        }
    }
}

/// Extracted color channel arrays from a colormap.
///
/// # See also
///
/// C Leptonica: `pixcmapToArrays()`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColormapArrays {
    /// Red channel values
    pub red: Vec<u8>,
    /// Green channel values
    pub green: Vec<u8>,
    /// Blue channel values
    pub blue: Vec<u8>,
    /// Alpha channel values (if requested)
    pub alpha: Option<Vec<u8>>,
}

impl PixColormap {
    // ---------------------------------------------------------------
    //  Creation / conversion to new colormaps
    // ---------------------------------------------------------------

    /// Create a "jet" false-color colormap from grayscale (8 bpp, 256 entries).
    ///
    /// The colormap maps gray values 0..255 through a blue→cyan→green→yellow→red
    /// spectrum modeled on the MATLAB "jet" palette.
    ///
    /// `gamma` controls brightness: 0.0 or 1.0 for default, >1.0 for brighter
    /// (2.0 is recommended).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGrayToFalseColor()` in `colormap.c`
    pub fn gray_to_false_color(gamma: f32) -> Self {
        let gamma = if gamma <= 0.0 { 1.0 } else { gamma };
        let inv_gamma = 1.0 / gamma;

        // Build a 64-entry transition curve
        let mut curve = [0i32; 64];
        for (i, entry) in curve.iter_mut().enumerate() {
            let x = i as f32 / 64.0;
            *entry = (255.0 * x.powf(inv_gamma) + 0.5) as i32;
        }

        let mut cmap = Self::new(8).unwrap();
        for i in 0..256 {
            let (rval, gval, bval) = if i < 32 {
                (0, 0, curve[i + 32])
            } else if i < 96 {
                (0, curve[i - 32], 255)
            } else if i < 160 {
                (curve[i - 96], 255, curve[159 - i])
            } else if i < 224 {
                (255, curve[223 - i], 0)
            } else {
                (curve[287 - i], 0, 0)
            };
            cmap.add_color(RgbaQuad::rgb(rval as u8, gval as u8, bval as u8))
                .unwrap();
        }
        cmap
    }

    /// Create a monochrome-tinted colormap (8 bpp, 256 entries).
    ///
    /// Maps gray 0 to the specified color and gray 255 to white,
    /// with intermediate grays linearly interpolated.
    ///
    /// `color_pixel` is a packed 0xRRGGBB00 value.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGrayToColor()` in `colormap.c`
    pub fn gray_to_color(color_pixel: u32) -> Self {
        let (rval, gval, bval) = color::extract_rgb(color_pixel);
        let mut cmap = Self::new(8).unwrap();
        for i in 0..256 {
            let r = rval as i32 + (i * (255 - rval as i32)) / 255;
            let g = gval as i32 + (i * (255 - gval as i32)) / 255;
            let b = bval as i32 + (i * (255 - bval as i32)) / 255;
            cmap.add_color(RgbaQuad::rgb(r as u8, g as u8, b as u8))
                .unwrap();
        }
        cmap
    }

    /// Create a grayscale colormap from an existing colormap using weighted
    /// channel mixing.
    ///
    /// The weights `rwt`, `gwt`, `bwt` should be non-negative and ideally sum
    /// to 1.0. If they don't, they are normalized to sum to 1.0.
    ///
    /// # Errors
    ///
    /// Returns an error if any weight is negative.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapColorToGray()` in `colormap.c`
    pub fn color_to_gray(&self, mut rwt: f32, mut gwt: f32, mut bwt: f32) -> Result<Self> {
        if rwt < 0.0 || gwt < 0.0 || bwt < 0.0 {
            return Err(Error::InvalidParameter(
                "weights must be non-negative".into(),
            ));
        }

        let sum = rwt + gwt + bwt;
        if sum == 0.0 {
            rwt = 1.0 / 3.0;
            gwt = 1.0 / 3.0;
            bwt = 1.0 / 3.0;
        } else if (sum - 1.0).abs() > 0.0001 {
            rwt /= sum;
            gwt /= sum;
            bwt /= sum;
        }

        let mut cmapd = self.clone();
        for i in 0..cmapd.len() {
            let (r, g, b) = cmapd.get_rgb(i).unwrap();
            let val = (rwt * r as f32 + gwt * g as f32 + bwt * b as f32 + 0.5) as u8;
            cmapd
                .set_color(i, RgbaQuad::new(val, val, val, cmapd.get(i).unwrap().alpha))
                .unwrap();
        }
        Ok(cmapd)
    }

    /// Convert a 2 bpp colormap to 4 bpp.
    ///
    /// Colors are preserved; only the depth changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the source depth is not 2.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapConvertTo4()` in `colormap.c`
    pub fn convert_to_4(&self) -> Result<Self> {
        if self.depth() != 2 {
            return Err(Error::InvalidParameter(format!(
                "source depth must be 2, got {}",
                self.depth()
            )));
        }
        let mut cmapd = Self::new(4)?;
        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            cmapd.add_rgb(r, g, b)?;
        }
        Ok(cmapd)
    }

    /// Convert a 2 bpp or 4 bpp colormap to 8 bpp.
    ///
    /// Colors are preserved; only the depth changes.
    /// If already 8 bpp, returns a clone.
    ///
    /// # Errors
    ///
    /// Returns an error if the source depth is not 2, 4, or 8.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapConvertTo8()` in `colormap.c`
    pub fn convert_to_8(&self) -> Result<Self> {
        if self.depth() == 8 {
            return Ok(self.clone());
        }
        if self.depth() != 2 && self.depth() != 4 {
            return Err(Error::InvalidParameter(format!(
                "source depth must be 2, 4, or 8, got {}",
                self.depth()
            )));
        }
        let mut cmapd = Self::new(8)?;
        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            cmapd.add_rgb(r, g, b)?;
        }
        Ok(cmapd)
    }

    // ---------------------------------------------------------------
    //  Array extraction and serialization
    // ---------------------------------------------------------------

    /// Extract color channel arrays from the colormap.
    ///
    /// If `include_alpha` is true, the alpha array is included.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapToArrays()` in `colormap.c`
    pub fn to_arrays(&self, include_alpha: bool) -> ColormapArrays {
        let n = self.len();
        let mut red = Vec::with_capacity(n);
        let mut green = Vec::with_capacity(n);
        let mut blue = Vec::with_capacity(n);
        let mut alpha = if include_alpha {
            Some(Vec::with_capacity(n))
        } else {
            None
        };

        for c in self.colors() {
            red.push(c.red);
            green.push(c.green);
            blue.push(c.blue);
            if let Some(ref mut a) = alpha {
                a.push(c.alpha);
            }
        }

        ColormapArrays {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Extract an RGBA packed table (Vec of 0xRRGGBBAA values).
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapToRGBTable()` in `colormap.c`
    pub fn to_rgb_table(&self) -> Vec<u32> {
        self.colors()
            .iter()
            .map(|c| color::compose_rgba(c.red, c.green, c.blue, c.alpha))
            .collect()
    }

    /// Serialize colormap to a compact binary format.
    ///
    /// Each color is stored as 3 bytes (RGB) or 4 bytes (RGBA) depending
    /// on `cpc`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapSerializeToMemory()` in `colormap.c`
    pub fn serialize_to_memory(&self, cpc: ComponentsPerColor) -> Vec<u8> {
        let bpc = cpc.bytes_per_color();
        let mut data = Vec::with_capacity(self.len() * bpc);
        for c in self.colors() {
            data.push(c.red);
            data.push(c.green);
            data.push(c.blue);
            if cpc == ComponentsPerColor::Rgba {
                data.push(c.alpha);
            }
        }
        data
    }

    /// Deserialize a colormap from a compact binary format.
    ///
    /// `data` contains `ncolors * cpc` bytes where cpc is 3 (RGB) or 4 (RGBA).
    ///
    /// # Errors
    ///
    /// Returns an error if the data length doesn't match, or ncolors > 256.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapDeserializeFromMemory()` in `colormap.c`
    pub fn deserialize_from_memory(
        data: &[u8],
        cpc: ComponentsPerColor,
        ncolors: usize,
    ) -> Result<Self> {
        if ncolors == 0 {
            return Err(Error::InvalidParameter("ncolors must be > 0".into()));
        }
        if ncolors > 256 {
            return Err(Error::InvalidParameter(format!(
                "ncolors {} exceeds 256",
                ncolors
            )));
        }
        let bpc = cpc.bytes_per_color();
        if data.len() < ncolors * bpc {
            return Err(Error::InvalidParameter(format!(
                "data too short: expected {} bytes, got {}",
                ncolors * bpc,
                data.len()
            )));
        }

        let depth = if ncolors > 16 {
            8
        } else if ncolors > 4 {
            4
        } else if ncolors > 2 {
            2
        } else {
            1
        };

        let mut cmap = Self::new(depth)?;
        for i in 0..ncolors {
            let r = data[bpc * i];
            let g = data[bpc * i + 1];
            let b = data[bpc * i + 2];
            let a = if cpc == ComponentsPerColor::Rgba {
                data[bpc * i + 3]
            } else {
                255
            };
            cmap.add_color(RgbaQuad::new(r, g, b, a))?;
        }
        Ok(cmap)
    }

    /// Convert serialized RGB data to a hex string for PDF embedding.
    ///
    /// Output format: `< r0g0b0 r1g1b1 ... rngnbn >`
    ///
    /// `data` should be the RGB-only serialization (3 bytes per color).
    ///
    /// # Errors
    ///
    /// Returns an error if the data length is not `3 * ncolors`.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapConvertToHex()` in `colormap.c`
    pub fn convert_to_hex(data: &[u8], ncolors: usize) -> Result<String> {
        if data.len() != 3 * ncolors {
            return Err(Error::InvalidParameter(format!(
                "data length {} != 3 * {}",
                data.len(),
                ncolors
            )));
        }

        let mut hex = String::with_capacity(2 + (7 * ncolors) + 1);
        hex.push_str("< ");
        for i in 0..ncolors {
            if i > 0 {
                hex.push(' ');
            }
            hex.push_str(&format!(
                "{:02x}{:02x}{:02x}",
                data[3 * i],
                data[3 * i + 1],
                data[3 * i + 2]
            ));
        }
        hex.push_str(" >");
        Ok(hex)
    }

    // ---------------------------------------------------------------
    //  In-place color transforms
    // ---------------------------------------------------------------

    /// Apply gamma TRC (tone reproduction curve) to all colors in-place.
    ///
    /// Each RGB component is independently mapped through a gamma curve.
    /// `gamma` must be > 0.0. Values in `[minval, maxval]` are mapped to
    /// `[0, 255]`.
    ///
    /// # Errors
    ///
    /// Returns an error if gamma <= 0 or minval >= maxval.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapGammaTRC()` in `colormap.c`
    pub fn gamma_trc(&mut self, gamma: f32, minval: i32, maxval: i32) -> Result<()> {
        if gamma <= 0.0 {
            return Err(Error::InvalidParameter("gamma must be > 0.0".into()));
        }
        if minval >= maxval {
            return Err(Error::InvalidParameter("minval must be < maxval".into()));
        }
        if gamma == 1.0 && minval == 0 && maxval == 255 {
            return Ok(()); // no-op
        }

        let lut = build_gamma_trc(gamma, minval, maxval);
        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            let alpha = self.get(i).unwrap().alpha;
            self.set_color(
                i,
                RgbaQuad::new(lut[r as usize], lut[g as usize], lut[b as usize], alpha),
            )?;
        }
        Ok(())
    }

    /// Apply contrast enhancement TRC to all colors in-place.
    ///
    /// `factor` >= 0.0 (0.0 = no change, higher = more contrast).
    /// Uses an arctan-based S-curve centered at 127.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapContrastTRC()` in `colormap.c`
    pub fn contrast_trc(&mut self, factor: f32) {
        let factor = if factor < 0.0 { 0.0 } else { factor };
        let lut = build_contrast_trc(factor);
        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            let alpha = self.get(i).unwrap().alpha;
            self.set_color(
                i,
                RgbaQuad::new(lut[r as usize], lut[g as usize], lut[b as usize], alpha),
            )
            .unwrap();
        }
    }

    /// Shift the intensity of all colors in-place.
    ///
    /// `fraction` in [-1.0, 1.0]:
    /// - Negative: move toward black (darken)
    /// - Positive: move toward white (fade)
    ///
    /// # Errors
    ///
    /// Returns an error if fraction is outside [-1.0, 1.0].
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapShiftIntensity()` in `colormap.c`
    pub fn shift_intensity(&mut self, fraction: f32) -> Result<()> {
        if !(-1.0..=1.0).contains(&fraction) {
            return Err(Error::InvalidParameter(
                "fraction must be in [-1.0, 1.0]".into(),
            ));
        }

        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            let alpha = self.get(i).unwrap().alpha;
            let (nr, ng, nb) = if fraction < 0.0 {
                (
                    ((1.0 + fraction) * r as f32) as u8,
                    ((1.0 + fraction) * g as f32) as u8,
                    ((1.0 + fraction) * b as f32) as u8,
                )
            } else {
                (
                    (r as i32 + (fraction * (255 - r as i32) as f32) as i32) as u8,
                    (g as i32 + (fraction * (255 - g as i32) as f32) as i32) as u8,
                    (b as i32 + (fraction * (255 - b as i32) as f32) as i32) as u8,
                )
            };
            self.set_color(i, RgbaQuad::new(nr, ng, nb, alpha))?;
        }
        Ok(())
    }

    /// Shift each color component independently toward a target.
    ///
    /// Maps the colormap so that `src_pixel` (0xRRGGBB00) would map to
    /// `dst_pixel` (0xRRGGBB00). Other colors shift proportionally per
    /// component.
    ///
    /// # See also
    ///
    /// C Leptonica: `pixcmapShiftByComponent()` in `colormap.c`
    pub fn shift_by_component(&mut self, src_pixel: u32, dst_pixel: u32) {
        for i in 0..self.len() {
            let (r, g, b) = self.get_rgb(i).unwrap();
            let alpha = self.get(i).unwrap().alpha;
            let (nr, ng, nb) = pixel_shift_by_component(r, g, b, src_pixel, dst_pixel);
            self.set_color(i, RgbaQuad::new(nr, ng, nb, alpha)).unwrap();
        }
    }
}

/// Shift a single pixel by component mapping.
///
/// Each component is shifted independently:
/// - If dst component < src component: scale down proportionally
/// - If dst component > src component: push toward 255 proportionally
/// - If equal: no change
///
/// # See also
///
/// C Leptonica: `pixelShiftByComponent()` in `coloring.c`
fn pixel_shift_by_component(r: u8, g: u8, b: u8, src_pixel: u32, dst_pixel: u32) -> (u8, u8, u8) {
    let (rs, gs, bs) = color::extract_rgb(src_pixel);
    let (rd, gd, bd) = color::extract_rgb(dst_pixel);

    let shift_component = |val: u8, src: u8, dst: u8| -> u8 {
        if dst == src {
            val
        } else if dst < src {
            ((val as i32 * dst as i32) / src as i32) as u8
        } else {
            (255 - (255 - dst as i32) * (255 - val as i32) / (255 - src as i32)) as u8
        }
    };

    (
        shift_component(r, rs, rd),
        shift_component(g, gs, gd),
        shift_component(b, bs, bd),
    )
}

/// Build a 256-entry gamma TRC lookup table.
///
/// Equivalent to C `numaGammaTRC()` in `enhance.c`.
fn build_gamma_trc(gamma: f32, minval: i32, maxval: i32) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let inv_gamma = 1.0 / gamma;

    for i in 0..256i32 {
        let val = if i < minval {
            0
        } else if i > maxval {
            255
        } else {
            let x = (i - minval) as f32 / (maxval - minval) as f32;
            let v = (255.0 * x.powf(inv_gamma) + 0.5) as i32;
            v.clamp(0, 255)
        };
        lut[i as usize] = val as u8;
    }
    lut
}

/// Build a 256-entry contrast TRC lookup table.
///
/// Equivalent to C `numaContrastTRC()` in `enhance.c`.
fn build_contrast_trc(factor: f32) -> [u8; 256] {
    let mut lut = [0u8; 256];

    if factor == 0.0 {
        // Linear map (identity)
        for (i, entry) in lut.iter_mut().enumerate() {
            *entry = i as u8;
        }
        return lut;
    }

    const ENHANCE_SCALE_FACTOR: f64 = 5.0;
    let factor = factor as f64;
    let scale = ENHANCE_SCALE_FACTOR;
    let ymax = (1.0 * factor * scale).atan();
    let ymin = (-127.0 * factor * scale / 128.0).atan();
    let dely = ymax - ymin;

    for (i, entry) in lut.iter_mut().enumerate() {
        let x = i as f64;
        let val =
            ((255.0 / dely) * (-ymin + (factor * scale * (x - 127.0) / 128.0).atan()) + 0.5) as i32;
        *entry = val.clamp(0, 255) as u8;
    }
    lut
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;

    // ---------------------------------------------------------------
    //  gray_to_false_color
    // ---------------------------------------------------------------

    #[test]
    fn test_gray_to_false_color_default_gamma() {
        let cmap = PixColormap::gray_to_false_color(1.0);
        assert_eq!(cmap.depth(), 8);
        assert_eq!(cmap.len(), 256);
        // Index 0 should be dark blue-ish
        let (r, g, b) = cmap.get_rgb(0).unwrap();
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert!(b > 0);
        // Index 255 should be dark red-ish
        let (r, g, b) = cmap.get_rgb(255).unwrap();
        assert!(r > 0);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_gray_to_false_color_bright_gamma() {
        let cmap = PixColormap::gray_to_false_color(2.0);
        assert_eq!(cmap.len(), 256);
        // Middle region (around 128) should be green-ish
        let (r, g, b) = cmap.get_rgb(128).unwrap();
        assert_eq!(g, 255);
    }

    #[test]
    fn test_gray_to_false_color_zero_gamma_uses_default() {
        let cmap = PixColormap::gray_to_false_color(0.0);
        assert_eq!(cmap.len(), 256);
    }

    // ---------------------------------------------------------------
    //  gray_to_color
    // ---------------------------------------------------------------

    #[test]
    fn test_gray_to_color_red_tint() {
        let cmap = PixColormap::gray_to_color(color::compose_rgb(255, 0, 0));
        assert_eq!(cmap.depth(), 8);
        assert_eq!(cmap.len(), 256);
        // Index 0 → the specified color (red)
        assert_eq!(cmap.get_rgb(0), Some((255, 0, 0)));
        // Index 255 → white
        assert_eq!(cmap.get_rgb(255), Some((255, 255, 255)));
    }

    #[test]
    fn test_gray_to_color_blue_tint_midpoint() {
        let cmap = PixColormap::gray_to_color(color::compose_rgb(0, 0, 200));
        // Midpoint (index 128) should be interpolated
        let (r, g, b) = cmap.get_rgb(128).unwrap();
        // r = 0 + (128 * 255) / 255 = 128
        assert!((r as i32 - 128).abs() <= 1);
        // g = 0 + (128 * 255) / 255 = 128
        assert!((g as i32 - 128).abs() <= 1);
        // b = 200 + (128 * 55) / 255 ≈ 228
        assert!((b as i32 - 228).abs() <= 1);
    }

    // ---------------------------------------------------------------
    //  color_to_gray
    // ---------------------------------------------------------------

    #[test]
    fn test_color_to_gray_equal_weights() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(90, 120, 150).unwrap();
        cmap.add_rgb(255, 0, 0).unwrap();
        let gray = cmap.color_to_gray(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0).unwrap();
        assert!(gray.is_grayscale());
        let (r, _, _) = gray.get_rgb(0).unwrap();
        assert!((r as i32 - 120).abs() <= 1); // (90+120+150)/3 = 120
    }

    #[test]
    fn test_color_to_gray_standard_weights() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(255, 128, 64).unwrap();
        let gray = cmap.color_to_gray(0.3, 0.6, 0.1).unwrap();
        let (r, g, b) = gray.get_rgb(0).unwrap();
        assert_eq!(r, g);
        assert_eq!(g, b);
        // 0.3*255 + 0.6*128 + 0.1*64 = 76.5 + 76.8 + 6.4 = 159.7 → 160
        assert!((r as i32 - 160).abs() <= 1);
    }

    #[test]
    fn test_color_to_gray_normalizes_weights() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 100, 100).unwrap();
        // Weights sum to 2.0, should be normalized
        let gray = cmap.color_to_gray(0.6, 0.8, 0.6).unwrap();
        let (r, _, _) = gray.get_rgb(0).unwrap();
        assert_eq!(r, 100); // uniform color → any weighting gives same gray
    }

    #[test]
    fn test_color_to_gray_negative_weight_error() {
        let cmap = PixColormap::new(8).unwrap();
        assert!(cmap.color_to_gray(-0.1, 0.6, 0.5).is_err());
    }

    // ---------------------------------------------------------------
    //  convert_to_4 / convert_to_8
    // ---------------------------------------------------------------

    #[test]
    fn test_convert_to_4() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();
        cmap.add_rgb(40, 50, 60).unwrap();
        let cmap4 = cmap.convert_to_4().unwrap();
        assert_eq!(cmap4.depth(), 4);
        assert_eq!(cmap4.len(), 2);
        assert_eq!(cmap4.get_rgb(0), Some((10, 20, 30)));
        assert_eq!(cmap4.get_rgb(1), Some((40, 50, 60)));
    }

    #[test]
    fn test_convert_to_4_wrong_depth() {
        let cmap = PixColormap::new(8).unwrap();
        assert!(cmap.convert_to_4().is_err());
    }

    #[test]
    fn test_convert_to_8_from_2() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(100, 200, 50).unwrap();
        let cmap8 = cmap.convert_to_8().unwrap();
        assert_eq!(cmap8.depth(), 8);
        assert_eq!(cmap8.len(), 1);
        assert_eq!(cmap8.get_rgb(0), Some((100, 200, 50)));
    }

    #[test]
    fn test_convert_to_8_from_4() {
        let mut cmap = PixColormap::new(4).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();
        let cmap8 = cmap.convert_to_8().unwrap();
        assert_eq!(cmap8.depth(), 8);
        assert_eq!(cmap8.get_rgb(0), Some((10, 20, 30)));
    }

    #[test]
    fn test_convert_to_8_already_8() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();
        let cmap8 = cmap.convert_to_8().unwrap();
        assert_eq!(cmap8.depth(), 8);
        assert_eq!(cmap8.get_rgb(0), Some((10, 20, 30)));
    }

    #[test]
    fn test_convert_to_8_wrong_depth() {
        let cmap = PixColormap::new(1).unwrap();
        assert!(cmap.convert_to_8().is_err());
    }

    // ---------------------------------------------------------------
    //  to_arrays
    // ---------------------------------------------------------------

    #[test]
    fn test_to_arrays_rgb() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgb(10, 20, 30).unwrap();
        cmap.add_rgb(40, 50, 60).unwrap();
        let arrays = cmap.to_arrays(false);
        assert_eq!(arrays.red, vec![10, 40]);
        assert_eq!(arrays.green, vec![20, 50]);
        assert_eq!(arrays.blue, vec![30, 60]);
        assert!(arrays.alpha.is_none());
    }

    #[test]
    fn test_to_arrays_rgba() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgba(10, 20, 30, 128).unwrap();
        let arrays = cmap.to_arrays(true);
        assert_eq!(arrays.alpha, Some(vec![128]));
    }

    // ---------------------------------------------------------------
    //  to_rgb_table
    // ---------------------------------------------------------------

    #[test]
    fn test_to_rgb_table() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgba(0xAA, 0xBB, 0xCC, 0xDD).unwrap();
        let table = cmap.to_rgb_table();
        assert_eq!(table.len(), 1);
        assert_eq!(table[0], color::compose_rgba(0xAA, 0xBB, 0xCC, 0xDD));
    }

    // ---------------------------------------------------------------
    //  serialize / deserialize
    // ---------------------------------------------------------------

    #[test]
    fn test_serialize_roundtrip_rgb() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgba(10, 20, 30, 200).unwrap();
        cmap.add_rgba(40, 50, 60, 128).unwrap();
        let data = cmap.serialize_to_memory(ComponentsPerColor::Rgb);
        assert_eq!(data.len(), 6); // 2 colors * 3 bytes
        let restored =
            PixColormap::deserialize_from_memory(&data, ComponentsPerColor::Rgb, 2).unwrap();
        // Alpha is lost with RGB, defaults to 255
        assert_eq!(restored.get_rgba(0), Some((10, 20, 30, 255)));
        assert_eq!(restored.get_rgba(1), Some((40, 50, 60, 255)));
    }

    #[test]
    fn test_serialize_roundtrip_rgba() {
        let mut cmap = PixColormap::new(2).unwrap();
        cmap.add_rgba(10, 20, 30, 200).unwrap();
        cmap.add_rgba(40, 50, 60, 128).unwrap();
        let data = cmap.serialize_to_memory(ComponentsPerColor::Rgba);
        assert_eq!(data.len(), 8); // 2 colors * 4 bytes
        let restored =
            PixColormap::deserialize_from_memory(&data, ComponentsPerColor::Rgba, 2).unwrap();
        assert_eq!(restored.get_rgba(0), Some((10, 20, 30, 200)));
        assert_eq!(restored.get_rgba(1), Some((40, 50, 60, 128)));
    }

    #[test]
    fn test_deserialize_bad_size() {
        // ncolors > 256
        assert!(
            PixColormap::deserialize_from_memory(&[0; 3], ComponentsPerColor::Rgb, 257).is_err()
        );
        // ncolors = 0
        assert!(PixColormap::deserialize_from_memory(&[], ComponentsPerColor::Rgb, 0).is_err());
    }

    // ---------------------------------------------------------------
    //  convert_to_hex
    // ---------------------------------------------------------------

    #[test]
    fn test_convert_to_hex() {
        let data = [0xFF, 0x00, 0xAB, 0x01, 0x02, 0x03];
        let hex = PixColormap::convert_to_hex(&data, 2).unwrap();
        assert_eq!(hex, "< ff00ab 010203 >");
    }

    #[test]
    fn test_convert_to_hex_bad_length() {
        let data = [0xFF, 0x00]; // not a multiple of 3
        assert!(PixColormap::convert_to_hex(&data, 1).is_err());
    }

    // ---------------------------------------------------------------
    //  gamma_trc
    // ---------------------------------------------------------------

    #[test]
    fn test_gamma_trc_identity() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 150, 200).unwrap();
        cmap.gamma_trc(1.0, 0, 255).unwrap(); // no-op
        assert_eq!(cmap.get_rgb(0), Some((100, 150, 200)));
    }

    #[test]
    fn test_gamma_trc_brightens() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 100, 100).unwrap();
        cmap.gamma_trc(2.0, 0, 255).unwrap(); // gamma > 1 brightens
        let (r, _, _) = cmap.get_rgb(0).unwrap();
        assert!(r > 100);
    }

    #[test]
    fn test_gamma_trc_invalid() {
        let mut cmap = PixColormap::new(8).unwrap();
        assert!(cmap.gamma_trc(0.0, 0, 255).is_err()); // gamma <= 0
        assert!(cmap.gamma_trc(1.0, 200, 100).is_err()); // minval >= maxval
    }

    // ---------------------------------------------------------------
    //  contrast_trc
    // ---------------------------------------------------------------

    #[test]
    fn test_contrast_trc_zero_no_change() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(64, 128, 192).unwrap();
        cmap.contrast_trc(0.0);
        assert_eq!(cmap.get_rgb(0), Some((64, 128, 192)));
    }

    #[test]
    fn test_contrast_trc_increases_contrast() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(64, 128, 192).unwrap();
        cmap.contrast_trc(1.0);
        let (r, _, b) = cmap.get_rgb(0).unwrap();
        // dark values should get darker, light values brighter
        assert!(r < 64);
        assert!(b > 192);
    }

    #[test]
    fn test_contrast_trc_endpoints() {
        // 0 → 0 and 255 → 255 for any factor
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(0, 0, 0).unwrap();
        cmap.add_rgb(255, 255, 255).unwrap();
        cmap.contrast_trc(0.5);
        assert_eq!(cmap.get_rgb(0), Some((0, 0, 0)));
        assert_eq!(cmap.get_rgb(1), Some((255, 255, 255)));
    }

    // ---------------------------------------------------------------
    //  shift_intensity
    // ---------------------------------------------------------------

    #[test]
    fn test_shift_intensity_darken() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(200, 100, 50).unwrap();
        cmap.shift_intensity(-0.5).unwrap();
        let (r, g, b) = cmap.get_rgb(0).unwrap();
        assert_eq!(r, 100);
        assert_eq!(g, 50);
        assert_eq!(b, 25);
    }

    #[test]
    fn test_shift_intensity_fade() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 100, 100).unwrap();
        cmap.shift_intensity(0.5).unwrap();
        let (r, _, _) = cmap.get_rgb(0).unwrap();
        // 100 + 0.5 * (255 - 100) = 100 + 77 = 177
        assert!((r as i32 - 177).abs() <= 1);
    }

    #[test]
    fn test_shift_intensity_out_of_range() {
        let mut cmap = PixColormap::new(8).unwrap();
        assert!(cmap.shift_intensity(-1.5).is_err());
        assert!(cmap.shift_intensity(1.5).is_err());
    }

    // ---------------------------------------------------------------
    //  shift_by_component
    // ---------------------------------------------------------------

    #[test]
    fn test_shift_by_component_identity() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 150, 200).unwrap();
        let src = color::compose_rgb(128, 128, 128);
        cmap.shift_by_component(src, src); // src == dst → no change
        assert_eq!(cmap.get_rgb(0), Some((100, 150, 200)));
    }

    #[test]
    fn test_shift_by_component_darken() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(200, 200, 200).unwrap();
        let src = color::compose_rgb(200, 200, 200);
        let dst = color::compose_rgb(100, 100, 100);
        cmap.shift_by_component(src, dst);
        let (r, g, b) = cmap.get_rgb(0).unwrap();
        // 200 * 100 / 200 = 100
        assert_eq!(r, 100);
        assert_eq!(g, 100);
        assert_eq!(b, 100);
    }

    #[test]
    fn test_shift_by_component_brighten() {
        let mut cmap = PixColormap::new(8).unwrap();
        cmap.add_rgb(100, 100, 100).unwrap();
        let src = color::compose_rgb(100, 100, 100);
        let dst = color::compose_rgb(200, 200, 200);
        cmap.shift_by_component(src, dst);
        let (r, _, _) = cmap.get_rgb(0).unwrap();
        // 255 - (255 - 200) * (255 - 100) / (255 - 100) = 255 - 55 = 200
        assert_eq!(r, 200);
    }

    // ---------------------------------------------------------------
    //  build_gamma_trc helper
    // ---------------------------------------------------------------

    #[test]
    fn test_build_gamma_trc_identity() {
        let lut = build_gamma_trc(1.0, 0, 255);
        for i in 0..256 {
            assert_eq!(lut[i], i as u8);
        }
    }

    #[test]
    fn test_build_gamma_trc_endpoints() {
        let lut = build_gamma_trc(2.0, 0, 255);
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
    }

    #[test]
    fn test_build_gamma_trc_minmax_range() {
        let lut = build_gamma_trc(1.0, 50, 200);
        // Values below minval should be 0
        assert_eq!(lut[0], 0);
        assert_eq!(lut[49], 0);
        // Values above maxval should be 255
        assert_eq!(lut[201], 255);
        assert_eq!(lut[255], 255);
    }

    // ---------------------------------------------------------------
    //  build_contrast_trc helper
    // ---------------------------------------------------------------

    #[test]
    fn test_build_contrast_trc_zero() {
        let lut = build_contrast_trc(0.0);
        for i in 0..256 {
            assert_eq!(lut[i], i as u8);
        }
    }

    #[test]
    fn test_build_contrast_trc_endpoints() {
        let lut = build_contrast_trc(1.0);
        assert_eq!(lut[0], 0);
        assert_eq!(lut[255], 255);
    }

    #[test]
    fn test_build_contrast_trc_monotonic() {
        let lut = build_contrast_trc(0.5);
        for i in 1..256 {
            assert!(lut[i] >= lut[i - 1], "not monotonic at {i}");
        }
    }

    // ---------------------------------------------------------------
    //  pixel_shift_by_component helper
    // ---------------------------------------------------------------

    #[test]
    fn test_pixel_shift_by_component_identity() {
        let src = color::compose_rgb(128, 128, 128);
        let (r, g, b) = pixel_shift_by_component(100, 150, 200, src, src);
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn test_pixel_shift_by_component_decrease() {
        let src = color::compose_rgb(200, 200, 200);
        let dst = color::compose_rgb(100, 100, 100);
        let (r, _, _) = pixel_shift_by_component(200, 200, 200, src, dst);
        assert_eq!(r, 100); // 200 * 100 / 200
    }

    #[test]
    fn test_pixel_shift_by_component_increase() {
        let src = color::compose_rgb(100, 100, 100);
        let dst = color::compose_rgb(200, 200, 200);
        let (r, _, _) = pixel_shift_by_component(100, 100, 100, src, dst);
        // 255 - (255 - 200) * (255 - 100) / (255 - 100) = 200
        assert_eq!(r, 200);
    }
}
