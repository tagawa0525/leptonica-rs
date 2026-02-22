# leptonica-io: C版 vs Rust版 関数レベル比較

調査日: 2026-02-22（700_recog-full-porting Phase 1-13 全完了を反映）

## サマリー

| 項目 | 数 |
|------|-----|
| ✅ 同等 | 68 |
| 🔄 異なる | 17 |
| ❌ 未実装 | 61 |
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
| readHeaderPng | ✅ 同等 | `png::read_header_png` | IHDR + pHYsチャンク解析 |
| freadHeaderPng | ✅ 同等 | `png::read_header_png` | Unified with stream |
| readHeaderMemPng | ✅ 同等 | `png::read_header_png` | Unified with stream |
| fgetPngResolution | ✅ 同等 | `png::read_header_png` | ImageHeader.x/y_resolution |
| isPngInterlaced | ❌ 未実装 | - | Interlace判定は未サポート |
| fgetPngColormapInfo | ❌ 未実装 | - | Colormap詳細情報取得は未サポート |
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
| readHeaderJpeg | ✅ 同等 | `jpeg::read_header_jpeg` | jpeg-decoderでinfo取得 |
| freadHeaderJpeg | ✅ 同等 | `jpeg::read_header_jpeg` | Unified with stream |
| fgetJpegResolution | ✅ 同等 | `jpeg::read_header_jpeg` | ImageHeader.x/y_resolution |
| fgetJpegComment | ❌ 未実装 | - | コメント取得は未サポート |
| pixWriteJpeg | 🔄 異なる | `jpeg::write_jpeg` | jpeg-encoder使用、C版はlibjpeg |
| pixWriteStreamJpeg | 🔄 異なる | `jpeg::write_jpeg` | jpeg-encoder使用 |
| pixReadMemJpeg | ✅ 同等 | `jpeg::read_jpeg` | Unified with stream |
| readHeaderMemJpeg | ✅ 同等 | `jpeg::read_header_jpeg` | Unified with stream |
| readResolutionMemJpeg | ✅ 同等 | `jpeg::read_header_jpeg` | ImageHeader.x/y_resolution |
| pixWriteMemJpeg | 🔄 異なる | `write_image_mem` → `jpeg::write_jpeg` | 統一メモリI/O API経由 |
| pixSetChromaSampling | ❌ 未実装 | - | Chroma sampling設定未サポート |

### pnmio.c (PNM/PBM/PGM/PPM/PAM I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadStreamPnm | ✅ 同等 | `pnm::read_pnm` | PBM/PGM/PPM/PAM対応 |
| readHeaderPnm | ✅ 同等 | `pnm::read_header_pnm` | PNMヘッダー解析 |
| freadHeaderPnm | ✅ 同等 | `pnm::read_header_pnm` | Unified with stream |
| pixWriteStreamPnm | ✅ 同等 | `pnm::write_pnm` | Binary format出力 |
| pixWriteStreamAsciiPnm | ✅ 同等 | `pnm::write_pnm_ascii` | P1/P2/P3 ASCII形式 |
| pixWriteStreamPam | ✅ 同等 | `pnm::write_pam` | P7 PAM形式 |
| pixReadMemPnm | ✅ 同等 | `pnm::read_pnm` | Unified with stream |
| readHeaderMemPnm | ✅ 同等 | `pnm::read_header_pnm` | Unified with stream |
| pixWriteMemPnm | ✅ 同等 | `pnm::write_pnm` | Unified with stream |
| pixWriteMemPam | ✅ 同等 | `pnm::write_pam` | Unified with stream |

### tiffio.c (TIFF I/O)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| pixReadTiff | ✅ 同等 | `tiff::read_tiff` | Top level wrapper |
| pixReadStreamTiff | ✅ 同等 | `tiff::read_tiff` | Uses tiff crate |
| pixWriteTiff | ✅ 同等 | `tiff::write_tiff` | Top level wrapper |
| pixWriteTiffCustom | ❌ 未実装 | - | カスタムタグ対応未実装 |
| pixWriteStreamTiff | ✅ 同等 | `tiff::write_tiff` | Uses tiff crate |
| pixWriteStreamTiffWA | 🔄 異なる | `tiff::write_tiff_append` | read-all-rewrite方式 |
| pixReadFromMultipageTiff | ✅ 同等 | `tiff::read_tiff_page` | 指定ページ読み取り |
| pixaReadMultipageTiff | ✅ 同等 | `tiff::read_tiff_multipage` | 全ページ読み取り |
| pixaWriteMultipageTiff | ✅ 同等 | `tiff::write_tiff_multipage` | 複数ページ書き込み |
| writeMultipageTiff | ✅ 同等 | `tiff::write_tiff_multipage` | 複数ページ書き込み |
| writeMultipageTiffSA | ❌ 未実装 | - | SARRAY版未実装 |
| fprintTiffInfo | ❌ 未実装 | - | TIFF情報表示は未サポート |
| tiffGetCount | ✅ 同等 | `tiff::tiff_page_count` | ページ数取得 |
| getTiffResolution | ✅ 同等 | `tiff::tiff_resolution` | 解像度取得 |
| readHeaderTiff | ✅ 同等 | `tiff::read_header_tiff` | TIFFディレクトリ情報 |
| freadHeaderTiff | ✅ 同等 | `tiff::read_header_tiff` | Unified with stream |
| readHeaderMemTiff | ✅ 同等 | `tiff::read_header_tiff` | Unified with stream |
| findTiffCompression | ✅ 同等 | `tiff::tiff_compression` | 圧縮形式検出 |
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
| readHeaderWebP | ✅ 同等 | `webp::read_header_webp` | VP8/VP8L/VP8Xチャンク解析 |
| readHeaderMemWebP | ✅ 同等 | `webp::read_header_webp` | Unified with stream |
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
| convertFilesToPdf | 🔄 異なる | `pdf::write_pdf_from_files` | パス群→PDF、異なるAPI |
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
| pixaWriteCompressedToPS | 🔄 異なる | `ps::write_ps_multi` | マルチページPS、異なるAPI |
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
| pixReadHeader | ✅ 同等 | `read_image_header` | ユニバーサルヘッダー読み取り |
| findFileFormat | 🔄 異なる | `detect_format` | ファイルフォーマット検出 |
| findFileFormatStream | 🔄 異なる | `detect_format_from_bytes` | Stream版フォーマット検出 |
| findFileFormatBuffer | 🔄 異なる | `detect_format_from_bytes` | Buffer版フォーマット検出 |
| fileFormatIsTiff | ❌ 未実装 | - | TIFF判定未実装 |
| pixReadMem | ✅ 同等 | `read_image_mem` | Memory読み取り |
| pixReadHeaderMem | ✅ 同等 | `read_image_header_mem` | Memory版header読み取り |
| writeImageFileInfo | ❌ 未実装 | - | 画像ファイル情報書き込み未実装 |
| ioFormatTest | ❌ 未実装 | - | I/Oフォーマットテスト未実装 |

