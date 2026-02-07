# C版テストのRust移植計画

## 1. 概要

### 目的

C版Leptonicaの回帰テスト（160個）をRust版に移植し、同等のテストカバレッジを実現する。

### 現状

| 項目 | C版 | Rust版 |
| ------ | ----- | -------- |
| 回帰テスト | 160個 | 3個（pngio, binmorph1, conncomp） |
| ユニットテスト | - | 919個 |
| テスト画像 | 100+個 | 9個 |
| テストフレームワーク | regutils.c | leptonica-test crate |

### 優先度の定義

| 記号 | 意味 | 条件 |
| ------ | ------ | ------ |
| ✅ | 完了 | Rust回帰テストが存在し、PASSしている |
| 🔵 P1 | 最優先 | 実装済み機能の基本テスト |
| 🟢 P2 | 高優先 | 実装済み機能の拡張テスト |
| 🟡 P3 | 中優先 | 実装済み機能の高度・エッジケーステスト |
| ⚪ P4 | 低優先 | Rust側が未実装のためテスト不可 |
| ⬜ Skip | 対象外 | alltests_reg等、移植不要 |

---

## 2. エージェント運用方針

### メインエージェントの役割

メインエージェントはコンテキストウィンドウ節約のため、**進捗管理に専念**する。

- この計画ファイルの進捗更新（状態・担当列の書き換え）
- サブエージェントへのタスク委譲（Task tool）
- サブエージェントの結果確認と次タスクの決定
- テスト実行結果の記録

**メインエージェントは自分でテストコードを書かない。**

### サブエージェントの種類と役割

| 種類 | subagent_type | 役割 |
| ------ | --------------- | ------ |
| 準備 | general-purpose | テスト画像コピー、テストフレームワーク拡張 |
| 実装 | general-purpose | 個別テストの移植（C読解→Rust実装→動作確認） |
| 検証 | Bash | テスト実行、PASS/FAIL確認 |

### サブエージェントへの委託テンプレート

#### テスト実装の委託（1テストずつ）

```yaml
Task tool:
  subagent_type: "general-purpose"
  prompt: |
    ## テスト移植タスク: {testname}_reg

    C版Leptonicaの回帰テスト `{testname}_reg.c` をRustに移植してください。

    ### 入力
    - C版テスト: reference/leptonica/prog/{testname}_reg.c
    - 対象クレート: crates/{crate_name}/

    ### 参照パターン
    既存のRust回帰テストを参考にしてください:
    - I/Oテスト例: crates/leptonica-io/tests/pngio_reg.rs
    - アルゴリズムテスト例: crates/leptonica-morph/tests/binmorph1_reg.rs
    - 領域テスト例: crates/leptonica-region/tests/conncomp_reg.rs
    - テストフレームワーク: crates/leptonica-test/src/params.rs

    ### 手順
    1. C版テストを読み、テスト内容を把握する
    2. Rustで対応する公開APIを確認する（crates/{crate_name}/src/）
    3. crates/{crate_name}/tests/{testname}_reg.rs を作成する
    4. 必要なテスト画像がtests/data/images/になければ、
       reference/leptonica/prog/ からコピーする
    5. テストを実行して動作確認:
       cargo test -p {crate_name} --test {testname}_reg
    6. C版で未実装のRust APIがあれば、テスト内でスキップする

    ### 完了基準
    - テストが `cargo test` でPASSすること
    - C版テストの主要なテストケースが網羅されていること
    - テストが既存のRegParamsパターンに従っていること

    ### 注意
    - C版のすべての機能がRustにあるとは限らない。
      Rustに対応APIがない部分はコメントで記録し、スキップしてよい。
    - テストフレームワーク(leptonica-test)の変更が必要な場合は、
      変更内容と理由を報告すること。
```

#### 同一カテゴリの一括委託（並行作業用）

```yaml
Task tool:
  subagent_type: "general-purpose"
  prompt: |
    ## テスト一括移植タスク: {crate_name} カテゴリ

    以下のテストをすべて移植してください:
    {test_list}

    [以下は個別テンプレートと同じ]
```

#### テスト画像準備の委託

