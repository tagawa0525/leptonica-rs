# io/jp2k: 書き込み + 高度読み込み（部分実装）

Status: IMPLEMENTED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 F)

## Context

C 版 `jp2kio.c` の以下が Rust に未移植。`tests/io/jp2kio_reg.rs` には 6 件の
`#[ignore]` が残っている。

| C 関数                                                    | 役割                         | Rust 実装可能性                |
| --------------------------------------------------------- | ---------------------------- | ------------------------------ |
| `pixWriteJp2k` / `pixWriteStreamJp2k` / `pixWriteMemJp2k` | JP2K 書き込み                | ❌ 純 Rust 不可                |
| `pixReadJp2k` (reduction 引数)                            | 縮小読み込み (1/2, 1/4, ...) | ✅ hayro `target_resolution`   |
| `pixReadJp2k` (box 引数)                                  | 範囲指定 cropped 読み込み    | ⚠ scale + clip で代替          |
| `pixReadStreamJp2k` (J2K codec)                           | raw J2K codestream 読み込み  | ✅ hayro が両 codec をサポート |

## 調査結果

### hayro-jpeg2000 0.3.4 の機能

[docs.rs](https://docs.rs/hayro-jpeg2000/0.3.4/) を確認した結果:

- **デコーダ専用**（encoder なし）
- `DecodeSettings { resolve_palette_indices, strict, target_resolution: Option<(u32, u32)> }` で

  ターゲット解像度ヒントが指定可能

- `Image::new(&data, &DecodeSettings)` → `decode()` で RGBA バイト列を取得
- raw J2C codestream / JP2 box 形式の両方をサポート

### 純 Rust JP2K エンコーダの状況 (2026-05 時点)

- `hayro-jpeg2000` は decoder-only (本クレートが作られた pdf-writer 用途では reading のみ必要)
- crates.io 検索で見つかる `jpeg2k` は libopenjp2 (C library) への bindings
- 純 Rust の JP2K encoder crate は事実上存在しない
- JP2K エンコーダは wavelet + arithmetic coding で実装規模が極めて大きい

## 採用方針

**部分実装 + 保留**:

### 実装する (本タスク)

- **`read_jp2k_scaled(reader, scale_denom)`**: hayro の `target_resolution`

  を使い、原寸の `1/scale_denom` の解像度で読み込む。`scale_denom` は 1/2/4/8/16
  をサポート（C 版の `reduction` 引数に相当）

- **`read_jp2k_cropped(reader, box_)`**: 全画像をデコードした後に `Pix::clip_rectangle`

  で box にクリップする方式で実装（hayro 単独では cropped デコード不可のため、
  performance trade-off を doc に明記）

### 保留 (別 issue)

- **JP2K 書き込み (`pixWriteJp2k` 系)**: `jpeg2k` 等 C-binding crate 追加が必要。

  ビルド複雑度・cross-compile 影響が大きいため別 issue にする

- **J2K codec variant**: hayro は両 codec をサポートするため、書き込みが入った時点で

  併せて実装する

## 配置先・API 設計

```rust
// 新規追加
pub fn read_jp2k_scaled<R: Read + Seek>(reader: R, scale_denom: u32) -> IoResult<Pix>;
pub fn read_jp2k_scaled_mem(data: &[u8], scale_denom: u32) -> IoResult<Pix>;
pub fn read_jp2k_cropped<R: Read + Seek>(reader: R, box_: &Box) -> IoResult<Pix>;
pub fn read_jp2k_cropped_mem(data: &[u8], box_: &Box) -> IoResult<Pix>;
```

## TDD ステップ

1. **RED**: `tests/io/jp2kio_reg.rs::jp2kio_reg_scaled_read` と

   `jp2kio_reg_cropped_read` の `#[ignore]` を RED に書き換え。stub を公開

2. **GREEN**: hayro の `DecodeSettings.target_resolution` を使って実装

書き込み系 (`jp2kio_reg_write`、`jp2kio_reg_j2k_codec`) と "no JP2K test image bundled"
の `#[ignore]` 2 件はそのまま残す。

## ブランチ・PR

- ブランチ: `feat/io-jp2k-scaled-read`
- PR: `feat(io): add scaled / cropped JP2K read using hayro target_resolution`
- 1PR、RED → GREEN

## ステータス

- [x] 計画書作成
- [x] hayro-jpeg2000 API 調査 (decoder-only、target_resolution あり)
- [x] 純 Rust encoder の不在を確認
- [ ] RED コミット (stub + テスト)
- [ ] GREEN コミット (実装)
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を「項目 F: 部分実装、書き込みは保留」に更新
