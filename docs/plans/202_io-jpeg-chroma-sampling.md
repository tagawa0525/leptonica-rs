# io/jpeg: クロマサブサンプリング設定

Status: PLANNED
親計画: [031_gap-fill-overall.md](031_gap-fill-overall.md) (項目 E)

## Context

C 版 `jpegio.c::pixSetChromaSampling()` (1174) は、Pix の `special` フィールドに
`L_NO_CHROMA_SAMPLING_JPEG = 1` を立てて JPEG 書き込み時のクロマサブサンプリングを
無効化する仕組み。デフォルトは 4:2:0（2x2 サブサンプリング有効）で、フル解像度の
クロマが必要な場合にこのフラグを立てる。

```c
if (sampling)
    pixSetSpecial(pix, 0);              /* default = subsample */
else
    pixSetSpecial(pix, L_NO_CHROMA_SAMPLING_JPEG);
```

## 現状

- `PixMut::set_special(i32)` は既存
- `src/io/jpeg.rs` の書き込み側は `special` を参照しておらず、`jpeg-encoder` のデフォルト

  挙動でエンコードしている

- `jpeg-encoder` crate には `Encoder::set_sampling_factors()` 相当の API があるか要調査

## 配置先・API 設計

- **定数**: `src/core/pix/mod.rs` か新ファイルに `pub const NO_CHROMA_SAMPLING_JPEG: i32 = 1;` を追加
- **ヘルパ**: `src/io/jpeg.rs` に

  ```rust
  pub fn set_chroma_sampling(pix: &mut PixMut, sampling: bool);
  ```

  を追加。`special` への代入はこのヘルパ越しに行うのが C 版と整合する。

- **書き込み側**: `write_jpeg` 内で `pix.special() == NO_CHROMA_SAMPLING_JPEG` なら

  4:4:4 サンプリング（YCbCr フル解像度）でエンコード、そうでなければデフォルト 4:2:0

## 事前調査タスク

- `jpeg-encoder` 0.7.0 のドキュメントを確認し、サンプリングファクタを設定できる

  API があるか確認。無ければサブサンプリング ON/OFF だけ切り替える限定実装にする

- 仮に API がなければ計画を見直し（このタスクは保留にする）

## TDD ステップ

1. **調査**（コード変更なし、計画書を更新）
2. **RED** (`test(io): pixSetChromaSampling の RED テスト`)
   - `tests/io/iomisc_reg.rs` の `#[ignore = "...pixSetChromaSampling..."]` を unignore
   - 同一画像をデフォルト/フルクロマで書き出し、ファイルサイズや復号後ピクセルの差を確認
3. **GREEN** (`feat(io): JPEG クロマサブサンプリング無効化フラグに対応`)
   - 上記 API を実装
4. **REFACTOR**: 不要

## テスト戦略

- 小さいカラー画像 (e.g. `tests/data/images/marge.jpg` のように既存の小サンプル) で

  `set_chroma_sampling(false)` した出力と、デフォルト出力を比較

- 出力 JPEG を再読み込みし、フルクロマの方が彩度方向の差が小さいことを検証

## ブランチ・PR

- ブランチ: `feat/io-jpeg-chroma-sampling`
- PR: `feat(io): support pixSetChromaSampling for JPEG writing`
- 1PR、調査結果を踏まえて RED → GREEN

## 想定リスク

- `jpeg-encoder` 0.7 にサンプリング設定 API が無い場合、別 crate へのスイッチが必要に

  なる可能性がある。その場合は本計画を保留し、依存切替を別計画として切り出す。

## ステータス

- [ ] jpeg-encoder API 調査
- [ ] RED コミット
- [ ] GREEN コミット
- [ ] PR 作成・Copilot レビュー対応
- [ ] /gh-pr-merge --merge
- [ ] 031 全体計画書を IMPLEMENTED に更新
