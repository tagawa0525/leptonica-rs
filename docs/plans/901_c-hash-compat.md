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

エントリ名は Rust manifest と同一キー (`<test_name>.<index>.<ext>`)。インデックスがズレるテストは初期段階では C 側を欠落として扱い、報告のみ行う。

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

### Phase 2: テストランタイムでの C 互換チェック

1. `tests/common/params.rs` に C manifest ロード + `check_hash()` 拡張
2. `tests/c_compat_report.txt` の append ロジック (テスト並列対応)
3. `tests/common/error.rs` に `CCompatMismatch` 等のレポート用構造体を追加 (テストは fail させない)
4. `REGTEST_C_COMPAT` 環境変数の追加

Phase 2 完了条件: `cargo test --test core` 等を実行すると `tests/c_compat_report.txt` に C 互換性差分が出力される

### Phase 3: 1モジュールでの検証

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

1. **PR-1** (Phase 1): スクリプトと `examples/gen_c_manifest.rs` + `tests/golden_manifest_c.tsv` の初回コミット
   - `feat(tests): C版互換性チェック用 manifest 生成ツール`
2. **PR-2** (Phase 2): `params.rs` 拡張と report-only 比較
   - `feat(tests): C-version hash comparison (report-only)`
3. **PR-3** (Phase 3+4): 検証結果ドキュメント + CI 統合
   - `docs(porting): C互換性ベースラインを記録` / `ci: C compat report を artifact 化`

各 PR は RED → GREEN → (必要なら REFACTOR) の TDD サイクルでコミット履歴を残す。

## オープン課題

- インデックスズレが多いテストの扱い: `golden_map.tsv` を参照する形にするか、ズレた範囲だけ別 manifest (`golden_manifest_c_mapped.tsv`) に逃すか。Phase 3 で判断
- C 版の jbig2 / pdf / ps テストでバイナリのみのケース: read_image できないファイルは `data_content_hash()` に流すが、Rust 側出力との整合性をどうとるかは Phase 1 の実装時に検討
- C 版を実行できない開発環境 (Windows 等) での扱い: `golden_manifest_c.tsv` はあくまで git 管理リソースとして配布し、生成スクリプトは Linux 前提とする

## C版対応ファイル

- `reference/leptonica/src/regutils.c` (`regTestCheckFile`, `filesAreIdentical`)
- `reference/leptonica/prog/*_reg.c` (各テストドライバ)
