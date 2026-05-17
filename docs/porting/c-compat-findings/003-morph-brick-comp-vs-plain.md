# C互換性調査 #003: brick morph の plain vs composite semantics 不一致 (binmorph1/3 計 7 件)

Phase 2.5 第二弾 (PR #383, TIFF 1bpp 直接書き出し) で depth stop が解消した
あとに残った 7 件の `Mismatch` (`binmorph1.{09,10,11,12}.tif` と
`binmorph3.{14,15,16}.tif`) の原因を調査した結果、**Rust の
`dilate_brick` / `erode_brick` / `open_brick` / `close_brick` が
"composite decomposition"** を採用しているのに対し、verify 用 C プログラム
`scripts/verify_binmorph.c` が **plain separable な
`pixDilateBrick` 等** を呼んでいることが原因と判明した。

`fhmtauto` 系 8 件はまた別の論点 (Sel origin / 境界処理) で、本書では概略の
切り分けまでに留め、詳細は後続の調査ノート (`004-hmt-impl-diff.md` を予定) に
回す。

## 観測

Phase 2 レポートの該当 7 件:

```text
[Mismatch] binmorph1 :: binmorph1.09.tif :: rust=fb2e160b859512ae, c[binmorph1_verify.00.tif]=33b60cb6c754a60e
[Mismatch] binmorph1 :: binmorph1.10.tif :: rust=171a1de4fa7c74bf, c[binmorph1_verify.01.tif]=b7f005730578dbee
[Mismatch] binmorph1 :: binmorph1.11.tif :: rust=e99305c26e85c7af, c[binmorph1_verify.02.tif]=b7f005730578dbee
[Mismatch] binmorph1 :: binmorph1.12.tif :: rust=e588e7c557e77cce, c[binmorph1_verify.03.tif]=4e03255827c5495e
[Mismatch] binmorph3 :: binmorph3.14.tif :: rust=ceaf6053504e87ff, c[binmorph3_verify.00.tif]=ea95536789d31c6f
[Mismatch] binmorph3 :: binmorph3.15.tif :: rust=ceaf6053504e87ff, c[binmorph3_verify.01.tif]=ea95536789d31c6f
[Mismatch] binmorph3 :: binmorph3.16.tif :: rust=d1fc7996d523973e, c[binmorph3_verify.02.tif]=90603db276f02f1f
```

`compare_golden` の pixel-level 数値:

| Test                 |    Diff/Total | MaxDiff |  %Diff |
| -------------------- | ------------: | ------: | -----: |
| `dilate_brick 21x15` | 129670/523800 |       1 | 24.76% |
| `erode_brick 21x15`  | 112527/523800 |       1 | 21.48% |
| `open_brick 21x15`   | 300049/523800 |       1 | 57.28% |
| `close_brick 21x15`  | 284500/523800 |       1 | 54.31% |

`max=1` は 1bpp の値域上限なので、これだけでは codec/アルゴリズムの区別は
できないが、**差分ピクセル数が大きい** (20-57%) ため codec 差では説明不可能。

## 根本原因

C 版 leptonica は brick morph で 2 系統の API を提供する:

- `pixDilateBrick(pixd, pixs, hsize, vsize)` — **plain separable**: 単純な

  brick SEL を horizontal と vertical で 2 回適用 (`reference/leptonica/
  src/morph.c:672`)

- `pixDilateCompBrick(pixd, pixs, hsize, vsize)` — **composite separable**:

  各次元を 2 つの factor (`selectComposableSels` で `f1 × f2 ≒ hsize`) に
  分解し、合成 SEL の積で実装 (`reference/leptonica/src/morph.c:1213`)

両者は **`hsize` が分解可能な場合は同じ結果** になるが、composite は SEL の
origin / 端の扱いで微小な差が出る場合がある。とくに分解時に **「product of
factors が `hsize` と一致しない」** ケース (上流 doc:
`reference/leptonica/src/morph.c:1198-1210` の Notes 8) では結果が積分的に
ズレる。

Rust 側 `src/morph/binary.rs::dilate_brick` (338 行) は実装内で
`dilate_1d_composite` を呼んでおり、**常に composite decomposition** を採用:

```rust
pub fn dilate_brick(pix: &Pix, width: u32, height: u32) -> MorphResult<Pix> {
    if width == 1 && height == 1 {
        return Ok(pix.clone());
    }
    // Separable: horizontal then vertical, each using composite decomposition.
    let tmp: Pix = dilate_1d_composite(pix, width, true)?.into();
    let mut result = dilate_1d_composite(&tmp, height, false)?;
    clear_unused_bits(result.data_mut(), pix.width(), pix.wpl() as usize);
    Ok(result.into())
}
```

doc コメントに「Uses separable + composite decomposition for optimal
performance」と書かれており、性能のために composite を選んでいるのは意図的。

一方 `scripts/verify_binmorph.c` (48-51 行) は plain 系の C API を呼ぶ:

```c
PIX *dil = pixDilateBrick(NULL, pixs, 21, 15);   /* plain separable */
PIX *ero = pixErodeBrick(NULL, pixs, 21, 15);
PIX *opn = pixOpenBrick(NULL, pixs, 21, 15);
PIX *cls = pixCloseBrick(NULL, pixs, 21, 15);
```

**Rust の `dilate_brick` は C の `pixDilateCompBrick` 相当、`verify` は C の
`pixDilateBrick` (= plain) を期待している** → ハッシュが合わない。

## 解決オプション (修正は別 PR で)

### A. `scripts/verify_*.c` を Comp 系に修正 (推奨、影響範囲最小)

- `pixDilateBrick` → `pixDilateCompBrick` 等に置換
- `tests/golden_manifest_c.tsv` の `binmorph1_verify.*` / `binmorph3_verify.*`

  7 件 hash を再生成

- Rust 実装は変えない (Rust 側 manifest 不変)
- 「Rust `dilate_brick` の semantics は `pixDilateCompBrick` 相当」と doc 明記

### B. Rust 実装を plain separable に修正

- `src/morph/binary.rs::dilate_brick` 等を `dilate(pix, brick_sel(W, H))` の

  単純呼び出しに戻す

- 既存の composite 実装を `dilate_comp_brick` のような別 API として残す
- 性能トレードオフ: 大サイズの brick で composite の方が高速 (上流の C 慣習も

  `pixOpenCompBrick` 等を別途用意)

- Rust manifest が広範に変わる (`binmorph1` / `binmorph3` の `tests/golden_

  manifest.tsv` 7 件、加えて呼び出し元のテストも要再生成)

- インターフェース不変だが意味論が変わる **breaking change**

### C. Rust に両方の API を提供 (C 慣習と揃える)

- `dilate_brick` (plain) と `dilate_comp_brick` (composite) を別関数として

  提供

- 既存テストの呼び出し先を必要に応じて切り替え
- 最も clean だが工数は B + α
- C 版と naming / semantics が完全一致して長期的には筋がよい

## 推奨

短期で `Mismatch` を解消するには **Option A**:

1. `scripts/verify_binmorph.c` の 4 関数を `pix*CompBrick` に置換
2. `bash scripts/gen_c_manifest.sh` を再実行 (manifest_c.tsv に 7 件の

   新 hash)

3. Phase 2 レポートで `binmorph1` / `binmorph3` の Mismatch が `Ok` に

   転換することを確認

長期的には **Option C** が望ましく、別フェーズ (Phase 3 以降) で
`docs/plans/` に新計画として整理する。

## fhmtauto 8 件について

`fhmtauto_hmt.{02,04,06,08,10,12,14}.tif` + `fhmtauto_id.01.tif` の 8 件は
別系統 (HMT、Hit-Miss Transform)。`src/morph/binary.rs::hit_miss_transform`
と C `pixHMT` の比較で:

- `verify_fhmtauto.c` は `pixHMT(NULL, pixs, sel)` を呼ぶ (plain HMT、

  composite なし)

- Rust `hit_miss_transform` も Sel 直接 (composite なし)
- なので brick とは別の根本原因がある

仮説:

- Sel の origin 解釈の差
- 出力初期化方針の差 (Rust は全 1 から AND、C は別?)
- 境界処理 (Rust は `clear_unused_bits` で src clone を変更、C は in-place で

  違う処理)

- HMT の hit / miss テーブルの初期値処理

詳細は本書のスコープを超えるため、後続の `004-hmt-impl-diff.md` に
まとめる予定。

## 関連

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](001-jpeg-codec-diffs.md)): JPEG codec 差
- Phase 2.5 第二弾 ([002-tiff-1bpp-write-limit.md](002-tiff-1bpp-write-limit.md)): TIFF 1bpp write
- 上流 leptonica の brick API 一覧: `reference/leptonica/src/morph.c:51-58`
