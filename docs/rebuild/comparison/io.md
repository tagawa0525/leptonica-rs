# leptonica-io: C版 vs Rust版 関数レベル比較

調査日: 2026-02-15

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 32 |
| 🔄 異なる | 15 |
| ❌ 未実装 | 99 |
| 合計 | 146 |

## 詳細

### bmpio.c (BMP I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamBmp | ✅ 同等 | `bmp::read_bmp` | Stream from reader |
| pixReadMemBmp | ✅ 同等 | `bmp::read_bmp` | Unified with stream reader |
| pixWriteStreamBmp | ✅ 同等 | `bmp::write_bmp` | Stream to writer |
| pixWriteMemBmp | ✅ 同等 | `bmp::write_bmp` | Unified with stream writer |

### pngio.c (PNG I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamPng | ✅ 同等 | `png::read_png` | Uses png crate |
| readHeaderPng | ❌ 未実装 | - | Header読み取りのみは未サポート |
| freadHeaderPng | ❌ 未実装 | - | Header読み取りのみは未サポート |
| readHeaderMemPng | ❌ 未実装 | - | Header読み取りのみは未サポート |
| fgetPngResolution | ❌ 未実装 | - | 解像度取得のみは未サポート |
| isPngInterlaced | ❌ 未実装 | - | Interlace判定は未サポート |
| fgetPngColormapInfo | ❌ 未実装 | - | Colormap情報取得は未サポート |
| pixWritePng | ✅ 同等 | `png::write_png` | Top level wrapper |
| pixWriteStreamPng | ✅ 同等 | `png::write_png` | Uses png crate |
| pixSetZlibCompression | ❌ 未実装 | - | 圧縮レベル設定は未サポート |
| l_pngSetReadStrip16To8 | ❌ 未実装 | - | 16bit→8bit変換フラグは未サポート |
| pixReadMemPng | ✅ 同等 | `png::read_png` | Unified with stream |
| pixWriteMemPng | ✅ 同等 | `png::write_png` | Unified with stream |

### jpegio.c (JPEG I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadJpeg | ✅ 同等 | `jpeg::read_jpeg` | Top level wrapper |
| pixReadStreamJpeg | ✅ 同等 | `jpeg::read_jpeg` | Uses jpeg-decoder crate |
| readHeaderJpeg | ❌ 未実装 | - | Header読み取りのみは未サポート |
| freadHeaderJpeg | ❌ 未実装 | - | Header読み取りのみは未サポート |
| fgetJpegResolution | ❌ 未実装 | - | 解像度取得のみは未サポート |
| fgetJpegComment | ❌ 未実装 | - | コメント取得は未サポート |
| pixWriteJpeg | 🔄 異なる | `jpeg::write_jpeg` | jpeg-encoder使用、C版はlibjpeg |
| pixWriteStreamJpeg | 🔄 異なる | `jpeg::write_jpeg` | jpeg-encoder使用 |
| pixReadMemJpeg | ✅ 同等 | `jpeg::read_jpeg` | Unified with stream |
| readHeaderMemJpeg | ❌ 未実装 | - | Memory版header読み取り未サポート |
| readResolutionMemJpeg | ❌ 未実装 | - | Memory版解像度取得未サポート |
| pixWriteMemJpeg | 🔄 異なる | `jpeg::write_jpeg_mem` | jpeg-encoder使用 |
| pixSetChromaSampling | ❌ 未実装 | - | Chroma sampling設定未サポート |

### pnmio.c (PNM/PBM/PGM/PPM I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamPnm | ✅ 同等 | `pnm::read_pnm` | PBM/PGM/PPM対応 |
| readHeaderPnm | ❌ 未実装 | - | Header読み取りのみは未サポート |
| freadHeaderPnm | ❌ 未実装 | - | Header読み取りのみは未サポート |
| pixWriteStreamPnm | ✅ 同等 | `pnm::write_pnm` | Binary format出力 |
| pixWriteStreamAsciiPnm | ❌ 未実装 | - | ASCII format出力は未サポート |
| pixWriteStreamPam | ❌ 未実装 | - | PAM format (P7) は未サポート |
| pixReadMemPnm | ✅ 同等 | `pnm::read_pnm` | Unified with stream |
| readHeaderMemPnm | ❌ 未実装 | - | Memory版header読み取り未サポート |
| pixWriteMemPnm | ✅ 同等 | `pnm::write_pnm` | Unified with stream |
| pixWriteMemPam | ❌ 未実装 | - | PAM format memory出力未サポート |

