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

### PR 29: findpattern1 の全 20 出力 (実施済み)

C 版ソース: `src/selgen.c` (pixGenerateSelBoundary /
pixSubsampleBoundaryPixels / adjacentOnPixelInRaster /
pixDisplayHitMissSel)、`src/morphapp.c` (pixDisplayMatchedPattern)、
`src/pixafunc2.c` (pixaDisplayTiledAndScaled)、
`prog/findpattern1_reg.c`。

`findpattern1_reg` は tribune-word.png / tribune-t.png /
tribune-page-4x.png という可逆 PNG のみを入力とし、**20 出力すべて**が
bit 一致比較できる。C 側の sel パラメータ (pixp/sel の寸法、原点、
hit/miss 数) を dump して段階的に突き合わせた。

実施結果:

- 実装差 47 件目: `generate_sel_boundary` が C の
  `pixClipToForeground` を行わず、パディング量も `missdist`
  (C は `missdist + 1`) だった。pixp の寸法が食い違っていた
  (例: 254x74 に対し 255x74)
- 実装差 48 件目: 同関数が C の `ppixe` (境界拡張後のパターン画像) を
  返していなかった。戻り値を `(Sel, Pix)` にした
- 実装差 49 件目: 境界画素の追跡順が C の `adjacentOnPixelInRaster`
  と異なっていた。**サブサンプリングでどの画素が残るかは追跡順で決まる**
  ため hit/miss 集合がずれていた (65/101 に対し C は 68/84)
- 実装差 50 件目: `display_matched_pattern` が 32bpp を返していた。
  C は 4bpp cmapped を返し、`scale < 1` では
  `scale_to_gray` + `threshold_to_4bpp` を通り、オフセットを
  `(l_int32)(scale * offset)` で切り捨てる。`nlevels` 引数も欠けていた
- 実装差 51 件目: `display_tiled_and_scaled` が C と別実装だった。
  1bpp を縮小して深い出力にする際の `scale_to_gray` 経路がなく、
  **背景の判定が C と逆** (非 1bpp では `background == 0` が白)、
  `border > tilewidth / 5` の無効化もなかった
- C `pixDisplayHitMissSel` 相当の `display_hit_miss_sel` を新規移植
- findpattern1 の **全 20 出力をマップ — 全件 Ok** (Ok 238 → 258、
  recog binary の Ok が 19 → 39)
- `findpattern1_reg_display_and_remove` は独自パラメータ (フル解像度の
  パターンを 4x 縮小ページに当てる) で、sel が C 準拠になると一致が
  1 件も出ない無意味なテストだったため削除した。同じ C プログラムは
  `findpattern1_c_compat` が厳密に覆う

### PR 30: newspaper のセグメンテーション連鎖 (実施済み)

C 版ソース: `src/morphseq.c` (morphSequence の 'c')、`src/pix3.c`
(pixSubtract)、`src/pixafunc2.c` (pixaDisplayRandomCmap)、
`prog/newspaper_reg.c`。

`newspaper_reg` は可逆な `scots-frag.tif` だけを入力とし、13 出力のうち
check 0 (C が JPEG で書く) を除く 12 件が bit 一致比較できる。

実施結果:

- 実装差 52 件目: `morph_sequence` の `'c'` が `close_brick` を呼んで
  いた。C `morphSequence` は `pixCloseSafeBrick` を、DWA 版
  `morphSequenceDwa` は `pixCloseSafeCompBrick` を使う。境界を安全に
  扱うためのボーダー付加の有無で結果が変わり、`"c50.1 + c1.10"` の
  出力がずれていた
- 実装差 53 件目: `Pix::diff` (subtract / abs_diff) が寸法一致を要求して
  いた。C `pixSubtract` / `pixXor` は寸法差を**警告するだけ**で、UL 角で
  揃えた交差領域に `pixRasterop` を適用する。結果は `pixs1` の寸法を保ち、
  重なりの外側は `pixs1` のまま。newspaper では 1450x1600 から
  1448x1600 を引くため、この差でエラーになっていた
- C `pixaDisplayRandomCmap` 相当の `Pixa::display_random_cmap` を新規移植
- newspaper の C check 1-9 と 11 を **10 ペアマップ — 全件 Ok**
  (Ok 258 → 268、recog binary の Ok が 39 → 49)
- **check 10 と 12 は Excluded**: `pixcmapCreateRandom` の乱数色が
  colormap 展開 (スケーリング時の `pixConvertTo8Or32`) で画素値に入る
  ため原理的に比較不能。check 11 は cmapped のまま出力されるので、
  pixel hash が colormap を含まない本方式では比較できる

### PR 31: overlap の box 統合 (実施済み)

C 版ソース: `src/boxbasic.c` (boxaGetValidBox)、`src/boxfunc1.c`
(boxaCombineOverlaps / boxaCombineOverlapsInPair / boxCompareSize)、
`prog/overlap_reg.c`。

`overlap_reg` は**画像入力を一切持たず**、`srand(45617)` と glibc の
`rand()` だけで全ての box を生成する。テスト側で glibc の TYPE_3
加算フィードバック生成器を再現すれば、13 出力すべてが決定的に比較できる
(再現できていることは C の `rand()` 出力と直接照合して確認した)。

実施結果:

- 実装差 54 件目: `Box::is_valid` が `w >= 0 && h >= 0` だった。C
  `boxaGetValidBox` は `w <= 0 || h <= 0` を無効とするため、leptonica が
  無効化マーカーとして書く `(0, 0, 0, 0)` が「有効」と判定され、
  combine 系の圧縮が効いていなかった
- 実装差 55 件目: `combine_overlaps` が C と別アルゴリズムだった。C は
  各パスで、吸収した相手を `(0,0,0,0)` に置き換えてから `boxaSaveValid`
  でまとめて圧縮し、件数が変わらなくなるまで繰り返す。C の構造をそのまま
  再現し、debug pixa (処理前を赤、処理後を緑で同じフレームに重ね描き) を
  受け取る `combine_overlaps_debug` を追加した
- 実装差 56 件目: `combine_overlaps_in_pair` も別物だった。C は面積合計の
  大きい方に先手を与え、交差する組では **厳密に面積が大きい** box だけが
  相手を吸収する (同面積なら双方残る)。旧実装は `>=` で比較していたため、
  同サイズの組でも片方が消えていた
- 検証系: C-compat の候補拡張子に `dat` を追加し、
  `regTestWriteDataAndCheck` 由来の出力を照合できるようにした
- overlap の全 13 出力のうちファイル出力 10 件をマップ — **全件 Ok**
  (Ok 268 → 278、core binary の Ok が 13 → 23)

**知見**: C の reg プログラムが `rand()` で入力を作る場合でも、glibc の
生成器を再現すれば比較可能になる。同種の prog は他にもあるため、この
`GlibcRand` ヘルパは再利用できる。

### PR 32: scale の PNG 出力 (実施済み)

C 版ソース: `src/scale2.c` (pixScaleToGray3/4/6/8/16 と
scaleToGray16Low / makeValTabSG*)、`prog/scale_reg.c`。

`scale_reg` は 50 出力のうち大半を JPEG で書くが、1bpp ブロック
(check 0-5)、2/4bpp ブロック (20-22, 24-26, 28-30)、`scale_to_size`
(check 35) の **16 件は PNG かつ可逆入力のみ**を使うため bit 一致比較が
できる。

実施結果:

- 実装差 57 件目: `scale_to_gray_N` に出力幅の切り詰めが無かった。C は
  destination 幅を `pixScaleToGray3` / `pixScaleToGray6` で
  `(ws / n) & 0xfffffff8`、`pixScaleToGray4` で `& 0xfffffffe` と
  マスクする (8x / 16x は素の除算)。切り詰めた結果が 0 になる小さい入力は
  C ではエラーになる
- 実装差 58 件目: 16x の値式が違った。C `scaleToGray16Low` は値テーブルを
  使わず `sum = L_MIN(sum, 255); 255 - sum` と**生の黒画素数をクランプ**
  する。比例配分 (`255 - black*255/256`) ではないため、16x16 全黒が 0、
  黒 1 画素が 254 になる
- 不足していたテスト画像 `weasel4.png` / `graytext.png` を追加
- scale の PNG 出力 16 件をマップ — **全件 Ok** (Ok 278 → 294、
  transform binary の Ok が 29 → 45)

### PR 33: affine の可逆性ブロック (実施済み)

C 版ソース: `src/affine.c` (pixAffineSequential / pixAffineSampledPta /
affineXformSampledPt)、`src/pix3.c` (pixAnd / pixOr / pixXor)、
`prog/affine_reg.c`。

`affine_reg` の check 0-19 (sequential と sampled の可逆性ブロック) は
`feyn.tif` のみを入力とし、C も PNG で書くため bit 一致比較ができる。
C 側に中間出力と変換係数を書き出して段階的に突き合わせた。

実施結果:

- 実装差 59 件目: `affine_sequential` の内部スケーリングが
  `ScaleMethod::Linear` 固定だった。C は `pixScale` (汎用ディスパッチ) を
  呼ぶため、1bpp 入力では `pixScaleBinary` に流れる
- 実装差 60 件目: `affineXformSampledPt` 相当の丸めが floor だった。C は
  `(l_int32)(vc[0]*x + vc[1]*y + vc[2] + 0.5)` すなわち **0 方向への
  切り捨て**で、負の結果で 1 ずれ、ソース範囲外判定が変わる
