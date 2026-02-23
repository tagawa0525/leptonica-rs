# ignore テスト実装計画

Status: IMPLEMENTED

## Context

現在 136 個の `#[ignore]` テストが存在する。これらを可能な限り実装することで、
Rust 移植の品質と完成度を高める。

調査の結果、以下の分類になった：

- **テスト本体を書くだけ**: プロダクションコードはすでに実装済み。テスト関数が空のままになっている
- **新規実装が必要**: 対応する Rust 実装がなく、C 版アルゴリズムを移植する必要がある
- **依存関係により保留**: 他 crate の未実装機能に依存しており、先に依存先を実装する必要がある

本計画では上位 6 PR で合計 7 個の ignore テストを解除する。

---

## PR 一覧と順序

| PR | ブランチ | 対象テスト | 分類 | 依存 |
|---|---|---|---|---|
| 1 | `feat/region-grayfill-hybrid-test` | `grayfill_reg_hybrid_comparison` | テスト本体のみ | なし |
| 2 | `feat/core-rasteropip-mirrored-border` | `rasteropip_reg_mirrored_border` | テスト本体のみ | なし |
| 3 | `feat/core-numa-morphology` | `numa3_reg_morphology` | 新規実装 | なし |
| 4 | `feat/core-numa-find-extrema` | `extrema_reg_find_extrema` | 新規実装 | なし |
| 5 | `feat/core-numa-threshold` | `numa3_reg_threshold_finding` | 新規実装 | PR3 |
| 6 | `feat/filter-scale-gray-rank` | `rank_reg_scale_gray_rank2/cascade/minmax` | 新規実装 | なし |

PR1〜4 は互いに独立。PR5 は PR3 に依存（`Numa::transform` が必要）。
同一 worktree 内では 1 PR ずつ順次進める。

---

## PR1: grayfill ハイブリッド比較テスト

**ブランチ**: `feat/region-grayfill-hybrid-test`

### 変更ファイル

- `crates/leptonica-region/tests/grayfill_reg.rs`（テスト本体の記述）

### 実装内容

`grayfill_reg_hybrid_comparison` の空の本体に、C 版 `grayfill_reg.c` の checks 19–34 相当を実装する。

使用する既存 API（プロダクションコード変更なし）:
- `PixMut::add_constant_inplace(i32)` — `crates/leptonica-core/src/pix/arith.rs:434`
- `seedfill_gray(&seed, &mask, connectivity)` — `leptonica-region`（公開済み）
- `seedfill_gray_simple(&seed, &mask, connectivity)` — `leptonica-region`（公開済み）
- `seedfill_gray_inv(&seed, &mask, connectivity)` — `leptonica-region`（公開済み）
- `seedfill_gray_inv_simple(&seed, &mask, connectivity)` — `leptonica-region`（公開済み）

### テスト内容（C 版 checks 19–34 相当）

```rust
fn grayfill_reg_hybrid_comparison() {
    let mut rp = RegParams::new("gfill_hybrid");

    let mask = make_mask_200();

    // seed1: 中央 3x3 に値 50（standard fill 用）
    let seed1 = { /* PixMut、中央 (99..=101, 99..=101) に 50 */ };
    // seed2: 中央 3x3 に値 205（inv fill 用）
    let seed2 = { /* PixMut、中央 (99..=101, 99..=101) に 205 */ };

    // add_constant_inplace でシード値を変化させてからフィル
    let mut s1 = seed1.deep_clone().try_into_mut().unwrap();
    s1.add_constant_inplace(-30);
    let s1: Pix = s1.into();

    let mut s2 = seed2.deep_clone().try_into_mut().unwrap();
    s2.add_constant_inplace(60);
    let s2: Pix = s2.into();

    // hybrid (seedfill_gray) vs iterative (seedfill_gray_simple) を比較
    let mask_inv = mask.invert();
    let h4  = seedfill_gray(&s1, &mask_inv, ConnectivityType::FourWay).unwrap();
    let i4  = seedfill_gray_simple(&s1, &mask_inv, ConnectivityType::FourWay).unwrap();
    rp.compare_values(1.0, if h4.equals(&i4) { 1.0 } else { 0.0 }, 0.0);

    // inv 版も同様
    let ih4 = seedfill_gray_inv(&s2, &mask, ConnectivityType::FourWay).unwrap();
    let ii4 = seedfill_gray_inv_simple(&s2, &mask, ConnectivityType::FourWay).unwrap();
    rp.compare_values(1.0, if ih4.equals(&ii4) { 1.0 } else { 0.0 }, 0.0);

    assert!(rp.cleanup());
}
```

### コミット構成

1. RED: `#[ignore]` を残したままテスト本体を記述（コンパイルは通るが ignore のまま）
2. GREEN: `#[ignore]` を除去してテストが通ることを確認

---