### tiffio.c (TIFF I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadTiff | ✅ 同等 | `tiff::read_tiff` | Top level wrapper |
| pixReadStreamTiff | ✅ 同等 | `tiff::read_tiff` | Uses tiff crate |
| pixWriteTiff | ✅ 同等 | `tiff::write_tiff` | Top level wrapper |
| pixWriteTiffCustom | ❌ 未実装 | - | カスタムタグ対応未実装 |
| pixWriteStreamTiff | ✅ 同等 | `tiff::write_tiff` | Uses tiff crate |
| pixWriteStreamTiffWA | ❌ 未実装 | - | Write-append mode未サポート |
| pixReadFromMultipageTiff | ✅ 同等 | `tiff::read_tiff_page` | 指定ページ読み取り |
| pixaReadMultipageTiff | ✅ 同等 | `tiff::read_tiff_multipage` | 全ページ読み取り |
| pixaWriteMultipageTiff | ✅ 同等 | `tiff::write_tiff_multipage` | 複数ページ書き込み |
| writeMultipageTiff | ✅ 同等 | `tiff::write_tiff_multipage` | 複数ページ書き込み |
| writeMultipageTiffSA | ❌ 未実装 | - | SARRAY版未実装 |
| fprintTiffInfo | ❌ 未実装 | - | TIFF情報表示は未サポート |
| tiffGetCount | ✅ 同等 | `tiff::tiff_page_count` | ページ数取得 |
| getTiffResolution | ✅ 同等 | `tiff::tiff_resolution` | 解像度取得 |
| readHeaderTiff | ❌ 未実装 | - | Header読み取りのみは未サポート |
| freadHeaderTiff | ❌ 未実装 | - | Header読み取りのみは未サポート |
| readHeaderMemTiff | ❌ 未実装 | - | Memory版header読み取り未サポート |
| findTiffCompression | ❌ 未実装 | - | 圧縮形式検出は未サポート |
| extractG4DataFromFile | ❌ 未実装 | - | G4データ抽出は未サポート |
| pixReadMemTiff | ✅ 同等 | `tiff::read_tiff` | Unified with stream |
| pixReadMemFromMultipageTiff | ✅ 同等 | `tiff::read_tiff_page` | Memory版ページ読み取り |
| pixaReadMemMultipageTiff | ✅ 同等 | `tiff::read_tiff_multipage` | Memory版全ページ読み取り |
| pixaWriteMemMultipageTiff | ✅ 同等 | `tiff::write_tiff_multipage` | Memory版複数ページ書き込み |
| pixWriteMemTiff | ✅ 同等 | `tiff::write_tiff` | Memory版書き込み |
| pixWriteMemTiffCustom | ❌ 未実装 | - | Memory版カスタムタグ未実装 |

### gifio.c (GIF I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamGif | ✅ 同等 | `gif::read_gif` | Uses gif crate |
| pixReadMemGif | ✅ 同等 | `gif::read_gif` | Unified with stream |
| pixWriteStreamGif | ✅ 同等 | `gif::write_gif` | Uses gif crate |
| pixWriteMemGif | ✅ 同等 | `gif::write_gif` | Unified with stream |

### webpio.c (WebP I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamWebP | ✅ 同等 | `webp::read_webp` | Uses webp crate |
| pixReadMemWebP | ✅ 同等 | `webp::read_webp` | Unified with stream |
| readHeaderWebP | ❌ 未実装 | - | Header読み取りのみは未サポート |
| readHeaderMemWebP | ❌ 未実装 | - | Memory版header読み取り未サポート |
| pixWriteWebP | ✅ 同等 | `webp::write_webp` | Top level wrapper |
| pixWriteStreamWebP | ✅ 同等 | `webp::write_webp` | Uses webp crate |
| pixWriteMemWebP | ✅ 同等 | `webp::write_webp` | Unified with stream |

### webpanimio.c (WebP Animation I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaWriteWebPAnim | ❌ 未実装 | - | アニメーションWebP未サポート |
| pixaWriteStreamWebPAnim | ❌ 未実装 | - | アニメーションWebP未サポート |
| pixaWriteMemWebPAnim | ❌ 未実装 | - | アニメーションWebP未サポート |

