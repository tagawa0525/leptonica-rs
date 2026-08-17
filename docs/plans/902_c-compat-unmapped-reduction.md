# Plan 902: C 互換 Unmapped の削減 — 分母の確定と高価値モジュールのマッピング拡充

- Status: IN_PROGRESS
- 前提: plan 901 (C hash 互換検証基盤、PR #377〜#391)、Phase 2.5〜3 (PR #382〜#405)
- 関連 findings: `docs/porting/c-compat-findings/001`〜`007`

## Why

v0.5.0 時点の C 互換ベースラインは
**Ok 44 / Mismatch 29 / MissingC 0 / Unmapped 500**。
Unmapped 500 の実測内訳は以下のとおりで、マップする価値が形式によって大きく異なる:

| 分類 | 件数 | 評価 |
| --- | --: | --- |
| PNG | 360 | lossless — hash 一致が意味を持つ。マップ対象 |
| TIFF | 82 | 同上。マップ対象 |
| JPEG | 45 | codec 差 (finding 001) で必ず Mismatch になる。マップ不能 |
| PDF | 8 | 非決定的形式 (PR #386 で hash 化除外済みの残り)。マップ不能 |
| ba/na | 5 | データストリーム。マップ対象 |

また「hash が C manifest と完全一致していて機械的にマップできる残り」は
22 件 (一意特定 7 件) しかなく、easy win は Phase 3 第一弾で回収済み。
残りは C prog と Rust テストの出力 index を突き合わせる semantic
ペアリング作業になる。

「Unmapped を 0 にする」のではなく、
**(1) マップ不能分を Excluded として明示的に分離して分母を確定し、
(2) 意味のある残り (PNG/TIFF 中心) を高価値モジュールから漸進的にマップする**。

## What

### PR 1: Excluded ステータスの導入 (本 PR)

C 版ソース: 対応なし (テストインフラのみ)。

- `tests/common/c_compat.rs`:
  - `CCompatStatus::Excluded` を追加
  - 除外ルールファイル `scripts/c_compat_exclude.tsv` のパーサを追加
    (フィールド: `kind` (`ext`|`prefix`) / `value` / `reason`)
  - 分類ロジック: golden_map にエントリがあれば従来どおり
    Ok/Mismatch/MissingC (**マッピングが除外より優先**)。無い場合のみ
    除外ルールを照合し、一致すれば `Excluded`、不一致なら従来どおり
    `Unmapped`
  - strict モードで `Excluded` は fail しない (Mismatch のみ fail)
- `scripts/c_compat_exclude.tsv` 初期ルール:
  - `ext jpg` / `ext jpeg` — JPEG codec 差 (finding 001)
  - `ext pdf` / `ext ps` — 非決定的形式 (PR #386)
- `.github/workflows/ci.yml`: Job Summary の集計に `Excluded` 列を追加
- docs 更新: `c-compat-status.md` / CLAUDE.md / README(en/ja) のベース
  ライン表記

期待効果: Unmapped 500 → 447 (jpg 45 + pdf 8 が Excluded へ)。

### PR 2: dither の semantic ペア + kernel 修正 (実施済み)

C 版ソース: `src/grayquant.c` (ditherToBinaryLineLow / ditherTo2bppLineLow)。

- Rust テストを C prog と同じ gamma 1.3 前処理に整列し、dither ペア 4 件を
  golden_map に追加 (Unmapped 447 → 445、Mismatch +4)
- この過程で **dither kernel の実装差** (古典 FS vs C 3近傍 3/8・3/8・1/4
  整数演算 + clip) を発見し、C 準拠に修正。同一入力での bit 一致を確定証明
- 詳細: finding 008。follow-up: scale_gray_2x/4x_li の LI 実装差 (発見 3)

### PR 3: scale_gray_2x/4x_li の C 専用整数補間化 (実施済み)

C 版ソース: `src/scale1.c` (scaleGray2xLILineLow / scaleGray4xLILineLow)。

- 汎用 fractional LI 委譲だった 2x/4x を C 専用整数補間に書き直し
  (finding 008 発見 3 の解消)
- 同一入力検証で dither.04/05 とも diff=0 の bit 一致を確認。これで
  dither 系 4 ペアはすべて「アルゴリズム等価、残差は JPEG decode 差のみ」

### PR 4: paintmask 19-21 の lossless ペア (実施済み)

C 版ソース: `prog/paintmask_reg.c` 19-21 (feyn.tif / rabi.png)。

- C と同条件 (同 box・outval) の 1bpp blend テストを追加し 3 ペアをマップ
- **全件 hash 完全一致 (Ok 44 → 47)**。clip_rectangle / invert /
  clip_masked の C 等価性を pixel-level で証明
- 教訓: **lossless 入力のペアは即 Ok になる**。JPEG 入力系列
  (decode 差で必ず Mismatch) より lossless 系列を優先してマップする

### PR 5: distance 系の整列 + boundary condition 修正 (実施済み)

C 版ソース: `prog/distance_reg.c`、`src/seedfill.c`
(pixDistanceFunction / distanceFunctionLow / pixSetMirroredBorder)。

- distance テスト 4 本を C prog と同条件に整列 (box 1480x1050、invert)
- ペアを張った結果 **bc=Foreground の全ペアが不一致** →
  `distance_function` の L_BOUNDARY_FG 実装差を発見し TDD で C 準拠に修正
  (境界1周の 255 セット → interior 2 パス → 隣接 interior ミラー)
- 17 ペア全件 hash 一致 (Ok 47 → 64)。C 対応が JPEG/不在の 26 キーは
  除外ルール (`key` 種別を新設) で分離し、distance 系 Unmapped は 0
- 教訓: lossless 系列の整列は「即 Ok」または「実バグ発見」のどちらかに
  なる。マッピング作業自体がバグ検出器として機能している

### PR 6: label 整列 — hash 規約の構造修正 + 3 実装バグ (実施済み)

C 版ソース: `prog/label_reg.c`、`src/pixlabel.c` / `src/rop.c` / `src/shear.c`。

- label 8 ペアを張る過程で 4 つの乖離を連鎖的に発見しすべて解消
  (finding 009): (1) **hash 比較規約の非対称** → C 比較のみ roundtrip
  hash に構造修正 (seedspread 4 件が自動解消)、(2) loc-to-color の
  alpha=255、(3) rasterop_hip/vip の 1bpp incolor 反転、(4) shear の
  band 量子化欠落
- label 8 ペア全件 Ok (Ok 64 → 76、Mismatch 33 → 29、Unmapped 410 → 406)
- 未対応: C check 1 (ConnCompTransform 8bpp) と check 5
  (pixMultConstantGray) は API 追加が必要 (finding 009 参照)

### PR 7: label 残り 2 ペア — conn_comp_transform_depth + multiply_constant (実施済み)

- `conn_comp_transform_depth` 新設 (C pixConnCompTransform 準拠) と
  `multiply_constant` 32bpp の C 準拠化 (実装差 8 件目) で
  C label_reg の PNG 出力 10 件が完全制覇 (Ok 76 → 78)

### PR 8: conncomp 整列 — pixaDisplay の合成修正 (実施済み)

C 版ソース: `prog/conncomp_reg.c`、`src/pixafunc2.c` (pixaDisplay)。

- conncomp の pixa 再構成ペアを張る過程で **pixaDisplay の合成が
  上書きコピーになっており、bbox が重なる成分の fg が消える**実装差
  (9 件目) を発見。C の PIX_PAINT (OR) + 白背景初期化に修正
- 4-cc/8-cc の再構成が原画像と bit 一致し 2 ペア Ok (Ok 78 → 80)
- 未対応: C 11 (pixaDisplayRandomCmap、乱数依存)、C 12-18
  (pixMakeCoveringOfRectangles — Rust 版はパラメータ意味論が異なり
  要整列。後続 PR 候補)

### PR 9: covering of rectangles の C 準拠化 (実施済み)

C 版ソース: `src/pix5.c` (pixMakeCoveringOfRectangles)。

- `make_covering_of_rectangles` を C 準拠 (maxiters 指定、PIX 返し、
  bbox 塗り→再ラベル→収束) に書き直し (旧 Rust 版は distance 拡張の
  Boxa 返しで意味論が異なった)
- C 12-17 の 6 ペア (rank cascade + covering ×5) が全件即 hash 一致
  (Ok 80 → 86)。conncomp は乱数依存の C 11 と composite の C 18 を
  除き全 PNG 出力が Ok

### PR 10: quadtree 整列 — 値テーブル修正 + 表示系 3 関数移植 (実施済み)

C 版ソース: `src/quadtree.c` / `src/pixafunc2.c` / `src/scale2.c`。

- **scale_to_gray_N の値テーブルが四捨五入になっていた実装差 (11 件目)**
  を発見・修正 (C は 255 - (black*255)/N² の切り捨て)
- boxaa_quadtree_regions / fpixa_display_quadtree /
  Pixa::display_tiled_in_rows を新規移植 (TDD)
- quadtree 4 ペア追加・全件 Ok (Ok 86 → 90)。Boxaa 直列化も byte 互換を実証
- **未対応**: quadtree 02-04 (fpixa display) は Rust の Bmf が合成フォント
  (base 7px スケール) で C のビットマップフォント実体と行高が異なるため
  pixel 不一致。C フォントデータ (bmfdata) の移植が必要 (後続 PR 候補)

### PR 11: falsecolor 整列 — color の合成入力系列 (実施済み)

C 版ソース: `prog/falsecolor_reg.c`、`src/pixconv.c` / `src/colormap.c`。

- C prog と同一の合成入力 (768x100 の 8/16bpp gradient) で
  `convert_gray_to_false_color` を gamma {1.0, 2.0, 3.0} で適用する
  `falsecolor_c_compat` を追加し、C の 8 出力 (全 PNG) と 8 ペア
  **全件即 hash 一致 (Ok 90 → 98)**。実装は既に C と等価だった
- pixel hash は colormap を含まないため、gamma 別 colormap
  (256 エントリ) は C 出力との decode 後比較で bit 一致を別途実証
- Rust 独自 API (pix_linear_map_to_target_color 等) の falsecolor.*
  4 件は C 対応が無く prefix 除外 (Unmapped 407 → 403)
- **見送り**: coloring_reg (harmoniam100-11.png、PNG 14 件) は全出力が
  pixAddSingleTextblock (bmf フォント) 経由のため、quadtree 02-04 と
  同じく bmfdata 移植が前提 (後続 PR 候補)

### PR 12: bmfdata 移植 — Bmf の C 準拠化 (実施済み)

C 版ソース: `src/bmf.c` / `src/bmfdata.h`、`prog/genfonts_reg.c`。

Rust の `Bmf` は合成 5x7 フォントのスケール生成で、C のビットマップ
フォント実体 (bmfdata.h の G4 TIFF) とグリフ・行高が根本的に異なる。
これが quadtree 02-04 / coloring 全 14 出力のブロッカー (PR 10/11 で
記録)。事前調査で以下を確認済み:

- fontdata_N (base64) を decode した TIFF は Rust の tiff crate で
  decode 可能 (G4 対応)、かつ `prog/fonts/chars-N.tif` と pixel 一致
- C genfonts_reg の出力 00-08 (ファイル経路) と 09-17 (文字列経路) は
  同一 hash — 経路によらず同一 pixa

作業内容:

1. fontdata の decode 済み TIFF (9 サイズ、計 30KB) を
   `src/core/fonts/` に置き `include_bytes!` で埋め込み
   (抽出スクリプトをコミット、C bmfdata.h との一致を検証)
2. `pixaGenerateFont` / `pixGetTextBaseline` / `bmfMakeAsciiTables` を
   C 移植し、`Bmf::new` を C 準拠に差し替え (合成フォント削除、
   fontsize は C 同様 4-20 偶数のみ)
3. genfonts_c_compat テスト (9 サイズの font pixa を
   pixaDisplayTiled(1500, 0, 15)) で C genfonts.09-17 と 9 ペア
4. quadtree.02-04 ↔ quadtree_c.03-05 の 3 ペア (fpixa display は
   PR 10 で移植済み、フォント差のみが残ブロッカー)
5. Bmf 依存の既存 golden (bmf_reg / writetext_reg / genfonts_reg /
   quadtree_c / gplot 系) を再生成

実施結果:

- 全 9 サイズで baseline / lineheight / kern / space / vertsep /
  グリフ寸法が C 実測値と一致 (bmf_c_compat_metrics)
- この過程で **pixaDisplayTiled の実装差 (12 件目)** を発見: Rust は
  詰め込み折り返しで、C は最大部分画像寸法ベースの均等格子。C 準拠に
  書き直し (TDD)
- quadtree 3 ペア + genfonts 9 ペア **全件 Ok (Ok 98 → 110)**。
  genfonts ペアは 95 グリフ x 9 サイズの bit 等価の完全証明
- fontsize 18 が新たに有効化。coloring 14 ペアのフォント面の前提が
  整った (残りは cmapped pixShiftByComponent、PR 13 候補)

### PR 13: coloring 整列 — cmapped shift 対応 + 14 ペア (実施済み)

C 版ソース: `prog/coloring_reg.c`、`src/coloring.c` / `src/colormap.c`。

PR 11 で見送った coloring 系列。フォント面の前提は PR 12 で解消済み。

1. `pix_shift_by_component` に cmapped 分岐を追加 (C は cmap を
   `pixcmapShiftByComponent` で変換するだけ。`PixColormap::
   shift_by_component` は移植済みで pixel 式も C と一致確認済み) (TDD)
2. テスト画像 `harmoniam100-11.png` を C prog から追加
3. coloring_c テスト: C checks 2-15 と同条件 (cmap reset 4 + cmapped
   shift 4 + rgb shift 4 + fg cmapped/rgb 2、全出力に
   pixAddSingleTextblock fontsize 8) で 14 出力を書き出し 14 ペア

実施結果:

- cmapped テキスト描画を C 準拠化 (paint_through_mask の
  pixSetMaskedCmap 分岐、add_single_textblock の色解決と cmapped 時の
  クランプ回避、baselinetab[93] 整合) (TDD)
- **実装差 13 件目**: convert_to_32 が colormap を無視して index 値を
  グレー複製していた → C pixConvert*To32 準拠で
  REMOVE_CMAP_TO_FULL_COLOR 経由に修正。あわせて
  remove_colormap(ToFullColor) の alpha byte を C 準拠の 0 に修正 (TDD)
- coloring 14 ペア **全件即 Ok (Ok 110 → 124)**。cmapped 描画・shift・
  cmap reset・RGB 展開の全経路が C と bit 一致。C compare 0-1 相当の
  cmapped/rgb 経路一致検証も rp.compare_pix で index を揃えて実施

### PR 14: smallpix 整列 — 変換 9 関数の合成入力系列 (実施済み)

C 版ソース: `prog/smallpix_reg.c`、`src/scale1.c` / `src/rotate.c` /
`src/pixafunc2.c`。

transform は Unmapped 78 で最大の未開拓プール。smallpix_reg は
**入力が完全合成 (9x9 の pixCreate + generatePtaLineFromPt)** で全 9
出力が PNG、しかも 1 出力 = 1 変換関数のスイープになっており、
codec 差なしで主要変換 9 種を一挙に検証できる:

| C check | 関数 | Rust 現状 |
| --- | --- | --- |
| 0 | pixScaleSmooth | `scale_smooth` |
| 1 | pixScaleAreaMap | 非公開 (`_to_size` のみ) |
| 2 | pixScaleBySampling | `scale_by_sampling` |
| 3 | pixRotateAM | 非公開 (corner 版のみ) |
| 4 | pixRotateBySampling | 非公開 |
| 5 | pixRotateAMCorner | `rotate_am_corner` |
| 6 | pixRotateAMColorFast | 非公開 (corner 版のみ) |
| 7 | pixScaleColorLI | `scale_color_li` |
| 8 | pixScaleLI | `scale_li` |

作業内容:

1. `Pixa::display_tiled_in_columns` を移植 (C
   pixaDisplayTiledInColumns。translate / shear2 / xformbox でも必要)
2. 未公開の 4 関数 (`scale_area_map` / `rotate_am` /
   `rotate_by_sampling` / `rotate_am_color_fast`) を C シグネチャで公開
3. smallpix_reg.rs を C と同条件の `smallpix_c_compat` に整列し 9 ペア

実施結果:

- Pixa::display_tiled_in_columns を移植し、未公開だった 4 関数を公開
  (rotate_am 系 3 種は初回から C と bit 一致)
- **この 1 PR で実装差 4 件 (14-17 件目) を発見・修正**:
  (14) sampling の index 規約 ((int)(ratio*i + shift)、
  scale_by_sampling の shift=0.5、rotate の切り捨て位置)、
  (15) bilinear の 1/16 サブピクセル規約 + 特別ケース、
  (16) scale_smooth の固定窓・clamp・isize^2 除算、
  (17) area map の 1/16 分解 (C の float/double 非対称まで再現)
- smallpix 9 ペア **全件 Ok (Ok 124 → 133)**。transform binary が
  Ok 4 → 13 になり、主要変換 9 種の C 等価性を実証

### PR 15: translate / shear2 整列 — transform の残り lossless (実施済み)

C 版ソース: `prog/translate_reg.c` / `prog/shear2_reg.c`、
`src/rop.c` / `src/warper.c`。

PR 14 で `display_tiled_in_columns` を移植し、両 prog の前提が揃った。
どちらも全出力が PNG で、入力は lossless (weasel2.4c.png) または完全合成。

| C prog | 出力 | 入力 | 使用関数 |
| --- | --: | --- | --- |
| translate | 3 | weasel2.4c.png | pixTranslate x 4 種 x 深度別 |
| shear2 | 4 | 合成 (RenderLineArb) | pixQuadraticVShear (sampled/interp) |

作業内容:

1. translate_c テスト: C と同条件 (3x sampling → clip 209x214、
   cmap 除去 2 種 + 1bpp 化 + rotate_am 4 種) で 3 出力
2. shear2_c テスト: C と同条件 (301/601 の合成 RGB に 6 本の色線、
   sampled/interp x left/right、border 3 + textblock) で 4 出力

実施結果:

- **cmapped 経路の実装差 4 件 (18-21 件目) を発見・修正**:
  (18) convert_to_8 が colormap を無視 (8bpp cmapped を deep copy)、
  (19) **clip_rectangle が colormap を落とす** — 以降の
  remove_colormap / convert_* が全て無効化される根本原因、
  (20) rasterop_hip/vip の cmapped 充填が生値 (0 / max) で
  get_rank_intensity の index を使っていない、
  (21) 32bpp warp の白充填が 0xffffff00 (C は pixSetAll = 0xffffffff)
- あわせて convert_to_1 (C pixConvertTo1) を新規実装
- translate 3 ペア + shear2 4 ペア **全件 Ok (Ok 133 → 140)**。
  transform binary は Ok 13 → 20

### PR 16: xformbox 整列 — hash box 描画と box 変換 (実施済み)

C 版ソース: `prog/xformbox_reg.c`、`src/boxfunc2.c` / `src/graphics.c`。

transform に残る最後の全 PNG 系列。入力は feyn.tif (lossless) で、
hash box 描画 3 種と boxa の直交回転・順序付き変換を検証する。

| C check | 内容 |
| --: | --- |
| 0-2 | render_hash_box / _color /_blend を成分 boxa に適用 |
| 3 | rotate_orth x 4 + boxa rotate_orth を tiled in rows |
| 4-5 | transform_ordered 6 種 (translate / scale 系) の重ね描き |

必要 API (`render_hash_box*`, `Boxa::rotate_orth`,
`Boxa::transform_ordered`, `display_tiled_in_rows`) は移植済み。

実施結果:

- **実装差 22 件目**: `render_pta_color` が cmapped 画像で colormap を
  使わず生の gray/RGB 値を書いていた (C pixRenderPtaArb は
  pixcmapAddNearestColor で index を解決)。TDD で修正
- C 0-2 の 3 ペア **全件 Ok (Ok 140 → 143)**。1bpp / 8bpp cmapped /
  32bpp blend の 3 描画経路を実証
- **C 3/4 は次段送り**: どちらも `pixaDisplayTiledIn*` 内部の
  `pixScale` を経由するが、Rust の `scale_general` は
  **(a) unsharp masking (C は sharpfract 0.2/0.4 を既定で適用)、
  (b) area map の 1/2 特別ケース、(c) 出力寸法の丸め**が C と異なる
  (実装差 23 件目)。理由付きで Excluded に分離し PR 17 で対応する
- C 5 はさらに `boxaAffineTransform` + 2D 行列ビルダが未移植

### PR 17: scale_general の C 準拠化 (実施済み)

C 版ソース: `src/scale1.c` (pixScale / pixScaleGeneral)、
`src/enhance.c` (pixUnsharpMasking* )。

PR 16 で記録した実装差 23 件目の解消。`pixScale` は移植済み関数の中でも
利用箇所が広く (`display_tiled_in_*` の scalefactor 経路を含む)、C との
乖離が xformbox 2 ペアのブロッカーになっている。

C `pixScaleGeneral` との差:

- **unsharp masking を適用していない** — C は `pixScale` から
  sharpfract 0.2 (maxscale < 0.7) / 0.4、sharpwidth 1 / 2 を既定で渡し、
  縮小時は maxscale > 0.2、拡大時は maxscale < 1.4 の条件で適用する。
  Rust は引数を `_sharpfract` / `_sharpwidth` として無視
- **sub-dispatch** — C は公開関数 (`pixScaleAreaMap` / `pixScaleGrayLI` /
  `pixScaleColorLI`) を呼ぶため 1/2 の特別ケースや寸法規約が効くが、
  Rust は impl を直接呼び `.round()` で寸法を決めている
- 1bpp は `pixScaleBinary`、それ以外は `pixConvertTo8Or32` を通す

あわせて **実装差 24 件目**: `unsharp_masking_gray_fast` が
`blockconv_gray` による全面ブラーで、C の分離型 box フィルタ
(内部のみ更新、border は原画コピー、`(int)(s + f*(s-L) + 0.5)`) と
異なる。

実施結果 — 追跡の過程で **実装差 5 件 (23-27) を発見・修正**:

- **23** `scale_general` の dispatch と unsharp masking (上記)
- **24** `unsharp_masking_gray_fast` の分離型 box 化
- **25** `scale_area_map_2` の 2x2 平均が `+2` の四捨五入 (C は
  `val >>= 2` の切り捨て)、alpha も平均していた
- **26** 対角 hash の spacing を f32 で計算しており、C の double と
  切り捨て位置がずれる (`generatePtaHashBox`)
- **27** **閾値比較の精度**。C は `l_float32` を double literal と
  比較するため float が昇格し、`0.7f` は「< 0.7」と判定される。
  Rust の f32 同士比較では false になり dispatch が分岐していた
- xformbox 5 ペア **全件 Ok (Ok 143 → 145)**、PR 16 の Excluded 2 件も
  解消。transform binary は Unmapped/Excluded ともに残り 0 の Ok 25

### PR 18: grayfill 整列 — filter 最大プールへの着手 (実施済み)

C 版ソース: `prog/grayfill_reg.c`、`src/seedfill.c`。

filter は Unmapped 57 で残り最大のプール。その中で grayfill_reg は
**入力が完全合成 (200x200 の pixCreate + 式で値を埋める)** で全 27 出力が
PNG のため、codec 差なしで gray seedfill 系を検証できる。

| C check | 内容 | 依存 |
| --- | --- | --- |
| 0-6 | seedfill_gray_inv (4/8 連結) + 閾値 + combine_masked + tiled | 済 |
| 7-12 | seedfill_gray (4/8 連結) + 閾値 + tiled | 済 |
| 13-18 | local_extrema + seedfill_gray_basin | **未** |
| 19-34 | 4 組の inv/正順 x simple 一致検証 | 済 |

**実装差 28 件目**: `local_extrema` は C `pixLocalExtrema` と
パラメータ意味論が異なる (Rust は erosion/dilation のカーネル径と
最小差分、C は 3x3 固定 + `pixQualifyLocalMinima` の閾値 maxmin/minmax)。
13-18 はこれの整列が前提のため次段送り。

実施結果:

- **実装差 29 件目**: `seedfill_gray_inv` が C と**別のアルゴリズム**
  だった。C `seedfillGrayInvLowSimple` は前後 2 方向の走査で
  「mask < 255 の画素について自身と走査済み近傍の最大値を取り、
  mask を超える場合のみ書き戻す」(mask は下側の障壁) のに対し、
  Rust は `max(seed, mask)` で初期化して最小値を伝播しており、
  結果が実質 mask になっていた。C 準拠に書き直し、
  `seedfill_gray_inv_simple` も同じ実装に委譲 (C 自身が両者の一致を
  reg test で保証しているため)
- 一方 `seedfill_gray` (正順) は初回から全件 Ok で、C と等価だった
- grayfill 21 ペア **全件 Ok (Ok 145 → 166)**。region binary は Ok 67

### PR 19: local_extrema の C 準拠化 (実施済み)

C 版ソース: `src/seedfill.c` (pixLocalExtrema / pixQualifyLocalMinima)。

PR 18 で記録した実装差 28 件目の解消。grayfill の C 13-18 が
これに依存している。

C `pixLocalExtrema(pixs, maxmin, minmax, &pixmin, &pixmax)`:

1. `pixErodeGray(pixs, 3, 3)` と `pixFindEqualValues` で候補を出す
   (3x3 固定。Rust はカーネル径を引数に取っていた)
2. `pixQualifyLocalMinima(pixs, pixmin, maxmin)` で候補成分を篩う:
   成分の代表値が `maxval` 超なら除去し、成分の**外周 1 画素**
   (dilate 3x3 と XOR で得る) がすべて代表値より大きくなければ除去
3. maxima は入力を反転して同じ処理 (閾値は `255 - minmax`)
4. `maxmin <= 0` は 254、`minmax <= 0` は 1 が既定

必要 API (`erode_gray` / `find_equal_values` / `conncomp_pixa` /
`dilate_brick` / `xor` / `next_on_pixel_in_raster`) は移植済み。

実施結果:

- 候補の篩い分けには**既存の公開 `qualify_local_minima`** をそのまま
  再利用できた (C と同仕様で移植済みだったが、`local_extrema` から
  呼ばれていなかった)
- 旧意味論に基づく unit テスト 2 件を C の実挙動に合わせて更新。
  平坦画像は「全体が 1 つの極小」(外周が画像外で反証されない) となり、
  maxima 側は反転後の 255 が閾値 254 を超えて全消去される
- grayfill が **27 ペア全件 Ok (Ok 166 → 172)**。grayfill_reg の全 PNG
  出力を完全制覇し、region binary は Ok 73

### PR 20: lineremoval 整列 — recog の lossless パイプライン (実施済み)

C 版ソース: `prog/lineremoval_reg.c`。

recog は Unmapped 45 で残る大きなプール。lineremoval_reg は入力が
`dave-orig.png` (lossless) の単一直線パイプラインで、全 10 出力が PNG。

閾値化 → skew 検出 → `rotate_am_gray` → gray close/erode/open →
`threshold_to_value` ×2 → 反転 → `arith_add` → `combine_masked` と、
gray morphology と算術の主要経路をまとめて検証できる。必要 API は
すべて移植済み (`find_skew` は `SkewDetectOptions` 経由)。

実施結果:

- skew 角は C と完全一致 (-0.656250) だったが `rotate_am_gray` の出力が
  3 画素だけずれた。追跡の結果 **実装差 30 件目**: C の
  `rotateAM{Gray,Color}Low` は `sina = 16.f * sin(angle)` を、sin() が
  double を返すため double 精度で計算し float に一度だけ丸める。
  Rust は f32 で三角関数を評価していた。area map 系カーネルが f64 の
  sin/cos を受け取るよう修正
- さらにテスト側の `deg2rad` も C は `3.14159 / 180.` を **double 除算**
  してから float に丸めており、f32 除算では sin が 1 ulp ずれる。
  C と同じ計算順に揃えて解消
- lineremoval 10 ペア **全件 Ok (Ok 172 → 182)**。recog binary は Ok 19

### PR 21: iomisc 整列 — alpha / colormap 変換系 (実施済み)

C 版ソース: `prog/iomisc_reg.c`。

io は Unmapped 41。iomisc_reg の PNG 出力は 8 件で、うち C check 13
(番号であって件数ではない) は既に Ok (`iomisc_regen_rgb_cmap`)。
残る 7 件が lossless 入力
(`books_logo.png` / `weasel4.11c.png` / `weasel4.5g.png`) 由来:

| C check | 内容 |
| --: | --- |
| 6 | alpha チャンネルの取り出し |
| 7 | `alpha_blend_uniform` (白背景) |
| 9 | `set_alpha_over_white` 後の alpha |
| 10 | `alpha_blend_uniform` (シアン背景) |
| 14 | `convert_rgb_to_colormap` |
| 15-16 | 8bpp cmapped の除去と `convert_gray_to_colormap` |

必要 API はすべて移植済み。

実施結果:

- **実装差 31 件目**: `convert_rgb_to_colormap` が常に 8bpp を返していた
  (C `pixFewColorsOctcubeQuant2` は色数で 2/4/8bpp を選ぶ)。C
  `pixConvertTo8Colormap` も 32bpp 入力をこれに委譲するため、
  `convert_to_8_colormap` は単色画像で 2bpp になるのが C 準拠
- **実装差 32 件目**: `set_alpha_over_white` が `255 - 輝度平均` の近似
  だった。C は距離変換ベース (反転 → RGB max → 閾値 → 反転 →
  `distance_function(8, 8, Foreground)` → x128) なので置き換え
- `alpha_blend_uniform` の丸めを C の切り捨てに合わせ、差分を
  4364 → 83 画素に削減
- iomisc 4 ペア Ok (Ok 182 → 186)
- **C 側の不整合を発見**: `pixAlphaBlendUniform` は白 x 白 (alpha 13) の
  ブレンドで 254 を返すが、公開ソースの式 `(1-f)*255 + f*255` は
  float/double いずれの評価でも 255。合成 1x1 入力でも再現し、C ソース
  からは説明できない。残る 83 画素はすべてこの形のため、checks 7/9/10 は
  理由付きで Excluded とした

**見送り**: `boxa3_reg` は `boxaDisplayTiled` のシグネチャが C と
大きく異なる (Rust は `(pixa, max_width)` のみ) ため、パラメータ整列が
前提。24 出力と規模も大きく別 PR とする。

### PR 22: boxaDisplayTiled の C 準拠化 (実施済み)

C 版ソース: `src/boxfunc4.c` (boxaDisplayTiled)、`prog/boxa3_reg.c`。

PR 21 で見送った `boxa3_reg` (24 出力) のブロッカー。C の
`boxaDisplayTiled(boxa, pixa, first, last, maxwidth, linewidth,
scalefactor, background, spacing, border)` に対し Rust は
`(pixa, max_width)` しか取らず、内部処理も異なる:

1. `boxaSaveValid` で無効 box を除去
2. `first`/`last` の範囲指定 (last < 0 は末尾)
3. scalefactor から fontsize を決める (0.8 超で 6、以下 10/14/18/20)
4. 各 box: 白背景 (または pixa の該当 pix) に 2px の青枠 →
   index を `add_single_textblock` で下に描画 → 赤の box を線幅
   `linewidth` で描画
5. `display_tiled_in_rows(32, maxwidth, scalefactor, background,
   spacing, border)` で合成

必要 API (`set_border_val` / `render_box_color` /
`add_single_textblock` / `display_tiled_in_rows`) は移植済み。

実施結果:

- `Boxa::display_tiled` を C シグネチャ・処理に書き換え (実装差 33 件目)
- boxa3 の **直列化 12 件が全件 byte 一致 (Ok 186 → 198)**。
  `transform_ordered` (= C boxaTransform)、
  `reconcile_size_by_median` の 3 種、`.ba` 直列化がいずれも
  C と bit 等価であることを実証
- **display 出力 12 件は次段送り**: タイル高が C と異なる
  (テキストブロック高の算出差、幅は一致)。boxa アルゴリズム自体は
  `.ba` で検証済みのため、理由付きで Excluded とした

### PR 23: colorcontent の RGB gamut 分類 (実施済み)

C 版ソース: `src/pix3.c` (pixMakeArbMaskFromRGB)、
`src/colorspace.c` (pixMakeGamutRGB)、`prog/colorcontent_reg.c`。

`colorcontent_reg` の 13 出力のうち check 0/1/5/8/9 は fish24.jpg・
wyom.jpg・map.057.jpg を入力とするため JPEG デコード差 (finding 001/008)
で bit 一致が原理的に不可能。一方 **check 10-17 の 8 件は入力画像を持たず**、
`pixMakeGamutRGB` で合成した RGB gamut を `pixMakeArbMaskFromRGB` で
分類するだけなので決定的に比較できる。

実施結果:

- 実装差 34 件目: `make_arb_mask_from_rgb` が f32 の重み付き和を
  そのまま閾値と比較していた。C は `pixConvertRGBToGrayArb` で
  8bpp gray 中間を作ってから `pixThresholdToBinary(pix1, thresh + 1)` +
  `pixInvert` するため、実際の判定は

  ```text
  clip(trunc(rc*R + gc*G + bc*B), 0, 255) >= trunc(thresh) + 1
  ```

  という整数意味論になる。例えば係数 (0.4, 0.3, 0.3)・閾値 60 で和が
  60.8 のとき、C は `trunc(60.8) = 60 < 61` で OFF、旧実装は
  `60.8 > 60.0` で ON となり乖離していた。切り捨て・[0,255] クリップ・
  閾値の整数化に加え、`thresh >= 255` を 254 にクランプする挙動と
  係数が全て非正のときのエラーも C に合わせた
- 実装差 35 件目 (レビュー指摘から派生): 既存の
  `convert_rgb_to_gray_arb` が `+ 0.5` で丸めていた。C
  `pixConvertRGBToGrayArb` は `val = (l_int32)(...)` で切り捨てるため
  同じ 60.8 が 61 になっていた。切り捨てに修正し、
  `make_arb_mask_from_rgb` を同関数経由に変更して量子化規約を 1 箇所に
  集約した (golden hash に変化なし)
- `Pix::make_gamut_rgb` (C pixMakeGamutRGB) を新規移植。32 個の
  32x32 サブ画像 (B 一定、R/G を 8 刻みで振る) を
  `display_tiled_in_columns(8, scale, 5, 0)` で並べる
- colorcontent の C check 10-17 を 8 ペアマップ — **全件即 Ok**
  (Ok 198 → 206)
- JPEG 入力側の 5 件は既存の finding 001/008 の範囲であり、
  今回は新規マップ対象外

### PR 24: grayquant の feyn.tif ブロック (実施済み)

C 版ソース: `src/paintcmap.c` (pixSetSelectCmap)、
`src/grayquant.c` (pixThresholdTo2bpp / pixThresholdTo4bpp /
makeGrayQuantIndexTable / makeGrayQuantTargetTable)、
`prog/grayquant_reg.c`。

`grayquant_reg` の 47 出力の大半は `test8.jpg` / `stampede2.jpg` 入力で
JPEG デコード差 (finding 001) の影響を受けるが、**check 28-39 の 12 件は
可逆な `feyn.tif` 入力**なので bit 一致比較ができる。

実施結果:

- 実装差 36 件目: `pix_set_select_cmap` が colormap のエントリ自体を
  上書きし、`region` を `let _ = region;` で捨てていた。C
  `pixSetSelectCmap` は新しい色を cmap から検索 (無ければ末尾に追加、
  既存エントリは不変) し、box 内の `old_index` の **ピクセル** だけを
  新 index に置き換える。box 外の同 index ピクセルは色が変わらない
- 実装差 37 件目: `threshold_to_2bpp` / `threshold_to_4bpp` の量子化
  テーブルが等幅バケット (`level = i / (256/nlevels)`) だった。C は
  `cmapflag` でテーブルを切り替える:

  - cmapflag: `makeGrayQuantIndexTable(nlevels)` — 閾値
    `255*(2j+1)/(2*nlevels-2)` による最近傍 index 割り当て
  - 非 cmapflag: `makeGrayQuantTargetTable(1<<d, d)` — `nlevels` を
    `2^depth` で上書きし、index ではなく量子化後のグレー **値** を格納

  `nlevels = 2` のときだけ両者が一致するため、これまで 2 レベルの
  テストだけが通っていた
- grayquant の C check 28-39 を 12 ペアマップ — **全件 Ok**
  (Ok 206 → 218)
- 量子化変更に伴い gquant_multi / pmask_clip / equal_8bpp_gray /
  writetext_multi / adaptnorm 系の golden を再生成 (いずれも Unmapped で
  C 側 Ok の退行なし)

### PR 25: checkerboard corner 検出 (実施済み)

C 版ソース: `src/checkerboard.c` (pixFindCheckerboardCorners /
makeCheckerboardCornerPixa)、`src/boxfunc2.c` (boxaExtractCorners)、
`prog/checkerboard_reg.c`。

`checkerboard_reg` は既に C の構造 (check 0/2/3/5) をそのまま写して
いたが、入力が可逆な `checkerboard1.tif` / `checkerboard2.tif` にも
かかわらず 4 件すべて Mismatch だった。

実施結果:

- 実装差 38 件目: corner 検出の hit-miss sel が象限全体を hit/miss で
  埋める独自構成だった。C `makeCheckerboardCornerPixa` は

  - 2 点 ((1,1) と (size-2, size-2)、cross 系は中央列の 2 点) を立てた
    1bpp マスクを dilation ブリックで膨張させたものを hit
  - 同マスクを 90 度時計回りに回転したものを miss
  - 残りは全て don't-care、原点は中心

  とする**疎な**構成で、対になる sel は hit/miss を入れ替える。
  `morph::dilate_brick` / `transform::rotate_90` で C の構成を再現した
- 実装差 39 件目: `Boxa::extract_corners(Center)` が
  `(left + right) as f32 / 2.0` と浮動小数で計算していた。C
  `boxaExtractCorners(L_BOX_CENTER)` は l_int32 の `(left + right) / 2`
  で、偶数幅の box では .5 にならず左上側へ切り捨てられる
- checkerboard の C check 0/2/3/5 を 4 ペアマップ — **全件 Ok**
  (Ok 218 → 222、Unmapped 400 → 396)
- **C check 1/4 (debug pixa の tiled display) は次段送り**:
  `selaDisplayInPix` / `selMakePlusSign` / `pixDisplaySelectedPixels` が
  未移植で、`find_checkerboard_corners` が中間画像を返さないため

### PR 26: paint の colormap 再構成 (実施済み)

C 版ソース: `src/paintcmap.c` (pixSetMaskedCmap)、`prog/paint_reg.c`。

`paint_reg` の入力は大半が JPEG (lucasta-frag.jpg / lucasta.150.jpg) だが、
末尾の **colormap 再構成ブロックは weasel2.4c.png / weasel4.11c.png /
weasel8.240c.png という可逆な cmapped PNG のみ**を使うため bit 一致比較が
できる。

実施結果:

- 実装差 40 件目: `pix_set_masked_cmap` が色を検索せずに必ず `add_color`
  し、失敗時は最近傍色へ黙ってフォールバックしていた。C
  `pixSetMaskedCmap` は `pixcmapGetIndex` で既存色を探して再利用し、
  無い場合のみ追加、空きが無ければ "no room in cmap" でエラーを返す
  (最近傍フォールバックは呼び出し側の責務)。深度 {2,4,8} の検証も追加。
  旧実装では `ReconstructByValue` のように既存 cmap を持つ pix を塗り直す
  ケースで重複エントリが積まれ index がずれていた
- paint の C check 24/26/28-31 を 6 ペアマップ — **全件 Ok**
  (Ok 222 → 228)
- **C ソースのコメント番号は実 index とずれている**: helper 内の
  `regTestComparePix` を数えていないため `/* 23 */` 〜 `/* 28 */` は実際には
  24/26/28〜31。C manifest に 23/25/27 が存在しないことで判明した。
  以後のマッピングでは manifest の実エントリを正とする
- **check 18-22 (feyn-fract.tif ブロック) は次段送り**: C
  `pixColorGrayRegions` / `pixColorGray` は 8bpp gray と cmapped を直接
  扱い boxa を取るのに対し、Rust 側は 32bpp 専用でシグネチャも異なるため、
  別 PR で C 準拠に書き換える必要がある

### PR 27: paint の feyn-fract ブロック (実施済み)

C 版ソース: `src/convolve.c` (pixConvolve / pixConvolveRGB)、
`src/grayquant.c` (pixThresholdOn8bpp)、`src/coloring.c` /
`src/paintcmap.c` (pixColorGray 系)、`prog/paint_reg.c`。

PR 26 で次段送りにした check 18-22。可逆な `feyn-fract.tif` を入力に
「ガウシアン畳み込み → 二値化 → 連結成分 → gray 領域の彩色」という連鎖を
通る。C 側に中間出力を書き出す dump プログラムを作り、段階ごとに
FNV-1a ハッシュを突き合わせて 3 箇所の乖離を切り分けた。

実施結果:

- 実装差 41 件目: `convolve` が C `pixConvolve` と別物だった。カーネル
  反転なし、正規化なし、境界が replicate (C は mirrored)、負の総和を
  0 クリップ (C は絶対値)、`outdepth` / `normflag` 引数なし。この段階で
  既に畳み込み結果が違い、連結成分数が C の 179 に対し 1360 だった。
  `convolve_color` は C `pixConvolveRGB` (成分ごとに outdepth 8 /
  normflag 1) に対応させた
- 実装差 42 件目: `threshold_on_8bpp` の量子化テーブルがビン中心方式
  だった。PR 24 と同じく C は `cmapflag` で
  `makeGrayQuantIndexTable(nlevels)` と
  `makeGrayQuantTargetTable(nlevels, 8)` を切り替える。colormap も
  `pixcmapCreateLinear` 相当の `i*255/(n-1)` に修正
- 実装差 43 件目: color_gray 系が 32bpp 専用だった。C は cmapped と
  8bpp gray を直接受け付け、`pixColorGrayRegions` は cmap に余裕が
  あれば cmapped のまま処理し、`PaintType` で式が変わり、閾値の境界は
  Light が `ave >= thresh` / Dark が `ave <= thresh`、Dark 側は
  `255.` が double リテラルのため倍精度評価、出力 alpha は 0
- paint の C check 18-22 を 5 ペアマップ — **全件 Ok** (Ok 228 → 233)
- convolve の mirrored 境界化に伴い colorize / gquant_adv /
  paint_cgray / convolve_custom_kernel の golden を再生成
  (いずれも Unmapped か Excluded で C 側 Ok の退行なし)
- `dreyfus8.png` は cmapped なので、C `pixConvolve` 同様 colormap 付き
  入力を拒否するようになった。テスト側で C の呼び出し順どおり
  colormap を外してから畳み込むよう修正した

### PR 28: filter 系の最初の整列 (実施済み)

C 版ソース: `src/convolve.c` (pixBlocksum / blocksumLow /
pixBlockconvAccum)、`src/adaptmap.c` (pixFillMapHoles)、
`src/enhance.c` (numaGammaTRC)、`prog/convolve_reg.c` /
`prog/adaptmap_reg.c`。

`filter` は Ok 2 件と最も未開拓なバイナリだった。lossless 入力を持つ
ブロックを探し、`convolve_reg` の check 2-4 (test1.png の
`pixBlockrank`) と `adaptmap_reg` の check 14-15 (weasel8.png と 3x3
合成マップの `pixFillMapHoles`) を対象にした。

実施結果:

- 実装差 44 件目: `blocksum` の正規化が 1 パスの f64 丸めだった。C
  `blocksumLow` は

  1. 全カーネル面積の `norm = 255/(fwc*fhc)` で正規化し、f32 の積を
     byte に切り捨てる
  2. 境界の行・列を、**切り捨て済みの byte** に対して `fhc/hn`・
     `fwc/wn` で再スケールし、また切り捨てる

  という 2 パス構成。理想値を 1 回丸めるのとは多くの画素で 1 ずれ、
  全 ON 画像でも角が 252 になる。accumulator も C 同様 1bpp を
  「ON 画素数」で積算するよう `blockconv_accum` を 1bpp 対応にした
- 実装差 45 件目: `fill_map_holes` が `filltype` を取らず
  `L_FILL_BLACK` 固定だった。C は
  `valtest = (filltype == L_FILL_WHITE) ? 255 : 0` で穴の値を切り替える。
  `MapFillType` を導入
- 実装差 46 件目: `gamma_trc` の LUT が
  `255. * powf(x, invgamma) + 0.5` を f32 で評価していた。C の `255.` は
  double リテラルのため倍精度評価になり、.5 境界に乗る値が 1 ずれる
  (maxval 270 で入力 153 が C 144 に対し 145)。**PR 27 の #41 / #43 と
  同種の「C の double リテラルによる評価精度」問題**で、この
  キャンペーンで繰り返し現れるパターン
- 5 ペアマップ — **全件 Ok** (Ok 233 → 238、filter binary の Ok が 2 → 7)

### PR 29 以降: semantic マッピングの漸進追加

Phase 3 と同じ進め方 (1 PR あたり 5〜20 ペア + 必要に応じて finding)。
優先順位はバイナリ別の未開拓度で決める:

| 優先 | binary | Unmapped | 現状 Ok | 備考 |
| --- | --- | --: | --: | --- |
| 1 | color | 114 | 0 | C 比較が全く無い最大の未開拓領域 |
| 2 | filter | 97 | 2 | 同上に近い。convolve/rank 系は lossless 出力が多い |
| 3 | transform | 78 | 4 | rotate/scale 系 |
| 4 | region | 72 | 0 | seedspread 6 件は finding 006 調査中 |
| 5 | io / recog / core | 130 | 8 | io は形式依存が強く個別判断 |
| - | morph | 9 | 30 | ほぼ完了。残りは低優先 |

各 PR の作業手順:

1. 対象 Rust テストと C prog (`reference/leptonica/prog/*_reg.c`) の出力
   順序を突き合わせ、`scripts/golden_map.tsv` にペアを追加
2. `cargo test --test <binary>` でレポートを再生成し、Ok / Mismatch を確認
3. 新規 Mismatch は root cause を調査して finding 化 (既知原因なら既存
   finding を参照)
4. C 版対応が存在しない Rust 出力は `prefix` ルールで
   `c_compat_exclude.tsv` に理由付きで追加

### 完了条件

- Unmapped のうち「マップ可能かつ未着手」が色・フィルタ系で解消され、
  残りが理由付き Excluded または調査中 finding に紐付く状態
- 数値目標は置かない (マッピングの副産物であるバグ発見が主目的のため)

## Impact

- テストインフラ (`tests/common/c_compat.rs`) と TSV データのみ。
  ライブラリ本体のコード・公開 API への影響なし
- CI Job Summary の表示列が 1 列増える
- ベースライン数値の意味が変わる (Unmapped = 「マップ可能な未着手」に純化)
