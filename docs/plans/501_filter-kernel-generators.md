# Filter: kernel.c の生成系 5 関数を移植

Status: IMPLEMENTED
作成日: 2026-05-10
完了日: 2026-05-10
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ G

## 対象 C 関数

`reference/leptonica/src/kernel.c` 1019-1239 行で定義された生成系 5 関数:

| C 関数                  | 行   | 用途                                                          |
| ----------------------- | ---- | ------------------------------------------------------------- |
| `parseStringForNumbers` | 1019 | 区切り文字列→ Numa 変換 (`kernelCreateFromString` の内部依存) |
| `makeFlatKernel`        | 1069 | 矩形フラットカーネル (height, width, cy, cx 指定)             |
| `makeGaussianKernel`    | 1113 | 矩形ガウシアンカーネル (halfh, halfw, stdev, max)             |
| `makeGaussianKernelSep` | 1166 | 分離可能ガウシアン (kely, kelx の二本)                        |
| `makeDoGKernel`         | 1210 | DoG (Difference of Gaussians) バンドパス                      |

## 既存実装との関係

`src/filter/kernel.rs` には既に以下が存在:

- `Kernel::box_kernel(size)` — 正方フラット (size 同じ前提、center は size/2)
- `Kernel::gaussian(size, sigma)` — 正方ガウシアン (size 同じ前提、自動正規化)

C 版の API はより汎用 (矩形・任意の center・正規化なし) のため、別関数として
追加する。既存の `box_kernel` / `gaussian` は単純化されたショートカットとして残す。

## API 設計

```rust
impl Kernel {
    /// C: makeFlatKernel(height, width, cy, cx)
    ///
    /// Rectangular flat (low-pass) kernel with explicit origin.
    /// Returns a normalized kernel (sum = 1).
    pub fn make_flat(height: u32, width: u32, cy: u32, cx: u32) -> FilterResult<Self>;

    /// C: makeGaussianKernel(halfh, halfw, stdev, max)
    ///
    /// Rectangular Gaussian kernel with peak value `max` at center.
    /// Size = (2*halfh + 1, 2*halfw + 1). NOT normalized — caller controls
    /// normalization via convolve options.
    pub fn make_gaussian(halfh: u32, halfw: u32, stdev: f32, max: f32) -> FilterResult<Self>;

    /// C: makeGaussianKernelSep(halfh, halfw, stdev, max) -> (kelx, kely)
    ///
    /// Separable Gaussian: returns (kelx, kely) such that convolving with both
    /// in sequence is equivalent to convolving with the full Gaussian kernel.
    /// kely uses max=1.0 internally so the product at center equals `max`.
    pub fn make_gaussian_sep(
        halfh: u32, halfw: u32, stdev: f32, max: f32,
    ) -> FilterResult<(Self, Self)>;

    /// C: makeDoGKernel(halfh, halfw, stdev, ratio)
    ///
    /// Difference of Gaussians (DoG) wavelet bandpass kernel. Sum is zero,
    /// so do NOT normalize when convolving.
    /// `ratio` is sigma_wide / sigma_narrow, must be >= 1.0.
    pub fn make_dog(halfh: u32, halfw: u32, stdev: f32, ratio: f32) -> FilterResult<Self>;
}
```

`parseStringForNumbers` は `Numa` を返す pub 関数として `src/core/numa/mod.rs`
または `src/core/numa/parse.rs` に追加:

```rust
impl Numa {
    /// C: parseStringForNumbers(str, seps)
    ///
    /// Parse a string of whitespace/separator-delimited numbers into a Numa.
    /// Each token is parsed as f32. Empty input or non-numeric tokens cause an
    /// error.
    pub fn parse_from_string(s: &str, separators: &str) -> Result<Self>;
}
```

## TDD 手順

### RED コミット

`tests/filter/kernel_reg.rs` に以下のテストを `#[ignore = "not yet implemented"]` 付きで追加:

- `kernel_reg_make_flat_5x3_off_center` — 5x3, cy=2, cx=1, normval=1/15
- `kernel_reg_make_gaussian_peak_value` — halfh=halfw=2, stdev=1.0, max=2.5
- `kernel_reg_make_gaussian_sep_product_equals_full` — separable と full の差分が 1e-5 以内
- `kernel_reg_make_dog_sum_zero` — DoG の sum が 1e-4 以内
- `kernel_reg_make_dog_invalid_ratio` — ratio<1.0 でエラー

`tests/core/numa1_reg.rs` または `tests/core/parse_reg.rs` に追加:

- `numa_reg_parse_from_string_basic` — "1.5 -2 3.14" + " " → Numa[1.5, -2.0, 3.14]
- `numa_reg_parse_from_string_csv` — "1,2,3" + "," → Numa[1.0, 2.0, 3.0]

### GREEN コミット

各関数を実装し `#[ignore]` を除去。

### REFACTOR

必要に応じて (おそらく不要)。

## 完了条件

- [x] cargo test --all-features 全通過
- [x] cargo clippy --all-features --all-targets -- -D warnings 通過
- [x] cargo fmt --all -- --check 通過
- [x] PR 作成・Copilot レビュー対応・マージ
- [x] docs/porting/comparison/filter.md の追加検証エントリで kernel.c 5 件を ❌ → ✅ に更新
- [x] docs/plans/032 のステータス表で「501」を IMPLEMENTED に更新