### jp2kio.c (JPEG 2000 I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadJp2k | ✅ 同等 | `jp2k::read_jp2k` | Top level wrapper |
| pixReadStreamJp2k | ✅ 同等 | `jp2k::read_jp2k` | Uses jpeg2000 crate |
| pixWriteJp2k | ❌ 未実装 | - | JP2K書き込み未実装 |
| pixWriteStreamJp2k | ❌ 未実装 | - | JP2K書き込み未実装 |
| pixReadMemJp2k | ✅ 同等 | `jp2k::read_jp2k_mem` | Memory版読み取り |
| pixWriteMemJp2k | ❌ 未実装 | - | Memory版書き込み未実装 |

### pdfio1.c (PDF I/O - High Level)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| convertFilesToPdf | ❌ 未実装 | - | ファイル群→PDF変換未実装 |
| saConvertFilesToPdf | ❌ 未実装 | - | SARRAY版ファイル群→PDF未実装 |
| saConvertFilesToPdfData | ❌ 未実装 | - | SARRAY版→PDFメモリ未実装 |
| selectDefaultPdfEncoding | ❌ 未実装 | - | デフォルトエンコード選択未実装 |
| convertUnscaledFilesToPdf | ❌ 未実装 | - | 非スケーリング版未実装 |
| saConvertUnscaledFilesToPdf | ❌ 未実装 | - | 非スケーリング版SARRAY未実装 |
| saConvertUnscaledFilesToPdfData | ❌ 未実装 | - | 非スケーリング版メモリ未実装 |
| convertUnscaledToPdfData | ❌ 未実装 | - | 非スケーリング版データ未実装 |
| pixaConvertToPdf | 🔄 異なる | `pdf::write_pdf_multi` | Pixa→PDF、異なるAPI |
| pixaConvertToPdfData | 🔄 異なる | `pdf::write_pdf_multi` | Pixa→PDFメモリ、異なるAPI |
| convertToPdf | ❌ 未実装 | - | 単一ページ変換未実装 |
| convertImageDataToPdf | ❌ 未実装 | - | 画像データ→PDF未実装 |
| convertToPdfData | ❌ 未実装 | - | 単一ページ→メモリ未実装 |
| convertImageDataToPdfData | ❌ 未実装 | - | 画像データ→メモリ未実装 |
| pixConvertToPdf | 🔄 異なる | `pdf::write_pdf` | Pix→PDF、シンプル化されたAPI |
| pixWriteStreamPdf | 🔄 異なる | `pdf::write_pdf` | Stream版、異なるAPI |
| pixWriteMemPdf | 🔄 異なる | `pdf::write_pdf_mem` | Memory版、異なるAPI |
| convertSegmentedFilesToPdf | ❌ 未実装 | - | セグメント化ファイル→PDF未実装 |
| convertNumberedMasksToBoxaa | ❌ 未実装 | - | マスク→BOXAA変換未実装 |
| convertToPdfSegmented | ❌ 未実装 | - | セグメント化→PDF未実装 |
| pixConvertToPdfSegmented | ❌ 未実装 | - | Pixセグメント化→PDF未実装 |
| convertToPdfDataSegmented | ❌ 未実装 | - | セグメント化→メモリ未実装 |
| pixConvertToPdfDataSegmented | ❌ 未実装 | - | Pixセグメント化→メモリ未実装 |
| concatenatePdf | ❌ 未実装 | - | PDF連結未実装 |
| saConcatenatePdf | ❌ 未実装 | - | SARRAY版PDF連結未実装 |
| ptraConcatenatePdf | ❌ 未実装 | - | PTRA版PDF連結未実装 |
| concatenatePdfToData | ❌ 未実装 | - | PDF連結→メモリ未実装 |
| saConcatenatePdfToData | ❌ 未実装 | - | SARRAY版PDF連結→メモリ未実装 |

