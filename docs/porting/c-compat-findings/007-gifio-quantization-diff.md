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

入力ファイル名は `tests/io/gifio_reg.rs` の定数 (`FILE_1BPP` 〜
`FILE_32BPP`) から確認:

| Test file (input)                | Rust check | C check | Rust file | C file | 状態            |
| -------------------------------- | ---------: | ------: | --------: | -----: | --------------- |
| FILE_1BPP (`feyn.tif`)           |          0 |       0 |        01 |     00 | ✅ Ok           |
| FILE_2BPP (`weasel2.4g.png`)     |          1 |       1 |        03 |     01 | ✅ Ok           |
| FILE_4BPP (`weasel4.16c.png`)    |          2 |       2 |        05 |     02 | ✅ Ok           |
| FILE_8BPP_1 (`dreyfus8.png`)     |          3 |       3 |        07 |     03 | ✅ Ok           |
| FILE_8BPP_2 (`weasel8.240c.png`) |          4 |       4 |        09 |     04 | ✅ Ok           |
| **FILE_8BPP_3** (`test8.jpg`)    |          5 |       5 |        11 |     05 | ⚠️ **Mismatch** |
| FILE_16BPP (`test16.tif`)        |          6 |       6 |        13 |     06 | ✅ Ok           |
| **FILE_32BPP** (`marge.jpg`)     |          7 |       7 |        15 |     07 | ⚠️ **Mismatch** |

GIF のテスト構造は「lossless r/w ラウンドトリップ」 (元画像 → GIF
write → read で得た `pix1` の hash を確認) なので、入力画像と Rust の
GIF encoder/decoder + C の GIF encoder/decoder の挙動差が hash 差に
直結する。

## Root cause 仮説 (未確定、要追加調査)

LZW 圧縮自体は lossless なので、GIF bytes が異なっても decoded pixel
値は同じになるはず。よって LZW 圧縮テーブル順序差は本 finding の root
cause **ではない**。差は encoder 前 (palette 構築 / quantization /
colormap 順序) または decoder 側 (palette index 解釈) にあると考え
られる。

| 仮説                                            | 影響                                                                                    | 切り分け方                                                    |
| ----------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| (1) 32bpp → 8bpp 量子化アルゴリズム差           | FILE_32BPP は GIF write 前に必ず量子化                                                  | Rust `convert_to_8` / `color_quant` 系と C の差               |
| (2) JPEG decoder 差 (FILE_8BPP_3 = `test8.jpg`) | 入力 8bpp の値が C/Rust で違う可能性                                                    | `test8.jpg` のロード直後の hash を C/Rust で比較              |
| (3) 8bpp colormap 順序                          | GIF が cmap-based のため、palette index 順序差で pixel 値が違う                         | round-trip 後の cmap entry 順序を C/Rust で比較               |
| (4) GIF encoder の palette 構築差               | cmap 未付与画像から GIF を書き出す際の Rust gif crate と giflib の生成 palette が異なる | C と Rust で同じ image を GIF write して decoded pixel を比較 |

`FILE_8BPP_3 = test8.jpg` は JPEG 8bpp なので、JPEG decoder 差 (仮説 2)
が直接効く可能性が高い。`FILE_32BPP = marge.jpg` は JPEG 32bpp で、
入力ロードと GIF write 前の量子化の **両方** の差が積み重なる。

## Next step (別 PR)

優先度順:

1. **入力の hash 確認**: `test8.jpg` と `marge.jpg` を C と Rust で

   load して、ロード直後の hash が一致するか確認 (仮説 2 = JPEG
   decoder 差を切り分け)

2. **同一入力 pix からの GIF write 比較**: C と Rust で同じ入力 pix

   を別々に GIF に書き出し、**同じ decoder で read** して decoded
   pixel が一致するか確認 (encoder 差 = 仮説 4 を切り分け)

3. 仮説 1 (量子化) / 3 (colormap 順序) はその後に検討

## 関連

- 本 finding 開始経緯: Phase 3 第三弾 PR (gifio Part 1 完成 + 2 件

  Mismatch 可視化)

- C 実装: `reference/leptonica/prog/gifio_reg.c`
- Rust 実装: `tests/io/gifio_reg.rs`, `src/io/gif.rs`
