# leptonica-morph 全未実装関数の移植計画

Status: PLANNED

## Context

leptonica-morph crateは基本的なbinary/grayscale/color形態学演算、DWA、thinningを
実装済みだが、C版leptonicaのmorph関数群に対して以下の重要な機能が欠落している:

1. **Safe closing** - 境界アーティファクト防止版のclose演算が未実装
2. **Generalized ops** - openGeneralized/closeGeneralizedが未実装
3. **Morphological applications** - コンポーネント単位・マスク付きの高度な演算が未実装
4. **SEL I/O** - 構造要素のシリアライゼーションが未実装
5. **DWA拡張** - Composite DWA、Extended DWA（>63px）が未実装
6. **シーケンス拡張** - DWAシーケンス、カラーシーケンスが未実装

### 現状の実装状況

| モジュール | 実装済み関数数 | 状態 |
|-----------|-------------|------|
| binary.rs | 13 | コア実装済み、safe closing / generalized未対応 |
| grayscale.rs | 7 | 基本演算完了 |
| color.rs | 7 | 基本演算完了 |
| sel.rs | 18+ | 生成・操作は充実、I/O未対応 |
| dwa.rs | 4 | 基本brick DWAのみ、composite/extended未対応 |
| sequence.rs | 3 | binary sequence実装済み、DWA/color未対応 |
| thin.rs | 2 | 完了 |
| thin_sels.rs | 4 | 完了 |

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `fmorphautogen*`, `fmorphgen*` | DWAコード自動生成はRustでは不要（手書き実装済み） |
| `selDisplayInPix`, `selaDisplayInPix` | 可視化専用 |
| `resetMorphBoundaryCondition`, `getMorphBorderPixelColor` | グローバル状態（Rustではオプション構造体で対応） |
| `pixaThinConnected` | Pixa操作はアプリケーション層でループすれば良い |
| `pixDisplayHitMissSel` | デバッグ可視化 |
| `pixRemoveMatchedPattern`, `pixDisplayMatchedPattern` | パターン可視化専用 |
| `selDestroy`, `selaExtendArray` 等 | Rustではメモリ管理がDrop / Vec自動拡張で対応済み |
| `pixErodeGray3`, `pixDilateGray3`, `pixOpenGray3`, `pixCloseGray3` | 特殊3x3最適化（hsize=3, vsize=3で同等の結果） |

---

## 実行順序

Phase 1 → 2 → 3 → 4 → 5 → 6 の順に直列で実行する。

```
Phase 1 (Safe closing + Generalized) ← 基盤
  → Phase 2 (Morphological applications) ← Phase 1のsafe closingを使用可能
    → Phase 3 (SEL I/O + 拡張)
      → Phase 4 (SEL生成)
        → Phase 5 (DWA拡張 + シーケンス拡張)
          → Phase 6 (Sela配列管理)
```

---

## Phase 1: Safe closing + Generalized ops（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/morph.c` L170-350

### 実装内容

Safe closing（境界アーティファクト防止）:
- `close_safe(pix, sel) -> MorphResult<Pix>` - 安全なcloseバイナリ形態学演算
- `close_safe_brick(pix, hsize, vsize) -> MorphResult<Pix>` - brick SEL版
- `close_safe_comp_brick(pix, hsize, vsize) -> MorphResult<Pix>` - composite brick版

動作: close演算で画像境界の白ピクセルが侵食されるアーティファクトを
防止するため、画像を拡張してからclose→元サイズに戻す。

Generalized ops:
- `open_generalized(pix, hsize, vsize, iterations) -> MorphResult<Pix>` - 反復付きopen
- `close_generalized(pix, hsize, vsize, iterations) -> MorphResult<Pix>` - 反復付きclose

### 修正ファイル

- `crates/leptonica-morph/src/binary.rs`: 上記5関数追加

### テスト

- close_safe vs close の境界アーティファクト比較
- 大きなSELでのclose_safe動作確認
- generalized ops の反復パラメータ検証
- テスト画像: 境界近くに要素がある1bpp画像

---

## Phase 2: Morphological applications（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/morphapp.c` 全体

