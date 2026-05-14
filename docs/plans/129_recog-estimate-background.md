# Recog: pixEstimateBackground 移植 (plan 032 残: recog ❌)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md (recog 残課題 6 件のうちの 1 件)

## 対象 C 関数 (1)

`pageseg.c` の `pixEstimateBackground`。8bpp グレースケール画像の
背景レベル (中央値ピクセル) を、暗いピクセルをマスクで除外しつつ
推定する。

## API 設計

```rust
/// C: `pixEstimateBackground`
/// Returns the estimated background gray level [0..255].
pub fn estimate_background(
    pix: &Pix,
    darkthresh: u32,
    edgecrop: f32,
) -> RecogResult<u32>;
```

- `pix`: 8 bpp (colormap は内部で gray に変換)
- `darkthresh`: それ以下を暗いとみなしてマスク除外。0 で無効化。
- `edgecrop`: 0.0..1.0 で内側のクロップ比率

## アルゴリズム

1. colormap があれば gray に変換
2. `edgecrop > 0.0` なら内側矩形をクリップ
3. 50000 サンプル以下になる sampling factor を計算
4. `darkthresh > 0` なら threshold-to-binary でマスクを作り、invert
5. `rank_value_masked(mask, 0, 0, sampling, 0.5)` で中央値を取得
6. round して u32 を返す

## 依存

- 既存 `Pix::remove_colormap`
- 既存 `Pix::clip_rectangle`
- 既存 `color::threshold::threshold_to_binary`
- 既存 `Pix::invert`
- 既存 `Pix::rank_value_masked`

## テスト方針

- 単色 (gray = 200) 8bpp 画像 → 推定 = 200
- 半分が dark, 半分が light → darkthresh で dark を除外し light のみ反映
- 非 8bpp で Err
- edgecrop の境界値 (0.0 / 1.0) で適切な範囲チェック

## 完了条件

- [x] cargo test/clippy/fmt 通過 (5 件パス)
- [x] recog.md 1 件 ❌ → ✅
- [x] plan 032 で 129 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
