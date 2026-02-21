# leptonica-io 全未実装関数の移植計画

Status: IMPLEMENTED

## Context

leptonica-io crateは10フォーマットの基本的なread/writeを実装済みだが、
C版leptonicaの I/O関数（約146関数）に対して以下の重要な機能が欠落している:

1. **JPEG書き込み** - 最も広く使われる画像フォーマットなのにread-only
2. **SPIX形式** - leptonica独自シリアライゼーション形式が完全未実装
3. **ヘッダー読み取り** - ピクセルデコードなしにメタデータ取得する手段がない
4. **フォーマットユーティリティ** - 拡張子からのフォーマット判定がない
5. **PNM拡張** - ASCII書き込み(P1/P2/P3)とPAM形式(P7)が未対応
6. **TIFF拡張** - ページ数取得、解像度取得、追記モードが未対応
7. **PDF/PS拡張** - DCT(JPEG)圧縮オプションが未対応

### スコープ除外（Rust移植に不適切なもの）

| 除外対象 | 理由 |
|----------|------|
| `pixDisplay`, `l_fileDisplay`, `l_chooseDisplayProg` | システム依存の表示機能 |
| `l_jpegSetQuality`, `pixSetZlibCompression` 等 | Rustではオプション構造体で対応済み |
| C版の file/stream/mem 3パターン分離 | Rustでは `Read`/`Write` トレイトで統一済み |
| `getPdfPageCount`, `getPdfPageSizes` | PDF解析ライブラリが必要（pdf-writerは書き込み専用） |
| `concatenatePdf`, `saConcatenatePdfToData` | 同上 |
| JP2K書き込み | `hayro-jpeg2000` はデコーダ専用、pure Rustエンコーダなし |
| WebPアニメーション | `image-webp` はアニメーション書き込み非対応 |
| `convertSegmentedPagesToPS`, `pixWriteSegmentedPageToPS` | 複雑なセグメント描画 |
| `convertJpegToPSEmbed`, `convertG4ToPSEmbed` | 生フォーマットデータの直接埋め込み |
| `pixaReadFiles`, `pixaWriteFiles` | アプリケーション層で `read_image`/`write_image` をループするだけ |
| `ioFormatTest`, `writeImageFileInfo`, `pixWriteDebug` | テスト/デバッグ用ユーティリティ |
| `pixUninterlaceGIF` | `gif` crateが内部で脱インターレース済み |

---

## 実行順序

Phase 1 → 2 → 3 → 4 → 5 → 6 → 7 の順に直列で実行する。

```
Phase 1 (JPEG書き込み) ← 最重要、Phase 6/7のJPEG圧縮を可能にする
  → Phase 2 (SPIX形式)
    → Phase 3 (ヘッダー読み取り + フォーマットユーティリティ)
      → Phase 4 (PNM拡張: ASCII書き込み + PAM)
        → Phase 5 (TIFF拡張)
          → Phase 6 (PDF DCT圧縮)
            → Phase 7 (PS拡張)
```

## 新規依存

| crate | version | feature | 用途 |
|-------|---------|---------|------|
| `jpeg-encoder` | latest stable | `jpeg` feature | JPEG エンコード |

---

## Phase 1: JPEG書き込み（1 PR）