### 実装内容

マスク付き・コンポーネント単位の演算:
- `morph_sequence_masked(pix, mask, sequence, dispsep) -> MorphResult<Pix>` - マスク領域のみに形態学シーケンス適用
- `morph_sequence_by_component(pix, sequence, connectivity, min_w, min_h, boxa) -> MorphResult<Pixa>` - 各連結成分に個別適用
- `morph_sequence_by_region(pix, mask, sequence, connectivity) -> MorphResult<Pixa>` - 各領域に個別適用

集合演算:
- `union_of_morph_ops(pix, sels) -> MorphResult<Pix>` - 複数SEL演算結果の和集合
- `intersection_of_morph_ops(pix, sels) -> MorphResult<Pix>` - 複数SEL演算結果の積集合

高度な形態学演算:
- `selective_conn_comp_fill(pix, connectivity, min_depth) -> MorphResult<Pix>` - 選択的な連結成分充填
- `seedfill_morph(pix_seed, pix_mask, max_iters, connectivity) -> MorphResult<Pix>` - 反復的な形態学seedfill
- `h_dome(pix, height, connectivity) -> MorphResult<Pix>` - H-dome抽出
- `fast_tophat(pix, hsize, vsize, tophat_type) -> MorphResult<Pix>` - 高速tophat
- `morph_gradient(pix, hsize, vsize, smoothing) -> MorphResult<Pix>` - 形態学グラディエント（グレースケール）
- `run_histogram_morph(pix, direction, range_type) -> MorphResult<Numa>` - グラニュロメトリ

### 修正ファイル

- `crates/leptonica-morph/src/morphapp.rs`（新規）: 上記関数群
- `crates/leptonica-morph/src/lib.rs`: `pub mod morphapp` 追加

### テスト

- morph_sequence_masked のマスク境界での正確性
- union/intersection の結果検証
- h_dome の高さパラメータ効果確認
- seedfill_morph の収束テスト

---

## Phase 3: SEL管理拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/sel1.c` L800-1200

### 実装内容

SEL I/O:
- `Sel::read<R: Read>(reader: R) -> MorphResult<Sel>` - SELファイル読み込み
- `Sel::write<W: Write>(&self, writer: W) -> MorphResult<()>` - SELファイル書き込み

SEL生成拡張:
- `Sel::from_color_image(pix: &Pix) -> MorphResult<Sel>` - カラー画像からSEL生成（RED=hit, GREEN=miss, BLUE=DC）
- `Sel::from_pta(pta: &Pta, cy, cx) -> MorphResult<Sel>` - 点群からSEL生成
- `Sel::print_to_string(&self) -> String` - SELのテキスト表現

SEL操作:
- `Sel::get_parameters(&self) -> (u32, u32, u32, u32)` - (height, width, cy, cx) を一括取得
- `Sel::copy(&self) -> Sel` - 明示的コピー（Clone traitとは別のセマンティクス）

### 修正ファイル

- `crates/leptonica-morph/src/sel.rs`: I/O、生成、操作関数追加

### テスト

- SEL read/write ラウンドトリップ（各種SEL形状）
- from_color_image でのhit/miss正確性
- from_pta での座標→SEL変換検証
- print_to_string のフォーマット確認

---

## Phase 4: SEL生成（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/sel2.c`

### 実装内容

標準SELライブラリ:
- `Sel::create_cross_junction(size) -> Sel` - 十字交差パターン
- `Sel::create_t_junction(size) -> Sel` - T字交差パターン
- `Sel::create_plus_sign(size) -> Sel` - プラス記号型

SEL生成ユーティリティ:
- `sela_add_basic() -> Vec<Sel>` - 基本brick SEセット
- `sela_add_hit_miss() -> Vec<Sel>` - hit-missパターンセット
- `sela_add_dwa_linear() -> Vec<Sel>` - DWA用線形パターンセット
- `sela_add_dwa_combs() -> Vec<Sel>` - DWA用コムパターンセット

### 修正ファイル

- `crates/leptonica-morph/src/sel.rs`: 上記関数追加

### テスト