## PR2: rasteropip ミラーボーダーテスト

**ブランチ**: `feat/core-rasteropip-mirrored-border`

### 変更ファイル

- `crates/leptonica-core/tests/rasteropip_reg.rs`（テスト本体の記述）

### 実装内容

使用する既存 API（プロダクションコード変更なし）:
- `Pix::remove_border(npix: u32)` — `crates/leptonica-core/src/pix/border.rs:156`
- `Pix::add_mirrored_border(left, right, top, bot: u32)` — `crates/leptonica-core/src/pix/border.rs:251`

### テスト内容（C 版 rasteropip_reg.c check 1 相当）

```rust
fn rasteropip_reg_mirrored_border() {
    let mut rp = RegParams::new("rasteropip_mirror");
    let pixs = load_test_image("test8.jpg").unwrap();
    let pixt = pixs.remove_border(40).unwrap();
    let pixd = pixt.add_mirrored_border(40, 40, 40, 40).unwrap();
    // 寸法は元画像と同じ
    rp.compare_values(pixs.width() as f64, pixd.width() as f64, 0.0);
    rp.compare_values(pixs.height() as f64, pixd.height() as f64, 0.0);
    rp.compare_pix(&pixs, &pixd);  // golden 比較
    assert!(rp.cleanup());
}
```

### コミット構成

1. RED: テスト本体を記述（`#[ignore]` のまま）
2. GREEN: `#[ignore]` を除去

---

## PR3: Numa 形態学操作 + transform

**ブランチ**: `feat/core-numa-morphology`

### 変更ファイル

- `crates/leptonica-core/src/numa/operations.rs`（実装追加）
- `crates/leptonica-core/src/numa/mod.rs`（メソッド公開）
- `crates/leptonica-core/tests/numa3_reg.rs`（テスト本体の記述）

### 実装する関数

C 版: `reference/leptonica/src/numafunc2.c:162–427`

```rust
impl Numa {
    /// C: numaErode()
    /// Linear morphological erosion with size `size` (forced odd).
    /// Pads boundary with f32::MAX (large values) before taking min in window.
    pub fn erode(&self, size: u32) -> Result<Numa>;

    /// C: numaDilate()
    /// Linear morphological dilation with size `size` (forced odd).
    /// Pads boundary with f32::MIN (small values) before taking max in window.
    pub fn dilate(&self, size: u32) -> Result<Numa>;

    /// C: numaOpen() = erode then dilate
    pub fn open(&self, size: u32) -> Result<Numa>;

    /// C: numaClose() = dilate then erode
    pub fn close(&self, size: u32) -> Result<Numa>;

    /// C: numaTransform()
    /// nad[i] = scale * (nas[i] + shift)
    pub fn transform(&self, shift: f32, scale: f32) -> Numa;
}
```

### アルゴリズム（erode/dilate 共通構造）

```
1. size が偶数なら size += 1
2. if size == 1 { return self.clone() }
3. hsize = size / 2
4. パディング配列 fas (len = n + 2*hsize) を作成:
   - erode: 境界を 1.0e37f32 で埋める
   - dilate: 境界を -1.0e37f32 で埋める
5. 内部 [hsize..hsize+n] に self の値をコピー
6. 各 i in 0..n について fas[i..i+size] の min/max を出力
```

### テスト内容（C 版 numa3_reg.c checks 2–6 相当）

```rust
fn numa3_reg_morphology() {
    // C版は lyra.5.na を読むが、Rust版はインラインで生成
    // シンプルな sin 波形を使ってerode/dilate/open/close の基本動作を検証
    let n = 200usize;
    let mut na = Numa::new();
    for i in 0..n {
        na.push((i as f32 * 0.1).sin());
    }

    let ne = na.erode(21).unwrap();
    let nd = na.dilate(21).unwrap();
    // dilated >= original >= eroded (各点で)
    for i in 0..n {
        assert!(nd[i] >= na[i] - 1e-5);
        assert!(ne[i] <= na[i] + 1e-5);
    }
    // open <= dilate, close >= erode
    let no = na.open(21).unwrap();
    let nc = na.close(21).unwrap();
    for i in 0..n {
        assert!(no[i] <= nd[i] + 1e-5);
        assert!(nc[i] >= ne[i] - 1e-5);
    }
    // transform: shift=1.0, scale=2.0 → 各値が 2*(x+1)
    let nt = na.transform(1.0, 2.0);
    rp.compare_values((2.0 * (na[0] + 1.0)) as f64, nt[0] as f64, 1e-4);
}
```

### コミット構成

1. RED: テスト本体を記述（`#[ignore]` のまま）
2. GREEN: `erode/dilate/open/close/transform` を実装して `#[ignore]` 除去

---

## PR4: Numa::find_extrema

**ブランチ**: `feat/core-numa-find-extrema`

