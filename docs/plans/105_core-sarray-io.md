# core/sarray: ファイル/ストリーム I/O とバイト列検索

Status: IN_PROGRESS
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

- ファイル: `src/core/sarray/serial.rs` を新設（または `mod.rs` に追記）
- 公開 API:

```rust
impl Sarray {
    /// sarrayWrite 相当
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// sarrayWriteStream 相当
    pub fn write_to_writer<W: Write>(&self, writer: &mut W) -> Result<()>;
    /// sarrayWriteMem 相当
    pub fn to_serialized_bytes(&self) -> Result<Vec<u8>>;

    /// sarrayRead 相当
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Sarray>;
    /// sarrayReadStream 相当
    pub fn read_from_reader<R: BufRead>(reader: R) -> Result<Sarray>;
    /// sarrayReadMem 相当
    pub fn from_serialized_bytes(bytes: &[u8]) -> Result<Sarray>;
}

/// arrayFindSequence 相当: data 中で sequence の最初の出現オフセットを返す。
///
/// 該当 C シグネチャは `(*poffset, *pfound)` だが、Rust では `Option<usize>`
/// を返す形に圧縮する。
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

- [x] 計画書作成
- [ ] RED コミット
- [ ] GREEN コミット
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
