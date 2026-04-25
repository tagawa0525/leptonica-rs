# `fill_map_holes` を C版 `pixFillMapHoles` と bit 同等に揃える

Status: PLANNED

## Context

### 現状

`src/filter/adaptmap.rs::fill_map_holes_inner` は C版 `pixFillMapHoles` (`reference/leptonica/src/adaptmap.c`) と挙動が異なる。
PR #293 で導入した `tests/filter/adaptmap_c_parity.rs::c_parity_weasel_known_divergence` が `#[ignore]` で固定化している通り、`scripts/verify_fillmapholes.c` で同入力に対する出力を比較すると以下の差が出る:

- 3×3 simple ケース: **bit-identical**
- weasel8 (82×73) ケース: 5550/5986 (92.7%) が一致、436 pixel (7.3%) が異なる、最大 channel delta = 233

入力（`pixGammaTRC` + 0穴パターン）は両者完全一致が確認済みなので、差は純粋に `fill_map_holes` 自体のアルゴリズム差である。

### アルゴリズムの違い

| 観点                              | C版 `pixFillMapHoles`                                            | Rust 現行 `fill_map_holes_inner`        |
| --------------------------------- | ---------------------------------------------------------------- | --------------------------------------- |
| ストラテジ                        | 列優先（縦方向に伝播 → 列まるごと複製）                          | 4近傍順次伝播（左→右→下→上の2pass）     |
| 除外列の扱い                      | データ無し列は最近接の有効列を `pixRasterop` で **列丸ごと複製** | 4近傍が全て無効ならエッジ拡張fallback   |
| 末尾partial tile (w > nx, h > ny) | `pixRasterop(pix, w-1, 0, 1, h, PIX_SRC, pix, w-2, 0)` で1列複製 | `valid_x..w` 範囲を最後の有効値で埋める |

`pixGetBackgroundGrayMap` 等の文書画像背景マップ用途では C版が経験的に良好（縦方向の連続性を活かす）。Rust版の4近傍伝播は一般的だが、document image での後段 `pixApplyInvBackgroundGrayMap` 出力にartifactを生む可能性がある。

### 直近のC版バグ修正との関係

`reference/leptonica` 上流コミット `737f969e` (2025-03) は `pixFillMapHoles` の列複製ループ境界を `j < w` → `j < nx` に修正した。Rust版は別アルゴリズムでこの境界バグの該当コードパス自体が無いため**自動的に未影響**だが、bit同等性が無いので C版「修正後」の結果も再現できていない。

## Goal

C版 `pixFillMapHoles(pix, nx, ny, L_FILL_BLACK)` と **bit-identical** な出力を Rust の `fill_map_holes(pix, nx, ny)` で得られるようにする。

具体的には `tests/filter/adaptmap_c_parity.rs::c_parity_weasel_known_divergence` の `#[ignore]` を解除し、C版出力 (`/tmp/c_fillmapholes_weasel.png`) との bit比較 assertion を追加して PASS させる。

### 非Goal

- C版の `L_FILL_WHITE` ケース対応（Rust API は現状 FILL_BLACK のみ。filltype引数追加は別PR）
- 4近傍伝播版の保持（用途が無い。C版に統一）
- C版コメント文 / `numa` 名称までの逐次移植（実装意図を保てば良い）

## C版アルゴリズム要約

`reference/leptonica/src/adaptmap.c` lines 1455-1581（doc comment + 関数本体）を Rust に移植する。擬似コードで:

```text
fn fill_map_holes_c_aligned(pix: &mut Pix, nx, ny):
    w, h = pix.dimensions()
    valtest = 0   // L_FILL_BLACK (Rust API 固定)
    column_has_data: Vec<bool> = vec![false; nx]
    nmiss = 0

    // ── Phase 1: 各列について縦方向に穴を埋める ──
    for j in 0..nx:
        // 列内の最初の非valtest pixel を探す。
        // 探索範囲は 0..ny（完全タイル領域）に限る — h > ny の partial 行は
        // 信頼できない値を含む可能性があるため、データ起点としては使わない。
        let first = (0..ny).find(|i| pix[j, i] != valtest)
        match first:
            None:
                column_has_data[j] = false
                nmiss += 1
            Some(y):
                column_has_data[j] = true
                let val = pix[j, y]
                // y より上を val で埋める
                for i in 0..y:
                    pix[j, i] = val
                // 下方向は h まで伝播（partial 行 ny..h も lastval で埋める）。
                let mut lastval = pix[j, 0]
                for i in 1..h:
                    if pix[j, i] == valtest:
                        pix[j, i] = lastval
                    else:
                        lastval = pix[j, i]

    if nmiss == nx:
        return Err("no bg found; no data in any column")

    // ── Phase 2: データ無し列を、最初の有効列から左右に複製 ──
    if nmiss > 0:
        let goodcol = (0..nx).find(|j| column_has_data[j]).unwrap()
        // goodcol より左を、右の列(j+1)の値で塗りつぶす（後ろから前に向かって）
        for j in (0..goodcol).rev():
            for i in 0..h:
                pix[j, i] = pix[j+1, i]
        // goodcol より右の data なし列を、左隣 (j-1) で複製
        for j in (goodcol+1)..nx:
            if !column_has_data[j]:
                for i in 0..h:
                    pix[j, i] = pix[j-1, i]

    // ── Phase 3: w > nx の場合、最後の partial 列を複製 ──
    if w > nx:
        for i in 0..h:
            pix[w-1, i] = pix[w-2, i]
    // ny < h は C版でも明示的に扱われない（Phase 1 の lastval 伝播で h-1 まで埋まる）
```