```yaml
Task tool:
  subagent_type: "general-purpose"
  prompt: |
    ## テスト画像準備

    以下のテスト画像をC版から Rust テストディレクトリにコピーしてください。

    コピー元: reference/leptonica/prog/
    コピー先: tests/data/images/

    ### 必要な画像
    {image_list}

    ### 手順
    1. コピー元に画像が存在するか確認
    2. コピー先にまだ存在しないものだけコピー
    3. コピーした画像の一覧を報告
```

---

## 3. 並行実装戦略

### 並行可能グループ

各クレートのテストは独立しているため、最大8並行で作業可能。

| グループ | クレート | P1 | P2 | P3 | 計 |
| ---------- | ---------- | ---- | ---- | ---- | ---- |
| A | leptonica-core | 3 | 7 | 3 | 13 |
| B | leptonica-io | 2 | 5 | 6 | 13 |
| C | leptonica-morph | 3 | 5 | 4 | 12 |
| D | leptonica-transform | 4 | 3 | 5 | 12 |
| E | leptonica-filter | 2 | 4 | 3 | 9 |
| F | leptonica-color | 2 | 6 | 4 | 12 |
| G | leptonica-region | 2 | 3 | 3 | 8 |
| H | leptonica-recog | 1 | 2 | 2 | 5 |

### Worktreeの活用

複数セッションで並行作業する場合はworktreeを使用。

```bash
git worktree add ../leptonica-rs-test-io    test/io
git worktree add ../leptonica-rs-test-morph  test/morph
git worktree add ../leptonica-rs-test-transform test/transform
# ...各グループ分
```

単一セッション内では、サブエージェントを並列起動（Task toolの並列呼び出し）で対応。

---

## 4. 実装手順

### Step 0: 準備

- [ ] テスト画像を追加コピー（サブエージェントに委託）
- [ ] leptonica-testクレートにヘルパー追加が必要であれば追加（サブエージェントに委託）

### Step 1: P1テスト（19個）

各カテゴリの最重要テストを実装。サブエージェントにカテゴリ単位で委託。

### Step 2: P2テスト（35個）

実装済み機能の拡張テスト。Step 1と同様にサブエージェントに委託。

### Step 3: P3テスト（30個）

高度機能・エッジケースのテスト。

### Step 4: P4テスト（67個）

Rust側の機能実装と並行してテストも追加。

---

## 5. 全テストケース一覧と進捗（160個）

### Core / Data Structures（21個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 1 | boxa1 | 🔵 P1 | TODO | leptonica-core | | Box基本操作 |
| 2 | boxa2 | 🔵 P1 | TODO | leptonica-core | | Box算術演算 |
| 3 | boxa3 | 🟢 P2 | TODO | leptonica-core | | Box配列操作 |
| 4 | boxa4 | 🟢 P2 | TODO | leptonica-core | | Box変換 |
| 5 | pta | 🔵 P1 | TODO | leptonica-core | | 点配列操作 |
| 6 | ptra1 | 🟢 P2 | TODO | leptonica-core | | ポインタ配列1 |
| 7 | ptra2 | 🟡 P3 | TODO | leptonica-core | | ポインタ配列2 |
| 8 | numa1 | 🟢 P2 | TODO | leptonica-core | | 数値配列基本 |
| 9 | numa2 | 🟢 P2 | TODO | leptonica-core | | 数値配列演算 |
| 10 | numa3 | 🟡 P3 | TODO | leptonica-core | | 数値配列高度 |
| 11 | fpix1 | 🟢 P2 | TODO | leptonica-core | | 浮動小数点画像1 |
| 12 | fpix2 | 🟡 P3 | TODO | leptonica-core | | 浮動小数点画像2 |
| 13 | pixa1 | 🟢 P2 | TODO | leptonica-core | | 画像配列基本 |
| 14 | pixa2 | 🟢 P2 | TODO | leptonica-core | | 画像配列操作 |
| 15 | pixadisp | 🟡 P3 | TODO | leptonica-core | | 画像配列表示 |
| 16 | pixalloc | ⚪ P4 | TODO | 未実装 | | メモリアロケータ |
| 17 | pixcomp | ⚪ P4 | TODO | 未実装 | | 圧縮画像 |
| 18 | pixmem | ⚪ P4 | TODO | 未実装 | | メモリ操作 |
| 19 | pixserial | ⚪ P4 | TODO | 未実装 | | シリアライズ |
| 20 | pixtile | ⚪ P4 | TODO | 未実装 | | タイル処理 |
| 21 | bytea | ⚪ P4 | TODO | 未実装 | | バイト配列 |