### writefile.c (汎用書き込み)
| C関数 | 状態 | Rust対応 | 備考 |
|-------|------|----------|------|
| l_jpegSetQuality | ❌ 未実装 | - | グローバル品質設定（RustはJpegOptionsで対応） |
| setLeptDebugOK | ❌ 未実装 | - | デバッグ設定未実装 |
| pixaWriteFiles | ❌ 未実装 | - | Pixa複数ファイル書き込み未実装 |
| pixWriteDebug | ❌ 未実装 | - | デバッグ書き込み未実装 |
| pixWrite | ✅ 同等 | `write_image` | ファイルパスへ書き込み |
| pixWriteAutoFormat | ✅ 同等 | `write_image_auto` | 拡張子推定による書き込み |
| pixWriteStream | ✅ 同等 | `write_image_format` | Stream書き込み |
| pixWriteImpliedFormat | ✅ 同等 | `write_image_auto` | 拡張子から判定書き込み |
| pixChooseOutputFormat | ✅ 同等 | `choose_output_format` | 深度/colormapに基づく自動選択 |
| getImpliedFileFormat | ✅ 同等 | `ImageFormat::from_path` | パスからフォーマット取得 |
| getFormatFromExtension | ✅ 同等 | `ImageFormat::from_extension` | 拡張子判定 |
| pixGetAutoFormat | ✅ 同等 | `choose_output_format` | 自動フォーマット取得 |
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
| pixReadStreamSpix | ✅ 同等 | `spix::read_spix` | SPIX読み取り |
| readHeaderSpix | ✅ 同等 | `spix::read_header_spix` | 先頭24バイト解析 |
| freadHeaderSpix | ✅ 同等 | `spix::read_header_spix` | Unified with stream |
| sreadHeaderSpix | ✅ 同等 | `spix::read_header_spix` | Unified with stream |
| pixWriteStreamSpix | ✅ 同等 | `spix::write_spix` | SPIX書き込み |
| pixReadMemSpix | ✅ 同等 | `spix::read_spix` | Unified with stream |
| pixWriteMemSpix | ✅ 同等 | `spix::write_spix` | Unified with stream |
| pixSerializeToMemory | ✅ 同等 | `spix::write_spix` | Pixシリアライズ |
| pixDeserializeFromMemory | ✅ 同等 | `spix::read_spix` | Pixデシリアライズ |

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
- gif crate
- image-webp crate
- hayro-jpeg2000 crate
- pdf-writer (PDF出力)
- miniz_oxide (Flate圧縮)

### 5. 未実装の主要機能カテゴリ

1. **PDF高レベル変換機能**: 複数ファイル→PDF（セグメント化、連結等）
2. **PostScript高レベル機能**: セグメント化PS、生フォーマット埋め込み
3. **アニメーションWebP**: WebPアニメーション対応
4. **Display機能**: pixDisplay等のGUI表示機能
5. **品質・圧縮設定**: グローバル変数による品質/圧縮レベル設定（RustではOptions構造体で対応済み）

## まとめ

Rust版leptonica-ioは、IO全移植計画（Phase 1-7）の完了により、C版146関数のうち85関数（58.2%）が同等または類似の機能を提供している（32.2% → 58.2%に改善）。

主な追加機能:
- JPEG書き込み（Phase 1）
- SPIX形式の読み書き（Phase 2）
- 全フォーマットのヘッダー読み取り + フォーマットユーティリティ（Phase 3）
- PNM ASCII書き込み + PAM形式（Phase 4）
- TIFF圧縮検出 + 追記モード（Phase 5）
- PDF DCT（JPEG）圧縮（Phase 6）
- PS マルチページ + Level 2 DCT圧縮（Phase 7）

残りの未実装42%は主に:
- PDF/PS高レベル変換・セグメント化機能
- GUI表示機能（pixDisplay等）
- アニメーションWebP
- JP2K書き込み（pure Rustエンコーダなし）
