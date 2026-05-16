# textops: pixAddSingleTextblock を移植 (plan 032 カテゴリ M 残)

Status: IMPLEMENTED
作成日: 2026-05-16
完了日: 2026-05-16
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ M

## 対象 C 関数

画像に複数行のテキストブロックを描画するヘルパー。配置位置を 4 通り
(画像の外側上下、画像の内側上下) に切り替えられる。
`docs/porting/comparison/misc.md` で ❌ 未実装として残っていた 1 件。

| C 関数                  | 役割                                        |
| ----------------------- | ------------------------------------------- |
| `pixAddSingleTextblock` | テキストブロックを画像に描画 (4 通りの位置) |

## API 設計

```rust
// in src/core/bmf.rs (extend existing module)

/// `pixAddSingleTextblock` 用の配置オプション。
/// `Bmf::add_textlines` 用の TextLocation とは別 (AtTop / AtBot がある)。
pub enum TextblockLocation {
    Above,   // C: L_ADD_ABOVE
    AtTop,   // C: L_ADD_AT_TOP
    AtBot,   // C: L_ADD_AT_BOT
    Below,   // C: L_ADD_BELOW
}

/// C: `pixAddSingleTextblock`
impl Bmf {
    pub fn add_single_textblock(
        &self,
        pix: &Pix,
        text: &str,        // 空ならコピーを返す
        val: u32,          // 描画色 (1bpp なら 0/1, 8bpp なら 0..=255, 32bpp は 0xRRGGBB00)
        location: TextblockLocation,
    ) -> Result<(Pix, bool)>; // bool: 水平方向に溢れた行があったか
}
```

## 依存 (すべて Rust 実装済み)

- `Bmf::get_line_strings` (テキスト折り返し)
- `Bmf::set_textline` (1 行描画)
- `Pix::new` / `to_mut` / `rop_region_inplace`
- `transform::*` は不要

## 設計差分 (C → Rust)

1. C の戻り値は overflow フラグを out-param で返すが、Rust は `(Pix, bool)` のタプルで一度に返す。
2. C は cmap (パレット) に色を追加する処理を含むが、Rust 実装では cmap 未対応 (set_pixel_unchecked で直接書き込む)。
3. `bmf` が None だった場合の "no bmf, return copy" パスは存在しない (Rust では Bmf が必須のメソッドとして実装)。
4. `textstr` が空の場合は入力画像のディープコピーを返す (C は `pixGetText(pix)` でフォールバックするが Rust 版は pix.text() のフォールバックも一応サポート)。

## テスト方針

- Above: 元画像より背高い PIX が返る、上部にテキスト
- Below: 元画像より背高い PIX が返る、下部にテキスト
- AtTop: 同じ寸法、上部にテキスト
- AtBot: 同じ寸法、下部にテキスト
- 空テキスト: 入力と同じ寸法、内容も等しい
- depth ガード: 1bpp/8bpp/32bpp で val 範囲を丸める
