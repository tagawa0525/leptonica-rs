//! glibc-compatible pseudo-random generator.
//!
//! Several C Leptonica routines draw from `rand()` — `pixcmapCreateRandom()`
//! for random colormaps, `pixRandomHarmonicWarp()` for warp parameters, and
//! the `prog/*_reg.c` programs that synthesise their own inputs. Reproducing
//! their output bit for bit means reproducing glibc's generator, not just
//! "some" pseudo-random sequence.
//!
//! # Why the sequence has to be passed around
//!
//! In C, `rand()` reads and advances one process-wide stream. A program that
//! calls `pixaDisplayRandomCmap()` four times gets four *different* colormaps,
//! each continuing where the last stopped. Rust code that re-seeds per call
//! would return the same colormap every time and diverge from C after the
//! first call.
//!
//! Rather than introduce global mutable state, the C-compatible entry points
//! take `&mut GlibcRand` (see [`crate::core::PixColormap::create_random_with`]
//! and [`crate::core::Pixa::display_random_cmap_with`]). Reproducing a whole
//! C program means creating one generator and threading it through every call
//! in the same order.

/// glibc's `rand()`, the TYPE_3 additive-feedback generator.
///
/// `GlibcRand::new(seed)` corresponds to `srand(seed)`. A C program that
/// never calls `srand()` behaves as if seeded with 1.
///
/// # Examples
///
/// ```
/// use leptonica::core::GlibcRand;
///
/// // The classic glibc sequence for seed 1.
/// let mut rng = GlibcRand::new(1);
/// assert_eq!(rng.next_u32(), 1_804_289_383);
/// assert_eq!(rng.next_u32(), 846_930_886);
/// ```
#[derive(Debug, Clone)]
pub struct GlibcRand {
    /// Ring over the last 31 values of the recurrence.
    r: [u32; DEG],
    /// Position of `r[i - 31]` for the next output.
    pos: usize,
}

/// Degree of the feedback polynomial (glibc `DEG_3`).
const DEG: usize = 31;
/// Separation of the feedback tap (glibc `SEP_3`).
const SEP: usize = 3;
/// Outputs glibc discards after seeding (`10 * DEG_3`).
const DISCARD: usize = 10 * DEG;

impl GlibcRand {
    /// Seed the generator, equivalent to C `srand(seed)`.
    pub fn new(seed: u32) -> Self {
        // glibc seeds 31 words with a Lehmer generator, mirrors the first
        // three, then runs the additive recurrence for 310 discarded steps.
        let mut r = vec![0u32; DEG + SEP + DISCARD];
        r[0] = seed;
        for i in 1..DEG {
            // r[i] = (16807 * r[i-1]) % 2147483647, via Schrage's trick to
            // stay inside 32 bits.
            let prev = i64::from(r[i - 1]);
            let hi = prev / 127_773;
            let lo = prev % 127_773;
            let mut word = 16_807 * lo - 2_836 * hi;
            if word < 0 {
                word += 2_147_483_647;
            }
            r[i] = word as u32;
        }
        for i in DEG..DEG + SEP {
            r[i] = r[i - DEG];
        }
        for i in DEG + SEP..r.len() {
            r[i] = r[i - DEG].wrapping_add(r[i - SEP]);
        }

        // Keep only the last DEG values. The next output needs r[len - DEG]
        // and r[len - SEP], which are the first and the SEP-th-from-last
        // entries of that window, so the read position starts at 0.
        let mut ring = [0u32; DEG];
        ring.copy_from_slice(&r[r.len() - DEG..]);
        Self { r: ring, pos: 0 }
    }

    /// Produce the next value, equivalent to C `rand()`.
    ///
    /// Named `next_u32` rather than `next` so it is not mistaken for
    /// [`Iterator::next`].
    ///
    /// The result is in `0..=RAND_MAX` (`i32::MAX`).
    pub fn next_u32(&mut self) -> u32 {
        // r[i] = r[i - DEG] + r[i - SEP]; the output is the high 31 bits.
        let back = self.r[self.pos];
        let tap = self.r[(self.pos + DEG - SEP) % DEG];
        let v = back.wrapping_add(tap);
        self.r[self.pos] = v;
        self.pos = (self.pos + 1) % DEG;
        v >> 1
    }

    /// The low byte of the next value, as C's colormap code uses it
    /// (`(l_uint32)rand() & 0xff`).
    pub fn next_byte(&mut self) -> u8 {
        (self.next_u32() & 0xff) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first outputs of glibc `rand()` after `srand(1)`, which is also
    /// the state a C program starts in when it never calls `srand()`.
    /// Values taken from a C program compiled against glibc.
    #[test]
    fn test_matches_glibc_seed_1() {
        let mut rng = GlibcRand::new(1);
        let got: Vec<u32> = (0..8).map(|_| rng.next_u32()).collect();
        assert_eq!(
            got,
            vec![
                1_804_289_383,
                846_930_886,
                1_681_692_777,
                1_714_636_915,
                1_957_747_793,
                424_238_335,
                719_885_386,
                1_649_760_492,
            ]
        );
    }

    /// `overlap_reg.c` uses `srand(45617)`; values dumped from C.
    #[test]
    fn test_matches_glibc_seed_45617() {
        let mut rng = GlibcRand::new(45617);
        let got: Vec<u32> = (0..5).map(|_| rng.next_u32()).collect();
        assert_eq!(
            got,
            vec![
                1_484_474_668,
                554_040_094,
                799_607_895,
                760_240_358,
                155_811_871
            ]
        );
    }

    /// The stream must keep advancing across long runs: a colormap consumes
    /// 3 * 254 = 762 values, and the next caller continues from there. Both
    /// figures come from C.
    #[test]
    fn test_stream_advances_across_a_full_colormap() {
        let mut rng = GlibcRand::new(1);
        let sum: u32 = (0..762).map(|_| u32::from(rng.next_byte())).sum();
        assert_eq!(sum, 99_666, "low-byte sum of the first 762 values");

        // C, with the three calls sequenced into separate statements —
        // passing them as printf arguments prints them in gcc's
        // right-to-left evaluation order, which is not the stream order.
        let second: Vec<u8> = (0..3).map(|_| rng.next_byte()).collect();
        assert_eq!(
            second,
            vec![30, 168, 172],
            "first RGB of the second colormap"
        );
    }
}
