//! Binary image 2x reduction by subsampling
//!
//! # See also
//! C Leptonica: `binreduce.c`

use crate::core::{Pix, PixelDepth, Result};

/// Make the permutation table for 2x binary subsampling
///
/// The table rearranges bits from the folded order (0,4,1,5,2,6,3,7)
/// to the natural order (0,1,2,3,4,5,6,7).
///
/// # See also
/// C Leptonica: `makeSubsampleTab2x()` in `binreduce.c`
pub fn make_subsample_tab_2x() -> Vec<u8> {
    let mut tab = vec![0u8; 256];
    for i in 0..256u16 {
        // Input bits in positions 0,4,1,5,2,6,3,7 (of a byte)
        // Output: rearrange to 0,1,2,3,4,5,6,7
        let mut val = 0u8;
        // bit 7 (MSB) of input → bit 7 of output
        if i & 0x80 != 0 {
            val |= 0x80;
        }
        // bit 6 of input (was pos 4) → bit 6 of output (pos 1 in output byte)
        if i & 0x40 != 0 {
            val |= 0x08;
        }
        // bit 5 of input → bit 5 of output
        if i & 0x20 != 0 {
            val |= 0x40;
        }
        // bit 4 of input → bit 4 of output
        if i & 0x10 != 0 {
            val |= 0x04;
        }
        // bit 3 of input → bit 3 of output
        if i & 0x08 != 0 {
            val |= 0x20;
        }
        // bit 2 of input → bit 2 of output
        if i & 0x04 != 0 {
            val |= 0x02;
        }
        // bit 1 of input → bit 1 of output
        if i & 0x02 != 0 {
            val |= 0x10;
        }
        // bit 0 (LSB) of input → bit 0 of output
        if i & 0x01 != 0 {
            val |= 0x01;
        }
        tab[i as usize] = val;
    }
    tab
}

/// 2x binary image reduction by subsampling
///
/// Takes every other pixel from every other row, producing an image
/// at half the original dimensions.
///
/// # See also
/// C Leptonica: `pixReduceBinary2()` in `binreduce.c`
pub fn reduce_binary_2(pix: &Pix) -> Result<Pix> {
    if pix.depth() != PixelDepth::Bit1 {
        return Err(crate::core::Error::UnsupportedDepth(pix.depth().bits()));
    }

    let ws = pix.width();
    let hs = pix.height();
    if hs <= 1 {
        return Err(crate::core::Error::InvalidParameter(
            "height must be at least 2".into(),
        ));
    }

    let wd = ws / 2;
    let hd = hs / 2;

    let pixd = Pix::new(wd, hd, PixelDepth::Bit1)?;
    let mut pixd = pixd
        .try_into_mut()
        .map_err(|_| crate::core::Error::AllocationFailed)?;

    // Simple subsampling: take pixel at (2j, 2i) for each output pixel (j, i)
    for id in 0..hd {
        let is = id * 2;
        for jd in 0..wd {
            let js = jd * 2;
            let val = pix.get_pixel(js, is).unwrap_or(0);
            if val != 0 {
                pixd.set_pixel(jd, id, 1)?;
            }
        }
    }

    Ok(pixd.into())
}
