# leptonica-recog 全未実装関数の移植計画

Status: PLANNED

## Context

leptonica-recog crateはbarcode、recog（文字認識）、jbclass（JBIG2）、dewarp（歪み補正）、
skew（傾き検出）、baseline（ベースライン検出）の基本機能を実装済みだが、
C版leptonicaのrecog関数群（約150+関数）に対して以下の重要な機能が欠落している:

1. **Recogシリアライゼーション** - 学習済みモデルの読み書きが未実装
2. **Recogアクセサ・設定** - テンプレート数取得、クラス名取得等が未実装
3. **Bootstrap digit recognizer** - ブートストラップ数字認識器が未実装
4. **高度な識別・フィルタリング** - 前処理フィルタ、数字抽出が未実装
5. **Dewarpシリアライゼーション** - 歪みモデルの読み書きが未実装
6. **Dewarpaコンテナ** - マルチページ歪み管理コンテナが未実装
7. **Dewarp高度モデル** - ページモデル構築、LSF、テキストライン検出拡張が未実装
8. **JbClassシリアライゼーション** - 分類データのI/Oが未実装
9. **Skew拡張** - sweep/search variants、deskew variantsが未実装
10. **Baseline拡張** - local skew、deskew localが未実装
11. **Barcode拡張** - 検出精度向上、追加フォーマットが未実装

### 現状の実装状況

| モジュール | 実装済み関数数 | 状態 |
|-----------|-------------|------|
| recog/ | ~15 | コア train/identify済み、I/O/bootstrap未対応 |
| jbclass/ | ~4 | init/add_page済み、I/O/word detection未対応 |
| dewarp/ | ~8 | single page済み、dewarpa/I/O未対応 |
| skew.rs | ~2 | 基本skew検出済み、variants未対応 |
| baseline.rs | ~2 | 基本baseline済み、local skew未対応 |
| barcode/ | ~12 | 7フォーマット対応済み、検出精度向上必要 |
| pageseg.rs | ~3 | 基本セグメンテーション済み |

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `recogShowAverageTemplates`, `recogShowContent`, `recogShowMatch` 等 | デバッグ可視化 |
| `recogShowMatchesInRange`, `recogDebugAverages` | デバッグ可視化 |
| `dewarpShowResults`, `dewarpDebug`, `dewarpaShowArrays` | デバッグ可視化 |
| `dewarpaShowSampledLines`, `dewarpShowBendingModel`, `dewarpShowDistortion` | デバッグ可視化 |
| `pixDisplayOutliers`, `recogDisplayOutlier` | デバッグ可視化 |
| `recogShowPath` | デバッグ可視化 |
| `showExtractNumbers`, `l_showIndicatorSplitValues` | デバッグ可視化 |
| `jbDataRender` | 可視化 |
| `pixRenderHorizEndPoints`, `pixRenderMidYs` | 可視化 |
| `dewarpShowReso` | デバッグ可視化 |
| `recogSkipIdentify` | デバッグ用スキップ関数 |

---

## 実行順序

Phase 1 → 2 → ... → 13 の順に直列で実行する。

```
Phase 1 (Recog I/O) ← 基盤。学習済みモデルの永続化
  → Phase 2 (Recog query) ← I/Oで読んだモデルの検査
    → Phase 3 (Bootstrap digit) ← trainedモデルを使用
      → Phase 4 (高度な識別) ← query結果を使用
        → Phase 5 (Dewarp I/O) ← 歪みモデルの永続化
          → Phase 6 (Dewarpa管理) ← マルチページ歪み管理
            → Phase 7 (Dewarpa モデル管理) ← dewarpaの検証・設定
              → Phase 8 (Dewarp2高度モデル) ← Phase 6/7のdewarpaを使用
                → Phase 9 (Dewarp3/4拡張適用)
                  → Phase 10 (JbClass I/O + 拡張)
                    → Phase 11 (Skew拡張)
                      → Phase 12 (Baseline拡張)
                        → Phase 13 (Barcode拡張)
```

---

## Phase 1: Recog シリアライゼーション（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/recogbasic.c` L800-1100

### 実装内容