- 実装差 61 件目: `Pix::and` / `or` / `xor` が寸法一致を要求していた。C の
  `pixAnd` / `pixOr` / `pixXor` は寸法差を警告するだけで、UL 角を揃えた
  交差領域に rasterop を適用する (**PR 30 で `pixSubtract` に入れたのと
  同じ規約**)。1bpp / 8bpp / 32bpp / その他深度の全経路に適用した
- affine の C check 0-19 を 20 ペアマップ — **全件 Ok** (Ok 294 → 314、
  transform binary の Ok が 45 → 65)

**知見**: 変換係数までは一致していたのに出力が違う場合、疑うべきは
(a) 丸め規約、(b) 内部で呼ぶ下位関数のディスパッチ、の 2 点。今回は
両方だった。C の係数を dump して先に一致を確認しておくと切り分けが速い。

### PR 34: projective / bilinear の可逆性ブロック (実施済み)

C 版ソース: `prog/projective_reg.c`、`prog/bilinear_reg.c`。

PR 33 で整列した affine と**同型のブロック**。`projective_reg` の
check 0-9 と `bilinear_reg` の check 0-6 は、いずれも `feyn.tif` のみを
入力とし C も PNG で書くため bit 一致比較ができる。

実施結果:

- **実装変更なしで 17 ペア全件 Ok** (Ok 314 → 331、transform binary の
  Ok が 65 → 82)。PR 33 までに入れたサンプル点の丸め (実装差 60) と
  rop の交差領域化 (実装差 61) がそのまま効いた
- bilinear の C 側ループは `for (i = 1; i < 3; i++)` で 2 点セットのみ
  使う点、residual を取る前に `pixInvert` を挟む点をテストで再現した

**知見**: 同型の C プログラムが複数ある場合、1 本を丁寧に整列させると
残りは実装変更なしで乗ることがある。affine → projective / bilinear が
その例で、投入コストに対する回収が大きい。次に狙うなら
`rotate1` / `rotate2` など同系統の残りが候補。

### PR 35: rotate1 の全 PNG 出力 (実施済み)

C 版ソース: `src/rotate.c` (pixRotate / pixEmbedForRotation /
pixRotateBySampling)、`src/rotateam.c` (pixRotateAMCorner)、
`src/rotateshear.c` (pixRotateShear / pixRotate2Shear / pixRotate3Shear)、
`src/pix2.c` (pixSetBlackOrWhite)、`prog/rotate1_reg.c`。

`rotate1_reg` の PNG 出力 32 件は `test1.png` / `weasel2.4c.png` /
`weasel4.11c.png` / `weasel4.16g.png` という可逆入力のみから作られる
(8/32bpp の 4 枚は JPEG)。

実施結果 (**このキャンペーン最多の 6 実装差**):

- 実装差 62 件目: `rotate()` が C `pixRotate` のパイプラインではなく
  独自実装だった。C は「メソッド上書き → area map なら cmap 除去 →
  埋め込みなしなら cmap に黒/白追加 → 埋め込み → area map かつ 8bpp 未満
  なら 8bpp 昇格 → 下位関数へ委譲」。既に C 準拠だった下位関数に委譲する
  形へ書き換えた
- 実装差 63 件目: 埋め込み判定の基準寸法が pix 自身の寸法だった。C は
  呼び出し側の width/height から maxside を出す。**繰り返し回転では C は
  常に原寸を渡す**ので、一度大きくなった後は追加の埋め込みが起きない。
  `RotateEmbed` で C の width/height を表現できるようにした
- 実装差 64 件目: メソッド上書きの条件が違った。C は 1bpp を常に
  shear/sampling に強制し、area map を浅い深度で sampling に落とさず
  8bpp に昇格させる。`LimitShearAngle` も 0.50 ではなく 0.35
- 実装差 65 件目: `rotate_am_corner` が 8bpp 未満をそのまま複製していた
- 実装差 66 件目: **cmapped 画像の背景色が生の極値だった**。C
  `pixSetBlackOrWhite` は colormap に黒/白を追加してその index で塗る。
  `set_black_or_white` を C 準拠にし、embed / sampling / shear の背景設定を
  これに統一した
- 実装差 67 件目: `rotate_shear` が独自実装だった。C は 2-shear / 3-shear の
  合成 (`|angle| <= MaxTwoShearAngle` で hshear+vshear、超えると
  vshear(a/2) + hshear(atan(sin a)) + vshear(a/2))
- rotate1 の PNG 出力 32 件をマップ — **全件 Ok** (Ok 331 → 363、
  transform binary の Ok が 82 → 114)

**知見**: 「上位のディスパッチ関数が独自実装で、下位の演算は既に C 準拠」
というパターンは、上位を C のパイプラインに置き換えるだけで一気に揃う。
cmapped 画像の背景色 (index か生値か) は横断的に効く論点で、他の
fill 系関数にも同じ確認が要る。

### PR 36: rotate2 の全 PNG 出力 (実施済み)

C 版ソース: `prog/rotate2_reg.c`。

PR 35 で整列した rotate1 と同系統。PNG 出力 8 件は同じ 4 枚の可逆入力から
作られ、1 枚あたり 2 出力 (shear の 8 変種 = 2 角度 x 2 fill x 埋め込み
有無、および sampling 4 変種 + area map 4 変種) を並べたもの。

実施結果:

- **実装変更なしで 8 ペア全件 Ok** (Ok 363 → 371、transform binary の
  Ok が 114 → 122)。PR 35 の `pixRotate` パイプライン化がそのまま効いた
- 1bpp では area map ブロックの前に `pixScaleToGray2` を挟む点、
  `L_BRING_IN_BLACK` と埋め込み無し (C の `0, 0`) の組み合わせも
  テストで再現した
- `rotateorth_reg` は `regTestComparePix` のみでファイル出力が無いため
  マップ対象が存在しない (C manifest のエントリも 0)

**次段送り**: `warper_reg` の 8 件 (feyn-word.tif、可逆) は C が
`srand(seed)` + glibc `rand()` で歪みパラメータを作る。PR 31 の
`GlibcRand` と同じ手が使えるが、ライブラリ側の乱数生成そのものを
glibc 互換にする必要があるため別 PR とする。

### PR 37: binmorph6 と skew の全 PNG 出力 (実施済み)

C 版ソース: `prog/binmorph6_reg.c`、`prog/skew_reg.c`。

どちらも入力は可逆 (`feyn-fract.tif` / `feyn.tif`) で、C の出力は全て
PNG なので pixel hash で完全比較できる。

実施結果:

- **binmorph6 は実装変更なしで 7 ペア全件 Ok**。`selCreateFromPix` で
  作った hit-only sel での dilate / open / close_safe を検証した
- **skew は 7 ペア全件 Ok**。ただし C 準拠にするために skew 探索の
  ほぼ全体を書き換えた (実装差 12 件)。index 2/4/5 が探索結果の角度に
  依存するため、アルゴリズムがずれていると hash が一致しない
- Ok 371 → 385 (morph 30 → 37、recog 49 → 56)

skew で見つかった主な実装差:

| 箇所 | 旧実装 | C |
| --- | --- | --- |
| center pivot | 中央 1/4 を切り出す | `pixVShearCenter` で支点を変えるだけ |
| sweep 用縮小 | 元画像から縮小 | search 用縮小画像をさらに縮小 |
| 縮小方法 | サブサンプリング | `pixReduceRankBinaryCascade` |
| シア | 画像を拡大 | 同サイズに書き込み、はみ出しは捨てる |
| スコア累算 | f64 | l_float32 (仮数部飽和で argmax が変わる) |
| confidence 分母 | sweep 分も含む | 二分探索のスコアのみ |
| confidence 閾値 | 元画像の寸法 | redsearch 縮小後の寸法 |
| sweep 端の最大 | 扱いなし | 警告して angle = conf = 0 |
| 直交探索 | `rotate_by_angle` + `+90` | `pixRotateOrth(1)` + `-90 + angle2` |
| sweep 単独版 | raw argmax | `numaFitMax` の放物線補間 |
| 差分二乗和 | `n < 2*nskip + 2` で 0 | 1 項だけでも加算。`0.05 * w` は double |
| deskew の回転 | embed して拡大 | `pixRotate(.., AREA_MAP, WHITE, 0, 0)` |

Rust 独自の `skew_reg` は index 5 (deskew 結果) の出力が変わるため
manifest を再生成した。

**次段送り**: 非 JPEG 入力で未マップかつ 5 件以上のプログラムのうち、
`watershed` (22 件) は C の `L_WSHED` 優先度キュー実装の移植、
`ptra1` (18 件) は `Ptra` 型と `lucasta.1.300.tif` の追加、
`ccbord` (14 件) は CCBORDA パイプライン、`circle` (13 件) は
`circles.pa` の pixa シリアライズ読み込みがそれぞれ必要。
`multitype` (17 件) は `test8.jpg` / `marge.jpg` を含むため原理的に
一致不可 (finding 001/008)。

### PR 38: findcorners の全 G4 TIFF 出力 (実施済み)

C 版ソース: `prog/findcorners_reg.c`。入力は `tickets.tif` のみ (lossless)
で、C の出力は全て G4 TIFF なので pixel hash で完全比較できる。

モルフォロジーでチケット領域を検出 → `pixFindSkew` でデスキュー →
再検出してクリップ、という流れなので、PR 37 で C 準拠にした skew 探索が
実データ上で検証される。

実施結果:

- **12 ペア全件 Ok** (Ok 385 → 397、recog 56 → 68)。9 件のデスキュー結果が
  すべて一致したのは PR 37 の直接の成果
- マッピングのために欠けていた C API を 2 つ追加した:

