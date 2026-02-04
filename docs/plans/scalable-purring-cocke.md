# C版Leptonicaテストのleptonica-rs移植戦略

## 概要

C版Leptonicaの回帰テスト（305個、うち160個が`*_reg.c`形式）をRust版に移植する戦略。

## 現状分析

### C版テスト

- **場所**: `reference/leptonica/prog/`
- **回帰テスト**: 160個（`*_reg.c`）
- **フレームワーク**: `L_REGPARAMS`（regutils.c）
  - 3モード: generate / compare / display
  - 関数: `regTestSetup`, `regTestCleanup`, `regTestComparePix`, `regTestWritePixAndCheck`等
- **テスト画像**: 262個（feyn.tif, test1.png, marge.jpg等）

### Rust版現状

- **テスト数**: 231個（インラインユニットテスト）
- **回帰テスト**: なし
- **テストデータ**: メモリ内生成のみ（外部画像なし）

---

## 実装計画

### Phase 0: テストインフラ構築

**目標**: 回帰テストフレームワークの作成

#### 1. `crates/leptonica-test`クレート作成

```rust
// src/lib.rs - 主要API
pub struct RegParams {
    pub test_name: String,
    pub index: usize,
    pub mode: RegTestMode,
    pub success: bool,
}

pub enum RegTestMode { Generate, Compare, Display }

impl RegParams {
    pub fn new(test_name: &str) -> Self;
    pub fn compare_values(
        &mut self,
        expected: f64,
        actual: f64,
        delta: f64,
    ) -> bool;
    pub fn compare_pix(&mut self, pix1: &Pix, pix2: &Pix) -> bool;
    pub fn write_pix_and_check(
        &mut self,
        pix: &Pix,
        format: ImageFormat,
    ) -> IoResult<()>;
    pub fn cleanup(self) -> bool;
}
```

#### 2. ディレクトリ構造

```text
tests/
├── data/images/          # テスト画像（Git LFS管理）
│   ├── feyn.tif
│   ├── test1.png
│   └── marge.jpg
├── golden/               # ゴールデンファイル
├── integration/          # 回帰テストコード
│   ├── region/
│   │   └── conncomp_reg.rs
│   ├── io/
│   ├── morph/
│   └── ...
└── common/               # 共通ユーティリティ
    ├── mod.rs
    └── test_images.rs
```

#### 3. 最小テスト画像セット（5ファイル）

- `feyn.tif` - 二値画像（モルフォロジー、連結成分）
- `test1.png` - 二値画像（回転）
- `marge.jpg` - RGB画像（色処理）
- `test8.jpg` - グレースケール（フィルタ）
- `rabi.png` - ドキュメント画像

---

### Phase 1: コア回帰テスト移植

**対象**: 実装済み機能の基本テスト（10個）

| テスト | C版ソース | 対象クレート |
| --- | --- | --- |
| conncomp_reg | conncomp_reg.c | leptonica-region |
| binmorph1_reg | binmorph1_reg.c | leptonica-morph |
| binmorph3_reg | binmorph3_reg.c | leptonica-morph |
| rotate1_reg | rotate1_reg.c | leptonica-transform |
| scale_reg | scale_reg.c | leptonica-transform |
| pngio_reg | pngio_reg.c | leptonica-io |
| pnmio_reg | pnmio_reg.c | leptonica-io |
| ioformats_reg | ioformats_reg.c | leptonica-io |
| colorspace_reg | colorspace_reg.c | leptonica-color |
| binarize_reg | binarize_reg.c | leptonica-color |

---

### Phase 2: I/O回帰テスト移植（5個）

| テスト | C版ソース |
| --- | --- |
| jpegio_reg | jpegio_reg.c |
| gifio_reg | gifio_reg.c |
| tiffio_reg | tiffio_reg.c |
| mtiff_reg | mtiff_reg.c |
| writeread_reg | writeread_reg.c |

---

### Phase 3: 高度な処理テスト移植（10個）

| テスト | C版ソース | 対象クレート |
| --- | --- | --- |
| skew_reg | skew_reg.c | leptonica-recog |
| baseline_reg | baseline_reg.c | leptonica-recog |
| pageseg_reg | pageseg_reg.c | leptonica-recog |
| convolve_reg | convolve_reg.c | leptonica-filter |
| edge_reg | edge_reg.c | leptonica-filter |
| watershed_reg | watershed_reg.c | leptonica-region |
| seedfill_reg | seedfill_reg.c | leptonica-region |
| label_reg | label_reg.c | leptonica-region |
| grayquant_reg | grayquant_reg.c | leptonica-color |
| colorquant_reg | colorquant_reg.c | leptonica-color |

---

### Phase 4: CI/ドキュメント整備

- GitHub Actions設定（Git LFS対応）
- テスト実行ガイド作成
- ゴールデンファイル生成・管理フロー文書化

---

## 移植例: conncomp_reg.rs

```rust
// tests/integration/region/conncomp_reg.rs
use leptonica_test::{RegParams, reg_test};
use leptonica_region::{find_connected_components, ConnectivityType};

#[test]
fn conncomp_reg() {
    let mut rp = RegParams::new("conncomp");

    let pixs = load_test_image("feyn.tif").unwrap();

    // 4連結テスト
    let comps_4 = find_connected_components(&pixs, ConnectivityType::FourWay).unwrap();
    let n1 = comps_4.len();
    rp.compare_values(n1 as f64, n1 as f64, 0.0);  // 0
    rp.compare_values(4452.0, n1 as f64, 0.0);     // 2: C版期待値

    // 8連結テスト
    let comps_8 = find_connected_components(&pixs, ConnectivityType::EightWay).unwrap();
    let n2 = comps_8.len();
    rp.compare_values(4305.0, n2 as f64, 0.0);     // 7: C版期待値

    assert!(rp.cleanup());
}
```

---

## テスト実行コマンド

```bash
# 比較モード（通常実行）
REGTEST_MODE=compare cargo test --test '*_reg'

# ゴールデンファイル生成
REGTEST_MODE=generate cargo test --test '*_reg'

# 特定テストのみ
cargo test conncomp_reg -- --nocapture
```

---

## 重要ファイル

| ファイル | 用途 |
| --- | --- |
| `reference/leptonica/src/regutils.c` | C版フレームワーク参照 |
| `reference/leptonica/prog/conncomp_reg.c` | 移植対象テスト例 |
| `crates/leptonica-region/src/conncomp.rs` | 既存Rust実装 |

---

## 検証方法

1. `cargo test`で全ユニットテストがパス
2. `REGTEST_MODE=compare cargo test --test '*_reg'`で回帰テストがパス
3. C版と同じ期待値（例: 4連結=4452, 8連結=4305）で成功