- `Recog::read<R: Read>(reader: R) -> RecogResult<Recog>` - バイナリ形式からの読み込み
- `Recog::read_from_file(path: &Path) -> RecogResult<Recog>` - ファイルから読み込み
- `Recog::write<W: Write>(&self, writer: W) -> RecogResult<()>` - バイナリ形式への書き込み
- `Recog::write_to_file(&self, path: &Path) -> RecogResult<()>` - ファイルへ書き込み
- `Recog::extract_pixa(&self) -> RecogResult<Pixa>` - テンプレート画像のPixaとして抽出
- `Recog::create_from_pixa(pixa: &Pixa, params: &RecogParams) -> RecogResult<Recog>` - Pixaから再構築

### シリアライゼーション形式

C版のバイナリ形式に互換:
1. バージョン番号
2. RecogParams（テンプレートサイズ、最小/最大幅高等）
3. テンプレートPixaの全データ
4. クラスラベル文字列

### 修正ファイル

- `crates/leptonica-recog/src/recog/io.rs`（新規）
- `crates/leptonica-recog/src/recog/mod.rs`: `pub mod io` 追加

### テスト

- 空のRecogのwrite/readラウンドトリップ
- 学習済みRecogのwrite/readラウンドトリップ（テンプレート数・クラス数一致）
- extract_pixa → create_from_pixa のラウンドトリップ

---

## Phase 2: Recog query/inspection（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/recogbasic.c` L500-800

### 実装内容

- `Recog::get_count(&self) -> usize` - テンプレート総数
- `Recog::get_class_count(&self) -> usize` - クラス数
- `Recog::get_class_index(&self, class_str: &str) -> Option<usize>` - クラス名→インデックス
- `Recog::get_class_string(&self, index: usize) -> Option<&str>` - インデックス→クラス名
- `Recog::string_to_index(class_str: &str) -> RecogResult<usize>` - 文字列のUTF-8コードポイントをインデックスに変換
- `Recog::set_params(&mut self, params: RecogParams)` - パラメータ更新

```rust
pub struct RecogParams {
    pub scaleh: u32,           // テンプレート正規化高さ
    pub linew: u32,            // ストローク幅正規化
    pub threshold: f32,        // 最低相関閾値
    pub maxyshift: u32,        // Y方向最大シフト
    pub charset_type: CharsetType,
    pub min_nopad: u32,        // パディング不要の最小テンプレート数
}
```

### 修正ファイル

- `crates/leptonica-recog/src/recog/types.rs`: `RecogParams` 構造体追加・拡張
- `crates/leptonica-recog/src/recog/train.rs`: query メソッド追加

### テスト

- get_count / get_class_count の正確性
- get_class_index / get_class_string の双方向変換
- set_params 後の動作変更確認

---

## Phase 3: Bootstrap digit recognizer（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/recogtrain.c` L800-1100

### 実装内容

- `Recog::make_boot_digit_recog(scale_h: u32) -> RecogResult<Recog>` - ブートストラップ数字認識器（0-9）を組み込みテンプレートから生成
- `Recog::train_from_boot(&mut self, boot_recog: &Recog) -> RecogResult<()>` - ブートストラップ認識器からの転移学習
- `Recog::pad_digit_training_set(&mut self, scale_h: u32) -> RecogResult<()>` - 不足する数字クラスをパディング
- `Recog::is_padding_needed(&self) -> bool` - パディングが必要か判定

### 動作

ブートストラップ認識器は内蔵のデフォルト数字テンプレートから作成される。
これを「教師」として使い、少数のサンプルからでも全10クラスの認識器を
構築できるようにする。

### 修正ファイル

- `crates/leptonica-recog/src/recog/train.rs`: 上記関数追加

### テスト

- make_boot_digit_recog で10クラスの認識器生成
- pad_digit_training_set でクラス数が10になること
- ブートストラップ認識器での基本認識テスト

---

## Phase 4: 高度な識別・フィルタリング（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/recogident.c` L600-900

### 実装内容

- `Recog::pre_splitting_filter(pix: &Pix) -> RecogResult<PreFilterResult>` - 分割前の前処理フィルタ（ノイズ除去、サイズ検証）
- `Recog::splitting_filter(pix: &Pix, min_aspect: f32, max_aspect: f32) -> RecogResult<bool>` - 分割フィルタ（アスペクト比チェック）
- `Recog::extract_numbers(pix: &Pix, sa: &Sarray, naid: &Numa) -> RecogResult<(Sarray, Numa, Pixa, Numa)>` - 画像から数字列を抽出