### I/O（18個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 22 | pngio | ✅ | 完了 | leptonica-io | - | PNG読み書き |
| 23 | jpegio | 🔵 P1 | TODO | leptonica-io | | JPEG読み書き |
| 24 | gifio | 🟢 P2 | TODO | leptonica-io | | GIF読み書き |
| 25 | webpio | 🟢 P2 | TODO | leptonica-io | | WebP読み書き |
| 26 | webpanimio | 🟡 P3 | TODO | leptonica-io | | WebPアニメーション |
| 27 | pnmio | 🟢 P2 | TODO | leptonica-io | | PNM読み書き |
| 28 | jp2kio | 🟡 P3 | TODO | leptonica-io | | JPEG2000読み書き |
| 29 | pdfio1 | 🟡 P3 | TODO | leptonica-io | | PDF出力基本 |
| 30 | pdfio2 | 🟡 P3 | TODO | leptonica-io | | PDF出力高度 |
| 31 | psio | 🟡 P3 | TODO | leptonica-io | | PostScript出力 |
| 32 | psioseg | 🟡 P3 | TODO | leptonica-io | | PSセグメント |
| 33 | ioformats | 🔵 P1 | TODO | leptonica-io | | フォーマット検出 |
| 34 | iomisc | 🟢 P2 | TODO | leptonica-io | | I/Oその他 |
| 35 | mtiff | 🟢 P2 | TODO | leptonica-io | | マルチページTIFF |
| 36 | selio | 🟢 P2 | TODO | leptonica-io | | SEL I/O |
| 37 | files | ⚪ P4 | TODO | 未実装 | | ファイル操作 |
| 38 | pdfseg | ⚪ P4 | TODO | 未実装 | | PDFセグメント |
| 39 | encoding | ⚪ P4 | TODO | 未実装 | | エンコーディング |

### Morphology（16個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 40 | binmorph1 | ✅ | 完了 | leptonica-morph | - | 二値形態学基本 |
| 41 | binmorph2 | 🔵 P1 | TODO | leptonica-morph | | safe closing |
| 42 | binmorph3 | 🔵 P1 | TODO | leptonica-morph | | 境界条件 |
| 43 | binmorph4 | 🟢 P2 | TODO | leptonica-morph | | 複合操作 |
| 44 | binmorph5 | 🟢 P2 | TODO | leptonica-morph | | 性能テスト |
| 45 | binmorph6 | 🟡 P3 | TODO | leptonica-morph | | 高度操作 |
| 46 | dwamorph1 | 🔵 P1 | TODO | leptonica-morph | | DWA基本 |
| 47 | dwamorph2 | 🟢 P2 | TODO | leptonica-morph | | DWA拡張 |
| 48 | graymorph1 | 🟢 P2 | TODO | leptonica-morph | | グレースケール形態学1 |
| 49 | graymorph2 | 🟡 P3 | TODO | leptonica-morph | | グレースケール形態学2 |
| 50 | colormorph | 🟢 P2 | TODO | leptonica-morph | | カラー形態学 |
| 51 | morphseq | 🟢 P2 | TODO | leptonica-morph | | 形態学シーケンス |
| 52 | ccthin1 | 🟡 P3 | TODO | leptonica-morph | | 細線化1 |
| 53 | ccthin2 | 🟡 P3 | TODO | leptonica-morph | | 細線化2 |
| 54 | fhmtauto | ⚪ P4 | TODO | 未実装 | | 自動HMT |
| 55 | fmorphauto | ⚪ P4 | TODO | 未実装 | | 自動形態学 |

### Transform（15個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 56 | rotate1 | 🔵 P1 | TODO | leptonica-transform | | 回転基本 |
| 57 | rotate2 | 🔵 P1 | TODO | leptonica-transform | | 回転高度 |
| 58 | rotateorth | 🔵 P1 | TODO | leptonica-transform | | 直交回転 |
| 59 | scale | 🔵 P1 | TODO | leptonica-transform | | スケーリング |
| 60 | affine | 🟢 P2 | TODO | leptonica-transform | | アフィン変換 |
| 61 | bilinear | 🟢 P2 | TODO | leptonica-transform | | 双線形変換 |
| 62 | projective | 🟢 P2 | TODO | leptonica-transform | | 射影変換 |
| 63 | shear1 | 🟡 P3 | TODO | leptonica-transform | | シアー1 |
| 64 | shear2 | 🟡 P3 | TODO | leptonica-transform | | シアー2 |
| 65 | warper | 🟡 P3 | TODO | leptonica-transform | | ワーピング |
| 66 | translate | 🟡 P3 | TODO | leptonica-transform | | 平行移動 |
| 67 | xformbox | 🟡 P3 | TODO | leptonica-transform | | Box変換 |
| 68 | expand | ⚪ P4 | TODO | 未実装 | | 拡張 |
| 69 | crop | ⚪ P4 | TODO | 未実装 | | クロップ |
| 70 | flipdetect | ⚪ P4 | TODO | 未実装 | | 反転検出 |

