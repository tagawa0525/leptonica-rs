# Plan 901: C版ハッシュとの互換性検証

Status: PLANNED
Phase: 9 (回帰テスト基盤)
関連: plan 900 (regression-test-full-porting)、`examples/compare_golden.rs`、`scripts/golden_map.tsv`

## 背景と問題

現状の `tests/golden_manifest.tsv` は **Rust 実装が出力した `Pix` の FNV-1a ハッシュ** を保存している。

- `tests/common/params.rs:45-69` の `pixel_content_hash()` は Rust の `Pix` を走査するだけ
- `REGTEST_MODE=generate` 時、`write_pix_and_check()` (`tests/common/params.rs:311`) が Rust 出力からハッシュを計算して manifest を更新
- C 版オリジナル (`reference/leptonica/src/regutils.c`) は `filesAreIdentical()` でファイル全一致を確認するが、leptonica-rs はその情報を取り込んでいない
- `examples/compare_golden.rs` は C↔Rust のピクセル比較ツールとして存在するが、CI / 通常テストの導線には組み込まれていない

つまり「Rust 実装が過去の自分と一致する」ことは保証されているが、「C 版の出力と一致する」ことは保証されていない。**回帰テストが C 互換を担保しているとは言えない。**

## ゴール

1. C 版 leptonica の出力からハッシュを再計算し、`tests/golden_manifest_c.tsv` として git 管理する
2. `REGTEST_MODE=compare` 実行時、Rust 出力ハッシュを既存の Rust manifest と C manifest の両方と照合する
3. C 互換性の不一致は **警告 + 別レポート** として扱い、既存テストは破壊しない
4. C manifest の再生成手順を `scripts/` 配下にスクリプト化し、再現可能にする

## 非ゴール

- C 版とのバイナリ（ファイル全一致）レベルの比較。PNG/JPEG エンコーダ差を許容するため、ピクセル単位 FNV-1a ハッシュで十分
- 計画当初から CI を fail にすること。最初は report-only。安定後に必要なら厳格化を検討
- `golden_map.tsv` のような C↔Rust 番号差マッピングの再構築。今回は **同一テストプレフィクス・同一インデックス** を前提とし、ズレるケースは「mapped」エントリとして別管理（後段で検討）

## 設計

### データフロー

```rust
[reference/leptonica/prog/*_reg generate]  ──► /tmp/lept/regout/*.{png,jpg,tif,...}
                                                  │
                                                  ▼
                                  [tools/gen_c_manifest (Rust example)]
                                  read_image() → pixel_content_hash()
                                                  │
                                                  ▼
                                       tests/golden_manifest_c.tsv  (git 管理)

[cargo test] ──► Rust が PIX 出力 ──► pixel_content_hash()
                                          │
                ┌─────────────────────────┴──────────────────────────┐
                ▼                                                    ▼
  golden_manifest.tsv と比較                     golden_manifest_c.tsv と比較
  (mismatch ⇒ FAIL: 既存挙動)            (mismatch ⇒ WARN + c_compat_report.txt)
```

### ハッシュ関数

既存の `pixel_content_hash()` をそのまま使う。FNV-1a で `(width, height, depth, pixel...)` を走査するため、C 版・Rust 版どちらの `Pix` でも同じ値が得られる。**圧縮形式差を吸収する**ためにバイナリ全一致比較ではなくピクセル一致比較を採用。

### ファイル構成

```text
tests/
├── golden_manifest.tsv           # 既存: Rust 出力ハッシュ
├── golden_manifest_c.tsv         # 新規: C 出力ハッシュ (git 管理)
└── c_compat_report.txt           # 新規: テスト後に生成される差分レポート (.gitignore)

scripts/
├── build_c_leptonica.sh          # 新規: reference/leptonica を nix develop でビルド
└── gen_c_manifest.sh             # 新規: prog/*_reg generate を一括実行 + Rust ツール呼び出し

tools/                            # 新規ディレクトリ
└── gen_c_manifest/
    └── main.rs                   # 新規: C 出力ディレクトリを走査し、manifest_c.tsv を生成
```