Outlier除去:
- `Recog::remove_outliers(&mut self, min_score: f32, min_fraction: f32, target: OutlierTarget) -> RecogResult<Recog>` - 相関スコアが低いテンプレートを除去
- `Recog::filter_pixa_by_size(pixa: &Pixa, min_w: u32, max_w: u32, min_h: u32, max_h: u32) -> RecogResult<Pixa>` - サイズによるテンプレートフィルタリング

```rust
pub enum OutlierTarget {
    /// 各クラスの平均テンプレートとの比較
    Average,
    /// 各クラスの最良テンプレートとの比較
    Individual,
}

pub struct PreFilterResult {
    pub is_valid: bool,
    pub width: u32,
    pub height: u32,
    pub reason: Option<String>,
}
```

### 修正ファイル

- `crates/leptonica-recog/src/recog/ident.rs`: 上記関数追加

### テスト

- pre_splitting_filter でノイズ画像の棄却確認
- splitting_filter のアスペクト比閾値検証
- remove_outliers でスコア低いテンプレートの除去確認

---

## Phase 5: Dewarp シリアライゼーション（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/dewarp1.c` L900-1200

### 実装内容

- `Dewarp::read<R: Read>(reader: R) -> RecogResult<Dewarp>` - 歪みモデル読み込み
- `Dewarp::write<W: Write>(&self, writer: W) -> RecogResult<()>` - 歪みモデル書き込み
- `Dewarp::read_from_file(path: &Path) -> RecogResult<Dewarp>` - ファイルから読み込み
- `Dewarp::write_to_file(&self, path: &Path) -> RecogResult<()>` - ファイルへ書き込み

### シリアライゼーション形式

1. バージョン番号
2. ページ番号、参照ページフラグ
3. 垂直/水平ディスパリティFPix（存在する場合）
4. モデル構築パラメータ

### 修正ファイル

- `crates/leptonica-recog/src/dewarp/io.rs`（新規）
- `crates/leptonica-recog/src/dewarp/mod.rs`: `pub mod io` 追加

### テスト

- Dewarp write/read ラウンドトリップ
- ディスパリティデータの保持確認
- 空のDewarp（モデル未構築）のI/O

---

## Phase 6: Dewarpa コンテナ管理（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/dewarp1.c` L70-400

### 実装内容

```rust
pub struct Dewarpa {
    dewarp_array: Vec<Option<Dewarp>>,
    max_pages: usize,
    // 設定
    max_linecurv: f32,
    min_lines: u32,
    max_edgeslope: f32,
    max_edgecurv: f32,
    max_diff: u32,
    use_both: bool,
    check_columns: bool,
}
```

- `Dewarpa::new(max_pages: usize, sampling: u32, reduction: u32, min_lines: u32, max_dist: u32) -> Dewarpa`
- `Dewarpa::insert(&mut self, dewarp: Dewarp) -> RecogResult<()>` - ページ番号に基づいて挿入
- `Dewarpa::get(&self, page: usize) -> Option<&Dewarp>` - ページ番号で取得
- `Dewarpa::destroy_dewarp(&mut self, page: usize)` - 特定ページのDewarpを削除
- `Dewarpa::set_curvatures(&mut self, max_linecurv: f32, min_lines: u32, max_edgeslope: f32, max_edgecurv: f32, max_diff: u32)` - 曲率制限設定
- `Dewarpa::use_both_arrays(&mut self, use_both: bool)` - 垂直/水平両方使用の設定
- `Dewarpa::set_check_columns(&mut self, check: bool)` - カラムチェック設定
- `Dewarpa::set_max_distance(&mut self, max_dist: u32)` - 最大参照ページ距離

I/O:
- `Dewarpa::read<R: Read>(reader: R) -> RecogResult<Dewarpa>` - 読み込み
- `Dewarpa::write<W: Write>(&self, writer: W) -> RecogResult<()>` - 書き込み

### 修正ファイル