| 追加 | C 対応 | 内容 |
| --- | --- | --- |
| `Box::transform` / `Boxa::transform` | `boxTransform` / `boxaTransform` | shift → scale。`max(0, ...)` / `max(1, ...)` で切り捨て |

- `Boxa::select_by_size` が幅と高さの両方しか見ておらず、C の
  `L_SELECT_WIDTH` / `L_SELECT_HEIGHT` を表現できていなかった。
  `box_/select.rs` に既にあった `Boxa::make_size_indicator`
  (C `boxaMakeSizeIndicator` の完全な移植) と `SizeSelectType`
  (Width/Height/Either/Both) を使い、C と同じく indicator 経由で
  選択する形に直した
- 重複していた `region::SizeSelectRelation` (Gte/Lte のみ) を廃止し、
  C の 4 relation を持つ `core::SizeRelation` に一本化。
  `region::pix_select_by_size` も連結成分の bounding box から Boxa を作って
  `make_size_indicator` に通す
- `pageseg` にあった「Rust の `pix_select_by_size` は IfBoth/IfEither しか
  無いので conncomp で代替」という回避策のコメントを実態に合わせて修正

- `compfilter_reg` にあった代替コード (Width/Height が無いので Both +
  未使用側の閾値を常に満たす値にする、strict `>` を `Gte` + 閾値+1 で
  代替、Either を手書きフィルタで数える) を C の引数そのままに戻した

**教訓**: 「無い」と判断する前に同一モジュール配下を確認する。今回
`SizeSelectType` を新設しかけたが、`box_/select.rs` に同名・同義の型が
既にあった (レビューで指摘され統合)。

**次段送り**: `core::pixa::properties::SizeIndicatorAxis` が
`SizeSelectType` と同義でありながら `IfEither` / `IfBoth` しか持たない。
C `pixaMakeSizeIndicator` も 4 type を取るため、`SizeSelectType` への
統合が望ましい (本 PR のスコープ外)。

**次段送り**: 非 JPEG 入力で未マップかつ 5 件以上のプログラムのうち、
`watershed` (22 件) は C の `L_WSHED` 優先度キュー実装の移植、
`ptra1` (18 件) は `Ptra` 型、`ccbord` (14 件) は CCBORDA パイプライン、
`circle` (13 件) は `circles.pa` の pixa シリアライズ読み込みがそれぞれ
必要。`multitype` (17 件) は `test8.jpg` / `marge.jpg` を含むため原理的に
一致不可 (finding 001/008)。

### PR 39: ptra1 の全 PNG 出力 (実施済み)

C 版ソース: `prog/ptra1_reg.c`。入力は `lucasta.1.300.tif` のみ (lossless)
で、C の出力は全て PNG なので pixel hash で完全比較できる。

ページの連結成分を `L_PTRA` に載せ、insert / remove / swap / compaction の
組み合わせで並べ替えた結果を毎回 `pixaDisplay` で復元する。

実施結果:

- **18 ペア全件 Ok** (Ok 397 → 415、core 23 → 41)
- C `L_PTRA` に相当する型が無かったので `Ptra<T>` を新設した。穴を許す
  動的配列で、imax / nactual の管理、3 種の downshift、compaction 有無の
  remove を移した。`swap` だけは C の remove → replace → insert 経由を
  踏襲していない (後述)
- 単体テストで C の細かい挙動が 2 つ判明した:
  - `ptraAdd` の拡張判定は格納**前**の `imax >= nalloc - 1` なので、容量
    ちょうどまで詰めても拡張されない
  - `ptraInsert` は「穴が無い」を `imax + 1 == nactual` で判定するが、
    nactual は挿入分を先に加算済み。このため穴が 1 つだけのときは
    `L_MIN_DOWNSHIFT` を指定しても full downshift に落ちる
- `ptraSwap` は C では remove → replace → insert を経由するが、index1 が
  最後の占有スロットで index2 がその下の穴の並びより手前にあると、remove が
  imax を下げた結果 replace が範囲外を弾き、取り出した item が失われる。
  Rust では両スロットを直接交換し、末尾が空いたときだけ imax を下げる
  (C が正しく扱えるケースでは結果は同じ)
- あわせて `Pixa::display` / `Boxa::get_extent` の実装差を修正した:

| 箇所 | 旧実装 | C |
| --- | --- | --- |
| 空 pixa | 常にエラー | サイズ指定があれば空の 1bpp 画像 |
| canvas サイズ | 負の原点を補正した独自計算 | `boxaGetExtent` (補正なし) |
| box を持たない成分 | 原点に配置 | 警告して読み飛ばす |
| `boxaGetExtent` | 全 box を含める | 幅・高さが非正の box を除外 |

**次段送り**: 残る非 JPEG 入力の未マップは `watershed` (22 件、C の
`L_WSHED` 優先度キュー実装の移植)、`ccbord` (14 件、CCBORDA パイプライン)、
`circle` (13 件、`circles.pa` の pixa シリアライズ読み込み)、`jbclass`
(8 件、`pixaDisplayOnLattice` と `jbDataRender`)。`rectangle` (9 件) は
C ライブラリが `/tmp` に書いたデバッグ画像を検証対象にしているため
マップ不能。

### PR 40: circle の全 PNG 出力 (実施済み)

C 版ソース: `prog/circle_reg.c`。入力は `circles.pa` (可逆 PNG を収めた
pixa シリアライズ) のみで、C の出力は全て PNG なので pixel hash で完全
比較できる。

各円について外側を seedfill で塗り、円盤を 3x3 で段階的に収縮しながら
元画像との積の連結成分数を数え、断片化が収まる収縮量を選ぶ。

実施結果:

- **13 ペア全件 Ok** (Ok 415 → 428、transform 122 → 135)
- 唯一の障害だった `circles.pa` の読み込みで pixa シリアライズの実装差が
  見つかった:

| 箇所 | 旧実装 | C |
| --- | --- | --- |
| pix ヘッダ | `xres`, `yres`, `size` | `xres`, `yres` のみ |
| PNG の終端 | `size` から決定 | PNG ストリームの自己終端に委ねる |

  C `pixaWriteStream` は `size` を書かず、`pixaReadStream` は PNG
  デコーダにストリームを渡して終端を任せている。このため C が書いた
  `.pa` を読めず、Rust が書いた `.pa` も C から読めなかった。読む側は
  PNG のチャンクを IEND までたどる方式にし (`size` 付きも引き続き受理)、
  書く側は C と同じヘッダにした

- 形態学と seedfill 側の実装差はゼロ

**次段送り**: 残る非 JPEG 入力の未マップは `watershed` (22 件、C の
`L_WSHED` 優先度キュー実装の移植)、`ccbord` (14 件、CCBORDA
パイプライン)、`jbclass` (8 件、`pixaDisplayOnLattice` と
`jbDataRender`)、`warper` (8 件、glibc 互換乱数)。`rectangle` (9 件) は
C ライブラリが `/tmp` に書いたデバッグ画像を検証対象にしているため
マップ不能。

### PR 41: watershed の極値・シード段 (実施済み)

C 版ソース: `prog/watershed_reg.c`。`DoWatershed()` を 2 枚の合成画像
(500x500, 8bpp) に適用し、check 0-11 と 12-23 を出力する。全て PNG と
Numa なので pixel hash で比較できる。

check 7-11 / 19-23 は `L_WSHED` (優先度キューによる流域拡張) を必要と
するため PR 42 に回し、本 PR は極値検出とシード生成の段 (check 0-6 /
12-18) に絞る。対象は **12 ペア**。

| C check | 内容 | 必要な Rust API |
| --: | --- | --- |
| 0 | 合成画像そのもの | - |
| 1 | 極小=赤・極大=緑で塗った 32bpp | `local_extrema`, `paint_through_mask` |
| 2 | 極小マスク (2 画素境界クリア後) | `set_or_clear_border` |
| 3 | シード点 | `select_min_in_conncomp`, `pix_generate_from_pta` |
| 4 | シードを緑で塗った 32bpp | 同上 |
| 5 | シードが載らなかった極小成分 | `remove_seeded_components` |
| 6 | 5 が空であることの値比較 | `is_zero` |

実施結果:

- **12 ペア全件 Ok** (Ok 428 → 440、region 73 → 85)
- 実装差を 2 件発見。どちらも C 版バイナリを直接ビルドして中間出力を
  ダンプし、ビット単位で突き合わせて確認した

**(1) 合成画像の浮動小数点セマンティクス**

C は `l_float32 f` に double 式を代入・加算するため、項ごとに f32 へ
丸められる:

```c
f = 128.0 + 26.3 * sin(0.0438 * (l_float32)i);   /* double -> f32 */
f += 33.4 * cos(0.0712 * (l_float32)i);          /* f32 -> double -> f32 */
```

Rust 側は式全体を f32 で評価していたため、入力画像自体が C と一致
しなかった。項ごとに f32 へ丸めるよう修正。

**(2) `pixQualifyLocalMinima` の走査ガード**

C は外部境界の走査を次のループで行う:

```c
for (i = 0, y = yc - 1; i < hc + 2 && y >= 0 && y < h; i++, y++)
    for (j = 0, x = xc - 1; j < wc + 2 && x >= 0 && x < w; j++, x++)
```

`y >= 0` / `x >= 0` は**行や列をスキップするのではなくループを終了
させる**。したがって外接矩形が左端 (`xc == 0`) または上端 (`yc == 0`)
に接する連結成分は外部境界を 1 画素も検査されず、無条件に極小として
残る。Rust 側は範囲外を単にスキップしていたため、これらの成分を
過剰に棄却していた。閾値 (`maxval`) の判定はこの走査より前にあるため、
端に接していても値が大きすぎる成分は消える。