`tools/gen_c_manifest` は `Cargo.toml` の `[[example]]` として登録 (新規ディレクトリを切らずに `examples/gen_c_manifest.rs` でもよい。実装時に決める)。

### manifest_c.tsv フォーマット

`tests/golden_manifest.tsv` と同一フォーマット:

```text
# C-version golden manifest - pixel-content hashes from C leptonica output
# Format: name<TAB>hash (FNV-1a hex)
adaptmap_bg_color.04.jpg <C版 PIX を pixel_content_hash() に通した値>
...
```

エントリ名は **C 出力のファイル名そのまま** (`edge.00.png` のように 0-indexed)。Rust manifest は 1-indexed (`edge.04.jpg`) で番号がズレるため、Phase 2 のランタイム比較では `scripts/golden_map.tsv` を介して C↔Rust のキー対応を解決する。マッピングが未登録のテストは「未カバー」として report-only に出す。

### params.rs の拡張

`tests/common/params.rs` に C manifest をロードする static を追加:

```rust
fn c_manifest() -> &'static HashMap<String, u64> { /* OnceLock */ }
```

`check_hash()` の `RegTestMode::Compare` 分岐で:

1. 既存通り `golden_manifest.tsv` と比較。不一致なら `failures` に追加 (FAIL)
2. 追加で `golden_manifest_c.tsv` を引き、不一致なら `tests/c_compat_report.txt` に append (WARN, success には影響させない)
3. C manifest にエントリが存在しないキーは「未カバー」として別カウント

レポートはテスト並列実行に耐えるよう `Mutex` で保護して追記。テストバイナリ起動時に一度トランケートする (`OnceLock` で初期化)。

### 環境変数

- `REGTEST_C_COMPAT=0` で C 比較を無効化 (デフォルトは有効、ただし report-only)
- `REGTEST_C_COMPAT=strict` で C 不一致も fail に昇格 (将来の C 互換性確保フェーズで使う)

## 実装フェーズ

### Phase 1: スクリプト整備とハッシュ生成ツール

1. `scripts/build_c_leptonica.sh`: `reference/leptonica` を `nix develop` 配下で `BUILD_PROG=ON` でビルド
2. `scripts/gen_c_manifest.sh`:
   - `prog/` で `*_reg generate` を一括実行 (`/tmp/lept/regout/` に C golden が落ちる)
   - `examples/gen_c_manifest.rs` を `cargo run --release --example gen_c_manifest --features all-formats -- --c-dir /tmp/lept/regout --out tests/golden_manifest_c.tsv` で呼ぶ
3. `examples/gen_c_manifest.rs`:
   - 入力ディレクトリの画像/データを走査
   - 画像は `read_image()` → `pixel_content_hash()`
   - その他バイナリ (`.ba`, `.na`, `.pdf`) は raw bytes → `data_content_hash()`
   - ソート済みの TSV を出力

Phase 1 完了条件: `tests/golden_manifest_c.tsv` が生成でき、`tests/golden_manifest.tsv` と diff を取って大まかな一致状況が分かる

### Phase 1.5: assertion-only テストの C 中間結果取り込み

PR #380 (`c-compat-coverage.md`) で判明した抜け穴。C 版 `prog/*_reg.c` が
`regTestWritePixAndCheck` を呼ばない assertion-only テスト (binmorph1/3、
fhmtauto、graymorph2) について、`scripts/verify_*.c` が C 中間結果を
`/tmp/c_*.{tif,jpg}` に出力する設計があるのに Phase 1 では取り込んでいなかった。

1. `scripts/c_verify_outputs.tsv` (新規): `/tmp/c_*.{tif,jpg}` 27 ファイル →
   `golden_map.tsv` の合成キー (`binmorph1_verify.NN.tif` 等) の対応表
2. `scripts/build_c_verify.sh` (新規): `verify_binmorph` / `verify_fhmtauto` /
   `verify_graymorph2` を libleptonica.a + cmake link line から拾った
   image libs で直接コンパイル (`nix develop` 不要)
