# C互換性調査 #002: 1bpp TIFF 出力が 8bpp に拡張される (binmorph / fhmtauto 計 15 件)

Phase 1.5 (PR #381) の取り込みで新たに観測された 15 件の `Mismatch` を調査した
結果、**Rust 側の `tiff` crate 経由の TIFF 書き出しが 1bpp Pix を 8bpp に拡張
してしまう**ことが原因と判明した。`tiff` crate 0.11.3 に 1bpp 書き出し
サポートがないため、修正は **別 PR (中〜大規模)** として独立計画する。

## 観測

`tests/c_compat_report.morph.txt` から該当 15 件:

| Rust 出力                                                                      | C 出力                               | bps 比較      |
| ------------------------------------------------------------------------------ | ------------------------------------ | ------------- |
| `binmorph1.{09,10,11,12}.tif` (4 件)                                           | `binmorph1_verify.{00,01,02,03}.tif` | C=1 / **R=8** |
| `binmorph3.{14,15,16}.tif` (3 件)                                              | `binmorph3_verify.{00,01,02}.tif`    | C=1 / **R=8** |
| `fhmtauto_hmt.{02,04,06,08,10,12,14}.tif` (7 件) + `fhmtauto_id.01.tif` (1 件) | `fhmtauto_verify.{00..07}.tif`       | C=1 / **R=8** |

`compare_golden` で 15 件すべてが **`DEPTH C=1 R=8`** で停止
(dimension/pixel 比較に進めない)。

`file(1)` で実物を確認:

```text
tests/regout/binmorph1.09.tif:           bps=8  compression=none      PhotometricInterpretation=BlackIsZero
/tmp/lept/regout/binmorph1_verify.00.tif: bps=1  compression=bi-level group 4  PhotometricInterpretation=WhiteIsZero
```

## 根本原因

`src/io/tiff.rs::write_pix_to_encoder` の `PixelDepth::Bit1` ブランチ
(844 行〜):

```rust
PixelDepth::Bit1 => {
    // 1-bit binary - convert to 8-bit for simplicity
    let mut data = vec![0u8; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let val = pix.get_pixel(x, y).unwrap_or(0);
            data[(y * width + x) as usize] = if val != 0 { 255 } else { 0 };
        }
    }
    encoder
        .write_image::<Gray8>(width, height, &data)
        .map_err(...)?;
}
```

明示的に「convert to 8-bit for simplicity」と書いてあり、`Gray8` (= 8bpp) で
書いている。

入力 Pix の `depth()` は `Bit1` のまま (`src/morph/morphapp.rs` の `dilate` /
`erode` / `hmt` 系はすべて `Pix::new(w, h, PixelDepth::Bit1)` で出力を作る)
が、TIFF への serialize 段階で 8bpp に拡張される。

## `tiff` crate 0.11.3 の制約

依存調査 (`scripts/verify` でなく Rust crate 直接):

- `tiff::encoder::colortype` に `Gray1` 相当の型は **存在しない**

  (`Gray8` / `Gray16` / `Gray32` / `Gray64` のみ)

- CCITT G3/G4 圧縮タグは decoder 側で定義されているが、encoder では

  `Uncompressed` にフォールバックされる

- crate 上流 (image-rs) のロードマップにも 1bpp encoder サポートの予定はない

## 影響範囲

`find tests -name '*.tif' -path '*/regout/*' | xargs file | awk -F'bps=' '{print $2}' | awk '{print $1}'`
集計:

|         bps | 件数 |
| ----------: | ---: |
|           8 |   89 |
| (16 bit 系) |   10 |

**Rust 側の TIFF 出力すべてが現状 `bps=8` 以上で書かれている** (1bpp で書かれ
ているものは皆無)。修正すると `tests/golden_manifest.tsv` の数十件のハッシュが
変わり、Rust 自身の回帰テストが一斉に fail する。

## 解決オプション (修正は別 PR で)

優先順位順:

### A. `tiff` crate を低レベル API で扱って 1bpp uncompressed TIFF を書く

- `tiff::encoder::DirectoryEncoder::write_tag()` で `BitsPerSample=1`、

  `PhotometricInterpretation=WhiteIsZero` 等を手動指定

- pixel buffer は 1bpp packed bits (`(width + 7) / 8` bytes per row)
- 圧縮は無圧縮 (C の G4 と bit 一致しないが、`pixel_content_hash` は decode

  後ピクセルで計算されるので **G4 圧縮の有無は影響しない**)

- **見込み**: 中規模 (`src/io/tiff.rs` の Bit1 ブランチを書き換え、unit test

  追加、Rust manifest 数十件再生成)

### B. `fax` crate (CCITT G4 圧縮実装) を導入して C と bit 一致

- 圧縮されたバイトまで含めて C と一致させる
- ただし `pixel_content_hash` は decoded pixel しか見ないので、A で必要十分
- 新規依存を増やす意味は薄い

### C. `pixel_content_hash` を depth 非依存に正規化

- 1bpp ({0,1}) と 8bpp ({0,255}) を同じ hash にする
- 既存の Rust manifest 1871 件すべてが影響を受け、全件再生成
- 「同じ画像表現でも異なる Pix を区別する」という hash semantics を弱める
- **採用しない**: 検査の厳密性が下がる

### D. 手書き TIFF writer

- 1bpp + 任意圧縮を完全制御
- 実装規模が大きく、メンテナンスコストも高い
- A が成立しない場合の最終手段

## 推奨

**A** を別 PR で実装する。手順:

1. `src/io/tiff.rs::write_pix_to_encoder` の `Bit1` ブランチを 1bpp 直接

   書き出しに変更 (`DirectoryEncoder` の low-level API)

2. `tests/io/` に 1bpp ↔ 1bpp round-trip 単体テストを追加 (write → read で

   pixel が保たれることを確認)

3. `REGTEST_MODE=generate cargo test` で `tests/golden_manifest.tsv` を再生成
4. 同じ生成サイクルで `bash scripts/gen_c_manifest.sh` を実行

   (verify_*.c 経由の C 1bpp と比較できる状態に)

5. Phase 2 レポートで 15 件の `Mismatch` が `Ok` に転換することを確認

## 関連

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](001-jpeg-codec-diffs.md)): JPEG codec 差
- Phase 1.5 (PR #381): verify_*.c 取り込みでこの 15 件の Mismatch が顕在化した