副次的に、plan 902 PR 19 で追加した `grayfill_local_extrema_matches_c`
の期待値が C 未検証の理想論 (「くぼみだけが極小」) だったことも判明した。
C の実出力は境界に接する平坦域も残す 16 画素で、修正後の実装と一致する。
期待値を C の実出力そのものに差し替えた。

**教訓**: 「C 準拠」を名乗るテストでも、期待値が C 版の実行結果ではなく
仕様の読み下しから書かれている場合がある。C 挙動の再現を疑うときは、
既存テストの期待値も含めて C 版バイナリで裏を取る。

**次段送り**: check 7-11 / 19-23 (10 ペア) は `L_WSHED` の移植が前提。

### PR 42: watershed の流域拡張段 — L_WSHED 移植 (実施済み)

PR 41 で送りにした check 7-11 / 19-23 (**10 ペア**) を対象にする。C の
`L_WSHED` (優先度キューによる流域拡張) の移植が前提。

移植対象 (`src/watershed.c`, 約 700 行):

| C 関数 | 役割 |
| --- | --- |
| `wshedCreate` | 32bpp ラベル画像を `MAX_LABEL_VALUE` (0x7fffffff) で初期化 |
| `wshedApply` | 優先度キューで低い値から充填し、盆地の衝突を解決 |
| `wshedSaveBasin` / `identifyWatershedBasin` | 確定した盆地を BFS で切り出す |
| `mergeLookup` | lut と backlink を正準形に保つ |
| `wshedGetHeight` | シード最小値からの高さ |
| `wshedRenderFill` / `wshedRenderColors` | 結果の描画 |

補助構造:

- `L_HEAP` (`heap.c`): f32 値で順序付ける二分ヒープ。`lheapSwapUp` /
  `lheapSwapDown` の実装が同値要素の順序を決めるため、逐語移植が必要
- `L_QUEUE` (`queue.c`): `identifyWatershedBasin` の BFS 用 FIFO

既存 API との関係: `watershed_segmentation` / `WatershedResult` は C に
対応物のない Rust 独自の便宜 API で、独自の充填アルゴリズムを持つ。本 PR
では触らず、`Wshed` を C の対応物として新設する。両者の統合は別途検討する
(現時点で統合すると `smoothedge_reg` を含む既存 golden が動くため)。

事前に判明している要修正点:

- `remove_seeded_components` に C の `bordersize` 引数がない。PR 41 の
  呼び出しでは入力が既に境界クリア済みだったため表面化しなかったが、
  `wshedApply` は境界クリアしていない `pixLocalExtrema` の出力に対して
  `bordersize = 2` で呼ぶため、この差が結果に出る

実施結果:

- **18/22 Ok** (Ok 440 → 446、region 85 → 91)。L_WSHED 本体の移植は正しく、
  盆地のランダム cmap (check 7/19)、レベルの Numa (8/20)、render_fill
  (9/21) がいずれもビット一致
- 実装差を 3 件発見:

**(1) `remove_seeded_components` の bordersize 欠落** (事前予測どおり)

**(2) Numa 直列化の末尾空行**

C `numaWriteStream()` は値の並びの後に必ず空行を出し、その後に任意の
`startx`/`delx` 行を書く。Rust 側は空行を出さず、代わりに `startx` 行の前に
だけ改行を付けていた。非デフォルトの場合は結果的に一致するが、デフォルト
では末尾の空行が欠ける。`.na` は生バイトをハッシュするため、これが
そのまま不一致になる。

**(3) `pixcmapCreateRandom` のグローバル乱数系列** (finding 010、PR 43 送り)

C は glibc の `rand()` を使い、これはプロセス全体で共有される 1 本の系列。
Rust は呼び出しごとに同じ LCG を初期化するため、2 回目以降の
`pixaDisplayRandomCmap` で色が食い違う。check 7/19 が一致するのは pixel
hash がカラーマップではなくインデックスを対象にするため。

C 忠実性の判断:

- C ヘッダが明記する既知の不具合 (重複した流域を見つけることがある) は
  修正せず再現した。12x12 の検証画像では C も Rust も同じ 1x1 の重複盆地を
  返す
- `wshedGetHeight` の `label >= nseeds` 分岐は C では `namh` を未シフトの
  `label` で引いており範囲外になる。`wshedApply` からは到達しないため、
  範囲外読みを再現せずエラーを返す形にした

**次段送り**: check 10/11/22/23 の 4 件は PR 43 (glibc 互換乱数を引数で
渡す API) で解消する。同じ仕組みは `warper_reg` の 8 件にも使える。

### PR 43: glibc 互換乱数で watershed を完遂 (実施済み)

finding 010 の解消。`watershed` の残り 4 ペア (C check 10/11/22/23) を Ok に
する。

C `pixcmapCreateRandom()` は glibc の `rand()` を呼ぶが、これは**プロセス
全体で共有される 1 本の系列**で、`srand()` が呼ばれなければ種は 1。
`watershed_reg.c` は `pixaDisplayRandomCmap` を 4 回実行し (各 762 個消費)、
それぞれ系列の異なる位置を使う。Rust 側は呼び出しごとに同じ LCG を
depth から初期化するため、2 回目以降で色が食い違う。

事前検証済み: 種 1 の glibc 系列から生成したカラーマップは、C の 1 回目
(check 7) と 3 回目 (check 19) のパレットと **256/256 完全一致**する。
2 回目の系列には C が check 10 で塗った色 (91, 3, 160) が index 16 に
存在する。

方針: グローバル可変状態は入れず、乱数源を引数で明示的に渡す。

| 追加 API | 対応する C |
| --- | --- |
| `core::GlibcRand` | glibc `rand()` (TYPE_3 additive-feedback) |
| `PixColormap::create_random_with` | `pixcmapCreateRandom()` |
| `Pixa::display_random_cmap_with` | `pixaDisplayRandomCmap()` |
| `Wshed::render_colors_with` | `wshedRenderColors()` |

引数なしの既存 API は種 1 の新しい系列に委譲する。C で `srand()` を呼ばず
最初に `rand()` を使った場合と一致するので、現行のアドホック LCG より
厳密になる。C の 1 本の系列を再現したいテストは `GlibcRand` を 1 つ作って
全ての呼び出しに渡す。

`tests/core/overlap_reg.rs` に同等の実装がテストローカルで存在するので、
ライブラリ側に移して重複を解消する。同じ仕組みは `warper_reg` (8 件、
`srand(seed)` + `rand()` で歪みパラメータを生成) にも使える。

実施結果:

- **watershed 22/22 全件 Ok** (Ok 446 → 450、region 91 → 95、
  Mismatch 33 → 29)。finding 010 を解消
- `tests/core/overlap_reg.rs` のテストローカル実装を削除し統合
- 引数なしの `create_random` / `display_random_cmap` は種 1 の系列に
  委譲するようにした。C の 1 回目の呼び出しと一致するため、アドホックな
  LCG を使っていた従来より厳密になる

**落とし穴**: C から期待値を採る際、3 つの `rand()` を `printf` の引数に
並べると gcc の右から左への評価順で出力が逆順になる。最初この形で採った
期待値がテストを誤らせた (実装は正しかった)。値を変数に受けてから表示する。

**再検討候補**: `newspaper` の Excluded 2 件は「`pixaDisplayRandomCmap` が
`rand()` の色を使う」ことを除外理由の 1 つに挙げている。乱数が再現可能に
なったので、残る理由 (スケーリング時の colormap 展開) だけで除外が妥当か
再確認する余地がある。

### PR 44: ccbord の境界追跡と再構成 (実施済み)

C 版ソース: `prog/ccbord_reg.c`。`RunCCBordTest()` を `feyn-fract.tif` と
`dreyfus1.png` に適用し、各 7 check (計 14) を出力する。全て PNG と
SVG 文字列なので比較できる。

C の `CCBORDA` パイプライン (`src/ccbord.c`、2578 行) 全体が必要なので、
3 PR に分ける。

| PR | C check | 内容 | ペア |
| --- | --- | --- | --: |
| 44 | 0,1,2 / 7,8,9 | 境界追跡・大域座標・ステップチェーン・再構成 | 6 |
| 45 | 3,4 / 10,11 | `.ccb` シリアライズの往復 | 4 |
| 46 | 5,6 / 12,13 | 単一パス境界と SVG 出力 | 4 |

本 PR (44) で移植する C 関数:

| C 関数 | 役割 |
| --- | --- |
| `pixGetAllCCBorders` / `pixGetCCBorders` | 連結成分ごとに外周と穴の境界を追う |
| `pixGetOuterBorder` / `pixGetHoleBorder` | 境界追跡本体 |
| `findNextBorderPixel` | 位置テーブル (`xpostab`/`ypostab`/`qpostab`) による次画素探索 |
| `locateOutsideSeedPixel` | 再構成用の外側シード決定 |
| `ccbaGenerateGlobalLocs` | 局所座標 → 大域座標 |
| `ccbaGenerateStepChains` | 局所座標 → ステップチェーン符号 |
| `ccbaStepChainsToPixCoords` | ステップチェーン → 局所/大域座標 |
| `ccbaDisplayBorder` | 大域座標の境界画素を描画 |
| `ccbaDisplayImage2` | 境界からの画像再構成 (seedfill) |

データ構造は C の `CCBORD` に合わせる (`boxa` / `start` / `local` /
`global` / `step`、後続 PR で `splocal` / `spglobal`)。

