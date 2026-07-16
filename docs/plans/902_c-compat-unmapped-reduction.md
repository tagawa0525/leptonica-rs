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

### PR 2 以降: semantic マッピングの漸進追加

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
