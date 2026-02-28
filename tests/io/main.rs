#[path = "../common/mod.rs"]
mod common;

mod encoding_reg;
mod files_reg;
#[cfg(feature = "gif-format")]
mod gifio_reg;
mod ioformats_reg;
mod iomisc_reg;
mod jp2kio_reg;
mod jpegio_reg;
#[cfg(feature = "tiff-format")]
mod mtiff_reg;
#[cfg(all(feature = "pdf-format", feature = "tiff-format"))]
mod pdfio1_reg;
#[cfg(all(feature = "pdf-format", feature = "tiff-format"))]
mod pdfio2_reg;
#[cfg(all(feature = "pdf-format", feature = "tiff-format"))]
mod pdfseg_reg;
mod pixtile_reg;
mod pngio_reg;
mod pnmio_reg;
#[cfg(feature = "ps-format")]
mod psio_reg;
#[cfg(feature = "ps-format")]
mod psioseg_reg;
mod spixio_reg;
#[cfg(feature = "webp-format")]
mod webpanimio_reg;
#[cfg(feature = "webp-format")]
mod webpio_reg;
mod writetext_reg;

mod convertfiles_reg;
mod io_coverage_reg;
mod partify_reg;