既存 API との関係: `src/region/ccbord.rs` の `Border` /
`ComponentBorders` は C に一対一対応のない Rust 独自 API。`Wshed` と同様、
C 対応物として `CcBorda` を別モジュールに新設し、既存 API は触らない。

必要な準備:

- `dreyfus1.png` をテストデータに追加する (`feyn-fract.tif` は既存)

実施結果:

- **6 ペア全件 Ok** (Ok 450 → 456、region 95 → 101)。`feyn-fract.tif`
  (464 成分) と `dreyfus1.png` (290 成分) の両方でビット一致
- 実装差は 1 件のみ:

**`next_on_pixel_in_raster` が開始画素を飛ばしていた**

C `nextOnPixelInRasterLow()` は走査を `(xstart, ystart)` から始めるため、
既に ON の画素から尋ねると同じ画素が返る。Rust 版は `start_x + 1` から
走査する仕様で、doc にも「開始画素は検査しない」と書かれ、既存テストも
その前提だった。

`pixGetOuterBorder()` は 1 画素境界を足した成分に対してこれを呼ぶ。1x1 の
成分では開始画素が唯一の前景画素なので、飛ばすと「開始画素が見つからない」
になる。`dreyfus1.png` には 1x1 成分が 8 個あり、そこで表面化した。

境界追跡そのもの (位置テーブルによる次画素探索、ステップチェーン、
seedfill による再構成) には実装差がなかった。

### PR 45: ccbord の `.ccb` シリアライズ往復 (実施済み)

C 版ソース: `prog/ccbord_reg.c` の check 3,4 / 10,11。PR 44 で作った
`CCBORDA` を `.ccb` ファイルに書き出して読み戻し、境界描画 (3,10) と
画像再構成 (4,11) をやり直す。

移植する C 関数:

| C 関数 | 役割 |
| --- | --- |
| `ccbaWriteStream` | ステップチェーン表現を直列化して zlib 圧縮 |
| `ccbaReadStream` | zlib 展開して直列化データを復元 |

`ccbaWrite` / `ccbaRead` はファイルを開閉するだけの薄い包みなので移植
しない。`std::fs::write(path, ccba.to_bytes()?)` /
`CcBorda::from_bytes(&std::fs::read(path)?)` で足りる。`RegionError` に
`Io` を足さずに済む利点もある。

**形式** (C の doc comment より、実測で確認済み):

```text
"ccba: %7d cc\n" を 18 バイト  (17 文字 + snprintf の NUL)
pix width   4B
pix height  4B
[成分ごと]
    ulx 4B / uly 4B / w 4B / h 4B     (w,h は復元に不要だが書かれる)
    境界数 nb 4B
    [境界ごと]
        startx 4B / starty 4B
        ステップ 2 個を 1 バイトに詰める (上位ニブルが先)
        終端 1B: n が奇数なら 0xz8 (z = 最後の値)、偶数なら 0x88
```

全体を zlib 圧縮する。整数は C が native order で書くため、x86 に
合わせてリトルエンディアンで実装する。

**復元されないもの**: `local` / `global` 座標と穴の bounding box
(`boxa` の index 1 以降)。読み戻した `CcBorda` は `boxa[0]` / `start` /
`step` だけを持ち、座標は `step_chains_to_pix_coords` で作り直す。
C の reg test がまさにその順序で呼ぶ。

**zlib 依存**: `miniz_oxide` は既に optional 依存 (`pdf-format` /
`ps-format`)。`.ccb` は 1 つのファイル形式なので、他の形式と同じ流儀で
`ccb-format` feature を足して `all-formats` に含める。テスト
`ccbord_c_compat` は feature ごと gate する (一部の check だけ落とすと
check 番号がずれて manifest が壊れるため)。`tests/core/pixa_select_to_pdf_reg.rs`
が `#![cfg(feature = "pdf-format")]` で同じことをしている前例。

**期待値**: C manifest では `ccbord.03 == ccbord.00`、`ccbord.04 ==
ccbord.02`、`ccbord.10 == ccbord.07`、`ccbord.11 == ccbord.09`。往復が
無損失なら自動的に一致するので、この 4 ペアは「往復でデータが落ちない」
ことの検証になる。

**RED に使う C 実測値** (`ccbaWrite` の出力を `zlibUncompress` して採取):

| 図形 | 成分 | ステップ列 | 終端バイト |
| --- | --- | --- | --- |
| 8x6 に 3x2 塊と孤立点 | 2 | `4 4 6 0 0 2` / なし | `44 60 02 88` / `88` |
| 9x9 の 5x5 リング | 1 (穴 1) | 外周 16 個 / 穴 12 個 | 各 `88` |
| 5x5 の L 字 3 画素 | 1 | `4 7 2` (奇数) | `47 28` |

実施結果:

- **4 ペア全件 Ok** (Ok 456 → 460、region 101 → 105)。実装差は 0 件
- `feyn-fract.tif` (464 成分、38272 バイト) と `dreyfus1.png`
  (290 成分、40987 バイト) で、C の `ccbaWrite()` が出す非圧縮ペイロード
  と Rust の `to_bytes()` がバイト完全一致することを確認した
- `ccbord_c_compat` の check が 2 つずつ増えたので、dreyfus1 側の Rust
  index が 4-6 から 6-10 にずれた。`golden_map.tsv` と manifest を更新

**C との意図的な差異** (いずれも rustdoc に記載):

- `to_bytes()` はステップチェーン未生成をエラーにする。C は黙って
  `ccbaGenerateStepChains()` を呼ぶが、同じモジュールの
  `step_chains_to_pix_coords` がエラーを返す方針なので揃えた
- `from_bytes()` は全読み出しを境界検査する。C の `ccbaReadStream()` は
  ファイル中の個数を信用して `memcpy` するため、切り詰められた入力で
  バッファ外を読む。個数からの事前確保もしない

### PR 46: ccbord の単一パス境界と SVG 出力 (実施済み)

C 版ソース: `prog/ccbord_reg.c` の check 5,6 / 12,13。穴を持つ成分の
境界を 1 本の閉 path にまとめ (`ccbaGenerateSinglePath`)、その大域座標を
描画 (5,12) して SVG 文字列を出力する (6,13)。

移植する C 関数:

| C 関数 | 役割 |
| --- | --- |
| `ccbaGenerateSinglePath` | 穴の境界を切断路で外周につなぎ 1 本にする |
| `getCutPathForHole` | 穴から外周への短い切断路を成分内部で探す |
| `ccbaGenerateSPGlobalLocs` | 単一パスを大域座標に変換 (全点 / 変曲点) |
| `ccbaDisplaySPBorder` | 単一パスの画素を描画 |
| `ccbaWriteSVGString` | polygon 要素の SVG 文字列を組む |

`ccbaWriteSVG` はファイルに書くだけなので PR 45 の `ccbaWrite` と同じく
移植しない。

**必要なデータ構造の追加**:

- `CcBord` に `splocal` / `spglobal` (`Pta`) を足す。C の `CCBORD` と同じ
- `CcBord` に `pix` (`Option<Pix>`) を足す。`getCutPathForHole` が成分の
  ビットマップを走査するため。C は `ccbCreate(pixs)` で保持している。
  `from_bytes` で読み戻した `CcBorda` は持たない (C の `ccbaRead` も
  復元しない) ので `Option`

`Pta` の `join` / `reverse` / `cyclic_perm` / `contains_pt` は core に
実装済みなのでそのまま使える。

**既存の Rust 独自実装との関係**: `region/ccbord.rs` にも
`generate_single_path` / `get_cut_path_for_hole` / `to_svg_string` がある
が、`get_cut_path_for_hole` が pix を取らないことから分かるとおり別の
アルゴリズムで、C 対応ではない。PR 44/45 と同じく `ccborda.rs` 側に新設
し、既存 API は触らない。

**check 6,13 の扱い**: C は `regTestWriteDataAndCheck(rp, svgstr,
strlen(svgstr), "ccb")` で SVG 文字列を `.ccb` 拡張子で書く。`.ccb` は
画像拡張子ではないので manifest は生バイトの FNV ハッシュになる
(`examples/gen_c_manifest.rs`)。Rust 側は `RegParams::write_data_and_check`
が同じ規則なので、文字列がバイト一致すれば Ok になる。

**RED に使う C 実測値** (`ring_and_dot` 図形):

- cc 0 (穴あり): `splocal` 35 点。切断路を往復するので `(3,0) (3,0)` の
  ように同じ点が連続する。`spglobal` (変曲点のみ) 17 点
- cc 1 (孤立点): `splocal` / `spglobal` とも 1 点
- SVG は 391 バイト。`</svg>` の後に空白 1 文字の行が付く
  (`sarrayToString` が各要素の後に改行を足すため)

実施結果:

- **4 ペア全件 Ok** (Ok 460 → 464、region 105 → 109)。実装差は 0 件。
  これで **ccbord は 14/14 全件 Ok**、C の `RunCCBordTest()` を全 7 check
  移植し終えた
- `feyn-fract.tif` (464 成分) と `dreyfus1.png` (290 成分) で、
  `splocal` / `spglobal` の点数 (47428/23104、10021/6115) と SVG 文字列
  (208621/63516 バイト) が C とバイト一致
- feyn-fract には切断路が見つからない穴が 16 個あるが、C と同じく
  パスから落とす形で一致した

**テストインフラの欠落を 1 件修正**: C は SVG 文字列を `.ccb` 拡張子で
書くが、`tests/common/c_compat.rs` の `CANDIDATE_C_EXTENSIONS` に `ccb`
が無く、正しい出力なのに `MissingC` になっていた。`.ccb` は画像拡張子
ではないので、C 側 manifest も Rust 側も生バイトの FNV ハッシュになる。

