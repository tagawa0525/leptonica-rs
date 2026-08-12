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

### PR 13: coloring 整列 — cmapped shift 対応 + 14 ペア (IN_PROGRESS)

C 版ソース: `prog/coloring_reg.c`、`src/coloring.c` / `src/colormap.c`。

PR 11 で見送った coloring 系列。フォント面の前提は PR 12 で解消済み。

1. `pix_shift_by_component` に cmapped 分岐を追加 (C は cmap を
   `pixcmapShiftByComponent` で変換するだけ。`PixColormap::
   shift_by_component` は移植済みで pixel 式も C と一致確認済み) (TDD)
2. テスト画像 `harmoniam100-11.png` を C prog から追加
3. coloring_c テスト: C checks 2-15 と同条件 (cmap reset 4 + cmapped
   shift 4 + rgb shift 4 + fg cmapped/rgb 2、全出力に
   pixAddSingleTextblock fontsize 8) で 14 出力を書き出し 14 ペア

### PR 14 以降: semantic マッピングの漸進追加

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