**Status: IMPLEMENTED** (PR #118)

**C参照**: `reference/leptonica/src/jpegio.c` L250-500

### 実装内容

- `write_jpeg<W: Write>(pix: &Pix, writer: W, options: &JpegOptions) -> IoResult<()>`
- `JpegOptions` 構造体: `quality: u8` (1-100, default 75)

### 変換ルール

| 入力深度 | 出力 |
|----------|------|
| 1bpp | 8bpp grayscale に変換 |
| 2bpp/4bpp | 8bpp grayscale に変換 |
| 8bpp (colormap有) | RGB に展開 |
| 8bpp (grayscale) | そのまま L8 |
| 16bpp | 8bpp に変換（上位バイト） |
| 32bpp (spp=3/4) | RGB24 (alpha無視) |

### 修正ファイル

- `Cargo.toml`（workspace）: `jpeg-encoder` 追加
- `crates/leptonica-io/Cargo.toml`: `jpeg-encoder` optional dep, `jpeg` feature に追加
- `crates/leptonica-io/src/jpeg.rs`: `write_jpeg`, `JpegOptions` 追加
- `crates/leptonica-io/src/lib.rs`: `write_image_format` の `Jpeg` 分岐追加

### テスト

- 8bpp grayscale ラウンドトリップ（read → write → read → ピクセル比較、JPEG非可逆なので閾値付き）
- 32bpp RGB ラウンドトリップ
- colormapped 画像の書き込み
- テスト画像: `karen8.jpg`, `fish24.jpg`, `marge.jpg`

---

## Phase 2: SPIX形式（1 PR）

**Status: IMPLEMENTED** (PR #119)

**C参照**: `reference/leptonica/src/spixio.c` 全504行

### バイナリフォーマット

```
"spix"    (4 bytes) -- マジックID
w         (4 bytes) -- 幅
h         (4 bytes) -- 高さ
d         (4 bytes) -- 深度
wpl       (4 bytes) -- words per line
ncolors   (4 bytes) -- カラーマップエントリ数; 0ならカラーマップなし
cdata     (4 * ncolors bytes) -- カラーマップデータ (RGBA per entry)
rdatasize (4 bytes) -- ラスタデータサイズ = 4 * wpl * h
rdata     (rdatasize bytes) -- 生ラスタデータ
```

### 実装内容

- `read_spix<R: Read>(reader: R) -> IoResult<Pix>`
- `write_spix<W: Write>(pix: &Pix, writer: W) -> IoResult<()>`
- バリデーション: `MaxWidth = 1_000_000`, `MaxHeight = 1_000_000`, `MaxArea = 400_000_000`
- エンディアン: C版はネイティブエンディアン。Rust版もネイティブエンディアンを使用（C版との互換性優先）

### 修正ファイル

- `crates/leptonica-io/src/spix.rs`（新規）
- `crates/leptonica-io/src/lib.rs`: `pub mod spix`, `read_image_format`/`write_image_format` に `Spix` 追加
- `crates/leptonica-io/src/format.rs`: SPIX マジックナンバー `b"spix"` 追加

### テスト

- 1bpp/8bpp/32bpp ラウンドトリップ
- colormapped 画像のラウンドトリップ
- 不正データ（切り詰め、サイズ超過、マジック不一致）のエラーハンドリング

---

## Phase 3: ヘッダー読み取り + フォーマットユーティリティ（1 PR）

**Status: IMPLEMENTED** (PR #120)

**C参照**: `reference/leptonica/src/readfile.c` L415-549, `writefile.c` L563-691

### 実装内容

#### ImageHeader 構造体

```rust
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub depth: u32,        // ビット深度
    pub bps: u32,          // bits per sample
    pub spp: u32,          // samples per pixel
    pub has_colormap: bool,
    pub format: ImageFormat,
    pub x_resolution: Option<u32>,  // DPI
    pub y_resolution: Option<u32>,  // DPI
}
```

#### ユニバーサルヘッダー読み取り

- `read_image_header<P: AsRef<Path>>(path: P) -> IoResult<ImageHeader>`
- `read_image_header_mem(data: &[u8]) -> IoResult<ImageHeader>`

#### フォーマット別ヘッダー読み取り（内部関数、ユニバーサルAPIから呼び出し）

- `bmp::read_header_bmp` - BMP info header 解析
- `pnm::read_header_pnm` - PNMヘッダー解析
- `png::read_header_png` - `png` crateのデコーダでIHDRチャンク + pHYsチャンク
- `jpeg::read_header_jpeg` - `jpeg-decoder` の info 取得
- `tiff::read_header_tiff` - `tiff` crateのディレクトリ情報
- `gif::read_header_gif` - GIF論理スクリーン記述子
- `webp::read_header_webp` - VP8/VP8L/VP8Xチャンク解析
- `jp2k::read_header_jp2k` - JP2コンテナ/J2Kコードストリームヘッダー
- `spix::read_header_spix` - 先頭24バイト解析

#### フォーマットユーティリティ

- `ImageFormat::from_extension(ext: &str) -> Option<ImageFormat>` - 拡張子→フォーマット
- `ImageFormat::from_path(path: &Path) -> Option<ImageFormat>` - パスから拡張子抽出→フォーマット
- `choose_output_format(pix: &Pix) -> ImageFormat` - 深度/colormapに基づく自動選択
- `write_image_auto<P: AsRef<Path>>(pix: &Pix, path: P) -> IoResult<()>` - 拡張子推定による書き込み

#### 拡張子マッピング（C版 writefile.c L153-170 準拠）

```
.bmp → Bmp, .jpg/.jpeg → Jpeg, .png → Png,
.tif/.tiff → Tiff, .pbm/.pgm/.pnm/.ppm → Pnm,
.gif → Gif, .jp2/.j2k → Jp2, .ps → Ps, .pdf → Lpdf,
.webp → WebP, .spix → Spix
```

### 修正ファイル

- `crates/leptonica-io/src/lib.rs`: `ImageHeader`, `read_image_header`, `read_image_header_mem`, `choose_output_format`, `write_image_auto` 追加
- `crates/leptonica-core/src/pix/mod.rs`: `ImageFormat::from_extension`, `from_path` メソッド追加
- 各フォーマットモジュール: `read_header_*` 関数追加

### テスト

- 各フォーマットのヘッダー読み取りテスト（寸法・深度・フォーマットの正確性）
- 拡張子マッピングテスト（全パターン + 大文字小文字）
- `choose_output_format` のロジックテスト

---

## Phase 4: PNM拡張 - ASCII書き込み + PAM（1 PR）

**Status: IMPLEMENTED** (PR #122)

**C参照**: `reference/leptonica/src/pnmio.c` L400-700

### 実装内容

- `write_pnm_ascii<W: Write>(pix: &Pix, writer: W) -> IoResult<()>` - P1/P2/P3 ASCII形式
- `read_pam<R: Read>(reader: R) -> IoResult<Pix>` - P7 PAM読み込み
- `write_pam<W: Write>(pix: &Pix, writer: W) -> IoResult<()>` - P7 PAM書き込み
- `PnmType` enum に `Pam` バリアント追加

### PAMヘッダー形式

```
P7
WIDTH <int>
HEIGHT <int>
DEPTH <int>        ; spp (1, 2, 3, or 4)
MAXVAL <int>       ; 255
TUPLTYPE <string>  ; BLACKANDWHITE, GRAYSCALE, RGB, RGB_ALPHA
ENDHDR
<binary data>
```

### 修正ファイル

- `crates/leptonica-io/src/pnm.rs`: ASCII write, PAM read/write 追加
- `crates/leptonica-io/src/format.rs`: `P7` マジック追加
- `crates/leptonica-io/src/lib.rs`: PAMフォーマットのディスパッチ追加（PnmとしてまとめるかPamを別にするか要検討）

### テスト

- ASCII PNM (P1/P2/P3) 書き込み → 読み込みラウンドトリップ
- PAM ラウンドトリップ（grayscale, RGB, RGBA）
- 既存テスト `pnmio_reg.rs` の拡張

---

## Phase 5: TIFF拡張（1 PR）

**Status: IMPLEMENTED** (PR #124)

**C参照**: `reference/leptonica/src/tiffio.c`

### 実装内容

- `tiff_page_count<R: Read + Seek>(reader: R) -> IoResult<u32>` - ページ数取得
- `tiff_resolution<R: Read + Seek>(reader: R) -> IoResult<(f32, f32)>` - X/Y解像度(DPI)取得
- `tiff_compression<R: Read + Seek>(reader: R) -> IoResult<TiffCompression>` - 圧縮方式検出
- `write_tiff_append<W: Write + Seek>(pix: &Pix, writer: W, compression: TiffCompression) -> IoResult<()>` - 既存TIFFへの追記

### 除外

- `pixWriteTiffCustom` - `tiff` crateがカスタムタグ書き込みAPIを公開していない
- `extractG4DataFromFile` - PS/PDF埋め込み用の特殊用途
- `fprintTiffInfo` - デバッグ出力

### 修正ファイル

- `crates/leptonica-io/src/tiff.rs`: 上記4関数追加

### テスト

- `tiff_page_count` でマルチページTIFFのページ数確認
- `tiff_resolution` で解像度取得確認
- `write_tiff_append` でページ追記→全ページ読み込みで確認
- テスト画像: `feyn.tif`, マルチページTIFF

---

## Phase 6: PDF DCT圧縮（1 PR）

**Status: IMPLEMENTED** (PR #126)

**C参照**: `reference/leptonica/src/pdfio2.c`

### 実装内容

- `PdfCompression::Jpeg` バリアント追加
- `PdfOptions::jpeg_quality` フィールドの実装（Phase 1のJPEGエンコーダを利用）
- PDF内でDCTFilter付きImage XObjectとしてJPEG圧縮データを埋め込む
- `write_pdf_from_files` - ファイルパス群からマルチページPDF生成

### 修正ファイル

- `crates/leptonica-io/src/pdf.rs`: JPEG圧縮パス追加、`write_pdf_from_files`

### テスト

- JPEG圧縮PDFの生成（ヘッダー検証 + ファイルサイズがFlateより小さいこと）
- `write_pdf_from_files` テスト

---

## Phase 7: PS拡張（1 PR）

**Status: IMPLEMENTED** (PR #128)

**C参照**: `reference/leptonica/src/psio1.c`, `psio2.c`

### 実装内容

- `write_ps_multi<W: Write>(images: &[&Pix], writer: W, options: &PsOptions) -> IoResult<()>` - マルチページPS
- `PsLevel::Level2` バリアント - DCT(JPEG)圧縮PS（Phase 1のJPEGエンコーダを利用）

### 修正ファイル

- `crates/leptonica-io/src/ps/mod.rs`: マルチページ対応、Level 2追加

### テスト

- マルチページPS生成（ヘッダー検証、ページ数確認）
- Level 2 JPEG圧縮PSの生成

---

## サマリー

| Phase | 対象 | PR数 | 関数数 |
|-------|------|------|--------|
| 1 | JPEG書き込み | 1 | 2 (write_jpeg + JpegOptions) |
| 2 | SPIX形式 | 1 | 2 (read_spix + write_spix) |
| 3 | ヘッダー + ユーティリティ | 1 | ~14 (ImageHeader + 9 format headers + 4 utils) |
| 4 | PNM拡張 | 1 | 3 (ascii write + pam read/write) |
| 5 | TIFF拡張 | 1 | 4 (page_count + resolution + compression + append) |
| 6 | PDF DCT圧縮 | 1 | 2 (Jpeg compression + from_files) |
| 7 | PS拡張 | 1 | 2 (multi-page + Level2) |
| **合計** | | **7** | **~29** |

C版の約146関数のうち:
- 既存実装でカバー済み: ~97関数（trait統一により少数のRust関数で対応）
- 本計画で追加: ~29関数
- スコープ除外(N/A): ~20関数（display, global state, PDF parsing, segmented等）
- 外部依存未対応: ~7関数（JP2K write, WebP anim）

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
└── feat/io-jpeg-write        ← Phase 1
└── feat/io-spix               ← Phase 2
└── feat/io-header-utils        ← Phase 3
└── feat/io-pnm-ext             ← Phase 4
└── feat/io-tiff-ext             ← Phase 5
└── feat/io-pdf-dct              ← Phase 6
└── feat/io-ps-ext               ← Phase 7
```

## 検証方法

各PRで以下を実行:

```bash
cargo fmt --check -p leptonica-io
cargo clippy -p leptonica-io -- -D warnings
cargo test -p leptonica-io
cargo test --workspace  # PR前に全ワークスペーステスト
```
