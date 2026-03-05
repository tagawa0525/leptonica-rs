# Phase 4: C版同等テスト内容強化

Status: IN_PROGRESS

## Context

C版とのベンチマーク比較が成り立つには、Rust版テストがC版と同等の検証を行っている必要がある。
Phase 3 (PR 1-8) は既存テストに golden manifest インフラ (`write_pix_and_check`) を追加したが、
**C版にあるテスト内容自体の追加は行われていない**。

監査結果 (`scripts/audit-regression-tests.py`):

- High divergence (>= 1.0): **25テスト**
- Medium divergence (0.3-1.0): **76テスト**
- うち実際にアクション必要: **~42テスト**（残りはRust側の方がチェック数多い or parity済み）

親計画: `docs/plans/014_regression-test-enhance.md`

## 方針

1. C版 `*_reg.c` の全チェックポイントをRust版と比較
2. Rust APIが存在する → 同等チェックを追加
3. Rust APIが存在しない → `#[ignore = "function_name not implemented"]` スケルトン追加
4. JPEGソーステストは golden 除外（デコーダ差異リスク）

## PR構成 (7 PR)

### PR 1: Easy Wins — API存在・RegParams不足 (4テスト)

ブランチ: `test/phase4-easy-wins`

- **seedspread** (region): C=7 WPAC, Rust=0. RegParams 6 WPAC (tiled display skip)
- **fpix2** (core): C=5 CP, Rust=0. FPix rotate/border not impl. 5 #[ignore]
- **checkerboard** (transform): C=6 WPAC, Rust=0. 4 WPAC + 2 #[ignore]
- **circle** (transform): C=2 WPAC, Rust=0. circles.pa missing. #[ignore] kept

見積: +10 WPAC, +7 #[ignore], ~10 golden

### PR 2-7: 後続 PR (別途計画)

(Color, Core, Region, Transform+IO, Morph, Skeletons)

## 検証

```bash
cargo test --test <module>
REGTEST_MODE=generate cargo test --test <module>
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/audit-regression-tests.py
```