- `crates/leptonica-recog/src/dewarp/dewarpa.rs`（新規）
- `crates/leptonica-recog/src/dewarp/mod.rs`: `pub mod dewarpa` 追加

### テスト

- Dewarpa作成、insert、getの基本操作
- 複数ページのDewarp管理
- Dewarpa I/Oラウンドトリップ
- 設定パラメータの動作確認

---

## Phase 7: Dewarpa モデル管理（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/dewarp4.c` L70-400

### 実装内容

- `Dewarpa::insert_ref_models(&mut self, use_both: bool) -> RecogResult<u32>` - 隣接ページモデルを参照モデルとして挿入
- `Dewarpa::use_single_model(&mut self, page: usize, use_both: bool) -> RecogResult<()>` - 全ページに単一モデル適用
- `Dewarpa::swap_pages(&mut self, page1: usize, page2: usize) -> RecogResult<()>` - ページ入れ替え
- `Dewarp::create_ref(page: usize, ref_page: usize) -> Dewarp` - 参照Dewarp作成（他ページのモデルを参照）
- `Dewarp::minimize(&mut self)` - メモリ節約のためフルレゾリューションデータを破棄
- `Dewarpa::strip_ref_models(&mut self)` - 全参照モデルを削除

### 修正ファイル

- `crates/leptonica-recog/src/dewarp/dewarpa.rs`: 上記関数追加
- `crates/leptonica-recog/src/dewarp/types.rs`: 参照ページフラグ追加

### テスト

- insert_ref_models でモデルがないページに参照が挿入されること
- use_single_model で全ページが同一モデルを参照
- minimize後もapply_disparityが動作すること

---

## Phase 8: Dewarp2 高度なモデル構築（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/dewarp2.c` 全体

### 実装内容

- `Dewarp::build_page_model(pix: &Pix) -> RecogResult<Dewarp>` - フルページモデル構築（テキストライン検出→ディスパリティ計算を統合）
- `Dewarp::find_vert_disparity(&mut self, pix: &Pix) -> RecogResult<()>` - 垂直ディスパリティの改良版（LSFフィッティング）
- `Dewarp::find_horiz_disparity(&mut self, pix: &Pix) -> RecogResult<()>` - 水平ディスパリティの改良版
- `pix_find_textline_flow_direction(pix: &Pix) -> RecogResult<f32>` - テキストラインの流れ方向検出
- `Dewarp::populate_full_res(&mut self, pix: &Pix, x: u32, y: u32) -> RecogResult<()>` - フルレゾリューションディスパリティ生成

### 修正ファイル

- `crates/leptonica-recog/src/dewarp/model.rs`: 上記関数追加・拡張
- `crates/leptonica-recog/src/dewarp/textline.rs`: flow direction検出追加

### テスト

- build_page_model で歪んだ文書のモデル構築
- populate_full_res の解像度精度検証
- テスト画像: 歪みのある文書画像

---

## Phase 9: Dewarp3/4 拡張適用（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/dewarp3.c`, `dewarp4.c`

### 実装内容

- `Dewarpa::apply_disparity(page: usize, pix: &Pix, x: u32, y: u32) -> RecogResult<Pix>` - dewarpaコンテナ経由のディスパリティ適用
- `Dewarpa::apply_disparity_boxa(page: usize, boxa: &Boxa) -> RecogResult<Boxa>` - 矩形群へのディスパリティ適用
- `dewarp_single_page_init(pix: &Pix) -> RecogResult<Dewarpa>` - 単一ページの初期化ワンショット
- `dewarp_single_page_run(dewarpa: &Dewarpa, pix: &Pix) -> RecogResult<Pix>` - 単一ページの適用ワンショット

### 修正ファイル

- `crates/leptonica-recog/src/dewarp/apply.rs`: dewarpa経由の適用追加
- `crates/leptonica-recog/src/dewarp/dewarpa.rs`: apply_disparity/boxa追加
- `crates/leptonica-recog/src/dewarp/mod.rs`: single_page_init/run追加

### テスト

- Dewarpa経由のdewarp適用の正確性
- boxa適用で矩形座標が正しく変換されること
- single_page_init/run のワンショットAPI動作確認

---

