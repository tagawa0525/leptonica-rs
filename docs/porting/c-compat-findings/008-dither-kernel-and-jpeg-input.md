# C互換性調査 #008: dither kernel の実装差 (修正済み) と JPEG 入力 decode 差

plan 902 PR 2 で dither の semantic ペア 4 件 (dither.00/02/04/05) を
golden_map に追加した際に発見・対応した内容を記録する。

## 発見 1: dither kernel が C と異なっていた (修正済み)

Rust の `dither_to_binary` / `dither_to_2bpp` / `scale_gray_*_li_dither` は
**古典的 Floyd-Steinberg** (7/16, 3/16, 5/16, 1/16 を浮動小数で伝播) を
実装していたが、C 版 `ditherToBinaryLineLow` / `ditherTo2bppLineLow`
(`src/grayquant.c`) は別のアルゴリズムを使う:

- 3 近傍伝播: **3/8 右、3/8 下、1/4 右下** (整数演算、切り捨て)
- clip: 黒/白から `DEFAULT_CLIP_LOWER/UPPER` (1bpp: 10、2bpp: 5) 以内の
  ピクセルは誤差を伝播しない
- 2bpp は `make8To2DitherTables()` のルックアップテーブル
  (レベル 0/85/170/255、分割点 43/85/128/170/213) で出力値と誤差を決定
- 最終行は右のみ、最終列は下のみ伝播

→ C kernel に書き直して解消 (fix コミット参照)。手計算期待値の単体テスト
2 件 + 下記の同一入力検証で確認。

## 発見 2: JPEG 入力の decode 差で hash 一致は不可能 (finding 001 の拡張)

`dither_reg.c` の入力は `test8.jpg`。C (libjpeg-turbo) と Rust
(jpeg-decoder) の decode 結果を直接比較したところ:

```text
total=234300 diff=3298 maxdiff=1
```

**3298 ピクセル (1.4%) が ±1 ずれる**。誤差拡散 dithering はこのズレを
下流に連鎖させるため、kernel が完全一致でも出力 hash は一致しない。

### 同一入力での bit 一致検証 (kernel 等価性の確定証明)

C 側で decode した test8.jpg を PNG 化し、同一ピクセルを両実装に与えた:

| C 出力    | 演算               | 結果                              |
| --------- | ------------------ | --------------------------------- |
| dither.00 | dither_to_binary   | **diff=0**                        |
| dither.02 | dither_to_2bpp     | **diff=0**                        |
| dither.04 | scale 2x LI dither | **diff=0** (PR 3 の発見 3 対応後) |
| dither.05 | scale 4x LI dither | **diff=0** (同上)                 |

dither kernel は **bit 一致を確定証明**。finding 001 が「仮説」に留めていた
「同一入力なら一致する」を、dither 系では確定させた。

## 発見 3: scale_gray_2x/4x_li の LI スケーリング実装差 (解消済み)

dither.04/05 の残差は dither ではなく **LI スケーリング自体の差**だった:

- C `scaleGray2xLILineLow` (`src/scale1.c`): 専用の整数 2x 補間
  (`(s1+s2)>>1`、`(s1+s2+s3+s4)>>2`)
- Rust `scale_gray_2x_li`: 汎用 `scale_gray_li(pix, 2.0, 2.0)` に委譲
  (fractional LI、サンプリング位置も異なる)

→ **plan 902 PR 3 で解消**: 2x/4x とも C 専用整数補間 (4x は 1/4・1/2・
3/4 重みの 4x4 ブロック、最終行・列は複製) に書き直し。同一入力検証で
dither.04/05 とも **diff=0 の bit 一致** を確認。

## 現状の分類

| ペア                    | 状態     | 原因                                    |
| ----------------------- | -------- | --------------------------------------- |
| dither.00 ↔ dither_bin  | Mismatch | JPEG 入力 decode 差のみ (kernel は一致) |
| dither.02 ↔ dither_2bpp | Mismatch | 同上                                    |
| dither.04 ↔ scaled 2x   | Mismatch | 同上 (発見 3 は PR 3 で解消済み)        |
| dither.05 ↔ scaled 4x   | Mismatch | 同上                                    |

## 再現手順

```bash
# C 側 golden の生成 (要 reference/leptonica ビルド)
cd reference/leptonica/prog && ../build/bin/dither_reg generate

# C decode の PNG 化
../build/bin/convertformat test8.jpg /tmp/lept/regout/test8_c_decoded.png

# Rust 側: /tmp/lept/regout/test8_c_decoded.png を読み gamma 1.3 →
# dither_to_binary / dither_to_2bpp を実行し C 出力と画素比較 (diff=0)
```
