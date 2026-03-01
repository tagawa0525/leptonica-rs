# Phase 3 PR 2/8: morph モジュール回帰テスト強化

Status: IMPLEMENTED

## Context

Phase 3 PR 1/8（filter, #258）がマージ済み。PR 2/8 として morph モジュールの
B分類テスト8件を強化する。filter で確立したパターン（RegParams + write_pix_and_check）
を踏襲し、C版チェックポイントとの対応を追加する。

## 修正対象（8テスト）

### 1. binmorph1_reg（難易度: 低）

- **C**: 1チェック（TestAll 集約結果の compare_values）
- **Rust現状**: 1テスト関数、7 compare_values（単調性・冪等性）
- **方針**: write_pix_and_check を追加し、代表的な dilate/erode/open/close 結果をgolden化。

  C版は複数アルゴリズム比較（rasterop, dwa, sequence）だが、Rust版は brick 実装のみ。
  DWA/sequence 比較は Rust API に存在しないため #[ignore] スタブ不要（テスト構造が異なる）。
  既存 compare_values による性質検証は残し、write_pix_and_check を追加する。

### 2. binmorph3_reg（難易度: 低）

- **C**: 1チェック（TestAll 集約結果の compare_values）
- **Rust現状**: 1テスト関数、9 compare_values（identity、separability）
- **方針**: binmorph1 と同様。write_pix_and_check を追加し、DWA/separable 比較結果をgolden化。

  C版の DWA vs rasterop 直接比較は Rust に DWA 実装がないため不可。
  brick 操作の分離可能性検証をgolden化して検証品質を向上。

### 3. ccthin1_reg（難易度: 中）

- **C**: 11チェック（全て write_pix_and_check — SEL可視化画像）
- **Rust現状**: 4テスト関数、~25 compare_values（SEL構造・カウント）
- **方針**: `thin_connected` の結果画像を write_pix_and_check でgolden化。

  C版の `selaDisplayInPix` はRust未実装 → #[ignore] スタブ。
  実際のthinning結果画像の検証を追加する。

### 4. ccthin2_reg（難易度: 中）

- **C**: 19チェック（全て write_pix_and_check — thinning結果画像）
- **Rust現状**: 4テスト関数、~15 compare_values（寸法保持・反拡張性）
- **方針**: `thin_connected` / `thin_connected_by_set` の結果画像を write_pix_and_check。

  C版の stroke width normalization（pixaDisplayTiledAndScaled）は Rust 未実装 → #[ignore]。
  5種の sel set によるthinning結果をgolden化。

### 5. colormorph_reg（難易度: 低）

- **C**: 8チェック（4 write_pix_and_check + 4 compare_pix）
- **Rust現状**: 1テスト関数、8 compare_values（チャネル単調性）
- **方針**: dilate/erode/open/close_color の結果を write_pix_and_check。

  C版の compare_pix は `pixColorMorphSequence` との比較 → `color_morph_sequence` がRust実装済みと
  判明したため、4つの compare_pix チェックも追加（C版と同一構造）。

### 6. fhmtauto_reg（難易度: 中）

- **C**: 20チェック（全て compare_pix — auto-gen FHMT vs HMT 比較）
- **Rust現状**: 2テスト関数 + 1 #[ignore]、~9 compare_values
- **方針**: `hit_miss_transform` の結果を write_pix_and_check でgolden化。

  C版の `pixFHMTGen_1` / `pixHMTDwa_1`（自動生成DWA）は Rust 未実装、既存 #[ignore] 維持。
  利用可能な thin sel set による HMT 結果画像のgolden検証を追加。

### 7. graymorph2_reg（難易度: 低）

- **C**: 12チェック（全て compare_pix — special case vs general 比較）
- **Rust現状**: 2テスト関数、8 compare_values（寸法・平均値単調性）
- **方針**: `dilate_gray` / `erode_gray` / `open_gray` / `close_gray` の結果を

  write_pix_and_check。C版は `pixDilateGray3` vs `pixDilateGray` の等価比較だが、
  Rust には `_3` 最適化バリアントがないため、一般実装結果のgolden化に留める。
  tophat/hdome テストの追加は可能（Rust 実装あり）。

### 8. selio_reg（難易度: 低）

- **C**: 8チェック（SELA I/O + 可視化）
- **Rust現状**: 18テスト関数 + 4 #[ignore]、~60 compare_values
- **方針**: Rust は SEL API テストが既に充実（C版より多い）。

  SELA file I/O（selaWrite/selaRead）は Rust 未実装、既存 #[ignore] 維持。
  SEL作成・操作結果の write_pix_and_check を追加（SEL→Pix変換が可能な場合）。
  追加できるgoldenが限定的なため、軽微な改善に留まる見込み。

## 重要ファイル

- `tests/morph/*.rs` — 修正対象テスト
- `reference/leptonica/prog/{binmorph1,binmorph3,ccthin1,ccthin2,colormorph,fhmtauto,graymorph2,selio}_reg.c`
- `src/morph/` — Rust実装（binary.rs, thin.rs, color.rs, grayscale.rs, sel.rs）
- `tests/common/params.rs` — RegParams インフラ

## 検証

1. `cargo test --test morph` — 全テスト通過
2. `cargo clippy --all-features --all-targets -- -D warnings`
3. `cargo fmt --all -- --check`
4. C版golden比較: `cargo run --example compare_golden --features all-formats -- --module morph`

   （golden_map.tsv にmorph用マッピングを追加した上で）