### Filter（13個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 71 | convolve | 🔵 P1 | TODO | leptonica-filter | | 畳み込み |
| 72 | edge | 🔵 P1 | TODO | leptonica-filter | | エッジ検出 |
| 73 | bilateral1 | 🟢 P2 | TODO | leptonica-filter | | バイラテラル1 |
| 74 | bilateral2 | 🟢 P2 | TODO | leptonica-filter | | バイラテラル2 |
| 75 | rank | 🟢 P2 | TODO | leptonica-filter | | ランクフィルタ |
| 76 | rankbin | 🟡 P3 | TODO | leptonica-filter | | 二値ランク |
| 77 | rankhisto | 🟡 P3 | TODO | leptonica-filter | | ランクヒストグラム |
| 78 | adaptmap | 🟢 P2 | TODO | leptonica-filter | | 適応マッピング |
| 79 | adaptnorm | 🟢 P2 | TODO | leptonica-filter | | 適応正規化 |
| 80 | kernel | 🟡 P3 | TODO | leptonica-filter | | カーネル操作 |
| 81 | enhance | ⚪ P4 | TODO | 未実装 | | エンハンス |
| 82 | compfilter | ⚪ P4 | TODO | 未実装 | | 複合フィルタ |
| 83 | locminmax | ⚪ P4 | TODO | 未実装 | | 局所最大最小 |

### Color（22個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 84 | colorspace | 🔵 P1 | TODO | leptonica-color | | 色空間変換 |
| 85 | binarize | 🔵 P1 | TODO | leptonica-color | | 二値化 |
| 86 | colorquant | 🟢 P2 | TODO | leptonica-color | | 色量子化 |
| 87 | cmapquant | 🟢 P2 | TODO | leptonica-color | | カラーマップ量子化 |
| 88 | colorseg | 🟢 P2 | TODO | leptonica-color | | 色セグメント |
| 89 | colorcontent | 🟢 P2 | TODO | leptonica-color | | 色コンテンツ分析 |
| 90 | colorfill | 🟢 P2 | TODO | leptonica-color | | 色塗りつぶし |
| 91 | coloring | 🟡 P3 | TODO | leptonica-color | | 着色 |
| 92 | colorize | 🟡 P3 | TODO | leptonica-color | | カラライズ |
| 93 | dither | 🟡 P3 | TODO | leptonica-color | | ディザリング |
| 94 | grayquant | 🟡 P3 | TODO | leptonica-color | | グレー量子化 |
| 95 | threshnorm | 🟢 P2 | TODO | leptonica-color | | しきい値正規化 |
| 96 | blackwhite | ⚪ P4 | TODO | 未実装 | | 白黒処理 |
| 97 | falsecolor | ⚪ P4 | TODO | 未実装 | | 疑似カラー |
| 98 | hardlight | ⚪ P4 | TODO | 未実装 | | ハードライト |
| 99 | lowsat | ⚪ P4 | TODO | 未実装 | | 低彩度 |
| 100 | colormask | ⚪ P4 | TODO | 未実装 | | カラーマスク |
| 101 | alphaops | ⚪ P4 | TODO | 未実装 | | アルファ操作 |
| 102 | alphaxform | ⚪ P4 | TODO | 未実装 | | アルファ変換 |
| 103 | blend1 | ⚪ P4 | TODO | 未実装 | | ブレンド1 |
| 104 | blend2 | ⚪ P4 | TODO | 未実装 | | ブレンド2 |
| 105 | blend3 | ⚪ P4 | TODO | 未実装 | | ブレンド3 |
| 106 | blend4 | ⚪ P4 | TODO | 未実装 | | ブレンド4 |
| 107 | blend5 | ⚪ P4 | TODO | 未実装 | | ブレンド5 |

