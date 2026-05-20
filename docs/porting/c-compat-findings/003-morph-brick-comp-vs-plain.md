# C互換性調査 #003: brick morph の plain vs composite semantics 不一致 (binmorph1/3 計 7 件)

Phase 2.5 第二弾 (PR #383, TIFF 1bpp 直接書き出し) で depth stop が解消した
あとに残った 7 件の `Mismatch` (`binmorph1.{09,10,11,12}.tif` と
`binmorph3.{14,15,16}.tif`) の原因を調査した結果、**Rust の
`dilate_brick` / `erode_brick` / `open_brick` / `close_brick` が
"composite decomposition"** を採用しているのに対し、verify 用 C プログラム
`scripts/verify_binmorph.c` が **plain separable な
`pixDilateBrick` 等** を呼んでいることが第一の差として確認できた。

> **更新 (2026-05-17)**: Option A (verify_\*.c を `pix*CompBrick` に置換) を
> 試した結果、**binmorph1/3 の 7 件はすべて Mismatch のまま残った**。Rust
> 側 `dilate_1d_composite` の実装は C の `pixDilateCompBrick` とも pixel-
> level で一致せず、**第三の挙動** をしている。詳細は末尾「Option A 実装
> 結果」を参照。

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

## Option A 実装結果 (2026-05-17 追記)

`scripts/verify_binmorph.c` を `pix*CompBrick` に置換 → `bash scripts/gen_c_manifest.sh`
で manifest 再生成、`cargo test --all-features` で Phase 2 レポート確認:

| キー                                    | 旧 hash (plain) | 新 hash (Comp) | 変化?    | Rust 側 hash | Mismatch? |
| --------------------------------------- | --------------- | -------------- | -------- | ------------ | --------- |
| `binmorph1_verify.00.tif` (dilate)      | 33b60c...       | 33b60c...      | **同じ** | fb2e16...    | **残る**  |
| `binmorph1_verify.01.tif` (erode)       | b7f005...       | b7f005...      | **同じ** | 171a1d...    | **残る**  |
| `binmorph1_verify.02.tif` (open)        | b7f005...       | b7f005...      | **同じ** | e99305...    | **残る**  |
| `binmorph1_verify.03.tif` (close)       | 4e0325...       | 9720f6...      | **変化** | e588e7...    | **残る**  |
| `binmorph3_verify.00.tif` (sep dilate)  | ea9553...       | 7285e6...      | **変化** | ceaf60...    | **残る**  |
| `binmorph3_verify.01.tif` (dir dilate)  | ea9553...       | 7285e6...      | **変化** | ceaf60...    | **残る**  |
| `binmorph3_verify.02.tif` (dilate 21,1) | 90603d...       | 90603d...      | **同じ** | d1fc79...    | **残る**  |

### 発見

1. **dilate / erode / open (21,15)** は plain も Comp も結果が同じ。21=3×7

   と 15=3×5 はともに因数分解の product が input size と一致するため、
   composite 化しても micro-shift が発生しない

2. **close (21,15)** は plain と Comp で結果が異なる (`pixCloseBrick` と

   `pixCloseCompBrick` の境界処理の差。後者は内部で別の padding を行う)

3. **binmorph3 sep/dir (11,7)** は plain と Comp で異なる (11 が prime で

   `selectComposableSels` が `3×4=12` に丸める)

4. **すべての Comp 後 hash が Rust 側 hash と一致しない** — Rust の

   `dilate_1d_composite` は C の `pixDilateCompBrick` の内部実装とも違う

### 仮説 (現時点)

Rust の `src/morph/binary.rs::dilate_1d_composite` は composite
decomposition を「実装している」が、その分解方法・SEL 構築・境界処理が C
版と異なる:

- factor 選択 (Rust の独自実装 vs C `selectComposableSels`) で異なる

  `(f1, f2)` を選ぶ可能性

- 1D Sel の origin (中心) 計算が違う
- intermediate result の境界処理が違う

### 次のアクション (新フェーズ Phase 2.5 第三弾-続)

`docs/porting/c-compat-findings/005-morph-1d-composite-impl-diff.md` (予定)
を作成し、`src/morph/binary.rs::dilate_1d_composite` を C
`pixDilateCompBrick` の内部実装 (`reference/leptonica/src/morph.c:1213-`)
と pixel-level で突き合わせる。

ただし、本 PR (Option A) で manifest_c.tsv を Comp 系に揃えたことは、
将来の調査の基盤として意義がある:

- Rust の semantics 探索の比較対象が plain ではなく Comp になり、Rust

  実装の意図 (composite 採用) と整合した

- 残った Mismatch の原因が「composite ベース実装の差」に絞り込まれた

## PR #398 結果 (2026-05-20)

`src/morph/binary.rs::dilate_brick / erode_brick / open_brick /
close_brick` を C `pixDilate/Erode/Open/CloseCompBrick`
(`reference/leptonica/src/morph.c`) 準拠に全面書き直し:

- `select_composable_sizes` を cost function (`4 * diff + rastcost`) +

  `ACCEPTABLE_COST = 5` ルールの C 準拠ロジックに置換 (prime size でも
  近似 composite を選ぶ。例: `(11) → (4, 3)`, `(13) → (4, 3)`)

- `select_composable_sels(size, horizontal)` ヘルパー新設 (C

  `selectComposableSels` 対応)

- `dilate_brick`: `add_border(32, 32, 32, 32)` → `dilate(selh1) →

  dilate(selh2) → dilate(selv1) → dilate(selv2)` → `remove_border(32)`
  (C `pixDilateCompBrick` 完全準拠)

- `erode_brick` / `close_brick` / `open_brick`: border なしで 4-step

  または 8-step の dilate/erode を順次適用 (C 完全準拠)

### 結果

| Test                             | 旧 hash (Rust) | 新 hash (Rust) | C hash    | 状態           |
| -------------------------------- | -------------- | -------------- | --------- | -------------- |
| `binmorph1.12 close(21, 15)`     | 4e0325...      | 9720f6...      | 9720f6... | ✅ **Ok 化**   |
| `binmorph3.14 dilate(11, 7) sep` | ea9553...      | 2a6524...      | 7285e6... | ⚠️ Mismatch 残 |
| `binmorph3.15 dilate(11, 7) dir` | ea9553...      | 2a6524...      | 7285e6... | ⚠️ Mismatch 残 |

完全分解できる `close(21, 15)` (= `(7, 3) × (5, 3)`) は **完全一致**。
prime 11 を含む `dilate(11, 7)` (= `(4, 3) × (7, 1)` で size 12) は
**依然不一致**。

### 残 root cause (要追加調査)

PR #398 で SEL pair (brick(4), comb(4, 3)) + border 32 + 4-step
構造を C と完全に揃えたが、`binmorph3.14/15` (dilate 11x7) は不一致
のまま。原因は Rust 基本 `dilate(pix, sel)` (= `dilate_rasterop` の
word/bit shift OR ループ) と C `pixDilate` (= `pixRasterop` with
`PIX_SRC | PIX_DST`) の **subtle な内部差** にあると推定。

### Next step (別 PR)

1. C `pixAddBorder(pix, 32, 0)` の出力 hash と Rust `add_border(pix,

   32, 32, 32, 32)` の出力 hash が一致するか確認 (border 構築 step
   の検証)

2. 各 step (`selh1`, `selh2`, `selv1`, `selv2`) 後の中間 hash を C と

   Rust で順次比較し、乖離が発生する step を特定

3. 該当 step (例: `shift_or_row` の MSB-first bit 順、または cx/cy

   origin 解釈) を C と pixel-level で一致させる

## 関連

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](001-jpeg-codec-diffs.md)): JPEG codec 差
- Phase 2.5 第二弾 ([002-tiff-1bpp-write-limit.md](002-tiff-1bpp-write-limit.md)): TIFF 1bpp write
- 上流 leptonica の brick API 一覧: `reference/leptonica/src/morph.c:51-58`
- 解消した発見: [004 HMT impl diff](004-hmt-impl-diff.md) (PR #397 で全 7 件解消)
