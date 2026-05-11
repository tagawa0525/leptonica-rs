# Core: arrayaccess.c の拡張ビット演算 4 関数

Status: IMPLEMENTED
作成日: 2026-05-10
完了日: 2026-05-10
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ H

## 対象 C 関数

| C 関数               | 行  | 用途                                    |
| -------------------- | --- | --------------------------------------- |
| `l_clearDataDibit`   | 195 | 2bit ピクセルを 0 にクリア              |
| `l_clearDataQbit`    | 248 | 4bit ピクセルを 0 にクリア              |
| `l_getDataFourBytes` | 343 | 32bit ピクセル取得 (内部 word アクセス) |
| `l_setDataFourBytes` | 359 | 32bit ピクセル書き込み                  |

## API 設計

`src/core/pix/access.rs` に追加:

```rust
/// Clear a 2-bit pixel to 0 (C: l_clearDataDibit)
pub fn clear_data_dibit(line: &mut [u32], x: u32);

/// Clear a 4-bit pixel to 0 (C: l_clearDataQbit)
pub fn clear_data_qbit(line: &mut [u32], x: u32);

/// Get a 32-bit pixel value (C: l_getDataFourBytes)
pub fn get_data_four_bytes(line: &[u32], x: u32) -> u32;

/// Set a 32-bit pixel value (C: l_setDataFourBytes)
pub fn set_data_four_bytes(line: &mut [u32], x: u32, val: u32);
```

`get_data_four_bytes` は実質的に `line[x as usize]` と等価だが、API 一貫性のため
他のアクセサと同じ呼び出し形を提供する。

## TDD 手順

`tests/core/lowaccess_reg.rs` または `tests/core/conversion_reg.rs` 等の既存
テストに `#[ignore = "not yet implemented (plan 113)"]` 付きで:

- `lowaccess_reg_clear_data_dibit_4_pixels`
- `lowaccess_reg_clear_data_qbit_4_pixels`
- `lowaccess_reg_get_set_data_four_bytes_round_trip`

## 完了条件

- [x] cargo test --all-features 全通過
- [x] cargo clippy --all-features --all-targets -- -D warnings
- [x] cargo fmt --all -- --check
- [x] PR + Copilot レビュー対応 + マージ
- [x] docs/porting/comparison/core.md の追加検証エントリで arrayaccess.c 4 件を ✅ に更新
- [x] docs/plans/032 のステータス表で 113 を IMPLEMENTED に更新
