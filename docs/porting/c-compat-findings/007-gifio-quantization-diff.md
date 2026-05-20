# C互換性調査 #007: `gifio` の 2 件 hash 不一致 (FILE_8BPP_3 / FILE_32BPP)

Phase 3 第三弾で `scripts/golden_map.tsv` の `gifio` Part 1 mapping
を完成させた結果、8 ファイル中 6 件は hash 一致で `Ok` だが
**FILE_8BPP_3** (Rust idx 11) と **FILE_32BPP** (Rust idx 15) が
`Mismatch` として残った。本 finding は観測結果と root cause 仮説を
記録する。

## 観測

```text
[Mismatch] gifio :: gifio.11.png :: rust=1fb83d3260ed8df7, c[gifio.05.gif]=281b063f4b473c57
[Mismatch] gifio :: gifio.15.png :: rust=e42ca00190f52b60, c[gifio.07.gif]=9b00cf8ffe512212
```

対応関係 (`tests/io/gifio_reg.rs` と `reference/leptonica/prog/
gifio_reg.c` の `test_gif()` を読み合わせて確定):

> **注**: 「Rust check」「C check」は **0-based の呼び出し順序** を
> 指す。実際の filename / manifest index は Rust 1-based / C 0-based
> で、`scripts/golden_map.tsv` の `c_index` / `rust_index` 欄は
> filename index を保持する。

| Test file (input)            | Rust check | C check | Rust file | C file | 状態            |
| ---------------------------- | ---------: | ------: | --------: | -----: | --------------- |
| FILE_1BPP (`feyn-fract.tif`) |          0 |       0 |        01 |     00 | ✅ Ok           |
| FILE_2BPP (`weasel2.png`)    |          1 |       1 |        03 |     01 | ✅ Ok           |
| FILE_4BPP (`weasel4.png`)    |          2 |       2 |        05 |     02 | ✅ Ok           |
| FILE_8BPP_1 (`map1.jpg`)     |          3 |       3 |        07 |     03 | ✅ Ok           |
| FILE_8BPP_2 (`weasel8.png`)  |          4 |       4 |        09 |     04 | ✅ Ok           |
| **FILE_8BPP_3** (`feyn.tif`) |          5 |       5 |        11 |     05 | ⚠️ **Mismatch** |
| FILE_16BPP (`marge.png`)     |          6 |       6 |        13 |     06 | ✅ Ok           |
| **FILE_32BPP** (`marge.jpg`) |          7 |       7 |        15 |     07 | ⚠️ **Mismatch** |

GIF のテスト構造は「lossless r/w ラウンドトリップ」 (元画像 → GIF
write → read で得た `pix1` の hash を確認) なので、入力画像と Rust の
GIF encoder/decoder + C の GIF encoder/decoder の挙動差が hash 差に
直結する。

## Root cause 仮説 (未確定、要追加調査)

| 仮説                                                     | 影響                                       | 切り分け方                                         |
| -------------------------------------------------------- | ------------------------------------------ | -------------------------------------------------- |
| (1) Rust `gif` crate と giflib の LZW 圧縮テーブル順序差 | round-trip 後の pixel 値も差うる可能性     | Rust write → C read で round-trip が成功するか確認 |
| (2) 32bpp → 8bpp 量子化アルゴリズム差                    | FILE_32BPP は GIF write 前に必ず量子化     | Rust `convert_to_8` / `color_quant` 系と C の差    |
| (3) JPEG decoder 差 (FILE_8BPP_3 = `feyn.tif`)           | 入力 8bpp の値が C/Rust で違う可能性       | `feyn.tif` のロード直後の hash を C/Rust で比較    |
| (4) 8bpp colormap 順序                                   | GIF が cmap-based のため順序差で hash 違う | round-trip 後の cmap entry 順序を C/Rust で比較    |

実は `FILE_8BPP_3 = feyn.tif` は **TIFF G4 (1bpp)** で、Rust 側 (`test_files` line 44 で "8bpp from JPEG" と書かれているがコメントは古い) の load 経路でも実態は 1bpp として load されるはず。であれば 1bpp → 8bpp → GIF とコンバートする経路で cmap 構築の差が出る (仮説 4)。

## Next step (別 PR)

優先度順:

1. **入力の hash 確認**: `feyn.tif` と `marge.jpg` を C と Rust で

   load して、ロード直後の hash が一致するか確認 (仮説 3 を切り分け)

2. **GIF write → 即 read round-trip の hash**: Rust と C で同じ入力

   pix から書き出した GIF を、逆側 (C 出力を Rust read / Rust 出力を
   C read) で逆 round-trip して hash 比較 (仮説 1 を切り分け)

3. 仮説 2 / 4 はその後に検討

## 関連

- 本 finding 開始経緯: Phase 3 第三弾 PR (gifio Part 1 完成 + 2 件

  Mismatch 可視化)

- C 実装: `reference/leptonica/prog/gifio_reg.c`
- Rust 実装: `tests/io/gifio_reg.rs`, `src/io/gif.rs`
