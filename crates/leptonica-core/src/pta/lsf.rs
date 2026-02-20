//! Least-squares fitting and Numa conversion for Pta.
//!
//! Corresponds to functions in C Leptonica's `ptafunc1.c`.

use crate::error::{Error, Result};
use crate::numa::Numa;
use crate::pta::Pta;

impl Pta {
    /// Linear least-squares fit y = ax + b.
    ///
    /// Returns `(a, b)`. Also returns a Numa of fitted values if `fit` is true.
    ///
    /// Special cases:
    /// - If `want_a` is false: fit a horizontal line (a=0), return (0.0, b).
    /// - If `want_b` is false: fit through origin (b=0), return (a, 0.0).
    ///
    /// C equivalent: `ptaGetLinearLSF()` in `ptafunc1.c`
    pub fn get_linear_lsf(
        &self,
        want_a: bool,
        want_b: bool,
        fit: bool,
    ) -> Result<(f32, f32, Option<Numa>)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Quadratic LSF: y = ax² + bx + c. Returns `(a, b, c, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetQuadraticLSF()` in `ptafunc1.c`
    pub fn get_quadratic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, Option<Numa>)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Cubic LSF: y = ax³ + bx² + cx + d. Returns `(a, b, c, d, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetCubicLSF()` in `ptafunc1.c`
    pub fn get_cubic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, f32, Option<Numa>)> {
        todo!("Phase 16.3 GREEN")
    }

    /// Quartic LSF: y = ax⁴+bx³+cx²+dx+e. Returns `(a,b,c,d,e, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetQuarticLSF()` in `ptafunc1.c`
    pub fn get_quartic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, f32, f32, Option<Numa>)> {
        todo!("Phase 16.3 GREEN")
    }
}

/// Evaluate y = ax + b at x.
///
/// C equivalent: `applyLinearFit()` in `ptafunc1.c`
pub fn apply_linear_fit(a: f32, b: f32, x: f32) -> f32 {
    todo!("Phase 16.3 GREEN")
}

/// Evaluate y = ax² + bx + c at x.
///
/// C equivalent: `applyQuadraticFit()` in `ptafunc1.c`
pub fn apply_quadratic_fit(a: f32, b: f32, c: f32, x: f32) -> f32 {
    todo!("Phase 16.3 GREEN")
}

/// Evaluate y = ax³ + bx² + cx + d at x.
///
/// C equivalent: `applyCubicFit()` in `ptafunc1.c`
pub fn apply_cubic_fit(a: f32, b: f32, c: f32, d: f32, x: f32) -> f32 {
    todo!("Phase 16.3 GREEN")
}

/// Evaluate y = ax⁴ + bx³ + cx² + dx + e at x.
///
/// C equivalent: `applyQuarticFit()` in `ptafunc1.c`
pub fn apply_quartic_fit(a: f32, b: f32, c: f32, d: f32, e: f32, x: f32) -> f32 {
    todo!("Phase 16.3 GREEN")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_linear_pta() -> Pta {
        // y = 2x + 1
        let mut p = Pta::new();
        for i in 0..5i32 {
            p.push(i as f32, 2.0 * i as f32 + 1.0);
        }
        p
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_linear_lsf_full() {
        let p = make_linear_pta();
        let (a, b, _) = p.get_linear_lsf(true, true, false).unwrap();
        assert!((a - 2.0).abs() < 0.01, "a={a}");
        assert!((b - 1.0).abs() < 0.01, "b={b}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_linear_lsf_through_origin() {
        // y = 3x  → a=3, b=0
        let mut p = Pta::new();
        for i in 1..=4i32 {
            p.push(i as f32, 3.0 * i as f32);
        }
        let (a, b, _) = p.get_linear_lsf(true, false, false).unwrap();
        assert!((a - 3.0).abs() < 0.01, "a={a}");
        assert_eq!(b, 0.0);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_linear_lsf_horizontal() {
        // y = 5 (constant) → a=0, b=5
        let mut p = Pta::new();
        for i in 0..4i32 {
            p.push(i as f32, 5.0);
        }
        let (a, b, _) = p.get_linear_lsf(false, true, false).unwrap();
        assert_eq!(a, 0.0);
        assert!((b - 5.0).abs() < 0.01, "b={b}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_linear_lsf_with_fit_numa() {
        let p = make_linear_pta();
        let (_, _, nafit) = p.get_linear_lsf(true, true, true).unwrap();
        let nafit = nafit.unwrap();
        assert_eq!(nafit.len(), 5);
        // nafit[i] ≈ 2i+1
        for i in 0..5 {
            let expected = 2.0 * i as f32 + 1.0;
            let got = nafit.get(i).unwrap();
            assert!(
                (got - expected).abs() < 0.01,
                "i={i}: got={got} expected={expected}"
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_quadratic_lsf() {
        // y = x² + 2x + 3
        let mut p = Pta::new();
        for i in 0..6i32 {
            let x = i as f32;
            p.push(x, x * x + 2.0 * x + 3.0);
        }
        let (a, b, c, _) = p.get_quadratic_lsf(false).unwrap();
        assert!((a - 1.0).abs() < 0.01, "a={a}");
        assert!((b - 2.0).abs() < 0.01, "b={b}");
        assert!((c - 3.0).abs() < 0.01, "c={c}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_cubic_lsf() {
        // y = x³ - x² + 2x - 1
        let mut p = Pta::new();
        for i in 0..6i32 {
            let x = i as f32;
            p.push(x, x * x * x - x * x + 2.0 * x - 1.0);
        }
        let (a, b, c, d, _) = p.get_cubic_lsf(false).unwrap();
        assert!((a - 1.0).abs() < 0.05, "a={a}");
        assert!((b + 1.0).abs() < 0.05, "b={b}");
        assert!((c - 2.0).abs() < 0.05, "c={c}");
        assert!((d + 1.0).abs() < 0.05, "d={d}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_quartic_lsf() {
        // y = x⁴ + x² + 1
        let mut p = Pta::new();
        for i in 0..6i32 {
            let x = i as f32;
            p.push(x, x * x * x * x + x * x + 1.0);
        }
        let (a, b, c, d, e, _) = p.get_quartic_lsf(false).unwrap();
        assert!((a - 1.0).abs() < 0.1, "a={a}");
        assert!(b.abs() < 0.1, "b={b}");
        assert!((c - 1.0).abs() < 0.1, "c={c}");
        assert!(d.abs() < 0.1, "d={d}");
        assert!((e - 1.0).abs() < 0.1, "e={e}");
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_apply_linear_fit() {
        assert!((apply_linear_fit(2.0, 1.0, 3.0) - 7.0).abs() < 1e-5);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_apply_quadratic_fit() {
        // y = 1*4 + 2*2 + 3 = 11
        assert!((apply_quadratic_fit(1.0, 2.0, 3.0, 2.0) - 11.0).abs() < 1e-5);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_apply_cubic_fit() {
        // y = 1*8 - 1*4 + 2*2 - 1 = 11
        assert!((apply_cubic_fit(1.0, -1.0, 2.0, -1.0, 2.0) - 11.0).abs() < 1e-5);
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_apply_quartic_fit() {
        // y = 1*16 + 0 + 1*4 + 0 + 1 = 21
        assert!((apply_quartic_fit(1.0, 0.0, 1.0, 0.0, 1.0, 2.0) - 21.0).abs() < 1e-5);
    }
}
