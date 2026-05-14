# Core: Numa::make_histogram_auto (plan 032 残: 109b)

Status: IMPLEMENTED
作成日: 2026-05-13
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ B (109b)

## 対象 C 関数 (1)

- `numaMakeHistogramAuto(na, maxbins) -> Numa` — 自動ビン幅で
  ヒストグラム生成。整数 + 小範囲なら単位幅整数ビン、それ以外は
  [minval, maxval] を maxbins 等分する float ビン。

## 実装

```rust
impl Numa {
    pub fn make_histogram_auto(&self, maxbins: u32) -> Result<Numa>;
}
```

- 整数ビン経路: `(maxval - minval) < maxbins` かつ整数のとき、
  `[imin, imax]` の単位幅ヒストグラム
- float ビン経路: `binsize = (maxval - minval) / maxbins`、
  edge case (`v == maxval`) は最後のビンにクランプ
- 全 constant 入力 (`range == 0`) は 1 ビン (count = n) を返す

## 完了条件

- [x] cargo test/clippy/fmt 通過 (5 件パス)
- [x] core.md 1 件 ❌ → ✅
- [x] plan 032 で 132 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ
