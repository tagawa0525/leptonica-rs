//! Least-squares fitting and Numa conversion for Pta.
//!
//! Corresponds to functions in C Leptonica's `ptafunc1.c`.

use crate::error::{Error, Result};
use crate::numa::Numa;
use crate::pta::Pta;

impl Pta {
    /// Linear least-squares fit y = ax + b.
    ///
    /// Returns `(a, b, Option<Numa>)`. The Numa contains fitted values if
    /// `fit` is true. Special cases:
    /// - `want_a=true, want_b=false`: fit through origin (b=0).
    /// - `want_a=false, want_b=true`: horizontal line (a=0).
    ///
    /// C equivalent: `ptaGetLinearLSF()` in `ptafunc1.c`
    pub fn get_linear_lsf(
        &self,
        want_a: bool,
        want_b: bool,
        fit: bool,
    ) -> Result<(f32, f32, Option<Numa>)> {
        let n = self.len();
        if n < 2 {
            return Err(Error::InvalidParameter("less than 2 pts found".to_string()));
        }
        let xa = self.x_coords();
        let ya = self.y_coords();

        let a;
        let b;
        if want_a && want_b {
            let mut sx = 0f32;
            let mut sy = 0f32;
            let mut sxx = 0f32;
            let mut sxy = 0f32;
            for i in 0..n {
                sx += xa[i];
                sy += ya[i];
                sxx += xa[i] * xa[i];
                sxy += xa[i] * ya[i];
            }
            let factor = n as f32 * sxx - sx * sx;
            if factor == 0.0 {
                return Err(Error::InvalidParameter("no solution found".to_string()));
            }
            let inv = 1.0 / factor;
            a = inv * (n as f32 * sxy - sx * sy);
            b = inv * (sxx * sy - sx * sxy);
        } else if want_a {
            // b = 0; line through origin
            let mut sxx = 0f32;
            let mut sxy = 0f32;
            for i in 0..n {
                sxx += xa[i] * xa[i];
                sxy += xa[i] * ya[i];
            }
            if sxx == 0.0 {
                return Err(Error::InvalidParameter("no solution found".to_string()));
            }
            a = sxy / sxx;
            b = 0.0;
        } else {
            // a = 0; horizontal line
            let sy: f32 = ya.iter().sum();
            a = 0.0;
            b = sy / n as f32;
        }

        let nafit = if fit {
            let mut na = Numa::with_capacity(n);
            for &x in xa {
                na.push(a * x + b);
            }
            Some(na)
        } else {
            None
        };

        Ok((a, b, nafit))
    }

