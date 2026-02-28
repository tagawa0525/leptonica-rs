# feature-comparison 検証レポート

調査日: 2026-02-27
ステータス: **修正済み** — 発見された乖離は各comparison/*.mdおよびfeature-comparison.mdに反映済み

## 概要

`docs/porting/feature-comparison.md` および `docs/porting/comparison/` 配下の各モジュール比較文書を、実際のRust実装と照合し正確性を検証した。

### 検証結果サマリー

| ファイル     | 精度        | 主要な問題                                                 |
| ------------ | ----------- | ---------------------------------------------------------- |
| core.md      | ⚠️ 要修正   | ~30件の🚫→✅修正（Ptaa/Pixaa実装済み）、~34件のboxfunc偽✅ |
| io.md        | ⚠️ 軽微     | 2件の偽✅、10件の関数名誤り、5件のメソッド/関数誤分類      |
| transform.md | ✅ 概ね正確 | 5件の軽微な命名誤差、1件のステータス境界ケース             |
| morph.md     | ⚠️ 要修正   | サマリーカウント不正（✅92→109、合計135→151）、重複2件     |
| filter.md    | ⚠️ 要修正   | 11件の偽✅（Kernel系10件 + adaptmap 1件）                  |
| color.md     | ⚠️ 要修正   | paintcmap.cセクション欠落（7関数）、Phase 3記述矛盾        |
| region.md    | ⚠️ 軽微     | 11件の関数名誤り（ccbord/pixlabel）、カウント正確          |
| recog.md     | ⚠️ 要修正   | 3件の偽✅、4件の重複エントリ、8件の命名誤り                |
| misc.md      | 🔴 要大修正 | 45件の偽✅、分析セクションの矛盾                           |

---

## モジュール別詳細

### 1. core.md

**カウント**: 文書のカウント（✅751 🔄24 ❌0 🚫107 合計882）はテーブル行数と一致するが、分類に誤りあり。

#### 🚫→✅ に修正すべき関数（~30件）

Ptaa/Pixaa が `Vec<Pta>`/`Vec<Pixa>` 代替として🚫とされているが、実際には専用構造体が完全実装されている:

- **Ptaa系（18件）**: `new`, `push`, `len`, `get`, `init_full`, `replace`, `add_pt`, `truncate`, `read_from_file`, `read_from_reader`, `read_from_bytes`, `write_to_file`, `write_to_writer`, `write_to_bytes`, `join` 等
- **Pixaa系（11件）**: `new`, `push`, `len`, `get`, `get_pix`, `is_full`, `init_full`, `replace`, `clear`, `join` 等
- **endian系（3件）**: `endian_byte_swap_new`, `endian_byte_swap`, `endian_two_byte_swap`

#### ✅だが実際は未実装（~34件）

- **boxfunc2.c**: 「全関数 ✅ 実装済み」と記載されるが、~21関数が未実装（`boxaTransform`, `boxTransform`, `boxaTransformOrdered`, `boxaBinSort`, `boxaSortByIndex`, `boxaSort2d` 等）
- **boxfunc5.c**: 「全関数 ✅ 実装済み」と記載されるが、~13関数が未実装（`boxaSmoothSequenceMedian`, `boxaWindowedMedian`, `boxaModifyWithBoxa` 等）

#### 関数名の誤り

| C関数                      | 文書の記載                  | 実際のRust名             |
| -------------------------- | --------------------------- | ------------------------ |
| `pixCreateWithCmap`        | `Pix::new_with_cmap`        | `Pix::new_with_colormap` |
| `pixAddGray`               | `add_gray()`                | `arith_add()`            |
| `pixSubtractGray`          | `subtract_gray()`           | `arith_subtract()`       |
| `pixRemoveColormapGeneral` | `remove_colormap_general()` | `remove_colormap()`      |
| `pixThreshold8`            | `threshold8`                | `threshold_8`            |

#### 不存在の✅関数

- `pixConvertTo1`: 汎用ディスパッチャが存在しない（`convert_to_1_adaptive` と `convert_to_1_by_sampling` のみ）

---

### 2. io.md

**カウント**: 正確（✅139 🔄18 ❌0 🚫45 合計202）

#### ✅だが存在しない関数（2件）

| C関数                    | 文書のRust名 | 問題               |
| ------------------------ | ------------ | ------------------ |
| `fget_png_colormap_info` | png.rs内     | 関数が見つからない |
| `fget_jpeg_comment`      | jpeg.rs内    | 関数が見つからない |

#### 関数名の誤り（10件）

| C関数                         | 文書の記載 | 実際のRust名                        |
| ----------------------------- | ---------- | ----------------------------------- |
| `extract_g4_data_from_file`   | 同名       | `extract_g4_data`                   |
| `write_mem_tiff_custom`       | 同名       | `write_tiff_custom_mem`             |
| `select_default_pdf_encoding` | 同名       | `select_default_encoding`           |
| `write_segmented_page_to_ps`  | 同名       | `pix_write_segmented_page_to_ps`    |
| `write_mixed_to_ps`           | 同名       | `pix_write_mixed_to_ps`             |
| `write_compressed_to_ps`      | 同名       | `pix_write_compressed_to_ps`        |
| `write_string_ps`             | 同名       | `pix_write_string_ps`               |
| `generate_uncompressed_ps`    | 同名       | `generate_uncompressed_ps_from_pix` |
| `write_stream_jp2k`           | 同名       | `write_jp2k`（統合）                |
| `write_mem_jp2k`              | 同名       | `write_jp2k_mem`                    |

#### メソッドではなくフリー関数（5件）

`Pixa::read_files`, `Pixa::write_files`, `Pixa::write_web_p_anim`, `Pixa::write_stream_web_p_anim`, `Pixa::write_mem_web_p_anim` — いずれもPixaメソッドではなくフリー関数

#### 備考

JP2K書き込み関数（`write_jp2k`, `write_jp2k_mem`）はスタブ実装で常に`Err(UnsupportedFormat)`を返す。✅は技術的に正しいがミスリーディング。

---

### 3. transform.md

**カウント**: 正確（✅104 🔄19 ❌0 🚫14 合計137）

#### 軽微な命名誤差（5件）

| C関数                 | 文書の記載                | 実際のRust名                |
| --------------------- | ------------------------- | --------------------------- |
| `pixScaleAreaMap2`    | `scale_area_map2`         | `scale_area_map_2`          |
| `pixScaleGrayMinMax2` | `scale_gray_min_max2`     | `scale_gray_min_max_2`      |
| (affinecompose)       | `AffineMatrix::invert`    | `AffineMatrix::inverse`     |
| (affinecompose)       | `AffineMatrix::translate` | `AffineMatrix::translation` |
| (affinecompose)       | `AffineMatrix::rotate`    | `AffineMatrix::rotation`    |

#### ステータス境界ケース（1件）

`pixScaleMipmap` が🚫（内部ヘルパー）とされるが、`scale_mipmap` としてプライベート関数が存在する。🚫の分類自体は妥当だが、理由の記述（「pixScaleToGrayMipmap内で処理」）はインライン化されていない実態と不一致。

---

### 4. morph.md

**カウント不正**:

| ステータス | 文書 | 実際 | 差分 |
| ---------- | ---- | ---- | ---- |
| ✅ 同等    | 92   | 109  | +17  |
| 🚫 不要    | 27   | 26   | −1   |
| 合計       | 135  | 151  | +16  |

原因: colormorph.c のRust専用エントリ7件、重複2件、サマリー更新漏れ。

#### 重複エントリ（2件）

- `pixMorphCompSequence`: morphapp.c と morphseq.c に重複
- `pixTophat`: morphapp.c と graymorph.c に重複（異なるRust関数にマッピング）

#### 命名誤り（2件）

- `Pixa::thin_connected` → 実際は `pixa_thin_connected`（フリー関数）
- `Sela::create_from_color_pixa` → 実際は `sela_create_from_color_pixa`（フリー関数）

#### C関数名のタイポ（1件）

- `selMakeThinSets` → 正しくは `selaMakeThinSets`

---

### 5. filter.md

**カウント**: テーブル行数は合計118と一致するが、分類に誤りあり。

#### ✅だが未実装（11件）

**Kernel系（10件）**:
`min_max`, `invert`, `read`, `read`(stream統合), `write`, `write`(stream統合), `from_string`, `from_file`, `from_pix`, `display_in_pix`

**adaptmap系（1件）**:
`get_foreground_gray_map` — コードベース全体に存在しない

#### 命名誤り（2件）

- `kernelSetOrigin` → 文書: `set_origin()` / 実際: `set_center()`
- `kernelGetParameters` → 文書: `cx()/cy()` / 実際: `center_x()/center_y()`

#### 🚫だがプライベート実装あり（3件）

`pixMinMaxTiles`, `pixSetLowContrast`, `pixLinearTRCTiled` — いずれも adaptmap.rs にプライベート関数として存在。🚫分類は妥当だが「Rust対応: -」は不正確。

**修正後カウント**: ✅95 🔄1 ❌11 🚫11 合計118

---

### 6. color.md

**カウント不正**: paintcmap.cセクションの欠落により✅と合計が過少。

#### paintcmap.c セクション欠落（7関数）

以下のすべてが `src/color/paintcmap.rs` に実装済み:
`pix_set_select_cmap`, `pix_color_gray_regions_cmap`, `pix_color_gray_cmap`, `pix_color_gray_masked_cmap`, `add_colorized_gray_to_cmap`, `pix_set_select_masked_cmap`, `pix_set_masked_cmap`

**修正後カウント**: ✅104（+7）、合計133（+7）

#### Phase 3 記述矛盾

- テーブル（90行目）: `pixColorSegment` → 🔄「Phase 3が未実装」 ← **コードと一致**
- 分析（199行目）:「Phase 1,2,3,4全て実装済み」 ← **矛盾**

実際: `color_segment_clean()` はスタンドアロン関数として存在するが、`color_segment()` のメインフロー内では Phase 3 はスキップされている。

#### 🚫だがプライベート実装あり（2件）

`makeRGBIndexTables`, `getRGBFromIndex` — `analysis.rs` にプライベート関数として存在。「C版LUT専用ヘルパー」という理由はRustでも同じアプローチを使用しているため不正確。

---

### 7. region.md

**カウント**: 正確（✅65 🔄8 ❌0 🚫22 合計95）

#### 関数名の誤り（11件）

**ccbord.c（6件）**:

| C関数                | 文書               | 実際                                        |
| -------------------- | ------------------ | ------------------------------------------- |
| `pixGetCCBorders`    | `get_cc_borders`   | `get_component_borders`                     |
| `pixGetHoleBorder`   | `get_hole_border`  | `pix_get_hole_border`                       |
| `ccbaWriteStream`    | `write_stream`     | 存在しない（`write<W: Write>()` に統合）    |
| `ccbaRead`           | `read`             | `read_from`                                 |
| `ccbaReadStream`     | `read_stream`      | 存在しない（`read_from<R: Read>()` に統合） |
| `ccbaWriteSVGString` | `write_svg_string` | `to_svg_string`                             |

**pixlabel.c（5件）**:

| C関数                        | 文書                         | 実際                             |
| ---------------------------- | ---------------------------- | -------------------------------- |
| `pixConnCompTransform`       | `label_connected_components` | `conn_comp_transform`            |
| `pixConnCompIncrInit`        | `conn_comp_incr_init`        | `pix_conn_comp_incr_init`        |
| `pixConnCompIncrAdd`         | `conn_comp_incr_add`         | `pix_conn_comp_incr_add`         |
| `pixGetSortedNeighborValues` | `get_sorted_neighbor_values` | `pix_get_sorted_neighbor_values` |
| `pixLocToColorTransform`     | `loc_to_color_transform`     | `pix_loc_to_color_transform`     |

---

### 8. recog.md

**カウント**: テーブル行数は一致（✅129 🔄26 ❌0 🚫18 合計173）だが重複・偽✅あり。

#### ✅だが未実装（3件）

- `pixFindWordAndCharacterBoxes` (classapp.c)
- `boxaExtractSortedPattern` (classapp.c)
- `numaaCompareImagesByBoxes` (classapp.c)

#### 重複エントリ（4件）

`jbCorrelation`, `jbRankHaus`, `pixGetWordsInTextlines`, `pixGetWordBoxesInTextlines` — 各2セクションに記載

#### メソッド/フリー関数の誤分類（5件）

`Recog::create_from_recog`, `Recog::create_from_pixa_no_finish`, `Pixa::accumulate_samples`, `Pixa::remove_outliers1`, `Pixa::remove_outliers2` — いずれもフリー関数

#### 関数名の誤り（3件）

- `Dewarp::new_ref` → 実際: `Dewarp::create_ref`
- `jbclass::correlation` → 実際: `jbclass::classify::jb_correlation`
- `jbclass::rank_haus` → 実際: `jbclass::classify::jb_rank_haus`

**修正後カウント**: ✅122 🔄26 ❌3 🚫18 ユニーク合計169（重複除外）

---

### 9. misc.md

**カウント大幅不正**: 45件の偽✅あり。

#### ✅だが未実装（45件）

| セクション     | 偽✅数 | 具体例                                                    |
| -------------- | ------ | --------------------------------------------------------- |
| textops.c      | 9      | `add_textlines`, `set_textblock`, `set_textline`, BMF関連 |
| bmf.c          | 5      | `bmf_create`, `bmf_get_pix` 等（BMFモジュール不存在）     |
| gplot.c        | 13     | 全13関数（GPlotモジュール不存在）                         |
| strokes.c      | 7      | 全7関数（公開API不存在）                                  |
| runlength.c    | 3      | 全3関数（runlengthモジュール不存在）                      |
| partition.c    | 2      | `Boxa::get_whiteblocks`, `Boxa::prune_sorted_on_overlap`  |
| partify.c      | 2      | `partify_files`, `partify_pixac`                          |
| binreduce.c    | 1      | `reduce_binary2`                                          |
| checkerboard.c | 1      | `find_checkerboard_corners`                               |
| convertfiles.c | 1      | `convert_files_to1bpp`                                    |
| finditalic.c   | 1      | `italic_words`                                            |

#### 分析セクションの矛盾

656-699行目の「実装状況」が古い情報のまま更新されていない:

- `pdfapp.c`: 「未実装 (0%)」→ 実際は 3/3 実装済み
- `paintcmap.c`: 「未実装 (0%)」→ 実際は 7/7 実装済み
- `pixcomp.c`: 「ほぼ未実装 (2%)」→ 実際は 36/36 実装済み
- `pixlabel.c`: 「未実装 (0%)」→ 実際は 6/6 実装済み
- `encoding.c`: 「部分実装 (14%)」→ 実際は 4/4 実装済み

#### 🚫だが実装済み（1件）

`pixTilingNoStripOnPaint` — `PixTiling::no_strip_on_paint()` として公開メソッドが存在

**修正後カウント**: ✅~99 ❌~45 🚫~180

---

## feature-comparison.md への影響

### サマリーテーブルの修正値

現在の文書では「❌ 未実装: 0」「実カバレッジ: 100.0%」と記載されているが、検証の結果、複数のモジュールで未実装関数が発見された。

| モジュール | 偽✅（実際は未実装） | 🚫→✅修正 | 命名誤り |
| ---------- | -------------------- | --------- | -------- |
| core       | ~34                  | ~30       | 5        |
| io         | 2                    | 0         | 15       |
| transform  | 0                    | 0         | 5        |
| morph      | 0                    | 0         | 3        |
| filter     | 11                   | 0         | 2        |
| color      | 0                    | 0         | 1        |
| region     | 0                    | 0         | 11       |
| recog      | 3                    | 0         | 8        |
| misc       | 45                   | 0         | 0        |
| **合計**   | **~95**              | **~30**   | **~50**  |

### 主要な是正事項

1. **「❌ 未実装: 0」は不正確** — 少なくとも~95関数が✅と記載されているが実際には未実装
2. **「実カバレッジ: 100.0%」は不正確** — 上記を考慮すると100%未満
3. **Ptaa/Pixaa の分類見直し** — 🚫から✅に~30件を移行すべき
4. **morph.md のサマリーカウント** — テーブル行数と一致していない
5. **misc.md の分析セクション** — 古い情報が残存しテーブルと矛盾
6. **color.md の paintcmap.c** — セクション自体が欠落

### 参考: Rust版ソースパス

文書中の `crates/*/src/` という記載（240行目）は誤り。正しくは `src/` 配下のモジュールディレクトリ。
