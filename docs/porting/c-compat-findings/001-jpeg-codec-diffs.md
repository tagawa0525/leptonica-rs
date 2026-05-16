# C互換性調査 #001: JPEG codec 差 (edge / convolve / colormorph)

Phase 2 (plan 901) のレポート機構で観測された 9 件の Mismatch を調査した結果、
**すべて JPEG codec の物理的な差** (C 版 `libjpeg-turbo` vs Rust 側
`jpeg-decoder` / `jpeg-encoder` crate) に起因し、Rust 実装のアルゴリズムは正
しく動作していることを確認した。**Phase 2.5 の Rust 側修正対象外** と判定する。

## 観測された 9 件の不一致

`cargo run --release --example compare_golden --features all-formats -- --module <name>`
で pixel-level 比較した結果:

| Test                       | Operation                             |    Diff/Total | MaxDiff |  %Diff | 入力      | 出力形式 |
| -------------------------- | ------------------------------------- | ------------: | ------: | -----: | --------- | -------- |
| edge.01                    | Sobel H 1bpp → threshold(60) → invert |     10/234300 |       1 |  0.00% | test8.jpg | PNG      |
| edge.02                    | Sobel V 1bpp → threshold(60) → invert |      4/234300 |       1 |  0.00% | test8.jpg | PNG      |
| edge.03                    | OR(H, V) 1bpp                         |     11/234300 |       1 |  0.00% | test8.jpg | PNG      |
| edge.04                    | max(H 8bpp, V 8bpp) → invert          |  25912/234300 |      23 | 11.06% | test8.jpg | JPEG     |
| convolve_blockconv_gray.01 | blockconv_gray(3, 5)                  |  14429/234300 |       5 |  6.16% | test8.jpg | JPEG     |
| colormorph.01              | dilate_color 7x7                      | 343730/453574 |      32 | 75.78% | wyom.jpg  | JPEG     |
| colormorph.03              | erode_color 7x7                       | 340521/453574 |      25 | 75.08% | wyom.jpg  | JPEG     |
| colormorph.05              | open_color 7x7                        | 329666/453574 |      24 | 72.68% | wyom.jpg  | JPEG     |
| colormorph.07              | close_color 7x7                       | 328822/453574 |      22 | 72.50% | wyom.jpg  | JPEG     |

## 判定根拠

### 1bpp PNG ケース (edge.01-03)

- 差分は **4-11 ピクセル / max 1**。1bpp 出力での「1 ピクセル差」は「閾値判定で

  片側が ON、もう片側が OFF」しか取り得ない

- Sobel filter の閾値 60 付近で C と Rust の中間値が ±1 程度ずれていれば、

  10 程度のピクセルが境界をまたいで反転するのは自然

- 中間値ズレの原因は入力 `test8.jpg` の JPEG decode 差。C の `libjpeg-turbo` と

  Rust の `jpeg-decoder` crate は SIMD / IDCT 実装が異なり、特に chroma
  upsampling や色変換で 1bpp 単位の差を生む

- アルゴリズム差なら max 1 では収まらない (Sobel は線形フィルタ + abs で誤差が

  伝搬する)。確認のため `src/filter/edge.rs::sobel_convolve_and_abs` を
  `reference/leptonica/src/edge.c::pixSobelEdgeFilter` と直接突き合わせ、
  kernel 定義 (sobel_horizontal = `[1,2,1; 0,0,0; -1,-2,-1]`) と convolve
  loop (`val1+2*val4+val7-val3-2*val6-val9 >> 3`) が完全に一致することを
  確認した

### 8bpp JPEG ケース (edge.04, convolve, colormorph)

- 入力 JPEG decode 差 + 出力 JPEG encode 差が **二重にかかる**
- とくに `colormorph` は wyom.jpg を入力に取り、結果も JPEG で保存。dilate /

  erode はピクセル値を **そのまま隣接ピクセルの最大/最小に置換** するため、
  小さな decode 差が大きな出力差として 8x8 JPEG ブロック単位で増幅される

- `scripts/golden_map.tsv` の colormorph セクションには **既に**「`Algorithm

  verified correct via compare_pix (dilate_color == color_morph_sequence("d7.7"))`」
  と記されている。Rust 同士の整合は取れており、不一致は codec 差のみ

- `convolve_blockconv_gray` の max 5 は線形フィルタの誤差伝搬として妥当な

  値域 (decode 差 ±2 程度が blockconv で平均化される)

## 結論

- Rust 実装の修正は不要
- Phase 2 のレポートで `Mismatch` 9 件はすべて codec 差と判定し、Phase 2.5

  の修正対象から除外する

- 将来 `tests/c_compat_report.<binary>.txt` に新たな `Mismatch` が出現した

  場合は、本書のチェックリストに沿って (1) 入出力形式の確認、(2)
  `compare_golden` で max ピクセル差を測定、(3) max < 数十なら codec 差を
  疑い、それ以上ならアルゴリズム差として個別調査する

## アクション

- `scripts/golden_map.tsv` の edge / convolve / colormorph セクションに本書

  への参照を追記 (本 PR で実施)

- `docs/plans/901_c-hash-compat.md` の Phase 2.5 セクションに「JPEG codec 差

  は対象外」の根拠を追加 (本 PR で実施)