### 変更ファイル

- `crates/leptonica-core/src/numa/operations.rs`（実装追加）
- `crates/leptonica-core/tests/extrema_reg.rs`（テスト本体の記述）

### 実装する関数

C 版: `reference/leptonica/src/numafunc2.c:2491–2567`

```rust
impl Numa {
    /// C: numaFindExtrema(nas, delta, pnav)
    /// ヒステリシス delta 付き極値位置を返す。
    /// 戻り値: 極値インデックスの Numa
    pub fn find_extrema(&self, delta: f32) -> Result<Numa>;

    /// delta に加えて極値の値も返す (pnav に対応)
    pub fn find_extrema_with_values(&self, delta: f32) -> Result<(Numa, Numa)>;
}
```

### アルゴリズム（C 版 numaFindExtrema 参照）

```
1. startval = self[0]
2. delta 以上離れた最初の点を検索
3. val > startval なら direction = Peak(+1), else direction = Valley(-1)
4. 以降を走査:
   - Peak方向: maxval 更新。maxval - val >= delta で峰を確定（nad に loc 記録）
                → direction = -1, minval = val, loc = i
   - Valley方向: minval 更新。val - minval >= delta で谷を確定（nad に loc 記録）
                → direction = +1, maxval = val, loc = i
5. 最後の loc は保存しない（C版コメントより）
```

### テスト内容

```rust
fn extrema_reg_find_extrema() {
    let mut rp = RegParams::new("extrema_find");
    let pi = std::f64::consts::PI;
    let mut na = Numa::new();
    for i in 0..500 {
        let angle = 0.02293 * i as f64 * pi;
        na.push(angle.sin() as f32);
    }
    // delta=0.1 で極値を検出
    let nax = na.find_extrema(0.1).unwrap();
    // サイン波500点、周期≈87点 → 約11-12個の極値が検出されるはず
    rp.compare_values(1.0, if nax.len() > 8 { 1.0 } else { 0.0 }, 0.0);
    // 全インデックスが有効範囲内
    for i in 0..nax.len() {
        rp.compare_values(1.0, if (nax[i] as usize) < 500 { 1.0 } else { 0.0 }, 0.0);
    }
    assert!(rp.cleanup());
}
```

### コミット構成

1. RED: テスト本体を記述（`#[ignore]` のまま）
2. GREEN: `find_extrema` を実装して `#[ignore]` 除去

---

## PR5: Numa::find_loc_for_threshold（PR3 依存）

**ブランチ**: `feat/core-numa-threshold`

### 変更ファイル

- `crates/leptonica-core/src/numa/operations.rs`（実装追加）
- `crates/leptonica-core/tests/numa3_reg.rs`（テスト本体の記述）

### 実装する関数

C 版: `reference/leptonica/src/numafunc2.c:2597–`

```rust
impl Numa {
    /// C: numaFindLocForThreshold(na, skip, pthresh, pfract)
    /// バイモーダルヒストグラムの谷（しきい値）の位置を返す。
    /// skip: ルックアヘッド距離（0 でデフォルト 20 を使用）
    /// 戻り値: (threshold_index, optional_fraction_below)
    pub fn find_loc_for_threshold(&self, skip: usize) -> Result<(usize, f32)>;
}
```

### テスト内容

```rust
fn numa3_reg_threshold_finding() {
    // バイモーダル分布を手動で生成
    let mut na = Numa::new();
    // 低い山: bin 0-100 にガウス的分布
    for i in 0..256usize {
        let x = i as f32;
        let peak1 = (-(x - 50.0).powi(2) / 200.0).exp() * 100.0;
        let peak2 = (-(x - 180.0).powi(2) / 200.0).exp() * 60.0;
        na.push(peak1 + peak2);
    }
    // transform で正規化 (PR3 の Numa::transform を使用)
    let sum: f32 = (0..256).map(|i| na[i]).sum();
    let nt = na.transform(0.0, 1.0 / sum);
    let (thresh, _frac) = nt.find_loc_for_threshold(0).unwrap();
    // 2つの山 (50, 180) の間 (100..=150 あたり) にしきい値があるはず
    assert!(thresh > 80 && thresh < 160, "thresh={thresh}");
}
```

### コミット構成

1. RED: テスト本体を記述（`#[ignore]` のまま）
2. GREEN: `find_loc_for_threshold` を実装して `#[ignore]` 除去

---

## PR6: scale_gray_rank2/cascade/minmax

**ブランチ**: `feat/filter-scale-gray-rank`

### 変更ファイル

- `crates/leptonica-filter/src/rank.rs`（実装追加）
  ※ scale 操作だが `pixScaleGrayRank2` は rank 操作に近く、filter crate に追加する
- `crates/leptonica-filter/src/lib.rs`（pub use 追加）
- `crates/leptonica-filter/tests/rank_reg.rs`（3 テストの本体記述）

