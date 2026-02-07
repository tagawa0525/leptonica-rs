# unsafe 削減プラン

## Context

leptonica-rs は C の Leptonica ライブラリから移植されたため、
899箇所の `unsafe` が残っている。
しかし調査の結果、**95%以上が不要な unsafe** であることが判明した。

Pix/PixMut の `get_pixel_unchecked` / `set_pixel_unchecked` は
`unsafe fn` として定義されているが、
内部実装は通常の Rust スライスインデックス（OOB ならパニック）を使用しており、
`get_unchecked` や生ポインタを一切含まない。
つまり UB（未定義動作）の可能性がゼロであり、
Rust の意味論的に `unsafe` マークは不正確。

この変更により、セマンティクスを一切変えずに
unsafe を 899 → 約3 に削減できる（99.7%減）。

## 変更対象の要約

| 変更内容 | 箇所数 | 種類 |
| --- | --- | --- |
| Pix/PixMut の unsafe fn → fn | 3 | シグネチャ変更 |
| FPix の unsafe fn → fn | 2 | 実装変更 |
| access.rs 内部の unsafe ブロック除去 | 3 | 呼び出し元 |
| 50ファイルの外部 unsafe ブロック除去 | ~891 | 呼び出し元 |

## ステップ 0: git-worktree でブランチ作成

`/git-worktree` スキルを使って独立した作業環境を作成し、
feature ブランチで作業する。
main ブランチには一切影響を与えない。

## ステップ 1: FPix の safe 化

**ファイル:** `crates/leptonica-core/src/fpix/mod.rs`
（280行目, 292行目）

- `get_pixel_unchecked`: `unsafe fn` → `fn`、
  `*self.data.get_unchecked(idx)` → `self.data[idx]`
- `set_pixel_unchecked`: `unsafe fn` → `fn`、
  `*self.data.get_unchecked_mut(idx) = value`
  → `self.data[idx] = value`
- `# Safety` ドキュメント → `# Panics`
- 呼び出し元はゼロのため影響なし

## ステップ 2: Pix/PixMut の access.rs 修正

**ファイル:** `crates/leptonica-core/src/pix/access.rs`

3つの関数定義から `unsafe` を除去（40行目, 62行目, 97行目）:

- `Pix::get_pixel_unchecked`
- `PixMut::get_pixel_unchecked`
- `PixMut::set_pixel_unchecked`

同ファイル内の呼び出し元（31行目, 52行目, 87行目）から
`unsafe { }` ブロック除去。
`# Safety` → `# Panics` にドキュメント変更。

## ステップ 3: 全呼び出し元から unsafe ブロック除去

edition = "2024" では `unused_unsafe` がエラーになるため、
全呼び出し元の更新が必須。
クレート別の対象ファイル一覧：

**leptonica-core** (5ファイル):

- `src/pix/arith.rs`, `src/pix/blend.rs`,
  `src/pix/compare.rs`, `src/pix/graphics.rs`,
  `src/pix/rop.rs`

**leptonica-io** (8ファイル):

- `src/bmp.rs`, `src/gif.rs`, `src/jpeg.rs`,
  `src/jp2k.rs`, `src/png.rs`, `src/pnm.rs`,
  `src/tiff.rs`, `src/webp.rs`

**leptonica-transform** (7ファイル):

- `src/affine.rs`, `src/bilinear.rs`,
  `src/projective.rs`, `src/rotate.rs`,
  `src/scale.rs`, `src/shear.rs`, `src/warper.rs`

**leptonica-filter** (5 src + 1 test):

- `src/adaptmap.rs`, `src/bilateral.rs`,
  `src/convolve.rs`, `src/edge.rs`, `src/rank.rs`
- `tests/rank_reg.rs`

**leptonica-morph** (5ファイル):

- `src/binary.rs`, `src/color.rs`, `src/dwa.rs`,
  `src/grayscale.rs`, `src/thin.rs`

**leptonica-color** (7 src + 5 test):

- `src/analysis.rs`, `src/colorfill.rs`,
  `src/coloring.rs`, `src/colorspace.rs`,
  `src/quantize.rs`, `src/segment.rs`,
  `src/threshold.rs`
- `tests/cmapquant_reg.rs`,
  `tests/colorcontent_reg.rs`,
  `tests/colorfill_reg.rs`,
  `tests/colorquant_reg.rs`,
  `tests/colorseg_reg.rs`

**leptonica-recog** (5 src + 1 test):

- `src/baseline.rs`, `src/dewarp/apply.rs`,
  `src/dewarp/textline.rs`, `src/pageseg.rs`,
  `src/skew.rs`
- `tests/pageseg_reg.rs`

### 置換パターン

単行（大部分）:

```text
unsafe { expr.get_pixel_unchecked(x, y) }
  →  expr.get_pixel_unchecked(x, y)

unsafe { expr.set_pixel_unchecked(x, y, v) };
  →  expr.set_pixel_unchecked(x, y, v);
```

複数行（rotate.rs 等に約28箇所）:

```text
unsafe {
    expr.set_pixel_unchecked(...);
    expr.set_pixel_unchecked(...);
}
  →
expr.set_pixel_unchecked(x, y, v1);
expr.set_pixel_unchecked(x, y, v2);
```

## スコープ外

`crates/leptonica-color/src/quantize.rs` の3箇所の
生ポインタ unsafe（`unsafe { &mut *node_ptr }`）は
octree 構造体の再帰的自己参照パターンであり、
arena allocator 等への構造的リファクタリングが必要。
本プランでは対象外とする。
変更後、これが唯一残存する unsafe（3箇所）となる。

## 結果の見込み

| 指標 | 変更前 | 変更後 |
| --- | --- | --- |
| unsafe fn 定義数 | 5 | 0 |
| unsafe ブロック総数 | ~894 | 3 |
| 影響ファイル数 | 51 | 1 |

## パフォーマンスへの影響

**ゼロ。**
Pix/PixMut は元々パニック型スライスインデックスを使用しており、
コンパイラ生成コードに差異なし。
FPix の unchecked メソッドは呼び出し元がゼロ。

## 検証

```bash
cargo build --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
# unsafe 残存確認
grep -rn "unsafe" --include="*.rs" crates/ \
  | grep -v "// " | wc -l
# 期待値: 3〜6行（quantize.rs の生ポインタのみ）
```