**C との意図的な差異**: `generate_sp_global_locs` は単一パス未生成を
エラーにする。C は黙って `ccbaGenerateSinglePath()` を呼ぶが、この
モジュールは生成段を全て明示する方針で統一している (PR 45 の
`to_bytes`、PR 44 の `step_chains_to_pix_coords` と同じ)。C の reg test
は `ccbaGenerateSinglePath` を先に呼ぶので、C 互換の呼び出し順は
そのまま通る。

**golden manifest の落とし穴**: check を途中に挿入して index の拡張子が
変わると (07 が png から ccb へ)、generate モードは同じキーを上書きする
だけなので旧 `ccbord_c.07.png` が取り残される。生成後は manifest の
diff で削除漏れがないか確認する。

### PR 47: jbclass の C 互換化 (実施済み)

C 版ソース: `prog/jbclass_reg.c`。`pageseg1.tif` / `pageseg4.tif` の上半分を
入力に、相関分類器 (check 0-3) と rank Hausdorff 分類器 (check 4-7) を
走らせる。計 8 check、全て TIFF G4 なので比較できる。

| C check | 内容 |
| --- | --- |
| 0 / 4 | テンプレート合成画像 (`jbDataSave` の `data->pix`) |
| 1,2 / 5,6 | テンプレートから再構成したページ (`jbDataRender`) |
| 3 / 7 | クラス別に並べた全インスタンス (`pixaDisplayTiledInColumns`) |

**現状**: Rust 側は `jbclass_haus` / `jbclass_corr` / `jbclass_wordmask` の
3 件が Unmapped。テストは「クラス数 > 0」等の緩い性質検査が中心で、
C との pixel 一致は見ていない。

**調査で判明した実装差** (C 版と Rust 版を同一入力で実測):

| 項目 | C | Rust | 影響 |
| --- | --- | --- | --- |
| `maxwidth` / `maxheight` 既定値 | 350 / 120 | 150 / 150 | 成分の取捨が変わる |
| テンプレート境界 `JB_ADDED_PIXELS` | 6 | `TEMPLATE_BORDER` = 4 | テンプレート寸法が全て違う |
| 相関しきい値 | `thresh + (1-thresh) * weight * area2/area` | `thresh + weight * fill_factor` | 式が別物 |
| テンプレート探索順 | `two_by_two_walk` の 25 手順 (近い順) | `dw`/`dh` の二重ループ | 貪欲一致の結果が変わる |
| 相関スコアの整列 | 重心差を四捨五入し `pix1` 側を走査 | 同左だが早期打ち切りなし | スコア自体は近いが `maxdiffw/h` 判定が無い |
| 分類結果 | nclass 1061 | nclass 2848 | 上記の複合結果 |

実測値 (相関、`pageseg1`+`pageseg4` の上半分、成分 5488 個):

- C: nclass=1061、lattice 87x130、composite 2784x4420
- Rust: nclass=2848、lattice 82x125、composite 4428x6625

**方針**: 差が広範なので 2 PR に分ける。

| PR | 内容 | ペア |
| --- | --- | --: |
| 47 | 分類器本体を C 準拠に直す (定数・しきい値式・探索順・スコア) | 0,4 |
| 48 | ページ再構成とインスタンス表示 (`jbGetULCorners` の最終位置合わせ含む) | 1,2,3 / 5,6,7 |

PR 47 で移植する C 関数:

| C 関数 | 役割 |
| --- | --- |
| `jbClassifyCorrelation` | 相関による貪欲分類 |
| `jbClassifyRankHaus` | rank Hausdorff による貪欲分類 |
| `findSimilarSizedTemplatesInit/Next` | `two_by_two_walk` による同寸法テンプレート探索 |
| `pixCorrelationScoreThresholded` | 早期打ち切り付き相関スコア |
| `jbDataSave` の lattice | `maxw+1` / `maxh+1` と `pixaDisplayOnLattice` の配置 |

**注意**: `TEMPLATE_BORDER` は公開定数なので、6 への変更は破壊的変更に
なる。C の `JB_ADDED_PIXELS` と名実を合わせる。

実施結果:

- **2 ペア全件 Ok** (Ok 464 → 466、recog 68 → 70)。テンプレート合成画像が
  相関・rank Hausdorff とも C と pixel 完全一致
- 実測 (`pageseg1`+`pageseg4` の上半分、成分 5488 個): 相関 nclass
  2848 → **1061 (C と一致)**、rank Hausdorff 1483 → **1036 (同)**。
  5488 成分すべてのクラス割当と 1061 個のテンプレート内容が一致

**最大の原因は成分の切り出し**だった。C の `pixConnCompPixa` は seedfill で
その成分の画素だけを取り出すが、Rust は bounding box で**ページを矩形
クロップ**していたため、隣接成分の画素が混入していた。テンプレートの
中身が違うので、以降の一致は原理的に不可能だった。

他に解消した差 (計画の表に加えて判明したもの):

- Hausdorff テストに重心整列が無かった。C の `pixRankHaustest` は重心差を
  丸めてずらしてから比較する。許容非被覆数の丸めも C に合わせた
- `pixHaustest` のサイズガード (`|wi-wt| > 2` で不一致) が無かった
- `add_page` がページ寸法を最大値で更新していた。C は最新ページの値

**構造的な問題も 1 件解消**: `add_page_components` が分類ロジックを
二重実装しており、しかもハッシュキーに境界込み寸法を使っていて
`classify_*` と食い違っていた。同じ関数に委譲するようにした。

**レイアウトの二重計算**: `templates_to_composite` を C の
`floor(sqrt(n))` 列に直したとき、`extract_templates` は
`ceil(sqrt(n))` のままで、正方数でないクラス数では別のセルを読んでいた
(nclass=501 で 22 列 vs 23 列、先頭 50 個中 28 個が別物)。C の
`pixaCreateFromPix` と同じく**合成画像の幅から列数を導く**ようにして、
ずれようがない形にした。レビュー指摘で発覚。

**残り**: check 1,2,3 / 5,6,7 (ページ再構成とインスタンス表示) は PR 48。
判明している要修正点:

- `jbGetULCorners` の最終位置合わせ (`finalPositioningForAlignment`) が
  未移植。C は 3x3 の範囲で XOR 画素数が最小になる位置を選ぶ
- `ptac` が境界なし成分の重心になっている (C は境界込み)。UL 座標の
  計算に効く
- `extract_templates` がセル全体を返す。C の `pixaCreateFromPix` は
  1bpp のとき `pixClipToForeground` で前景に切り詰めるので、配置される
  テンプレートの寸法が違う
- check 3,7 の `pixaDisplayTiledInColumns` と、テンプレートに白 3 +
  黒 1 の枠を付ける `PixaOutlineTemplates` が未移植

### PR 48: jbclass のページ再構成とインスタンス表示 (実施済み)

C 版ソース: `prog/jbclass_reg.c` の check 1,2,3 / 5,6,7。PR 47 で分類器が
C 一致になったので、テンプレートからページを再構成する段 (1,2 / 5,6) と、
クラス別に全インスタンスを並べる段 (3 / 7) を合わせる。

移植する C 関数:

| C 関数 | 役割 |
| --- | --- |
| `jbGetULCorners` | 各インスタンスの配置位置を重心差から決める |
| `finalPositioningForAlignment` | 3x3 の範囲で XOR 画素数が最小の位置を選ぶ |
| `pixaCreateFromPix` | 合成画像を格子で切り出す (1bpp は前景に切り詰め) |
| `pixaaFlattenToPixa` | クラス別インスタンス配列を平坦化 |
| `PixaOutlineTemplates` | 各クラス先頭に白 3 + 黒 1 の枠を付ける (reg test 側) |
| `pixaDisplayTiledInColumns` | 40 列・間隔 10 で並べる |

**要修正点** (PR 47 の調査で判明):

- `ptac` が境界なし成分の重心になっている。C は境界込み
  (`JB_ADDED_PIXELS` を足した画像) の重心を使う。UL 座標に直接効く
- `extract_templates` が格子セル全体を返す。C は 1bpp のとき
  `pixClipToForeground` で前景に切り詰めるので、配置されるテンプレートの
  寸法が違う
- `finalPositioningForAlignment` が未移植。UL 座標が重心差だけで決まって
  いる

**`finalPositioningForAlignment` の注意点**: 切り出し矩形
`(x - idelx - 6, y - idely - 6, w, h)` は画像外にはみ出しうる。C の
`pixClipRectangle` は矩形を画像に切り詰めて**小さい pix を返す**ので、
続く XOR も切り詰められた枠の中で行われる。画像端の成分では、この効果で
選ばれる位置が変わる。

**RED に使う C 実測値** (PR 47 と同じ 4 成分の fixture、相関):

- `ptac` (境界込み重心): `(8,9) (8,9) (8,9) (7,8)`
- `ptaul`: `(3,6) (22,6) (40,6) (58,6)`。成分 0 は元が x=4 なのに x=3 に
  なる (上記の画像端の効果)
- 合成画像から切り出したテンプレート: `5x7 / 5x7 / 3x5` (格子セルの
  18x20 ではない)

実施結果:

- **6 ペア全件 Ok** (Ok 466 → 472、recog 70 → 76)。これで **jbclass は
  8/8 全件 Ok**、C の `jbclass_reg.c` を全 check 移植し終えた
- 相関・rank Hausdorff とも、再構成ページ 2 枚とインスタンス表示が
  C と pixel 完全一致

計画の 3 点に加えて解消した差:

