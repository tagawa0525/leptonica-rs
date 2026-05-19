# C互換性調査 #005: TIFF 1bpp 読み込みの PhotometricInterpretation invert 条件が C 版と逆

Phase 2.5 第四弾 (PR #387, fhmtauto HMT 調査) で「Identity 1x1 brick が
100% diff」の真因を追跡した結果、`src/io/tiff.rs` の TIFF 読み込みで
**1bpp の bit 反転条件が C 版 leptonica と逆になっている重大 bug** を
発見した。修正は単純 (数行) だが、影響が広範 (24+ Rust テストが fail) で
**write 側との規約整合性** も含めた見直しが必要なため、本書で根本原因を
記録し、修正は別 PR で慎重に進める。

修正方向はユーザー指示「bit反転はCから受け継いでいるleptonicaの仕様
なので適切に修正するべきでしょう」に従い、C 版 `src/tiffio.c` の挙動に
揃える方向を採用する。

## 真因の特定経緯

PR #387 の 004 finding で「Identity 1x1 brick が 100% diff になる」と
記録した時点では `Sel::create_brick(1, 1)` / `hit_offsets` /
`shift_and_row` のどれかに bug があると推測していた。

実機で `examples/hmt_roundtrip_debug.rs` (一時) を走らせると:

```text
in-mem output    count_pixels(FG)=425188 depth=Bit1
rust regout file count_pixels(FG)=425188 depth=Bit1
c verify file    count_pixels(FG)=98612  depth=Bit1
rust-file vs c-file pixel diff = 523800
```

Rust 側の `hit_miss_transform(pix, brick(1,1))` は **in-memory で正しく
identity を返している** (FG count が入力と一致)。差分は C 出力ファイル
(98612 FG) と Rust 出力ファイル (425188 FG) の **どちらかの読み込みが
反転している** ことを示唆。

C 側で `verify_fhmtauto.c` に self-check を追加すると:

```text
C self-check: pixHMT(1x1 hit) == pixs? YES (FG in=98612, FG out=98612)
```

C 側 `pixHMT` は正しく identity を返している。すなわち `pixRead("feyn-fract.tif")`
が C で FG=98612、Rust で FG=425188 という **同じ TIFF ファイルの読み込み
結果が違う** のが真因。523800 - 98612 = 425188 で、両者は **正確に反転**。

## 根本原因

`feyn-fract.tif` は **1bpp + CCITT G4 + BlackIsZero**。

### C 版 (`reference/leptonica/src/tiffio.c:760-762`)

```c
if ((d == 1 && photometry == PHOTOMETRIC_MINISBLACK) ||
    (d == 8 && photometry == PHOTOMETRIC_MINISWHITE))
    pixInvert(pix, pix);
```

- 1bpp + BlackIsZero (= MINISBLACK) → invert
- 8bpp + WhiteIsZero (= MINISWHITE) → invert

これは leptonica 慣習「in-memory で foreground = 1」を保つための変換。

### Rust 側 (`src/io/tiff.rs:339-342`、修正前)

```rust
let invert = matches!(
    (pix_depth, photometric),
    (PixelDepth::Bit1, PhotometricInterpretation::WhiteIsZero)
);
```

**完全に逆** (Bit1 + WhiteIsZero で invert)。これにより:

- BlackIsZero TIFF (`feyn-fract.tif` 等) → invert なし → 内部表現が「逆

  leptonica 慣習」(FG=0, BG=1)

- WhiteIsZero TIFF (Rust 自身が PR #383 以降書き出すもの) → invert あり

  → 同じく内部表現が「逆 leptonica 慣習」

つまり **Rust の全 1bpp TIFF 読み込みが逆規約で in-memory に置かれている**。
これは Rust 内部での compare_pix / count_pixels 等の演算では `1` を FG と
扱っていない可能性があり、伝統的な leptonica 規約と分岐している。

write 側 (`src/io/tiff.rs::write_1bpp_pix_to_tiff` in PR #383) は、内部
表現の `val != 0` を WhiteIsZero TIFF の bit=1 として書き出している。
read 側が逆規約のため、round-trip は内部で「逆 → 反転 → 逆」と相殺されて
偶然合致する形だった。

## 単純修正 (1 行) と影響範囲

```rust
let invert = matches!(
    (pix_depth, photometric),
    (PixelDepth::Bit1, PhotometricInterpretation::BlackIsZero)
        | (PixelDepth::Bit8, PhotometricInterpretation::WhiteIsZero)
);
```

`cargo test --all-features --no-fail-fast` 実行で **24+ 件の Rust テスト
が fail**:

| Test binary                          | Fail | 内訳例                                                                              |
| ------------------------------------ | ---: | ----------------------------------------------------------------------------------- |
| color                                |    2 | `blend4_reg_add_alpha`, `colorfill_reg_expand_replicate`                            |
| filter                               |    1 | `convolve_windowed_stats_reg`                                                       |
| io                                   |    7 | **`tiff_1bpp_reg` 3 件 (本 PR で追加したテスト)**, `pdfio1_reg` 3 件, `gifio_reg`   |
| morph                                |    6 | `fhmtauto_reg` 2 件, `binmorph1_reg`, `binmorph3_reg`, `ccthin1_reg`, `ccthin2_reg` |
| recog                                |   8+ | `baseline_*`, `flipdetect_*`, `newspaper_reg`                                       |
| (続きの binary は実行されたか未確認) |      |                                                                                     |

とくに **`tiff_1bpp_reg::round_trip_preserves_*` (3 件、本 PR で追加した
テスト)** が fail する。私の `write_1bpp_pix_to_tiff` は WhiteIsZero TIFF
を書き出すが、修正後の read で「pixel (0,0) = 1 が 0 として返る」状態
(round-trip で値が反転)。これは write 側の bit semantics と read 側の
新規約が**整合していない**ことを示す。

## 修正方針 (別 PR で慎重に)

1. **read 側の invert 条件を C 版に揃える** (本書「単純修正」)
2. **write 側 (`write_1bpp_pix_to_tiff`) の PhotometricInterpretation 選択

   を見直す**: 修正後の read 規約 (BlackIsZero で invert / WhiteIsZero で
   そのまま) に合わせて、write 側も `BlackIsZero` で書くか、bit 反転して
   `WhiteIsZero` で書くか、C 慣習に揃える

3. **影響を受けるすべての Rust テスト** (compare_values / write_pix_and_check

   経由含む) を analysis:

   - `tests/golden_manifest.tsv` の影響 entry (おそらく 1bpp TIFF 出力

     entry 数十件) を `REGTEST_MODE=generate` で再生成

   - assertion 値ベース (compare_values) のテストは期待値を更新するか、

     内部処理がそもそも leptonica 慣習に依存していたか確認

4. **Phase 2 レポートで Mismatch 改善** を確認 (binmorph / fhmtauto /

   ccthin の系統が一気に Ok に転換する見込み)

5. CI で `cargo test --all-features` が pass することを確認

## 推定される改善効果 (修正完了後)

- 修正後の Phase 2 レポート Mismatch 数の見込み:
  - 21 件 (JPEG codec 差、修正対象外) は変化なし
  - 7 件 (binmorph1/3) は input feyn-fract.tif の解釈が正しくなることで

    in-memory dilate_brick の出力も C 版と一致する可能性 → Ok 化見込み

  - 8 件 (fhmtauto) は **本書で原因が解明済み** で、修正後 Ok 化見込み
- ただし、Rust manifest の更新が大量に発生するため、修正の各 step を

  明確に commit 分割して PR レビュー可能にする

## 関連

- 001 ([JPEG codec 差](001-jpeg-codec-diffs.md))
- 002 ([TIFF 1bpp write limit](002-tiff-1bpp-write-limit.md))
- 003 ([brick morph plain vs composite](003-morph-brick-comp-vs-plain.md))
- 004 ([HMT 実装差](004-hmt-impl-diff.md)) — 本書で真因が解明されたため、

  「HMT 自体には bug がない」可能性が高い (TIFF 読み込みが原因)

- C 版 `pixReadFromTiffStream`: `reference/leptonica/src/tiffio.c:760-762`
- Rust 読み込み: `src/io/tiff.rs:339-342` (修正前)、

  `convert_u8_to_pix(..., invert)`: 同 `:390-`

- Rust 書き出し: `src/io/tiff.rs::write_1bpp_pix_to_tiff` (PR #383 で追加)
