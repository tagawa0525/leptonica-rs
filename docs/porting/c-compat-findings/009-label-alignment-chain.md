# C互換性調査 #009: label 整列で発見した 4 つの乖離 (すべて解消)

plan 902 PR 6 で label_reg の semantic ペア 8 件を張る過程で、連鎖的に
4 つの乖離を発見し、すべて解消した。

## 発見 1: hash 比較規約の非対称 (構造的問題、解消)

C 側 manifest (`golden_manifest_c.tsv`) は「C が書いたファイルを Rust io
で decode した結果」の hash だが、Rust 側は「メモリ上の pix」の hash で
比較していた。PNG は 32bpp の alpha byte を書き出しで落とすため、
**32bpp 出力はどちらのアルゴリズムが正しくても一致し得なかった**。

→ C 比較用の hash のみ「書き出したファイルを読み戻して計算」に変更
(Rust-vs-Rust の golden manifest は従来どおりメモリ hash)。

**副次効果**: seedspread の Mismatch 6 件中 4 件 (finding 006) が
この規約差だけで Ok に転じた。finding 006 の主因は実装差ではなく
比較方法だった。

## 発見 2: loc-to-color の alpha byte (解消)

`pix_loc_to_color_transform` が alpha=255 で合成していたが、C
`pixCreateRGBImage` は alpha=0 (PR #405 の pixConvert8To32 と同じ規約)。
blue channel も `(area & 0xffff).min(255)` に厳密化。

## 発見 3: rasterop_hip/vip の 1bpp incolor 反転 (解消)

C `pixRasteropHip/Vip` は 1bpp の `L_BRING_IN_WHITE` を **PIX_CLR (0=白)**、
`L_BRING_IN_BLACK` を PIX_SET (1=黒) にマップする (leptonica の 1bpp は
1=黒)。Rust は深度によらず White=max_value で埋めており 1bpp で白黒が
反転していた。`translate` 経由で label_color.04 の Mismatch として発覚。
既存の `test_translate_binary` は誤った期待値 (白=1) を固定していた。

## 発見 4: shear の band 量子化 (解消)

C `pixHShear/pixVShear` (IP 含む) は行/列を |1/tan(angle)| 幅の帯単位で
シフトし、境界を `trunc(invangle*(shift±0.5)+0.5)` で決める。Rust は
行ごとの `round((yloc-y)*tan)` で、帯境界の行が 1px ずれていた。
`rotate_shear_center_ip` 経由で label_color.05 の Mismatch として発覚。
`band_shifts` ヘルパーに集約して 4 関数 (h/v × 非IP/IP) を統一。

## 結果

label 8 ペア全件 hash 完全一致 (Ok +8)。roundtrip 規約で seedspread +4。
全体: **Ok 64 → 76、Mismatch 33 → 29、Unmapped 410 → 406**。

残る Mismatch 29 の内訳: JPEG codec 差 21 (finding 001) + dither 4
(finding 008、JPEG 入力 decode 差) + seedspread 2 (finding 006 残) +
gifio 2 (finding 007)。

## 未対応 (後続 PR 候補)

- C label_reg の check 1 (`pixConnCompTransform` の 8bpp 出力):
  Rust の `conn_comp_transform` は出力 depth パラメータを持たず、
  ラベル値の規約 (1 + i % 254) と成分列挙順の一致確認も必要
- C check 5 (`pixMultConstantGray` 0.3 倍): 対応 API が未移植