実装上の注意:

- C版は in-place; Rust版は `&Pix → Pix` を保つため `try_into_mut().unwrap()` で書き込み可能 buffer に変換する（既存実装と同じ）
- C版は `pixRasterop(...PIX_SRC...)` で1列まるごとコピー。Rust では `for i in 0..h: set_pixel(j, i, get_pixel(j+1, i))` で十分（`unchecked` API 使用）
- **8bpp Pix チェックを追加する**: 現状の `fill_map_holes` 入口に depth check が無く、C版 `pixGetDepth(pix) != 8` 相当のガードを欠いている。GREEN PR で `if pix.depth() != PixelDepth::Bit8 { return Err(FilterError::UnsupportedDepth { ... }) }` を冒頭に追加する

## 変更対象ファイル

### 1. `src/filter/adaptmap.rs`

`fill_map_holes_inner(pix: &Pix, valid_x: u32, valid_y: u32) -> FilterResult<Pix>` を上記C版アルゴリズムで書き直す。

- 引数は現状の `valid_x, valid_y` (= C の nx, ny) をそのまま使う
- 戻り値は新しい `Pix`（in-place ではない）
- nmiss == nx の場合は `FilterError`（既存のエラー型を使う）

呼び出し側 (`fill_map_holes_inner(&pixt2, valid_x, valid_y)?`) は API 変更不要。

### 2. `tests/filter/adaptmap_c_parity.rs`

`c_parity_weasel_known_divergence` を以下に書き換える:

- `#[ignore]` 削除、テスト名から `_known_divergence` を取る (`c_parity_weasel`)
- C版出力の **FNV-1a ピクセルハッシュ** を `EXPECTED_C_WEASEL_HASH: u64 = 0x...` 等の定数としてテストファイルに直接埋め込み、Rust出力ハッシュと比較する
- PNG ファイル自体はリポジトリに追加しない（既存 `golden_manifest.tsv` 方式と整合）

C版期待ハッシュの作り方: `scripts/verify_fillmapholes.c` を1度実行して `/tmp/c_fillmapholes_weasel.png` を生成 → `tests/common::params::pixel_content_hash` 相当の関数で FNV-1a ハッシュを採取し、それを定数としてコミットする。`reference/leptonica` の build や PNG コミットを伴わず、`tests/golden_manifest.tsv` の運用方針とも整合する。

### 3. `tests/filter/adaptmap_reg.rs`

既存の `adaptmap_reg_fill_map_holes_weasel` は Rust 出力 hash を `golden_manifest.tsv` と比較しているので、アルゴリズム変更で hash が変わる。`REGTEST_MODE=generate cargo test --test filter adaptmap_reg_fill_map_holes` で manifest を再生成する。

### 4. `tests/golden_manifest.tsv`

`adaptmap_fill_holes_weasel.04.png` のハッシュ更新。`adaptmap_fill_holes_simple.06.png` は変わらない可能性が高い（3×3 ケースは元々 bit-identical）。

### 5. `scripts/verify_fillmapholes.c`

実装変更後、再実行で `IDENTICAL` になることを確認するだけ。コード変更は不要。

## 影響範囲

`fill_map_holes_inner` を呼ぶ社内コード:

```bash
$ grep -rn "fill_map_holes_inner\|fill_map_holes(" src/ | grep -v test
src/filter/adaptmap.rs:477:    let pixt2_filled = fill_map_holes(&pixt2, valid_x, valid_y)?;
src/filter/adaptmap.rs:758:    fill_map_holes_inner(&map_pix, nx, ny)
src/filter/adaptmap.rs:1186:    let pix_min = fill_map_holes_inner(&pix_min, map_w, map_h)?;
src/filter/adaptmap.rs:1187:    let pix_max = fill_map_holes_inner(&pix_max, map_w, map_h)?;
src/filter/adaptmap.rs:1433:    let filled = fill_map_holes_inner(&pix3, nx, ny)?;
src/filter/adaptmap.rs:1499:    fill_map_holes_inner(&pix3, nx, ny)
```