- 各パターンの幾何学的正確性検証
- DWAパターンのサイズ範囲確認
- 基本SELセットの完全性確認

---

## Phase 5: DWA拡張 + シーケンス拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/morphdwa.c` L400-900, `morphseq.c` L300-600

### 実装内容

Composite DWA:
- `dilate_comp_brick_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - コンポジットDWA dilate
- `erode_comp_brick_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - コンポジットDWA erode
- `open_comp_brick_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - コンポジットDWA open
- `close_comp_brick_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - コンポジットDWA close

Extended DWA（>63px対応）:
- `dilate_comp_brick_extend_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - 拡張DWA dilate
- `erode_comp_brick_extend_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - 拡張DWA erode
- `open_comp_brick_extend_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - 拡張DWA open
- `close_comp_brick_extend_dwa(pix, hsize, vsize) -> MorphResult<Pix>` - 拡張DWA close

シーケンス拡張:
- `morph_sequence_dwa(pix, sequence) -> MorphResult<Pix>` - DWAシーケンス
- `morph_comp_sequence_dwa(pix, sequence) -> MorphResult<Pix>` - コンポジットDWAシーケンス
- `color_morph_sequence(pix, sequence) -> MorphResult<Pix>` - カラー形態学シーケンス

### 修正ファイル

- `crates/leptonica-morph/src/dwa.rs`: composite/extended DWA追加
- `crates/leptonica-morph/src/sequence.rs`: DWA/colorシーケンス追加

### テスト

- composite DWA vs 通常DWAの結果一致確認
- extended DWA（hsize=100等）の大きなSELでのテスト
- DWAシーケンスの解析と実行
- カラーシーケンスの各チャンネル独立処理確認

---

## Phase 6: Sela配列管理（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/sel1.c` L70-200

### 実装内容

```rust
pub struct Sela {
    sels: Vec<Sel>,
}
```

- `Sela::new() -> Sela` - 空のSela作成
- `Sela::add(&mut self, sel: Sel)` - SEL追加
- `Sela::get(&self, index: usize) -> Option<&Sel>` - インデックスアクセス
- `Sela::find_by_name(&self, name: &str) -> Option<&Sel>` - 名前検索
- `Sela::count(&self) -> usize` - SEL数
- `Sela::read<R: Read>(reader: R) -> MorphResult<Sela>` - ファイルから読み込み
- `Sela::write<W: Write>(&self, writer: W) -> MorphResult<()>` - ファイルに書き出し

### 修正ファイル

- `crates/leptonica-morph/src/sel.rs`: `Sela` 構造体と上記メソッド追加
- `crates/leptonica-morph/src/lib.rs`: `Sela` をpublic APIに追加

### テスト

- Sela作成、追加、検索のユニットテスト
- find_by_name の一致/不一致テスト
- Sela read/write ラウンドトリップ

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Safe closing + Generalized | 1 | 5 |
| 2 | Morphological applications | 1 | ~11 |
| 3 | SEL管理拡張 | 1 | ~7 |
| 4 | SEL生成 | 1 | ~7 |
| 5 | DWA拡張 + シーケンス拡張 | 1 | ~11 |
| 6 | Sela配列管理 | 1 | ~7 |
| **合計** | | **6** | **~48** |

## 共通ワークフロー

### TDD

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### PRワークフロー

1. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
2. `/gh-pr-create` でPR作成
3. `/gh-actions-check` でCopilotレビュー到着を確認
4. `/gh-pr-review` でレビューコメント対応
5. CIパス確認後 `/gh-pr-merge --merge` でマージ
6. ブランチ削除

### ブランチ命名

```
main
└── feat/morph-safe-closing      ← Phase 1
└── feat/morph-applications      ← Phase 2
└── feat/morph-sel-io            ← Phase 3
└── feat/morph-sel-gen           ← Phase 4
└── feat/morph-dwa-ext           ← Phase 5
└── feat/morph-sela              ← Phase 6
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-morph
cargo clippy -p leptonica-morph -- -D warnings
cargo test -p leptonica-morph
cargo test --workspace  # PR前に全ワークスペーステスト
```