### pdfio2.c (PDF I/O - Low Level)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixConvertToPdfData | 🔄 異なる | `pdf::write_pdf_mem` | 内部実装、異なるAPI |
| ptraConcatenatePdfToData | ❌ 未実装 | - | PTRA版連結未実装 |
| convertTiffMultipageToPdf | ❌ 未実装 | - | TIFF複数ページ→PDF未実装 |
| l_generateCIDataForPdf | ❌ 未実装 | - | CIデータ生成未実装 |
| l_generateCIData | ❌ 未実装 | - | CIデータ生成未実装 |
| l_generateFlateDataPdf | ❌ 未実装 | - | Flateデータ生成未実装 |
| l_generateJpegData | ❌ 未実装 | - | JPEGデータ生成未実装 |
| l_generateJpegDataMem | ❌ 未実装 | - | JPEGデータメモリ生成未実装 |
| l_generateG4Data | ❌ 未実装 | - | G4データ生成未実装 |
| pixGenerateCIData | ❌ 未実装 | - | PixからCIデータ生成未実装 |
| l_generateFlateData | ❌ 未実装 | - | Flateデータ生成未実装 |
| cidConvertToPdfData | ❌ 未実装 | - | CID→PDFデータ変換未実装 |
| l_CIDataDestroy | ❌ 未実装 | - | CIDataデストラクタ未実装 |
| getPdfPageCount | ❌ 未実装 | - | PDFページ数取得未実装 |
| getPdfPageSizes | ❌ 未実装 | - | PDFページサイズ取得未実装 |
| getPdfMediaBoxSizes | ❌ 未実装 | - | MediaBoxサイズ取得未実装 |
| getPdfRendererResolution | ❌ 未実装 | - | レンダラー解像度取得未実装 |
| l_pdfSetG4ImageMask | ❌ 未実装 | - | G4イメージマスク設定未実装 |
| l_pdfSetDateAndVersion | ❌ 未実装 | - | 日付・バージョン設定未実装 |

### psio1.c (PostScript I/O - High Level)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| convertFilesToPS | ❌ 未実装 | - | ファイル群→PS変換未実装 |
| sarrayConvertFilesToPS | ❌ 未実装 | - | SARRAY版ファイル群→PS未実装 |
| convertFilesFittedToPS | ❌ 未実装 | - | フィット版ファイル群→PS未実装 |
| sarrayConvertFilesFittedToPS | ❌ 未実装 | - | フィット版SARRAY未実装 |
| writeImageCompressedToPSFile | ❌ 未実装 | - | 圧縮画像→PSファイル未実装 |
| convertSegmentedPagesToPS | ❌ 未実装 | - | セグメント化ページ→PS未実装 |
| pixWriteSegmentedPageToPS | ❌ 未実装 | - | Pixセグメント化ページ→PS未実装 |
| pixWriteMixedToPS | ❌ 未実装 | - | 混合コンテンツ→PS未実装 |
| convertToPSEmbed | ❌ 未実装 | - | 埋め込みPS変換未実装 |
| pixaWriteCompressedToPS | ❌ 未実装 | - | Pixa圧縮→PS未実装 |
| pixWriteCompressedToPS | ❌ 未実装 | - | Pix圧縮→PS未実装 |

### psio2.c (PostScript I/O - Low Level)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixWritePSEmbed | 🔄 異なる | `ps::write_ps` | 埋め込みPS、異なるAPI |
| pixWriteStreamPS | 🔄 異なる | `ps::write_ps` | Stream版、異なるAPI |
| pixWriteStringPS | ❌ 未実装 | - | 文字列版未実装 |
| generateUncompressedPS | ❌ 未実装 | - | 非圧縮PS生成未実装 |
| convertJpegToPSEmbed | ❌ 未実装 | - | JPEG→PS埋め込み未実装 |
| convertJpegToPS | ❌ 未実装 | - | JPEG→PS変換未実装 |
| convertG4ToPSEmbed | ❌ 未実装 | - | G4→PS埋め込み未実装 |
| convertG4ToPS | ❌ 未実装 | - | G4→PS変換未実装 |
| convertTiffMultipageToPS | ❌ 未実装 | - | TIFF複数ページ→PS未実装 |
| convertFlateToPSEmbed | ❌ 未実装 | - | Flate→PS埋め込み未実装 |
| convertFlateToPS | ❌ 未実装 | - | Flate→PS変換未実装 |
| pixWriteMemPS | 🔄 異なる | `ps::write_ps_mem` | Memory版、異なるAPI |
| getResLetterPage | ✅ 同等 | `ps::get_res_letter_page` | レター用紙解像度計算 |
| l_psWriteBoundingBox | ❌ 未実装 | - | BoundingBox設定未実装 |

