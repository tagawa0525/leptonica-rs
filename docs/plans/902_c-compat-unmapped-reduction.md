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

### PR 7 以降: semantic マッピングの漸進追加

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