### Region / Geometry（15個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 108 | conncomp | ✅ | 完了 | leptonica-region | - | 連結成分 |
| 109 | label | 🔵 P1 | TODO | leptonica-region | | ラベリング |
| 110 | ccbord | 🔵 P1 | TODO | leptonica-region | | 境界追跡 |
| 111 | seedspread | 🟢 P2 | TODO | leptonica-region | | シード拡散 |
| 112 | watershed | 🟢 P2 | TODO | leptonica-region | | 分水嶺 |
| 113 | quadtree | 🟢 P2 | TODO | leptonica-region | | 四分木 |
| 114 | maze | 🟡 P3 | TODO | leptonica-region | | 迷路 |
| 115 | grayfill | 🟡 P3 | TODO | leptonica-region | | グレー塗りつぶし |
| 116 | distance | 🟡 P3 | TODO | leptonica-region | | 距離変換 |
| 117 | partition | ⚪ P4 | TODO | 未実装 | | パーティション |
| 118 | overlap | ⚪ P4 | TODO | 未実装 | | オーバーラップ |
| 119 | rectangle | ⚪ P4 | TODO | 未実装 | | 矩形検出 |
| 120 | circle | ⚪ P4 | TODO | 未実装 | | 円検出 |
| 121 | checkerboard | ⚪ P4 | TODO | 未実装 | | チェッカーボード |
| 122 | projection | ⚪ P4 | TODO | 未実装 | | 投影 |

### Recognition / Document（22個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 123 | skew | 🔵 P1 | TODO | leptonica-recog | | スキュー検出 |
| 124 | baseline | 🟢 P2 | TODO | leptonica-recog | | ベースライン検出 |
| 125 | pageseg | 🟢 P2 | TODO | leptonica-recog | | ページセグメント |
| 126 | dewarp | 🟡 P3 | TODO | leptonica-recog | | デワーピング |
| 127 | jbclass | 🟡 P3 | TODO | leptonica-recog | | JBIG2分類 |
| 128 | genfonts | ⚪ P4 | TODO | 未実装 | | フォント生成 |
| 129 | italic | ⚪ P4 | TODO | 未実装 | | イタリック検出 |
| 130 | lineremoval | ⚪ P4 | TODO | 未実装 | | 線除去 |
| 131 | wordboxes | ⚪ P4 | TODO | 未実装 | | 単語Box |
| 132 | writetext | ⚪ P4 | TODO | 未実装 | | テキスト書き込み |
| 133 | newspaper | ⚪ P4 | TODO | 未実装 | | 新聞処理 |
| 134 | findcorners | ⚪ P4 | TODO | 未実装 | | コーナー検出 |
| 135 | findpattern1 | ⚪ P4 | TODO | 未実装 | | パターン検出1 |
| 136 | findpattern2 | ⚪ P4 | TODO | 未実装 | | パターン検出2 |
| 137 | nearline | ⚪ P4 | TODO | 未実装 | | 近接線 |
| 138 | texturefill | ⚪ P4 | TODO | 未実装 | | テクスチャ塗り |
| 139 | speckle | ⚪ P4 | TODO | 未実装 | | スペックル除去 |
| 140 | smoothedge | ⚪ P4 | TODO | 未実装 | | エッジ平滑化 |
| 141 | subpixel | ⚪ P4 | TODO | 未実装 | | サブピクセル |
| 142 | extrema | ⚪ P4 | TODO | 未実装 | | 極値 |
| 143 | smallpix | ⚪ P4 | TODO | 未実装 | | 小画像 |
| 144 | splitcomp | ⚪ P4 | TODO | 未実装 | | 成分分割 |

### Utility / Low-level（16個）