### readfile.c (汎用読み取り)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixaReadFiles | ❌ 未実装 | - | ディレクトリ読み取り未実装 |
| pixaReadFilesSA | ❌ 未実装 | - | SARRAY版読み取り未実装 |
| pixRead | ✅ 同等 | `read_image` | ファイルパスから読み取り |
| pixReadWithHint | ❌ 未実装 | - | ヒント付き読み取り未実装 |
| pixReadIndexed | ❌ 未実装 | - | インデックス指定読み取り未実装 |
| pixReadStream | ✅ 同等 | `read_image_format` | Stream読み取り |
| pixReadHeader | ❌ 未実装 | - | Header読み取り未実装 |
| findFileFormat | 🔄 異なる | `detect_format` | ファイルフォーマット検出 |
| findFileFormatStream | 🔄 異なる | `detect_format_from_bytes` | Stream版フォーマット検出 |
| findFileFormatBuffer | 🔄 異なる | `detect_format_from_bytes` | Buffer版フォーマット検出 |
| fileFormatIsTiff | ❌ 未実装 | - | TIFF判定未実装 |
| pixReadMem | ✅ 同等 | `read_image_mem` | Memory読み取り |
| pixReadHeaderMem | ❌ 未実装 | - | Memory版header読み取り未実装 |
| writeImageFileInfo | ❌ 未実装 | - | 画像ファイル情報書き込み未実装 |
| ioFormatTest | ❌ 未実装 | - | I/Oフォーマットテスト未実装 |

### writefile.c (汎用書き込み)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_jpegSetQuality | ❌ 未実装 | - | JPEG品質設定未実装 |
| setLeptDebugOK | ❌ 未実装 | - | デバッグ設定未実装 |
| pixaWriteFiles | ❌ 未実装 | - | Pixa複数ファイル書き込み未実装 |
| pixWriteDebug | ❌ 未実装 | - | デバッグ書き込み未実装 |
| pixWrite | ✅ 同等 | `write_image` | ファイルパスへ書き込み |
| pixWriteAutoFormat | ❌ 未実装 | - | 自動フォーマット書き込み未実装 |
| pixWriteStream | ✅ 同等 | `write_image_format` | Stream書き込み |
| pixWriteImpliedFormat | ❌ 未実装 | - | 拡張子から判定書き込み未実装 |
| pixChooseOutputFormat | ❌ 未実装 | - | 出力フォーマット選択未実装 |
| getImpliedFileFormat | ❌ 未実装 | - | 拡張子からフォーマット取得未実装 |
| getFormatFromExtension | ❌ 未実装 | - | 拡張子判定未実装 |
| pixGetAutoFormat | ❌ 未実装 | - | 自動フォーマット取得未実装 |
| getFormatExtension | ❌ 未実装 | - | フォーマット→拡張子変換未実装 |
| pixWriteMem | ✅ 同等 | `write_image_mem` | Memory書き込み |
| l_fileDisplay | ❌ 未実装 | - | ファイル表示未実装 |
| pixDisplay | ❌ 未実装 | - | Pix表示未実装 |
| pixDisplayWithTitle | ❌ 未実装 | - | タイトル付き表示未実装 |
| pixMakeColorSquare | ❌ 未実装 | - | カラー四角形生成未実装 |
| l_chooseDisplayProg | ❌ 未実装 | - | 表示プログラム選択未実装 |
| changeFormatForMissingLib | ❌ 未実装 | - | ライブラリ欠落時フォーマット変更未実装 |
| pixDisplayWrite | ❌ 未実装 | - | 表示用書き込み未実装 |

### spixio.c (SPIX serialization)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamSpix | ❌ 未実装 | - | SPIX読み取り未実装 |
| readHeaderSpix | ❌ 未実装 | - | SPIXヘッダー読み取り未実装 |
| freadHeaderSpix | ❌ 未実装 | - | SPIXヘッダーファイル読み取り未実装 |
| sreadHeaderSpix | ❌ 未実装 | - | SPIXヘッダー文字列読み取り未実装 |
| pixWriteStreamSpix | ❌ 未実装 | - | SPIX書き込み未実装 |
| pixReadMemSpix | ❌ 未実装 | - | SPIXメモリ読み取り未実装 |
| pixWriteMemSpix | ❌ 未実装 | - | SPIXメモリ書き込み未実装 |
| pixSerializeToMemory | ❌ 未実装 | - | Pixシリアライズ未実装 |
| pixDeserializeFromMemory | ❌ 未実装 | - | Pixデシリアライズ未実装 |

