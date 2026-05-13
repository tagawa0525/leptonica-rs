# Core: Pixa rotate / clip / render 3 関数 (plan 032 カテゴリ A-3 残り)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3 (108b)

## 対象 C 関数 (3)

`pixafunc1.c` の残課題 (plan 108b 10 件) のうち、純粋に既存
Pix 操作のラッパーとして実装可能な 3 関数を切り出す。

- `pixaRotate(pixas, angle, type, incolor, w, h) -> Pixa` —
  各 Pix を任意角で回転 (boxa は空のまま; C 版同様)
- `pixaClipToPix(pixas, pixs) -> Pixa` — 各 box で `pixs` を
  矩形クリップし、対応する `pixa` の Pix と AND
- `pixaRenderComponent(pixs, pixa, index) -> Pix` — 1bpp の
  単一コンポーネントを `pixs` に OR で描画 (`pixs = None`
  ならボックス extent でゼロ画像を生成)

## API 設計

```rust
impl Pixa {
    /// C: `pixaRotate` (angle はラジアン; `< MIN_ANGLE_TO_ROTATE` は
    /// 単に deep_clone)
    pub fn rotate(
        &self,
        angle: f32,
        options: &crate::transform::RotateOptions,
    ) -> Result<Pixa>;

    /// C: `pixaClipToPix` (`pixs` の各 box 矩形を切り出し AND)
    pub fn clip_to_pix(&self, pixs: &Pix) -> Result<Pixa>;

    /// C: `pixaRenderComponent` (`pixs = None` のときボックス extent
    /// から 1bpp ゼロ画像を生成して描画)
    pub fn render_component(
        &self,
        pixs: Option<&Pix>,
        index: usize,
    ) -> Result<Pix>;
}
```

## 依存

- 既存 `transform::rotate(pix, angle, options)`
- 既存 `Pix::clip_rectangle(x, y, w, h)`、`Pix::and(&other)`
- 既存 `Boxa::extent()` (extent rect)
- 既存 `PixMut::set_pixel` (render_component の OR ループ)

## テスト方針

- rotate:
  - 微小角度 (< MIN_ANGLE_TO_ROTATE) で全 Pix が deep_clone される
  - 任意角度で各 Pix 寸法が rotate 後の値に一致
  - 元 Pixa の ref_count は影響を受けない (deep_clone)
- clip_to_pix:
  - 単純画像で AND の結果が期待値と一致
  - box 数と pixa 数が異なるときも box 数の最小で動作
- render_component:
  - `pixs = Some(...)` で指定された Pix の上に OR で描画
  - `pixs = None` で boxa extent から新 Pix を生成し描画
  - index が範囲外で Err
  - 1bpp 以外の component / pixs で Err

## 完了条件

- [x] cargo test/clippy/fmt 通過 (9 件パス)
- [x] core.md 3 件 ❌ → ✅
- [x] plan 032 で 123 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `rotate`: `transform::rotate` を順に呼ぶだけ。C 版同様 boxa は
  空のまま (回転後の box は意味を持たないため)
- `clip_to_pix`: box が pixa 数より少ない場合は box 数までで終了
  (C 版は両者一致を前提だがチェックしていない; Rust 側は明示)
- `render_component`: 1bpp 制約は早期 Err。pixs=None のとき
  boxa.extent() でゼロ画像生成
