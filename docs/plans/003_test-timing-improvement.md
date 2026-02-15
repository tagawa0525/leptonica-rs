# 003: テスト実行時間の改善

Status: IN_PROGRESS

## 背景

回帰テスト (`graymorph1_reg`, `colormorph_reg`) が遅く、nextest の slow-timeout に引っかかる。
原因はグレースケール形態学演算の O(hsize×vsize) なナイーブ実装。
C版 leptonica は van Herk/Gil-Werman (vHGW) アルゴリズムで O(1)/pixel（1ピクセルあたり最大3回の比較）を実現している。

## 対応C版ファイル

※ `reference/leptonica` は git submodule のため、`git submodule update --init` 済みの環境を前提とする。

- `reference/leptonica/src/graymorph.c` — vHGW アルゴリズム、3x3 fast path
- `reference/leptonica/src/pageseg.c` — 形態学演算を使うページセグメンテーション

## PR構成（直列実行、各PRマージ後に次へ）

### PR 1: nextest slow-timeout 設定

- `.config/nextest.toml` に `slow-timeout = "10s"` を設定
- 根本対策完了までの一時的な緩和策

### PR 2: pageseg 形態学演算の leptonica-morph 委譲

**対象ファイル**: `crates/leptonica-recog/src/pageseg.rs`

ナイーブな pixel-by-pixel 実装を `leptonica_morph` の brick API に委譲:

| pageseg 関数 | 委譲先 |
|---|---|
| `morphological_erode` (L490) | `leptonica_morph::erode_brick` |
| `morphological_dilate` (L526) | `leptonica_morph::dilate_brick` |
| `morphological_open` (L478) | `leptonica_morph::open_brick` |
| `morphological_close` (L484) | `leptonica_morph::close_brick` |
| `seed_fill` (L560) | ワードレベル AND 演算に最適化 |
| `subtract_images` (L604) | ワードレベル AND-NOT 演算に最適化 |

**注意**: `seed_fill` に depth/wpl 検証、`subtract_images` に depth 検証を追加すること。

### PR 3: vHGW アルゴリズム実装

**対象ファイル**: `crates/leptonica-morph/src/grayscale.rs`

C版 `dilateGrayLow` (L1161), `erodeGrayLow` (L1268) に対応する vHGW 実装:

- `dilate_gray` (L39): ナイーブ → vHGW (水平/垂直 separable)
- `erode_gray` (L99): ナイーブ → vHGW (水平/垂直 separable)
- ナイーブ版は `#[cfg(test)]` でテスト用に保持

**vHGW アルゴリズム概要**:
- forward/backward 部分最大/最小配列をスライディングウィンドウで計算
- ウィンドウサイズ: 2*size+1
- ピクセルあたり最大3回の比較で O(1)/pixel
- ボーダー処理: dilation は 0 padding、erosion は 255 padding

**ヘルパー関数**:
- `add_border` → `Pix::add_border_general` (core) に委譲
- `remove_border` → `Pix::remove_border_general` (core) に委譲
- `set_border` は独自実装を維持 (core の `PixMut::set_border_val` とはセマンティクスが異なる)

### PR 4: 3x3 fast path 実装

**対象ファイル**: `crates/leptonica-morph/src/grayscale.rs`

C版 `pixDilateGray3h/3v`, `pixErodeGray3h/3v` に対応:

- `dilate_gray_3h`: 水平 3x1 dilation、8ピクセル unroll
- `dilate_gray_3v`: 垂直 1x3 dilation、8行 unroll
- `erode_gray_3h`: 水平 3x1 erosion、8ピクセル unroll
- `erode_gray_3v`: 垂直 1x3 erosion、8行 unroll

**注意**: unroll ループの末尾に fallback ループが必要（最後の 8-9 ピクセル/行のスカラー処理）。

`dilate_gray` / `erode_gray` の冒頭で hsize <= 3 && vsize <= 3 の場合に fast path にディスパッチ。

### PR 5 (将来): DWA ワードレベルシフト

binary morphology の DWA 演算をワードレベルのビット操作で高速化。

### PR 6 (将来): bilateral PBC 近似

bilateral フィルタの高速近似実装。

## テスト戦略

各PRで以下の回帰テストが既存結果と一致することを確認:
- `cargo nextest run -p leptonica-morph --test graymorph1_reg`
- `cargo nextest run -p leptonica-morph --test colormorph_reg`
- `cargo nextest run -p leptonica-recog --test pageseg_reg`

vHGW テストでは naive 実装との同値性テストを追加:
- 様々なサイズ (3x3, 7x5, 11x1, 1x9 等) で結果が一致
- dilation / erosion 両方をカバー