## 設計上の相違点

### 1. Stream vs Reader/Writer trait抽象化

**C版**: FILE*ポインタベース
- `pixReadStreamBmp(FILE *fp)`
- `pixWriteStreamBmp(FILE *fp, PIX *pix)`

**Rust版**: ジェネリックなRead/Write trait
- `read_bmp<R: Read>(reader: R)`
- `write_bmp<W: Write>(pix: &Pix, writer: W)`

### 2. Memory I/O の統合

**C版**: Stream版とMemory版が別関数
- `pixReadStreamBmp()` と `pixReadMemBmp()` が独立
- Memory版は内部でtempファイル使用の場合あり

**Rust版**: Read/Write traitで統一
- `std::io::Cursor<Vec<u8>>`を使えば同じ関数でMemory I/O可能
- Stream版とMemory版の区別なし

### 3. エラーハンドリング

**C版**: NULL返却 + グローバルエラーメッセージ
- `PIX *pixReadStreamBmp(FILE *fp)` → NULLでエラー
- エラー詳細は`ERROR_PTR`マクロ経由で出力

**Rust版**: Result型
- `IoResult<Pix>` で明示的なエラー情報
- `thiserror`によるstructured error

### 4. 依存ライブラリ

**C版**:
- libjpeg
- libpng
- libtiff
- giflib
- libwebp
- openjpeg

**Rust版**:
- jpeg-decoder / jpeg-encoder
- png crate
- tiff crate
- gif crate (image-rsベース)
- webp crate
- jpeg2000 crate

### 5. 未実装の主要機能カテゴリ

1. **Header-only読み取り**: 画像本体を読まずにメタデータのみ取得する関数群
2. **PostScript高レベル機能**: 複数ファイル→PS、セグメント化PS等
3. **PDF高レベル機能**: 複数ファイル→PDF、PDF連結、セグメント化PDF等
4. **SPIX serialization**: Leptonica独自のシリアライゼーション形式
5. **アニメーションWebP**: WebPアニメーション対応
6. **Display機能**: pixDisplay等のGUI表示機能
7. **品質・圧縮設定**: グローバル変数による品質/圧縮レベル設定

## 推奨される次のステップ

### 優先度: 高

1. **Header読み取り機能**: メタデータのみ取得する軽量API
   - 各フォーマット用の`read_header_*`関数
   - `ImageHeader { width, height, depth, format, ... }`型の導入

2. **JPEG品質設定**: `JpegOptions`構造体でのオプション指定
   - C版の`pixSetChromaSampling`相当

3. **PNG圧縮レベル設定**: `PngOptions`構造体でのオプション指定
   - C版の`pixSetZlibCompression`相当

### 優先度: 中

4. **PDF高レベル機能**:
   - 複数画像→単一PDF (`pixaConvertToPdf`相当)
   - PDF連結 (`concatenatePdf`相当)

5. **PostScript基本機能**:
   - 複数画像→PSファイル
   - 圧縮PS出力

6. **TIFF拡張機能**:
   - カスタムタグ対応 (`pixWriteTiffCustom`相当)

### 優先度: 低

7. **アニメーションWebP**: 静止画中心なら不要
8. **SPIX serialization**: Leptonica特有、他形式で代替可能
9. **Display機能**: I/Oライブラリの範囲外

## まとめ

Rust版leptonica-ioは、基本的な画像I/O機能（BMP, PNG, JPEG, TIFF, GIF, WebP, JP2K）の読み書きは実装済みで、C版の約22%の関数が同等または類似の機能を提供している。

未実装の68%は主に以下のカテゴリ:
- Header-only読み取り（メタデータのみ）
- PDF/PS高レベル変換機能
- 品質・圧縮レベルのグローバル設定
- GUI表示機能
- Leptonica独自フォーマット（SPIX）

Rust版は外部crateを活用したモダンなI/O抽象化（Read/Write trait）を採用し、C版より型安全でメモリ安全なAPIを提供している。