- `keep_pixaa` の既定が `false` だった。C の `jbRankHausInit` /
  `jbCorrelationInit` はどちらも 1 にする (インスタンス表示に必要)

**C の `ptaJoin` が重心を丸める**: C は `ptaJoin()` で `ptac` に重心を
追加するが、この関数は内部で `ptaGetIPt()` を通すため**値が整数に
丸められる**。一方 `ptact` (テンプレート重心) は `ptaAddPt()` 直接なので
小数のまま残る。この非対称が UL 座標に効く。

見つけ方: 相関は全一致したのに rank Hausdorff の page1 だけ不一致で、
`ptaul` の y 合計が 2 ずれていた。成分ごとに突き合わせると 5488 個中
2 個だけ配置が 1 画素ずれており、C の `ptac` が (14.000000, 22.000000)
と**整数ちょうど**なのに対し、同じ成分を C 内で直接 `pixCentroid` すると
(14.361702, 22.446808) になった。丸めは 3x3 の最終位置合わせがたいてい
吸収するので、窓の端に最適解が来た 2 個だけ表面化していた。

### PR 49: warper の C 互換化 (実施済み)

C 版ソース: `prog/warper_reg.c`。`feyn-word.tif` に 25 画素の枠を付けた
245x106 の 8bpp 画像を入力に、8 check を出力する。全て PNG なので比較できる。

| C check | 内容 |
| --- | --- |
| 0-3 | `pixRandomHarmonicWarp` を 4 通りのパラメータで 50 枚、色付けして並べる |
| 4-7 | `pixSimpleCaptcha` を nterms 1-4 で 50 枚、同様に並べる |

**乱数の扱い**: `pixRandomHarmonicWarp` は先頭で `srand(seed)` を呼び、
`generateRandomNumberArray(5 * (nx + ny))` で `rand()` を消費する。
reg test 側は各画像の直後 (captcha は直前) に色決定で `rand()` を 3 回
使う。`srand` が毎回呼ばれるので系列は完全に決定的。

**実装差** (実測):

| 項目 | C | Rust |
| --- | --- | --- |
| 乱数 | glibc `rand()` (`srand(seed)`) | 独自 LCG `SimpleRng` |
| 値の作り方 | `0.5 * (1 + rand() / RAND_MAX)` | `0.5 * (1 + next() / u64::MAX)` |

