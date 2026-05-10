# Core: numabasic.c の拡張 5 関数 (plan 032 カテゴリ L)

Status: IMPLEMENTED
作成日: 2026-05-10
完了日: 2026-05-10
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ L

## 対象 C 関数

- `numaCreateFromString` — カンマ区切り文字列から Numa 生成
- `numaCopyParameters` — startx/delx を別 Numa にコピー
- `numaConvertToSarray` — Numa を整形済み Sarray に変換
- `numaaCreateFull` — Numaa を初期 nptr 個の空 Numa で構築
- `numaaGetNumberCount` — Numaa 全体の総値数 (既存 `total_count` で代替)

## API

```rust
impl Numa {
    pub fn create_from_string(s: &str) -> Result<Self>;
    pub fn copy_parameters(&mut self, other: &Numa);
    pub fn convert_to_sarray(&self, width, precision, pad_zeros, value_type) -> Sarray;
}
impl Numaa {
    pub fn create_full(nptr: usize, n: usize) -> Self;
    // numaaGetNumberCount → 既存 total_count() で代替
}
pub enum NumaSarrayType { Integer, Float }
```

## 完了条件

- [x] cargo test/clippy/fmt 通過
- [x] PR + Copilot レビュー対応 + マージ
- [x] docs/porting/comparison/core.md で 5 件更新
- [x] docs/plans/032 で L を IMPLEMENTED に