3. `scripts/gen_c_manifest.sh` を拡張: フルラン時に verify プログラムを実行
   し、`/tmp/c_*` を `c_verify_outputs.tsv` に従ってリネーム copy → 既存の
   `examples/gen_c_manifest` で hash 化される

Phase 1.5 完了条件: `cargo test --all-features` 実行後に
`tests/c_compat_report.*.txt` の `MissingC` が 0 件 (1871 → 1898 entries)

### Phase 2: テストランタイムでの C 互換チェック (レポート機構)

1. `tests/common/c_compat.rs` を新規追加し、以下を実装:
   - `c_manifest()` static: `tests/golden_manifest_c.tsv` をロード
   - `golden_map()` static: `scripts/golden_map.tsv` をパースして `(rust_prefix, rust_index) → (c_prefix, c_index)` の対応表
   - `parse_manifest_key()`: `"edge.04.jpg"` を `(prefix, index, ext)` に分解
   - `check_c_hash()`: Rust 側キーから C 側ハッシュを引き、状態 (`Ok` / `Mismatch` / `Unmapped` / `MissingC`) を返す
   - `tests/c_compat_report.txt` への append (Mutex、起動時 truncate、`#[cfg(test)]` のもとで)
2. `tests/common/params.rs::RegParams::check_hash` を拡張し、`Compare` モード時に `check_c_hash()` を呼んで結果をレポート
3. `REGTEST_C_COMPAT` 環境変数: `off` で無効化、`report` (デフォルト) で報告のみ、`strict` で `Mismatch` を fail に昇格
4. `tests/c_compat_report.txt` を `.gitignore` に追加

Phase 2 完了条件: `cargo test --all-features` を実行すると `tests/c_compat_report.txt` に C 互換性差分が出力され、既存テストの成否には影響しない

### Phase 2.5: 個別不一致の調査と Rust 側修正 (複数 PR)

Phase 2 のレポートを元に、不一致が出ている Rust テストを 1 件ずつ調査する:

1. **修正対象**: Rust 実装のアルゴリズム差・ピクセル境界処理ミス・型変換誤りなど移植品質の問題。差分が小さくても挙動として誤っていれば修正
2. **修正対象外と判定する根拠**:
   - JPEG/WebP 等の lossy エンコーダ差 (例: libjpeg vs jpeg-encoder)。`compare_pix` で **Rust 同士の整合性** が取れていて、かつ C↔Rust のピクセル差が圧縮歪み範囲ならスキップ
   - C 側にバグがあり、修正版が Rust にのみ反映済みのケース (これは leptonica 上流の issue を確認)
   - golden_map のマッピング誤りに起因する見かけの不一致

3. 各不一致について `docs/porting/c-compat-findings/<NNN-<title>>.md` を作って所見を記録 (任意)
4. Rust 側修正が入った PR ごとに `tests/golden_manifest.tsv` を再生成

#### Findings ログ