`GlibcRand` は plan 902 PR 43 (#451) で移植済みなので、`SimpleRng` を
置き換えるだけで系列が一致するはず。

**色決定の評価順**: C の
`((rand() >> 16) & 0xff) << L_RED_SHIFT | ... << L_GREEN_SHIFT | ... << L_BLUE_SHIFT`
は 3 つの `rand()` の評価順が未規定。手元の `cc` では左から右
(1 番目が R) だったが、**リファレンスビルドのコンパイラと一致する保証は
ない**。C manifest のハッシュと突き合わせて確定させる。

**必要な部品**:

- `pixColorizeGray` (色付け) の移植状況を確認する
- `pixaDisplayTiledInColumns(pixac, 10, 1.0, 20, 0)` は core に実装済み

実施結果:

- **8 ペア全件 Ok** (Ok 472 → 480、transform 135 → 143)。warp 400 枚の
  画素・色付け・タイル配置がすべて C と一致
- **色決定の評価順は左から右**と確定。リファレンスビルドの出力から実際の
  色 (108,140,58) を読み出し、逐次の 1,2,3 番目と一致することを確認した

解消した実装差:

| 箇所 | 内容 |
| --- | --- |
| 乱数 | 独自 LCG `SimpleRng` → `GlibcRand` |
| `twopi` | `2*PI` → C の切り詰めリテラル `6.283185` |
| 補間 | f32 双一次 + 四捨五入 → C の 1/16 量子化 + 整数演算 + 切り捨て |
| `simple_captcha` | 枠の値と順序、パラメータ表、色付けの欠落 |

**C の行ストライドの癖**: `linearInterpolatePixelGray` は最終行で
`wpls` を 0 にするが、これが `lines = datas + yp * wpls` より前にあるため
基準行のポインタごと画像先頭に潰れる。下端では行 `yp` ではなく**行 0 を
2 回**読む。全 warp 画像の下端はこれで作られている。

**`GlibcRand` の seed 0 バグを発見**: glibc は種 0 を 1 に読み替えるが
(そうしないと Lehmer 段が 0 しか生まない)、PR 43 の移植でこれが漏れて
いて `srand(0)` が常に 0 を返していた。warper が seed 0 を使うため
表面化した。

### PR 50: color 領域の棚卸しと alphaops のマッピング (実施済み)

`color` binary は Unmapped 108 件で最大の未開拓領域。C 側の reg test と
入力形式を突き合わせて棚卸しした。

**調査でやり直した点**: 最初 Rust の prefix から C のテスト名を機械的に
推測して「C 対応なし 23 件」と分類したが、これは誤りだった。Rust 側は
`gquant_*` / `pmask_*` / `bw_*` と略しているのに対し、C は
`grayquant` / `paintmask` / `blackwhite` という名前で、単純な前方一致では
見つからない。テストファイルの由来を 1 件ずつ確認して訂正した。

**訂正後の分類**:

| 区分 | 内訳 |
| --- | --- |
| JPEG 入出力でマップ不能 | blend1-5、binarize、colorfill、colorize、paint の残り、colorspace、colorcontent の残り、cmapquant、coloring、dither、hardlight、threshnorm、colorseg、colorquant、`blackwhite` (11 枚のタイル合成に `marge.jpg` を含む) |
| マップ可能・本 PR | `alphaops` の check 0,1,3,4 (4 件) |
| マップ可能・次 PR | `grayquant` の check 28 以降 (22 件、入力は `feyn.tif`)、`paintmask` の PNG 出力 (入力 `feyn.tif` / `rabi.png` の分) |

代表例:

- `colorcontent` は既に 8 件 Ok。残る C 側 00/01/05/08/09 は
  `fish24.jpg` / `wyom.jpg` / `map.057.jpg` が入力。C のソース自身が
  「jpeg 展開の丸めで色数が数 % 変わる」と注記している
- `colorspace` は 24 check あるが 1-9 が `IFF_JFIF_JPEG` 出力で
  入力も `wyom.jpg`
- `paint` は 11 件マップ済み。残る PNG 出力 02-09/11/13-17 は入力が
  `lucasta-frag.jpg`
- `blend3` は入力を読まないように見えるが、ヘルパー内で `marge.jpg` /
  `test8.jpg` を読み JPEG で書く

実施結果:

- **4 ペア全件 Ok** (Ok 480 → 484、color 56 → 60)。さらに誤った
  Excluded の撤回で io が 3 件増え、最終的に **Ok 487**
- `alphaops` の check 0 は入力をそのまま書き出すだけなので、PNG の
  読み書きが C と一致することの検査にもなっている

**実装差を 1 件解消 — 混在精度の再現**:

C の `pixBlendWithGrayMask` は
`(l_int32)((1.0 - fract) * dval + fract * sval)` と書くが、**2 つの積は
同じ幅で評価されない**。`1.0` が double リテラルなので前者は double 乗算、
後者は `float * int` で float に丸めてから加算される。

`books_logo.png` を白に合成すると 30360 画素中 83 画素がこれで変わる。
例えばアルファ 13 の白画素では、C は `fract * sval` を float で丸めて
ちょうど 13 を得るため合計が 254.99999982 となり切り捨てで 254。両方を
f64 で計算すると 255.0 ちょうどになり 255 になる。

この修正で既存の `alphaops_uniform` / `blend4_offset` の出力も変わり、
`alphaops_uniform.03` は **C のハッシュと一致する値**になった。

**誤った Excluded を 1 件撤回**: plan 902 PR 21 は `iomisc_c.02/03/04` を
「C の `pixAlphaBlendUniform` が自身の公開式から逸脱する。合成 1x1 入力でも
再現し C ソースからは説明できない」として Excluded にしていた。そこで
観測された「白 x 白 (アルファ 13) が 254 になる」現象は、まさに上記の
混在精度が原因だった。式は公開どおりで、**評価される幅が項ごとに違う**
というのが答え。3 件とも Ok になったので除外ルールを削除した
(Excluded 100 → 97)。

### PR 51: color の再調査と snap_color の C 準拠化 (実施済み)

PR 50 で「次 PR 候補」とした `grayquant` を調べたところ、**既に
`grayquant_c` として 12 件すべて Ok 済み**だった。PR 50 の見積もり
(「22 件がマップ可能」) は二重に誤っていた:

- 件数: C の check 28 以降を数えたが、40-49 は `stampede2.jpg` 入力で
  対象外。lossless (`feyn.tif`) なのは 28-39 の 12 件
- 状態: その 12 件は既にマッピング済みで Ok

`paintmask` も同様で、lossless (`feyn.tif` / `rabi.png`) の check 19-21 は
`pmask_1bpp` として Ok 済み。残る 02-18 は `test24.jpg` 入力。

**color 全 22 テストの入力を機械的に洗い直した結果**、未マップで
lossless 入力のものは **`blend5` の check 2,3 だけ**だった:

| C check | 入力 | 内容 |
| --- | --- | --- |
| 2 | `google-searchbox.png` | 白 (`0xffffff00`) を黄 (`0xffffe400`) に snap、diff 30 |
| 3 | `weasel4.11c.png` | `0xfefefe00` を `0x80800000` に snap、diff 50 |

**実装差**: C の `pixSnapColor(pixd, pixs, srcval, dstval, diff)` は
**src 色と dst 色を別に取る**が、Rust の `snap_color_cmap(pix, target, diff)`
は 1 色しか取らず「target に近い色を target 自身に潰す」別物になっている。
C 準拠にするには src/dst を分離する必要がある。

さらに C の `pixSnapColorCmap` は colormap に空きがあるかで挙動を変える:

- 空きあり → dst 色を**追加**して新しい index を使う
- 空きなし → src に近い既存 entry を 1 つ**乗っ取って** dst 色に書き換える

`google-searchbox.png` は 256 色すべて使用済み (C のコメントに明記) なので
後者の経路を通る。両方の経路を移植する必要がある。

**本 PR でやること**:

1. `snap_color_cmap` を C 準拠にする (src/dst 分離、空き有無の分岐)
2. colormap を持たない 8bpp / 32bpp 向けの `snap_color` も移植する
3. `blend5` の check 2,3 をマッピングする

実施結果:

- **2 ペア全件 Ok** (Ok 487 → 489、color 60 → 62)
- `snap_color_cmap` を C 準拠にした。src/dst を分離し、colormap の空き
  有無で経路を分け、最後に未使用色を除去する。`pix_snap_color` は
  colormap 付き入力をこちらに委譲する (C と同じ)
- `google-searchbox.png` (256 色・空きなし = 乗っ取り経路) と
  `weasel4.11c.png` (11 色・空きあり = 追加経路) の両方で C と pixel
  完全一致

**color 領域の結論**: マップ可能な pair は出し切った。残る Unmapped
108 件は、C 側テストが JPEG 入力かつその Rust 側テストが独自に追加した
出力で、原理的に pixel 一致しない。件数を減らすなら
`c_compat_exclude.tsv` への移送になるが、prefix 単位では
`grayquant_c` / `pmask_1bpp` / `blend5_c` のように同じテストファイル内に
Ok のものが混在するため、`key` 単位で 108 行書く必要がある。費用対効果を
見て別途判断する。

### PR 52: shear1 のマッピング (実施済み)

`transform` binary は Unmapped 74 件。棚卸ししたところ、大半は既に
マップ済みか JPEG 縛りだった:

| C テスト | C の PNG 出力 | マップ済み | 残り |
| --- | --: | --: | --- |
| `scale` | 17 | 16 | 49 のみ |
| `rotate1` | 32 | 32 | なし |
| `rotate2` | 8 | 8 | なし |
| `shear2` / `translate` / `smallpix` | 4 / 3 / 9 | 同数 | なし |
| `affine` | 33 | 20 | 13 |
| **`shear1`** | **6** | **0** | **6 件すべて** |

`shear1` は 6 件とも未着手で、しかも**入力が全て lossless**
(`test1.png` / `weasel2.4c.png` / `weasel4.11c.png` / `weasel4.16g.png` /
`dreyfus8.png`)。JPEG 出力の check 4/6/7 だけが対象外。

| C check | 入力 | 深度 |
| --- | --- | --- |
| 0 | `test1.png` | 1bpp |
| 1 | `weasel2.4c.png` | 2bpp cmap (満杯) |
| 2 | `weasel4.11c.png` | 4bpp cmap (空きあり) |
| 3 | `weasel4.16g.png` | 4bpp cmap (満杯) |
| 5 | `dreyfus8.png` | 8bpp cmap |
| 12 | `weasel4.11c.png` | 4bpp cmap、in-place 系 |

C の `shearTest1()` は入力ごとに以下を組み合わせて 4 列に並べる:

- `pixHShear` / `pixVShear` を yloc/xloc 2 通り x fill 2 通り
- colormap なしの場合のみ `pixHShearIP` / `pixVShearIP` を同 4 通り
- 8bpp / 32bpp / colormap 付きの場合のみ `pixHShearLI` / `pixVShearLI`
  を同 4 通り

check 1 は**入力の colormap を書き換えてから**シアーする点に注意
(黒 (40,44,40) を暗赤 (100,0,0) に。満杯の colormap では
`L_BRING_IN_BLACK` が黒を引き込めないことを見せるため)。

使う Rust API はいずれも実装済み: `h_shear` / `v_shear` /
`h_shear_ip` / `v_shear_ip` / `h_shear_li` / `v_shear_li`。
角度は `ANGLE1 = 3.14159265 / 12`。

実施結果:

- **5 ペア全件 Ok** (Ok 489 → 494、transform 143 → 148)

**実装差を 2 件解消** (どちらも `pixHShearLI` / `pixVShearLI`):

1. **サブピクセル位置の整数化**: `x >> 6` を使っていたが C は `x / 64`
   で 0 方向に切り捨てる。`x` が -1..-63 のとき C は列 0 を読むのに
   Rust は -1 になって画素をスキップし、画像の左端・上端に充填色が
   残っていた
2. **32bpp の白充填**: `0xFFFFFF00` だった。C は `pixSetAll` で全ビットを
   立てるのでアルファも `0xff` になる

`weasel4.11c.png` に対する 8 通りの shear 演算すべてで C と pixel 完全
一致することを確認した。

**C の未初期化 index の落とし穴**: check 1 は colormap から
`(40, 44, 40)` を探して暗赤に塗り替えるが、`weasel2.4c.png` にその色は
無く `pixcmapGetIndex` は失敗する (近黒は `(48, 44, 40)`)。C は戻り値を
無視し、この関数が出力を 0 に初期化してから探索するため **index 0 が
塗り替えられる**。たまたま目的の近黒エントリなので意図どおり動いている。
テストは探索ではなく index を再現する。

shear の補間修正により `rotate1_shear` / `warper_stereo` の出力も変わる
(いずれも Unmapped な Rust 独自出力)。

### PR 53: affine のマッピング (実施済み)

PR 52 の棚卸しで残った `transform` の未着手分。`affine` は C 側 PNG 出力
33 件のうち 0-19 がマップ済みで、**40-52 の 13 件が未着手**。入力は
`feyn.tif` / `lucasta.1.300.tif` でいずれも lossless。

| C check | 内容 | 入力 |
| --- | --- | --- |
| 40-43 | 逐次変換 vs サンプリング変換の比較 (0.22 倍縮小) | `feyn.tif` |
| 44-49 | 大きな歪みでの逐次 / サンプリング / 補間の比較 | `feyn.tif` |
| 50-52 | boxa への affine 変換と逆変換 | `lucasta.1.300.tif` |

**本 PR は 40-49 の 10 件に絞る**。50-52 は
`createMatrix2dTranslate` / `createMatrix2dScale` / `createMatrix2dRotate` /
`l_productMat3` / `affineInvertXform` / `pixAffine` が未移植で、範囲が
別物になるため。

使う C 関数 (Rust 実装は確認済み):

| C 関数 | Rust |
| --- | --- |
| `pixAffineSequential` | `transform::affine_sequential` |
| `pixAffineSampledPta` | `transform::affine_sampled_pta` |
| `pixAffinePta` | `transform::affine_pta` |
| `pixScaleToGray6` | `transform::scale_to_gray_6` |
| `pixXor` / `pixInvert` | `Pix::xor` / `Pix::invert` |

**注意点**:

- check 40 は `pixAffineSequential` に `ADDED_BORDER_PIXELS = 1000` を
  border として渡す
- 対応点は C の配列から取る。40-43 は index 3、44-49 は index 4
- check 42/47/48 は XOR による差分画像で、**両辺が一致していないと
  意味を持たない**。実装差があればここで真っ先に出る

実施結果:

- **10 ペア全件 Ok** (Ok 494 → 504、transform 148 → 158)

**実装差を 1 件解消 — 同じ C 関数の二重移植**:

`affine_gray` が独自の面積重み付けを持っており、C の
`linearInterpolatePixelGray` と 3 点食い違っていた:

- 端の扱い: C は `xp+1` が幅を超えると自身に折り返し、最終行では
  行ストライドごと画像先頭に潰れる (PR 49 の調査で判明した癖)。Rust は
  `xp > w-2` で充填色のまま残していた
- 丸め: C は `(v00+v01+v10+v11) / 256` で切り捨てるが `+128` して
  四捨五入していた
- サブピクセル位置: C は `x` を求めてから 16 倍するが、行列係数を先に
  16 倍していた

これは **PR 49 で warper 向けに C 準拠へ直したのと同じ関数**だった。
`linear_interpolate_gray` を `pub(crate)` にして共有し、同じ C 関数の
移植を 2 つ持たない形にした。

`feyn.tif` を 1/6 縮小した 416x550 で `pixAffinePta` の出力が C と pixel
完全一致 (修正前は 42080 画素が相違、最大差 145)。サンプリング版
(`pixAffineSampledPta`) は修正前から一致していたので、補間経路だけの
問題だったと切り分けられた。

**レビュー指摘から波及**: 移植表の `linearInterpolatePixelGray` が
「不要・インライン処理」のままだと指摘され、実態を確認する過程で
**C では `affine.c` だけでなく `bilinear.c` と `projective.c` も同じ
ヘルパーを呼ぶ**ことが分かった。Rust 側の `bilinear_gray` /
`projective_gray` も同じ独自実装を持っていて同じ 3 点で食い違って
いたので、まとめて共通実装に寄せた。こちらも C と pixel 完全一致を確認
(修正前は約 13 万画素、最大差 255)。

**残り**: check 50-52 は `createMatrix2dTranslate` / `createMatrix2dScale` /
`createMatrix2dRotate` / `l_productMat3` / `affineInvertXform` /
`pixAffine` が未移植。行列合成 API 一式の移植になるため別 PR。

### PR 37 以降: semantic マッピングの漸進追加

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
