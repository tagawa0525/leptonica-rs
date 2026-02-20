# Debug版テスト速度改善

Status: IMPLEMENTED

## Context

`cargo test --workspace` が debug版(259秒) vs release版(14秒) で約18.5倍遅い。
上位3テスト（bilateral×2 + binmorph5）だけで182秒、全体の70%を占める。

2つの対策を実施する:
- **A**: Cargo profile設定で全体の最適化レベルを引き上げ（bilateral含む全テストに効果）
- **B**: DWA実装をワードレベルに最適化（binmorph系のアルゴリズム的改善）

## 計測結果（参考）

| テスト | debug (秒) | release (秒) | 倍率 |
|--------|-----------|-------------|------|
| binmorph5_reg | 81.77 | 13.51 | 6x |
| bilateral1_reg | 64.97 | 2.67 | 24x |
| bilateral2_reg | 35.15 | 1.44 | 24x |
| binmorph4_reg | 8.50 | 1.21 | 7x |
| skew_reg | 7.41 | 0.62 | 12x |

## 実装計画

### Step 1: Cargo profile設定（案A）

**ファイル**: `Cargo.toml`

ワークスペースのルート `Cargo.toml` に以下を追加:

```toml
[profile.dev]
opt-level = 1
```

`opt-level = 1` は基本的な最適化（インライン化、ループ最適化）を有効にする。
コンパイル時間への影響は軽微で、デバッグ情報は保持される。

bilateral系テスト（浮動小数点4重ループ）に最も効果が大きい。

### Step 2: DWA水平演算のワードシフト化（案B）

**ファイル**: `crates/leptonica-morph/src/dwa.rs`

#### 現状の問題

`dilate_horizontal_dwa` / `erode_horizontal_dwa` がピクセル単位でビットを走査している。
O(h × w × hsize) の計算量。

同ファイルに `dilate_horizontal_shift` / `erode_horizontal_shift` が既に存在するが、
hsize <= 3 に限定されており `#[allow(dead_code)]` で未使用。

#### 方針

`dilate_horizontal_shift` / `erode_horizontal_shift` を任意の hsize に一般化し、
`dilate_horizontal_dwa` / `erode_horizontal_dwa` を置き換える。

#### アルゴリズム

水平dilation (hsize) の場合:
1. origin = hsize / 2, shifts = [-(hsize/2), ..., hsize - 1 - hsize/2]
2. 各行を word 単位で処理:
   - accumulator を 0 で初期化（dilate: OR） / 全1で初期化（erode: AND）
   - 各シフト量 d について:
     - d > 0: `(word << d) | (next_word >> (32 - d))` を accumulator に OR
     - d < 0: `(word >> |d|) | (prev_word << (32 - |d|))` を accumulator に OR
     - d == 0: `word` をそのまま OR
3. accumulator を出力ワードに書き込み

計算量: O(h × wpl × hsize) — ビット単位の O(h × w × hsize) に対し 32倍高速。

**注意**: シフト量が 32 以上の場合、隣接ワードだけでなくさらに先のワードも参照する必要がある。
シフト量 d のとき、参照先は `word_idx + d/32` と `word_idx + d/32 + 1` のワード。

```rust
fn shift_row_or(row: &[u32], wpl: usize, shift: i32) -> Vec<u32> {
    // shift > 0: 左シフト（ビット番号が大きい方向）
    // shift < 0: 右シフト（ビット番号が小さい方向）
    let abs_shift = shift.unsigned_abs() as usize;
    let word_offset = abs_shift / 32;
    let bit_shift = (abs_shift % 32) as u32;

    let mut result = vec![0u32; wpl];
    for i in 0..wpl {
        if shift > 0 {
            // 左シフト: src_idx = i + word_offset
            let src_idx = i + word_offset;
            let hi = if src_idx < wpl { row[src_idx] } else { 0 };
            let lo = if src_idx + 1 < wpl { row[src_idx + 1] } else { 0 };
            result[i] = if bit_shift == 0 { hi } else { (hi << bit_shift) | (lo >> (32 - bit_shift)) };
        } else {
            // 右シフト: src_idx = i - word_offset
            let hi = if i >= word_offset + 1 { row[i - word_offset - 1] } else { 0 };
            let lo = if i >= word_offset { row[i - word_offset] } else { 0 };
            result[i] = if bit_shift == 0 { lo } else { (lo >> bit_shift) | (hi << (32 - bit_shift)) };
        }
    }
    result
}
```

各行の処理:
```rust
let mut acc = vec![0u32; wpl];  // dilate: OR accumulator
for d in left..=right {
    let shifted = shift_row_or(src_row, wpl, d);
    for i in 0..wpl { acc[i] |= shifted[i]; }
}
// acc を dst_row にコピー
```

erosion は OR を AND に、初期値を !0 に変えるだけ。

#### 削除対象

- `dilate_horizontal_dwa` (現行ピクセル単位版) → 新しいワードシフト版に置換
- `erode_horizontal_dwa` (現行ピクセル単位版) → 新しいワードシフト版に置換
- `dilate_horizontal_shift` (hsize<=3限定版) → 一般化版に統合、削除
- `erode_horizontal_shift` (hsize<=3限定版) → 一般化版に統合、削除

#### 保持する関数（変更なし）

- `dilate_vertical_dwa` — 垂直方向はワードレベル最適化が困難（ビット列が非連続）。
  現行のピクセル単位実装を維持。プロファイル設定で十分高速化される。
- `erode_vertical_dwa` — 同上
- `dilate_brick_dwa`, `erode_brick_dwa`, `open_brick_dwa`, `close_brick_dwa` — 呼び出し側は変更不要

### Step 3: 境界処理の確認

現行コードの境界処理（画像端で範囲外を 0 として扱う asymmetric boundary condition）を
ワードシフト版でも維持する。`shift_row_or` で範囲外ワードを 0 とすることで実現。

## 修正対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `Cargo.toml` | `[profile.dev]` に `opt-level = 1` を追加 |
| `crates/leptonica-morph/src/dwa.rs` | 水平dilation/erosionをワードシフト版に置換 |

## TDDサイクル

既存テスト（binmorph5_reg, binmorph4_reg, dwamorph1_reg, dwamorph2_reg + ユニットテスト）が
DWA実装の正確性を検証する。GREEN → REFACTOR の流れ:

1. **GREEN**: `Cargo.toml` に profile設定を追加 → コミット
2. **GREEN**: `dwa.rs` の水平関数をワードシフト版に置換 → 既存テスト全パス → コミット
3. **REFACTOR**: 不要コードの削除、コメント更新 → コミット

## 検証方法

```bash
# 全テスト正常性確認
cargo test --workspace

# debug版の所要時間計測（259秒が目標: ~30-60秒）
time cargo test --workspace

# 個別テスト（release版との比が3-5倍以内なら成功）
time cargo test --package leptonica-morph --test binmorph5_reg
time cargo test --package leptonica-filter --test bilateral1_reg

# release版の退行確認
time cargo test --workspace --release

# クリッピーチェック
cargo clippy --workspace
```