## Phase 10: JbClass シリアライゼーション + 拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/jbclass.c` L1500-1900

### 実装内容

I/O:
- `JbData::write<W: Write>(&self, writer: W) -> RecogResult<()>` - 分類データ書き込み
- `JbData::read<R: Read>(reader: R) -> RecogResult<JbData>` - 分類データ読み込み
- `JbData::write_to_file(&self, path: &Path) -> RecogResult<()>` - ファイルへ書き込み
- `JbData::read_from_file(path: &Path) -> RecogResult<JbData>` - ファイルから読み込み

Word detection:
- `pix_word_mask_by_dilation(pix: &Pix, max_dil: u32) -> RecogResult<(Pix, u32)>` - dilationによる単語マスク生成
- `pix_word_boxes_by_dilation(pix: &Pix, max_dil: u32) -> RecogResult<Boxa>` - dilationによる単語矩形検出

### 修正ファイル

- `crates/leptonica-recog/src/jbclass/io.rs`（新規）
- `crates/leptonica-recog/src/jbclass/classify.rs`: word detection追加
- `crates/leptonica-recog/src/jbclass/mod.rs`: `pub mod io` 追加

### テスト

- JbData write/read ラウンドトリップ
- pix_word_mask_by_dilation で単語領域の検出確認
- pix_word_boxes_by_dilation の矩形精度

---

## Phase 11: Skew拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/skew.c` L200-700

### 実装内容

高レベルインターフェース:
- `deskew(pix: &Pix) -> RecogResult<Pix>` - 自動deskew（angle検出+回転）
- `deskew_both(pix: &Pix) -> RecogResult<(Pix, Pix)>` - 元画像と1bpp版の両方deskew
- `deskew_general(pix: &Pix, options: &DeskewOptions) -> RecogResult<(Pix, f32)>` - オプション付きdeskew

Sweep and search variants:
- `find_skew_sweep_and_search(pix: &Pix, options: &SkewSearchOptions) -> RecogResult<SkewResult>` - 粗いsweep + 精密search
- `find_skew_sweep_and_search_score(pix: &Pix, options: &SkewSearchOptions) -> RecogResult<(f32, f32, f32)>` - スコア付き（angle, conf, endscore）
- `find_skew_sweep_and_search_score_pivot(pix: &Pix, options: &SkewSearchOptions, pivot: SkewPivot) -> RecogResult<(f32, f32, f32)>` - ピボット指定

```rust
pub struct DeskewOptions {
    pub reduce_factor: u32,    // 縮小係数（1, 2, or 4）
    pub sweep_range: f32,      // sweep角度範囲
    pub sweep_delta: f32,      // sweep角度刻み
    pub search_reduction: u32, // search時の追加縮小
    pub threshold: u32,        // 2値化閾値
}

pub enum SkewPivot {
    Corner,    // 左上角
    Center,    // 画像中心
}
```

### 修正ファイル

- `crates/leptonica-recog/src/skew.rs`: 上記関数・構造体追加

### テスト

- deskew の自動角度検出精度（既知角度の画像）
- sweep_and_search のsweep範囲パラメータ効果
- pivot=Corner と pivot=Center の結果比較
- テスト画像: 既知角度で回転させた文書画像

---

## Phase 12: Baseline拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/baseline.c` L300-500

### 実装内容

- `deskew_local(pix: &Pix, nslice: u32, reduction: u32, redsweep: u32, redsearch: u32, sweep_range: f32, sweep_delta: f32, min_bs_delta: f32) -> RecogResult<Pix>` - 局所的なdeskew（画像を水平に分割して各スライスを個別にdeskew）
- `get_local_skew_transform(nslice: u32, ny: u32, reduction: u32, angles: &[f32], cx: f32, cy: f32) -> RecogResult<Pta>` - 局所skew角度から変換マップ生成
- `get_local_skew_angles(pix: &Pix, nslice: u32, reduction: u32, sweep_range: f32, sweep_delta: f32, min_bs_delta: f32) -> RecogResult<(Numa, f32, f32)>` - 各スライスのskew角度計算

### 動作

文書画像を水平スライスに分割し、各スライスで独立にskew角度を測定する。
これにより、ページ全体が均一に傾いていない場合（曲がった本のページ等）にも
局所的な歪み補正が可能。

