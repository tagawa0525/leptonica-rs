# core/sarray: ファイル/ストリーム I/O とバイト列検索

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 D)

## Context

C 版の以下が Rust に未移植。`tests/core/string_reg.rs` には 4 件の `#[ignore]`
が残っている（うち hash 系 2 件は 031 で「不要」分類）。

| C 関数              | 行             | 役割                                   |
| ------------------- | -------------- | -------------------------------------- |
| `sarrayWrite`       | sarray1.c:1483 | ASCII シリアライズしてファイル書き出し |
| `sarrayWriteStream` | sarray1.c:1518 | 上記の Stream 版                       |
| `sarrayWriteMem`    | sarray1.c:1581 | 上記の Memory 版                       |
| `sarrayRead`        | sarray1.c:1351 | ASCII シリアライズからファイル読み込み |
| `sarrayReadStream`  | sarray1.c:1388 | 上記の Stream 版                       |
| `sarrayReadMem`     | sarray1.c:1457 | 上記の Memory 版                       |
| `arrayFindSequence` | utils2.c:1205  | バイト列内で部分列を線形探索           |

シリアライズフォーマット:

```text
\nSarray Version 1\n
Number of strings = N\n
  0[len0]:  string0\n
  1[len1]:  string1\n
  ...
\n
```

`SARRAY_VERSION_NUMBER` は 1。

## 配置先・API 設計

- ファイル: 既存 `src/core/sarray/serial.rs` で全シリアライズ I/O が
  実装済みだったため、本タスクではテストの `#[ignore]` 解除のみ
- 既存公開 API（再掲、本 PR で新規追加するものではない）:

```rust
impl Sarray {
    /// sarrayWrite 相当
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()>;
    /// sarrayWriteStream 相当
    pub fn write_to_writer(&self, writer: &mut impl Write) -> Result<()>;
    /// sarrayWriteMem 相当
    pub fn write_to_bytes(&self) -> Result<Vec<u8>>;

    /// sarrayRead 相当
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Sarray>;
    /// sarrayReadStream 相当
    pub fn read_from_reader(reader: &mut impl Read) -> Result<Sarray>;
    /// sarrayReadMem 相当
    pub fn read_from_bytes(data: &[u8]) -> Result<Sarray>;
}

/// arrayFindSequence 相当: data 中で sequence の最初の出現オフセットを返す。
/// 該当 C シグネチャは `(*poffset, *pfound)` だが、Rust では `Option<usize>`
/// を返す形に圧縮する。本 PR で新規追加。
pub fn array_find_sequence(data: &[u8], sequence: &[u8]) -> Option<usize>;
```

`array_find_sequence` は既存の Sarray 関連でなく純粋なバイト操作なので、
`src/core/sarray/mod.rs` に free function として置く。

## TDD ステップ

1. **RED** (`test(core): RED - sarray file I/O and arrayFindSequence`)
   - `tests/core/string_reg.rs::string_reg_binary_sequence` の `#[ignore]` を

     RED に書き換え、空配列 / 1 byte / 部分一致 / 末尾一致のケースを追加

   - `string_reg_file_io` の `#[ignore]` を RED に書き換え、

     write_to_writer + read_from_reader / write_to_file + read_from_file
     のラウンドトリップを 4 ケース (empty / 1 string / multiple / containing
     spaces) で検証

   - 公開 API stub を `unimplemented!()` で公開
2. **GREEN** (`feat(core): port sarray file I/O and arrayFindSequence`)
   - シリアライズ/デシリアライズ実装、`#[ignore]` 解除

## 不要分類

- `sarrayFindStringByHash`, `sarrayReplaceString` (hash) — 031 で不要分類済。

  これらの ignored test (`string_reg_find_by_hash`, `string_reg_replace_string`)
  はそのまま残す。

## ブランチ・PR

- ブランチ: `feat/core-sarray-io`
- PR: `feat(core): port sarray file/stream I/O and arrayFindSequence`
- 1PR、RED → GREEN

## ステータス

- [x] 計画書作成（c7612a2）
- [x] sarray I/O は実装済みと確認（テスト `#[ignore]` 解除のみ）
- [x] `array_find_sequence` 実装 + テスト追加（17ccb94）
- [x] PR 作成・Copilot レビュー対応（PR #320）
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書の表に PR #320 を反映