### 実装する関数

C 版: `reference/leptonica/src/scale2.c:974–1270`

```rust
/// C: pixScaleGrayMinMax()
/// 8bpp 画像を (xfact, yfact) ブロックごとに min/max/maxdiff でダウンスケール
pub fn scale_gray_min_max(pix: &Pix, xfact: u32, yfact: u32, op: MinMaxOp) -> FilterResult<Pix>;

#[derive(Debug, Clone, Copy)]
pub enum MinMaxOp { Min, Max, MaxDiff }

/// C: pixScaleGrayRank2()
/// 8bpp 画像を 2x ダウンスケール（2x2 ブロックから rank 番目の値を選択）
/// rank: 1=min, 2, 3, 4=max
pub fn scale_gray_rank2(pix: &Pix, rank: u8) -> FilterResult<Pix>;

/// C: pixScaleGrayRankCascade()
/// scale_gray_rank2 を最大 4 回カスケード適用
/// level: 各段の rank (0 は適用なし)
pub fn scale_gray_rank_cascade(
    pix: &Pix,
    level1: u8, level2: u8, level3: u8, level4: u8,
) -> FilterResult<Pix>;
```

### アルゴリズム（scale_gray_rank2）

```
入力: 8bpp 画像、rank ∈ {1,2,3,4}
出力: w/2 × h/2 の 8bpp 画像

各 (i,j) について 2x2 ブロック [2i..2i+2, 2j..2j+2] の 4 値をソートして
rank 番目（1-indexed）の値を出力ピクセルに設定。
rank=1 は min（= grayscale erosion）、rank=4 は max（= grayscale dilation）。
```

### テスト内容

```rust
fn rank_reg_scale_gray_rank2() {
    let pixs = load_test_image("test8.jpg").unwrap();
    for rank in 1u8..=4 {
        let pixd = scale_gray_rank2(&pixs, rank).unwrap();
        assert_eq!(pixd.width(), pixs.width() / 2);
        assert_eq!(pixd.height(), pixs.height() / 2);
        assert_eq!(pixd.depth(), PixelDepth::Bit8);
    }
}

fn rank_reg_scale_gray_rank_cascade() {
    let pixs = load_test_image("test8.jpg").unwrap();
    let pixd = scale_gray_rank_cascade(&pixs, 1, 2, 3, 4).unwrap();
    // 4回 2x ダウンスケール → w/16 × h/16
    assert_eq!(pixd.width(), pixs.width() / 16);
}

fn rank_reg_scale_gray_min_max() {
    let pixs = load_test_image("test8.jpg").unwrap();
    let pixd = scale_gray_min_max(&pixs, 2, 2, MinMaxOp::Min).unwrap();
    assert_eq!(pixd.width(), pixs.width() / 2);
    assert_eq!(pixd.height(), pixs.height() / 2);
}
```

### コミット構成

1. RED: 3 テストの本体を記述（`#[ignore]` のまま）
2. GREEN: 3 関数を実装して `#[ignore]` 除去

---

## 検証方法

各 PR で以下を確認：

```bash
# 対象 PR のテストが通ること
cargo test --package <crate> <test_name>

# workspace 全体で回帰がないこと
cargo test --workspace

# lint/fmt チェック
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

golden ファイル生成（初回）:
```bash
REGTEST_MODE=generate cargo test --package <crate> <test_name>
```

---

## 重要ファイル一覧

| ファイル | 用途 |
|---|---|
| `crates/leptonica-region/tests/grayfill_reg.rs` | PR1 テスト |
| `crates/leptonica-core/src/pix/arith.rs` | PR1 使用: `add_constant_inplace` (行 434) |
| `crates/leptonica-region/src/seedfill.rs` | PR1 使用: `seedfill_gray_simple` (行 1840) |
| `crates/leptonica-core/tests/rasteropip_reg.rs` | PR2 テスト |
| `crates/leptonica-core/src/pix/border.rs` | PR2 使用: `remove_border` (行 156), `add_mirrored_border` (行 251) |
| `crates/leptonica-core/src/numa/operations.rs` | PR3/4/5 実装場所 |
| `crates/leptonica-core/tests/numa3_reg.rs` | PR3/5 テスト |
| `crates/leptonica-core/tests/extrema_reg.rs` | PR4 テスト |
| `crates/leptonica-filter/src/rank.rs` | PR6 実装場所 |
| `crates/leptonica-filter/tests/rank_reg.rs` | PR6 テスト |
| `reference/leptonica/src/numafunc2.c` | PR3/4/5 C版参照 (erode:162, transform:407, find_extrema:2491, threshold:2597) |
| `reference/leptonica/src/scale2.c` | PR6 C版参照 (ScaleGrayMinMax:997, Rank2:1245, Cascade:1183) |