### 修正ファイル

- `crates/leptonica-recog/src/baseline.rs`: 上記関数追加

### テスト

- deskew_local で局所的な傾きが補正されること
- get_local_skew_angles で各スライスの角度が取得できること
- 均一傾きの画像ではglobal deskewと同等の結果

---

## Phase 13: Barcode拡張（1 PR）

**Status: PLANNED**

**C参照**: `reference/leptonica/src/bardecode.c`, barcode検出関連

### 実装内容

検出精度向上:
- `locate_barcodes_morphological(pix: &Pix) -> RecogResult<Boxa>` - 形態学ベースのbarcode領域検出（現在のplaceholder実装を完全実装に置き換え）
- `extract_barcode_widths(pix: &Pix, direction: Direction) -> RecogResult<Numa>` - バーコードのbar幅配列抽出
- `find_barcode_peaks(numa: &Numa, threshold: f32) -> RecogResult<Numa>` - 幅ヒストグラムのピーク検出
- `barcode_gen_mask(pix: &Pix) -> RecogResult<Pix>` - バーコードマスク生成

追加フォーマット（可能であれば）:
- Code 128 サポート
- EAN-8 サポート

### 修正ファイル

- `crates/leptonica-recog/src/barcode/detect.rs`: 形態学検出の完全実装
- `crates/leptonica-recog/src/barcode/signal.rs`: width extraction、peak detection追加
- `crates/leptonica-recog/src/barcode/formats/code128.rs`（新規、可能であれば）
- `crates/leptonica-recog/src/barcode/formats/ean8.rs`（新規、可能であれば）

### テスト

- locate_barcodes_morphological でバーコード領域の検出率
- width extraction の精度検証
- 追加フォーマットのデコードテスト
- テスト画像: バーコード含む文書画像

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | Recog I/O | 1 | 6 |
| 2 | Recog query/inspection | 1 | 6 |
| 3 | Bootstrap digit recognizer | 1 | 4 |
| 4 | 高度な識別・フィルタリング | 1 | 5 |
| 5 | Dewarp I/O | 1 | 4 |
| 6 | Dewarpa コンテナ管理 | 1 | 10 |
| 7 | Dewarpa モデル管理 | 1 | 6 |
| 8 | Dewarp2 高度モデル構築 | 1 | 5 |
| 9 | Dewarp3/4 拡張適用 | 1 | 4 |
| 10 | JbClass I/O + 拡張 | 1 | 6 |
| 11 | Skew拡張 | 1 | 6 |
| 12 | Baseline拡張 | 1 | 3 |
| 13 | Barcode拡張 | 1 | ~5 |
| **合計** | | **13** | **~70** |

## 共通ワークフロー

### TDD

1. **RED**: テスト作成コミット（`#[ignore = "not yet implemented"]`付き）
2. **GREEN**: 実装コミット（`#[ignore]`除去、テスト通過）
3. **REFACTOR**: 必要に応じてリファクタリングコミット

### PRワークフロー

1. `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --all -- --check`
2. `/gh-pr-create` でPR作成
3. `/gh-actions-check` でCopilotレビュー到着を確認
4. `/gh-pr-review` でレビューコメント対応
5. CIパス確認後 `/gh-pr-merge --merge` でマージ
6. ブランチ削除

### ブランチ命名

```
main
└── feat/recog-io               ← Phase 1
└── feat/recog-query             ← Phase 2
└── feat/recog-bootstrap         ← Phase 3
└── feat/recog-ident-ext         ← Phase 4
└── feat/dewarp-io               ← Phase 5
└── feat/dewarp-dewarpa          ← Phase 6
└── feat/dewarp-model-mgmt       ← Phase 7
└── feat/dewarp-advanced-model   ← Phase 8
└── feat/dewarp-apply-ext        ← Phase 9
└── feat/jbclass-io              ← Phase 10
└── feat/skew-ext                ← Phase 11
└── feat/baseline-ext            ← Phase 12
└── feat/barcode-ext             ← Phase 13
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-recog
cargo clippy -p leptonica-recog -- -D warnings
cargo test -p leptonica-recog
cargo test --workspace  # PR前に全ワークスペーステスト
```
