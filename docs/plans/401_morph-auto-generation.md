# morph 自動生成 (DWA): Rust 移植は保留

Status: HOLD
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 I)

## Context

C 版 `fmorphgen.1.c` / `fhmtgen.1.c` / `dwacomb.2.c`（および対応する low-level
ヘルパ `*low.*.c`）は、`prog/fmorphautogen.c` と `prog/fhmtautogen.c` という
コード生成プログラムが SEL（構造化要素）定義から機械生成した DWA
(Destination Word Addressing) 形態学関数群。`pixFMorphopGen_1`、
`pixFHMTGen_1`、`pixHMTDwa_1` 等を提供する。

機械生成済み行数:

| ファイル           | 行数          |
| ------------------ | ------------- |
| `fmorphgen.1.c`    | 273           |
| `fmorphgenlow.1.c` | 5,862         |
| `fhmtgen.1.c`      | 173           |
| `fhmtgenlow.1.c`   | 445           |
| `dwacomb.2.c`      | 295           |
| `dwacomblow.2.c`   | 4,970         |
| **合計**           | **約 12,000** |

生成元 `prog/fmorphautogen.c` 自体は 73 行（テンプレート出力ロジック）で、
SEL 定義テーブル + 出力テンプレートのみで構成される。

`tests/morph/fmorphauto_reg.rs::fmorphauto_reg` と
`tests/morph/fhmtauto_reg.rs::fhmtauto_reg` の `#[ignore]` 2 件が残っている。

## 判断: HOLD

### Rust 側で代替可能な機能

DWA 系関数は「特定 SEL での brick erosion / dilation / open / close /
hit-miss を、word 演算で並列に処理する高速化版」。Rust 側にはすでに以下が
ある:

- `morph::binary::{erode_brick, dilate_brick, open_brick, close_brick}`

  — 一般的な brick 形態学（任意サイズ）

- `morph::sequence::morph_sequence` — シーケンス文字列で複合操作
- `morph::dwa::{erode_brick_dwa, dilate_brick_dwa, ...}` — DWA 風の高速版が

  既に部分的に実装済み（限定 SEL 用）

つまり「機能的に同等の処理」は既に Rust から呼び出せる。残るは「ベンチマーク
で C 版機械生成の DWA に肉薄する性能を出せるか」のみ。

### コスト試算

- 案 A (生成スクリプト移植): `fmorphautogen.c` 73 行 + SEL テーブル + コード

  テンプレート × 2 を Rust に移植。`build.rs` で生成 → コンパイル時にコード
  膨張。実装は 1〜2 週間規模、生成テンプレートのバグが見えにくい

- 案 B (機械生成済み 12,000 行を手で移植): 各ビット演算チャンクを 1 つずつ

  Rust に書き写す。数週間規模、可読性ゼロでメンテ困難

- 案 C (HOLD): 既存の `dilate_brick` / DWA で代替し、性能ボトルネックが

  実測で確認された時点で再検討

### 性能向上の見込み

- C 版 DWA は MMX 風の word-level SIMD を手書きしたコード。Rust の

  `dilate_brick` (一般 brick 実装) は LLVM が auto-vectorize するため、
  最近のターゲットでは大差がない可能性が高い

- ベンチマークなしで「12,000 行手書きで X% 速くなる」を保証できない

### Rust 側で見るべきベンチマーク

`docs/plans/026_post-phase5-benchmark.md` (Phase 5.1) では「Rust 全体で
C 版より 23% 速い」結果が出ており、形態学系は既に C 版と同等以上の性能を
達成している。DWA 自動生成の Rust 移植は、Phase 5.1 のベンチマーク結果を
鑑みると **コストパフォーマンスが見合わない可能性が高い**。

## 採用方針

**HOLD**:

- 本タスクでは実装しない
- `tests/morph/{fmorphauto_reg,fhmtauto_reg}.rs` の 2 件の `#[ignore]` は

  そのまま残す

- 利用者には既存の `dilate_brick` / `morph_sequence` / `morph::dwa::*` を

  案内する

- 将来、特定の SEL 形態学が性能ボトルネックと実測されたら以下の優先順で

  検討:

  1. ターゲット SEL のみ手書き (~50–200 行)
  2. 案 A: 生成スクリプト Rust 移植
  3. 案 B: 全 12,000 行の手作業移植は採用しない

## ステータス

- [x] 計画書作成
- [x] C 版生成スクリプト構造の調査 (`fmorphautogen.c` 73 行)
- [x] 機械生成済み行数の計測（合計 ~12,000 行）
- [x] Rust 既存代替機能の確認（`dilate_brick`、`morph_sequence`、

      `morph::dwa::*`）

- [x] Phase 5.1 ベンチマーク結果との照合（既に C 版を上回る性能を達成）
- [x] HOLD 判断（性能ボトルネックが実測されたら再検討）
- [ ] 実装（**保留**: 性能要求が出たら案 A or 部分手書きを再検討）
- [ ] 031 全体計画書を「項目 I: 保留」に更新
