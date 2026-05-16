# C互換性調査 #001: JPEG codec 差 (edge / convolve / colormorph) — 仮説段階

Phase 2 (plan 901) のレポート機構で観測された 9 件の Mismatch を調査した結果、
**有力仮説として全件が JPEG codec の物理的な差** (C 版 `libjpeg-turbo` vs
Rust 側 `jpeg-decoder` / `jpeg-encoder` crate) に起因する、と判定する。
ただし C 版 Sobel/blockconv/colormorph を **同一 decoded pixel に対して走らせて
出力を bit-比較する確定検証は未実施**。本書は「Phase 2.5 の対応優先度を下げる
ための仮説整理」であり、後述「将来の確定検証」のステップを踏むまではあくまで
仮説とする。

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

- 差分は **4-11 ピクセル / max 1**。**注意**: 1bpp 出力では値域が `{0, 1}` の
  ため MaxDiff は常に 1 で頭打ち。MaxDiff は codec/アルゴリズムの切り分けに
  使えない (Copilot 指摘 #1 を反映)
- 切り分けの根拠は **差分ピクセル数の少なさ** (4-11/234,300 = 0.005%) と、
  入力 `test8.jpg` が JPEG であること:
  - C 版 `libjpeg-turbo` と Rust 側 `jpeg-decoder` crate は SIMD / IDCT 実装が
    異なり、特に chroma upsampling や色変換で個別ピクセルの ±1〜±2 のズレを
    生むことが知られている
  - そのズレが Sobel filter の閾値 (60) 境界を跨ぐピクセルでのみ ON/OFF を
    反転させると、差分ピクセル数は「閾値付近のピクセル数 × ズレ確率」程度に
    抑えられる。観測値 4-11 px はこのオーダーと整合
- 補強的な観察として、`src/filter/edge.rs::sobel_convolve_and_abs` を
  `reference/leptonica/src/edge.c::pixSobelEdgeFilter` と直接突き合わせ、
  kernel 定義 (sobel_horizontal = `[1,2,1; 0,0,0; -1,-2,-1]`) と convolve
  loop (`val1+2*val4+val7-val3-2*val6-val9 >> 3`) が **コードレベルで等価**
  であることを確認。ただしコード等価性は pixel-level 出力一致を保証しない
  (整数オーバーフローの扱い等で隠れた差が起きうる) ため、確定証明は後述の
  検証手順を要する

### 8bpp JPEG ケース (edge.04, convolve, colormorph)

- 入力 JPEG decode 差 + 出力 JPEG encode 差が **二重にかかる**
- とくに `colormorph` は wyom.jpg を入力に取り、結果も JPEG で保存。dilate /
  erode はピクセル値を **そのまま隣接ピクセルの最大/最小に置換** するため、
  小さな decode 差が大きな出力差として 8x8 JPEG ブロック単位で増幅される
- `scripts/golden_map.tsv` の colormorph セクションには既に「`Algorithm
  verified correct via compare_pix (dilate_color == color_morph_sequence("d7.7"))`」
  と記されている。**ただしこれは Rust 同士 (`dilate_color` vs
  `color_morph_sequence("d7.7")`) の整合性に過ぎず、両 Rust 実装が共通して
  C 版と semantic 差を持っている可能性を排除できない** (Copilot 指摘 #2)。
  C 一致の確定証明には別途 pixel-level 比較が必要
- `convolve_blockconv_gray` の max 5 は線形フィルタの誤差伝搬として妥当な
  値域 (decode 差 ±2 程度が blockconv で平均化される) — これも仮説

## 結論 (仮説段階)

- **暫定判定**: Phase 2 のレポートで観測された `Mismatch` 9 件はすべて JPEG
  codec 差の **可能性が高い**。Phase 2.5 の修正対象から **一旦除外** する
- ただし、上記の通り 1bpp 出力での MaxDiff は値域上限、Rust 同士の整合性は
  C 一致を保証しないなど、本書の論拠は確定証明ではない。**「Rust に bug が
  存在しない」とは言い切れていない**

## 将来の確定検証 (Phase 2.5 で別途 PR 化を検討)

- **Sobel filter**: PPM (lossless) 形式で同一 8bpp 画像を Rust と C にそれぞれ
  入力し、出力 Sobel の bit-by-bit 一致を確認する unit test を追加
  - 具体的には `tests/data/images/test8.jpg` を Rust で 1 回 decode、結果を
    PPM で書き出し、C 版 `pixSobelEdgeFilter` をその PPM に対して走らせて
    Rust 出力と比較
  - PPM 経由で渡せば JPEG codec の影響を排除でき、純粋にアルゴリズム差を
    観測できる
- **blockconv_gray / colormorph**: 同様に PPM 入力で C と Rust を比較
- 一致しなかった場合: 該当 Rust 実装を C と一致するように修正
- 一致した場合: 本書を「確定」に格上げし、`scripts/golden_map.tsv` の KNOWN
  コメントから「(仮説)」を外す

## アクション (本 PR で完了)

- `scripts/golden_map.tsv` の edge / convolve / colormorph セクションに本書
  への参照を追記
- `docs/plans/901_c-hash-compat.md` の Phase 2.5 セクションに Findings ログ
  リンクを追加