    /// Quadratic LSF: y = ax² + bx + c. Returns `(a, b, c, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetQuadraticLSF()` in `ptafunc1.c`
    pub fn get_quadratic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, Option<Numa>)> {
        let n = self.len();
        if n < 3 {
            return Err(Error::InvalidParameter("less than 3 pts found".to_string()));
        }
        let xa = self.x_coords();
        let ya = self.y_coords();

        let mut sx = 0f64;
        let mut sy = 0f64;
        let mut sx2 = 0f64;
        let mut sx3 = 0f64;
        let mut sx4 = 0f64;
        let mut sxy = 0f64;
        let mut sx2y = 0f64;
        for i in 0..n {
            let x = xa[i] as f64;
            let y = ya[i] as f64;
            sx += x;
            sy += y;
            sx2 += x * x;
            sx3 += x * x * x;
            sx4 += x * x * x * x;
            sxy += x * y;
            sx2y += x * x * y;
        }

        let f = vec![
            vec![sx4, sx3, sx2],
            vec![sx3, sx2, sx],
            vec![sx2, sx, n as f64],
        ];
        let rhs = vec![sx2y, sxy, sy];
        let g = gauss_jordan_n(&f, &rhs)
            .ok_or_else(|| Error::InvalidParameter("quadratic solution failed".to_string()))?;

        let a = g[0] as f32;
        let b = g[1] as f32;
        let c = g[2] as f32;

        let nafit = if fit {
            let mut na = Numa::with_capacity(n);
            for &x in xa {
                na.push(a * x * x + b * x + c);
            }
            Some(na)
        } else {
            None
        };

        Ok((a, b, c, nafit))
    }

    /// Cubic LSF: y = ax³ + bx² + cx + d. Returns `(a, b, c, d, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetCubicLSF()` in `ptafunc1.c`
    pub fn get_cubic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, f32, Option<Numa>)> {
        let n = self.len();
        if n < 4 {
            return Err(Error::InvalidParameter("less than 4 pts found".to_string()));
        }
        let xa = self.x_coords();
        let ya = self.y_coords();

        let mut sx = 0f64;
        let mut sy = 0f64;
        let mut sx2 = 0f64;
        let mut sx3 = 0f64;
        let mut sx4 = 0f64;
        let mut sx5 = 0f64;
        let mut sx6 = 0f64;
        let mut sxy = 0f64;
        let mut sx2y = 0f64;
        let mut sx3y = 0f64;
        for i in 0..n {
            let x = xa[i] as f64;
            let y = ya[i] as f64;
            sx += x;
            sy += y;
            sx2 += x * x;
            sx3 += x * x * x;
            sx4 += x * x * x * x;
            sx5 += x * x * x * x * x;
            sx6 += x * x * x * x * x * x;
            sxy += x * y;
            sx2y += x * x * y;
            sx3y += x * x * x * y;
        }

        let f = vec![
            vec![sx6, sx5, sx4, sx3],
            vec![sx5, sx4, sx3, sx2],
            vec![sx4, sx3, sx2, sx],
            vec![sx3, sx2, sx, n as f64],
        ];
        let rhs = vec![sx3y, sx2y, sxy, sy];
        let g = gauss_jordan_n(&f, &rhs)
            .ok_or_else(|| Error::InvalidParameter("cubic solution failed".to_string()))?;

        let a = g[0] as f32;
        let b = g[1] as f32;
        let c = g[2] as f32;
        let d = g[3] as f32;

        let nafit = if fit {
            let mut na = Numa::with_capacity(n);
            for &x in xa {
                na.push(a * x * x * x + b * x * x + c * x + d);
            }
            Some(na)
        } else {
            None
        };

        Ok((a, b, c, d, nafit))
    }

    /// Quartic LSF: y = ax⁴+bx³+cx²+dx+e. Returns `(a,b,c,d,e, Option<Numa>)`.
    ///
    /// C equivalent: `ptaGetQuarticLSF()` in `ptafunc1.c`
    pub fn get_quartic_lsf(&self, fit: bool) -> Result<(f32, f32, f32, f32, f32, Option<Numa>)> {
        let n = self.len();
        if n < 5 {
            return Err(Error::InvalidParameter("less than 5 pts found".to_string()));
        }
        let xa = self.x_coords();
        let ya = self.y_coords();

        let mut sx = 0f64;
        let mut sy = 0f64;
        let mut sx2 = 0f64;
        let mut sx3 = 0f64;
        let mut sx4 = 0f64;
        let mut sx5 = 0f64;
        let mut sx6 = 0f64;
        let mut sx7 = 0f64;
        let mut sx8 = 0f64;
        let mut sxy = 0f64;
        let mut sx2y = 0f64;
        let mut sx3y = 0f64;
        let mut sx4y = 0f64;
        for i in 0..n {
            let x = xa[i] as f64;
            let y = ya[i] as f64;
            sx += x;
            sy += y;
            sx2 += x * x;
            sx3 += x * x * x;
            sx4 += x * x * x * x;
            sx5 += x * x * x * x * x;
            sx6 += x * x * x * x * x * x;
            sx7 += x * x * x * x * x * x * x;
            sx8 += x * x * x * x * x * x * x * x;
            sxy += x * y;
            sx2y += x * x * y;
            sx3y += x * x * x * y;
            sx4y += x * x * x * x * y;
        }

        let f = vec![
            vec![sx8, sx7, sx6, sx5, sx4],
            vec![sx7, sx6, sx5, sx4, sx3],
            vec![sx6, sx5, sx4, sx3, sx2],
            vec![sx5, sx4, sx3, sx2, sx],
            vec![sx4, sx3, sx2, sx, n as f64],
        ];
        let rhs = vec![sx4y, sx3y, sx2y, sxy, sy];
        let g = gauss_jordan_n(&f, &rhs)
            .ok_or_else(|| Error::InvalidParameter("quartic solution failed".to_string()))?;

        let a = g[0] as f32;
        let b = g[1] as f32;
        let c = g[2] as f32;
        let d = g[3] as f32;
        let e = g[4] as f32;

        let nafit = if fit {
            let mut na = Numa::with_capacity(n);
            for &x in xa {
                na.push(a * x * x * x * x + b * x * x * x + c * x * x + d * x + e);
            }
            Some(na)
        } else {
            None
        };

        Ok((a, b, c, d, e, nafit))
    }
}