呼び出し元:

- `get_background_gray_map`
- `get_background_rgb_map`
- `get_foreground_gray_map`
- `get_background_gray_map_morph`
- `get_background_rgb_map_morph`
- `clean_background_to_white_*`

これら全てが C版アルゴリズムに揃うため、後段の `background_norm_*` / `clean_background_to_white_*` の出力も C 同等に近付く（完全 bit同等は別議論）。これらの既存 regression test (golden_manifest) も更新が必要になる可能性が高い。

## TDD サイクル（PR分割）

PR毎に別ブランチを切る。

### PR 1: RED — テスト更新（PR上は `#[ignore]` 維持、ローカルで fail 確認）

ブランチ: `test/c-parity-fill-map-holes-weasel`

- `c_parity_weasel_known_divergence` を `c_parity_weasel` に rename
- C版出力の FNV-1a ハッシュを `EXPECTED_C_WEASEL_HASH` 定数として埋め込み、`assert_eq!(pixel_content_hash(&filled), EXPECTED_C_WEASEL_HASH)` を追加
- ハッシュは事前に `scripts/verify_fillmapholes.c` を1度実行 → `/tmp/c_fillmapholes_weasel.png` から `pixel_content_hash` で採取（手作業）
- **ローカル確認**: `#[ignore]` を一時的に外して `cargo test c_parity_weasel` が **fail** することを確認
- **PR 共有時**: `#[ignore = "RED: blocked on plan 028 GREEN PR"]` を維持して CI を通す
- 意図: assertion を先にコミット履歴に残すことで、GREEN PR で `#[ignore]` を外すだけで RED→GREEN 遷移が一目瞭然になる

### PR 2: GREEN — 実装書き換え

ブランチ: `feat/filter-fill-map-holes-c-aligned`

- `fill_map_holes_inner` を C版アルゴリズムで書き直し
- PR 1 の `#[ignore]` を解除
- `c_parity_weasel` が PASS することを確認
- `scripts/verify_fillmapholes.c` を再実行して **両方 IDENTICAL** を確認
- `REGTEST_MODE=generate cargo test --test filter` で `adaptmap_fill_holes_weasel.04.png` の manifest hash を再生成
- 上流の `get_background_*` / `clean_background_*` テストの hash も再生成（差分が出れば）
- **全テスト + clippy + fmt 通過確認**

### PR 3: REFACTOR (optional)

実装が C版 1:1 移植から離れて読みやすく整理できる場合に。不要なら省略。

## リスク

1. **上流API（`get_background_*`）の挙動変化**:

   現行の Rust 出力に依存している既存の `*_reg` テストの hash 差分が広範囲に発生する可能性がある。`generate` モードで再生成すれば数値的には更新できるが、視覚 artifact が悪化していないか目視確認が望ましい（PR 2 の test plan に含める）

2. **`nmiss == nx` のエラー型**:

   C版は `return 1` (warning + error)。Rust では `FilterError::Other(...)` か新しい variant を作るか議論。既存 `fill_map_holes` シグネチャが `FilterResult<Pix>` なので Err で返す方針

3. **API 互換性**:

   Rust の `fill_map_holes(pix: &Pix, ...)` は `&Pix` を取り新規 `Pix` を返すので、in-place な C版とは API が違う。新アルゴリズムでも `try_into_mut()` 経由で書き込んで `into()` で戻す既存パターンを踏襲

## 完了条件

- [ ] `cargo test --test filter c_parity_weasel` PASS
- [ ] `scripts/verify_fillmapholes.c` 実行で両ケースとも `IDENTICAL`
- [ ] 全 regression test (`cargo test --all-features`) PASS（manifest 更新後）
- [ ] `cargo clippy --all-features --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] PR 2 の merge 後に本計画書の Status を `IMPLEMENTED` に更新

## 参考

- C版実装: `reference/leptonica/src/adaptmap.c` lines 1455-1581
- C版ドキュメント: 同ファイル lines 1457-1493 のコメント
- 比較ヘルパー: `scripts/verify_fillmapholes.c`
- C版直近修正: `737f969e Fix indexing error in pixFillMapHoles()` (Rust側未影響だが上流追従の参考)
- 関連PR: #293 (C parity tests 導入), #294 (clippy 1.95 lints 修正)
