# Phase 3: 回帰テスト修正計画

Status: IN_PROGRESS (PR 1/8: filter 完了、後続バグ修正 #255/#256/#257 全完了)

## Context

Phase 1-2（PR #254）で全149テストの監査が完了し、B分類84テストの修正が必要と判定された。
B分類 = 対応するRust関数は存在するが、テストのチェック数・検証品質がC版に不足している。

本計画はPhase 3の全体戦略と、最初のモジュール（filter）の具体的な修正内容を定義する。

## 全体戦略

### PRの粒度

モジュール単位で8つのPRに分割:

| 順序 | モジュール | B数 | 方針 |
| ---- | ---------- | --- | ---- |
| 1 | filter | 8 | 関数の大半が実装済み。最初に着手してパターンを確立 |
| 2 | morph | 8 | div=0.0が4件。修正量が少ない可能性 |
| 3 | io | 6 | 最少テスト数 |
| 4 | transform | 9 | 中程度 |
| 5 | color | 15 | 件数多いが多くがdiv=0.5 |
| 6 | region | 9 | 高divergenceあり（seedspread 1.8） |
| 7 | recog | 10 | 中程度 |
| 8 | core | 19 | 最多。string/ptra等で高divergence |

### 修正原則（013_regression-test-audit.md Phase 3より）

1. C版 `*_reg.c` の全チェックポイントを把握
2. 対応するRust関数が存在するチェックを追加
3. `write_pix_and_check()` / `compare_pix()` でピクセル単位検証
4. `load_test_image()` で実画像を使用
5. C版と同等の計算量を再現
6. 既存ユニットテストは残し、回帰テストチェックを追加
7. 未実装関数は `#[ignore = "関数名 not implemented"]` スケルトン追加

### コミット戦略

テストの強化は既存関数のテスト追加であり、新機能実装ではないため TDD RED-GREEN サイクルではなく:

- `test(filter): enhance X_reg with C-equivalent checks` — テスト強化コミット
- 複数テストの修正を1コミットにまとめず、1テスト=1コミットを基本とする
- 全テスト修正後に `cargo test --test filter` で通過を確認

### 実行方法

- サブエージェント（haiku）でC版・Rust版を読み比べ、修正コードを生成
- 2-3テストずつ並列処理
- 生成されたコードをレビューし、テスト通過を確認してからコミット

## PR 1: filter モジュール（8テスト）

ブランチ: `test/filter-regression-enhance`

### 修正対象と方針

#### 1. edge_reg（難易度: 低）

- **C**: 4チェック（Sobel水平/垂直、合成、8bpp出力）
- **Rust**: 7チェック（既に十分カバー）
- **方針**: RegParams + write_pix_and_check を追加し、golden比較を有効化

#### 2. enhance_reg（難易度: 低）

- **C**: 9チェック（gammaTRC, modifyHue, modifySaturation, contrastTRC, unsharp）
- **Rust**: 18+チェック（値比較は十分）
- **方針**: RegParams + write_pix_and_check を追加。pixMapWithInvariantHue は未実装 → #[ignore]

#### 3. adaptmap_reg（難易度: 低）

- **C**: 16チェック（background_norm, contrast_norm, gamma）
- **Rust**: 30+チェック（値比較は十分）
- **方針**: RegParams + write_pix_and_check を追加。低レベルmap関数は設計上非公開 → D扱い

#### 4. adaptnorm_reg（難易度: 中）

- **C**: 18チェック（contrast_norm + background_norm パイプライン）
- **Rust**: 24+チェック
- **方針**: RegParams + write_pix_and_check を追加。pixSeedfillGrayBasin は未実装 → #[ignore]

#### 5. convolve_reg（難易度: 中）

- **C**: 18チェック（blockconv, blockrank, blocksum, census, windowed stats）
- **Rust**: 9チェック
- **方針**: census_transform, windowed_mean/variance/stats のテストを追加。blockrank/blocksum は未実装 → #[ignore]

#### 6. rankhisto_reg（難易度: 低-中）

- **C**: 5チェック（rank color array, color mapping）
- **Rust**: 6チェック
- **方針**: RegParams + write_pix_and_check を追加。pixGetRankColorArray 未実装 → #[ignore]

#### 7. rankbin_reg（難易度: 中）

- **C**: 14チェック（rank bin values, discretize, strip）
- **Rust**: 6チェック
- **方針**: rank_filter テストを拡充。Numa rank bin 操作は未実装 → #[ignore]

#### 8. compfilter_reg（難易度: 高）

- **C**: 30+チェック（selectBySize, selectByPerimToAreaRatio, indicators）
- **Rust**: 6チェック
- **方針**: pix_select_by_size テストを拡充。Perimeter/area selectors は未実装 → #[ignore]

### 重要ファイル

- `tests/filter/*.rs` — 修正対象テスト
- `tests/common/params.rs` — RegParams, write_pix_and_check, compare_values
- `tests/common/mod.rs` — load_test_image, test_data_path
- `reference/leptonica/prog/*_reg.c` — C版リファレンス
- `src/filter/` — Rust実装（adaptmap.rs, convolve.rs, edge.rs, enhance.rs, rank.rs, windowed.rs）

### 既存パターン（再利用）

```rust
// テスト関数の基本構造
#[test]
fn testname_reg() {
    let mut rp = RegParams::new("testname");
    let pix = load_test_image("image.tif").expect("load");

    // 処理 + 検証
    let result = some_operation(&pix, params).expect("op");
    rp.write_pix_and_check(&result, ImageFormat::Png).expect("check");

    // 値比較
    rp.compare_values(expected, actual, delta);

    assert!(rp.cleanup(), "testname regression test failed");
}
```

## 検証

1. `cargo test --test filter` — 全テスト通過
2. `cargo clippy --all-features --all-targets -- -D warnings`
3. `cargo fmt --all -- --check`
4. `python3 scripts/audit-regression-tests.py` — 乖離スコア改善を確認
5. filter モジュールのB分類テスト数が減少していることを確認

## PR 1 bit一致検証結果

`examples/compare_golden.rs` + `scripts/golden_map.tsv` で C版goldenとRust版goldenをピクセル比較。

### 比較可能なチェックポイント

| テスト | C idx | 差異率 | MaxDiff | 分類 | Issue |
| ------ | ----- | ------ | ------- | ---- | ----- |
| edge Sobel H (1bpp) | 0 | 0.00% | 1 | fp差異 | #255 ✅修正済 |
| edge Sobel V (1bpp) | 1 | 0.00% | 1 | fp差異 | #255 ✅修正済 |
| edge OR combined | 2 | 0.00% | 1 | fp差異 | #255 ✅修正済 |
| edge 8bpp max(H,V) | 3 | 11.1% | 23 | JPEG codec差 | #255 ✅修正済(残差はcodec由来) |
| convolve blockconv gray | 0 | 6.16%(JPEG) / 0.83%(PNG) | 5/2 | JPEG codec差 | #257 ✅修正済(PR #262) |
| compfilter fill_closed_borders | 0 | 0.00% | 0 | 完全一致 | #256 ✅修正済 |
| compfilter render_hash_box | 1 | 0.00% | 0 | 完全一致 | #256 ✅修正済 |

### 比較不可能（DIM_MISMATCH）

- enhance: C版は20変異をタイル表示→1画像。Rust版は個別画像。直接比較不可
- adaptmap: C版とRust版でテスト構造が異なる（サブ画像抽出等）

### 根本原因と修正状況

- **edge (#255)** ✅: Rust版 `sobel_edge` の `>> 3` 正規化欠如 + ボーダー処理差異 → 1bpp極性修正(PR #256)で解決。1bppは完全一致、8bppの残差11%はJPEG codec差のみ
- **compfilter (#256)** ✅: `fill_closed_borders` / `render_hash_box` の1bpp極性反転 → PR #256で解決。完全一致(IDENTICAL)
- **convolve (#257)** ✅: `blockconv_gray` のボーダー正規化方式がC版と異なっていた → PR #262でC版 `blockconvLow()` の two-pass 方式に書き直し。PNG lossless比較でMaxDiff=2/0.83%（JPEG入力デコーダ差のみ）