/// Evaluate y = ax + b at x.
///
/// C equivalent: `applyLinearFit()` in `ptafunc1.c`
pub fn apply_linear_fit(a: f32, b: f32, x: f32) -> f32 {
    a * x + b
}

/// Evaluate y = ax² + bx + c at x.
///
/// C equivalent: `applyQuadraticFit()` in `ptafunc1.c`
pub fn apply_quadratic_fit(a: f32, b: f32, c: f32, x: f32) -> f32 {
    a * x * x + b * x + c
}

/// Evaluate y = ax³ + bx² + cx + d at x.
///
/// C equivalent: `applyCubicFit()` in `ptafunc1.c`
pub fn apply_cubic_fit(a: f32, b: f32, c: f32, d: f32, x: f32) -> f32 {
    a * x * x * x + b * x * x + c * x + d
}

/// Evaluate y = ax⁴ + bx³ + cx² + dx + e at x.
///
/// C equivalent: `applyQuarticFit()` in `ptafunc1.c`
pub fn apply_quartic_fit(a: f32, b: f32, c: f32, d: f32, e: f32, x: f32) -> f32 {
    a * x * x * x * x + b * x * x * x + c * x * x + d * x + e
}

/// Gauss-Jordan elimination for an n×n system Ax = b.
/// Returns the solution x, or None if singular.
fn gauss_jordan_n(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    // Build augmented matrix [A | b]
    let mut m: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = a[i].clone();
            row.push(b[i]);
            row
        })
        .collect();

    for col in 0..n {
        // Find pivot (partial pivoting)
        let pivot = (col..n).max_by(|&i, &j| {
            m[i][col]
                .abs()
                .partial_cmp(&m[j][col].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        m.swap(col, pivot);

        let diag = m[col][col];
        if diag.abs() < 1e-12 {
            return None;
        }
        // Scale pivot row
        for v in &mut m[col] {
            *v /= diag;
        }
        // Eliminate column
        let col_vals = m[col].clone();
        for (row, m_row) in m.iter_mut().enumerate().take(n) {
            if row == col {
                continue;
            }
            let factor = m_row[col];
            for (rv, &cv) in m_row.iter_mut().zip(col_vals.iter()) {
                *rv -= cv * factor;
            }
        }
    }

    Some((0..n).map(|i| m[i][n]).collect())
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
    fn test_linear_lsf_full() {
        let p = make_linear_pta();
        let (a, b, _) = p.get_linear_lsf(true, true, false).unwrap();
        assert!((a - 2.0).abs() < 0.01, "a={a}");
        assert!((b - 1.0).abs() < 0.01, "b={b}");
    }

    #[test]
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
    fn test_apply_linear_fit() {
        assert!((apply_linear_fit(2.0, 1.0, 3.0) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_quadratic_fit() {
        // y = 1*4 + 2*2 + 3 = 11
        assert!((apply_quadratic_fit(1.0, 2.0, 3.0, 2.0) - 11.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_cubic_fit() {
        // y = 1*8 - 1*4 + 2*2 - 1 = 7
        assert!((apply_cubic_fit(1.0, -1.0, 2.0, -1.0, 2.0) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_quartic_fit() {
        // y = 1*16 + 0 + 1*4 + 0 + 1 = 21
        assert!((apply_quartic_fit(1.0, 0.0, 1.0, 0.0, 1.0, 2.0) - 21.0).abs() < 1e-5);
    }
}