- [c-compat-coverage.md](../porting/c-compat-coverage.md): Phase 2.5 移行前の完全性確認。⚠️ Phase 1 は **未完成** と判明。`prog/*_reg` 由来は 155 件取得済み (SKIP_REGS 4 件は出力なしで対象外、これは正しい) だが、`scripts/verify_*.c` 由来の C 中間結果 27 件が manifest_c.tsv に未取り込み (Phase 2 レポートで MissingC 27 件として現れる)。Rust 側回帰テスト数は 159/159 だが、機能カバー率は file 数とは別物 (dwamorph2/fmorphauto/morphseq でカバー漏れあり)。Phase 1.5 完全化 PR の実施が次のアクション
- [001-jpeg-codec-diffs.md](../porting/c-compat-findings/001-jpeg-codec-diffs.md): Phase 2 のレポートで観測された 9 件の Mismatch (edge / convolve_blockconv_gray / colormorph) はすべて **JPEG codec 差** (仮説段階)。Rust 側修正対象外
- [002-tiff-1bpp-write-limit.md](../porting/c-compat-findings/002-tiff-1bpp-write-limit.md): Phase 1.5 で顕在化した 15 件の 1bpp Mismatch (binmorph1/3、fhmtauto) は `src/io/tiff.rs` が 1bpp Pix を 8bpp に拡張して書き出すことが原因。`tiff` crate 0.11.3 に 1bpp encoder サポートがない。修正は PR #383 で完了 (DirectoryEncoder の low-level API で 1bpp 直接書き出し)
- [003-morph-brick-comp-vs-plain.md](../porting/c-compat-findings/003-morph-brick-comp-vs-plain.md): PR #383 後に残った 7 件の Mismatch (binmorph1/3) は Rust `dilate_brick` 等が composite decomposition を使う一方、`verify_binmorph.c` が plain `pixDilateBrick` を呼ぶことが第一の差。PR #385 で Option A (verify を Comp に) を実装するも 7 件 Mismatch は残った → Rust `dilate_1d_composite` は C `pixDilateCompBrick` とも違う「第三の挙動」。次は Phase 2.5 第三弾-続 (005 finding 予定)
- [004-hmt-impl-diff.md](../porting/c-compat-findings/004-hmt-impl-diff.md): fhmtauto 系 8 件の Mismatch。7 件 (sel_4_*, sel_8_*) は C `pixHMT` の「Clear near edges」処理が Rust `hit_miss_transform` に無いことが疑わしい。8 件目 (Identity 1x1 brick, 100% diff) は theoretical には identity になるはずだが実機で all-1 出力に近い → Sel/HMT パスのどこかに bug がある。修正は別 PR (Step 1: Identity debug, Step 2: Clear near edges 実装)

### Phase 3: 1モジュールでの検証ベースライン記録

1. `tests/io/` か `tests/morph/` を先行検証
2. 一致率・主要な不一致パターンを `docs/porting/c-compat-status.md` (新規) にまとめる
3. インデックスズレが多いテストの一覧を作成 (golden_map.tsv の補完情報)

Phase 3 完了条件: 主要モジュールで C 互換性のベースラインが得られている

### Phase 4: 全モジュール展開とドキュメント整備

1. 全モジュールで C compat report を取得し、ベースラインを記録
2. `CLAUDE.md` に C manifest 再生成手順を追記
3. CI で `REGTEST_C_COMPAT=warn` (デフォルト) を有効化し、レポートを artifact として保存

Phase 4 完了条件: CI で C 互換差分が継続的に観測できる状態

## PR 構成

このプランは複数 PR に分割する:

1. **PR-1** (Phase 1, ✅ merged as #377): スクリプトと `examples/gen_c_manifest.rs` + `tests/golden_manifest_c.tsv`
2. **PR-2** (Phase 2, in progress): `tests/common/c_compat.rs` 新規 + `params.rs` 拡張 + report-only 比較
   - `feat(tests): C-version hash comparison (report-only)`
3. **PR-2.5/N** (Phase 2.5, 複数 PR): Phase 2 のレポートで判明した個別の不一致を Rust 側で修正
   - `fix(<module>): C版と一致するように <op> を修正` のような形で、不一致 1 件ごとに独立 PR
4. **PR-3** (Phase 3+4): 検証結果ドキュメント + CI 統合
   - `docs(porting): C互換性ベースラインを記録` / `ci: C compat report を artifact 化`

各 PR は RED → GREEN → (必要なら REFACTOR) の TDD サイクルでコミット履歴を残す。

## オープン課題

- インデックスズレが多いテストの扱い: `golden_map.tsv` を参照する形にするか、ズレた範囲だけ別 manifest (`golden_manifest_c_mapped.tsv`) に逃すか。Phase 3 で判断
- C 版の jbig2 / pdf / ps テストでバイナリのみのケース: read_image できないファイルは `data_content_hash()` に流すが、Rust 側出力との整合性をどうとるかは Phase 1 の実装時に検討
- C 版を実行できない開発環境 (Windows 等) での扱い: `golden_manifest_c.tsv` はあくまで git 管理リソースとして配布し、生成スクリプトは Linux 前提とする

## C版対応ファイル

- `reference/leptonica/src/regutils.c` (`regTestCheckFile`, `filesAreIdentical`)
- `reference/leptonica/prog/*_reg.c` (各テストドライバ)