| # | テスト名 | 優先度 | 状態 | Rust対応 | 担当 | 備考 |
| --- | ---------- | -------- | ------ | ---------- | ------ | ------ |
| 145 | rasterop | ⚪ P4 | TODO | 未実装 | | ラスター演算 |
| 146 | rasteropip | ⚪ P4 | TODO | 未実装 | | ラスター演算IP |
| 147 | logicops | ⚪ P4 | TODO | 未実装 | | 論理演算 |
| 148 | lowaccess | ⚪ P4 | TODO | 未実装 | | 低レベルアクセス |
| 149 | compare | ⚪ P4 | TODO | 未実装 | | 比較 |
| 150 | conversion | ⚪ P4 | TODO | 未実装 | | 変換 |
| 151 | equal | ⚪ P4 | TODO | 未実装 | | 等価判定 |
| 152 | hash | ⚪ P4 | TODO | 未実装 | | ハッシュ |
| 153 | heap | ⚪ P4 | TODO | 未実装 | | ヒープ |
| 154 | insert | ⚪ P4 | TODO | 未実装 | | 挿入 |
| 155 | multitype | ⚪ P4 | TODO | 未実装 | | 複合型 |
| 156 | paint | ⚪ P4 | TODO | 未実装 | | ペイント |
| 157 | paintmask | ⚪ P4 | TODO | 未実装 | | ペイントマスク |
| 158 | string | ⚪ P4 | TODO | 未実装 | | 文字列 |
| 159 | dna | ⚪ P4 | TODO | 未実装 | | DNA配列 |
| 160 | alltests | ⬜ Skip | - | - | | テストランナー |

---

## 6. 進捗サマリー

| カテゴリ | P1 | P2 | P3 | P4 | Skip | 完了 | 総数 |
| ---------- | ---- | ---- | ---- | ---- | ------ | ------ | ------ |
| Core | 3 | 7 | 3 | 6 | 0 | 0 | 21 |
| I/O | 2 | 5 | 6 | 3 | 0 | 1 | 18 |
| Morph | 3 | 5 | 4 | 2 | 0 | 1 | 16 |
| Transform | 4 | 3 | 5 | 3 | 0 | 0 | 15 |
| Filter | 2 | 4 | 3 | 3 | 0 | 0 | 13 |
| Color | 2 | 6 | 4 | 12 | 0 | 0 | 22 |
| Region | 2 | 3 | 3 | 6 | 0 | 1 | 15 |
| Recognition | 1 | 2 | 2 | 17 | 0 | 0 | 22 |
| Utility | 0 | 0 | 0 | 15 | 1 | 0 | 16 |
| **合計** | **19** | **35** | **30** | **67** | **1** | **3** | **160** |

### フェーズ別目標

| フェーズ | 対象 | テスト数 | 累計 | カバレッジ |
| ---------- | ------ | ---------- | ------ | ----------- |
| 現状 | - | 3 | 3 | 2% |
| Phase 1 | P1 | 19 | 22 | 14% |
| Phase 2 | P2 | 35 | 57 | 36% |
| Phase 3 | P3 | 30 | 87 | 54% |
| Phase 4 | P4 | 67 | 154 | 96% |

---

## 7. 参考情報

### 既存テストパターン

```rust
//! <テスト名> regression test
//!
//! C版: reference/leptonica/prog/<testname>_reg.c

use leptonica_test::{RegParams, load_test_image};

#[test]
fn <testname>_reg() {
    let mut rp = RegParams::new("<testname>");
    let pixs = load_test_image("<image>").expect("Failed to load");

    // テスト処理
    rp.compare_values(expected, actual, delta);
    rp.write_pix_and_check(&result, ImageFormat::Png).unwrap();

    assert!(rp.cleanup());
}
```

### 検証コマンド

```bash
# 全テスト
cargo test --workspace

# 回帰テストのみ
cargo test --workspace --test '*_reg'

# 特定テスト
cargo test -p leptonica-io --test jpegio_reg --features all-formats

# ゴールデンファイル生成
REGTEST_MODE=generate cargo test -p leptonica-io --test jpegio_reg

# 進捗確認（回帰テストファイル数）
find crates/*/tests -name '*_reg.rs' | wc -l
```

### 重要ファイル

| ファイル | 用途 |
| ---------- | ------ |
| `crates/leptonica-test/src/params.rs` | テストフレームワーク（RegParams） |
| `crates/leptonica-test/src/lib.rs` | テストヘルパー（load_test_image等） |
| `crates/leptonica-io/tests/pngio_reg.rs` | I/Oテスト参考パターン |
| `crates/leptonica-morph/tests/binmorph1_reg.rs` | アルゴリズムテスト参考パターン |
| `crates/leptonica-region/tests/conncomp_reg.rs` | 領域テスト参考パターン |
| `reference/leptonica/prog/*_reg.c` | C版テスト（移植元） |
| `tests/data/images/` | テスト画像格納先 |
