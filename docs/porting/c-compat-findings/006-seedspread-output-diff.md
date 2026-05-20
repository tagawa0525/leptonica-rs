# C互換性調査 #006: `seedspread` 出力の Rust/C hash 不一致 (6 件)

Phase 3 第二弾で `scripts/golden_map.tsv` に
`region/seedspread` の 6 件 (Rust idx 1-6 ↔ C idx 0-5) を semantic
マッピングとして追加した結果、すべて `Mismatch` として観測された。
本 finding は観測結果と root cause 仮説を記録する。完全特定 + 解消は
別 PR で行う。

## 観測

`tests/c_compat_report.region.txt` (Phase 3 第二弾 mapping 追加後):

```text
[Mismatch] seedspread :: seedspread.01.png :: rust=f1fe8b0727797eb9, c[seedspread.00.png]=f43982d8c41041f5
[Mismatch] seedspread :: seedspread.02.png :: rust=7c67d196cedb5d24, c[seedspread.01.png]=77fbf3890b3b064b
[Mismatch] seedspread :: seedspread.03.png :: rust=7f2824aa04c8b4f1, c[seedspread.02.png]=848d70ab157c3f49
[Mismatch] seedspread :: seedspread.04.png :: rust=fff18377c11f6ba6, c[seedspread.03.png]=9a84d7d3792a075e
[Mismatch] seedspread :: seedspread.05.png :: rust=b5d3b5815aebf8fe, c[seedspread.04.png]=4a1fb784c66eaff6
[Mismatch] seedspread :: seedspread.06.png :: rust=0237cde81f82b0de, c[seedspread.05.png]=9bf43d12df62b68e
```

セマンティクスの対応 (`tests/region/seedspread_reg.rs` と
`reference/leptonica/prog/seedspread_reg.c` の `write_pix_and_check` /
`regTestWritePixAndCheck` 順序を読み合わせて確定):

> **注**: 「Rust check」「C check」は **0-based の呼び出し順序** を
> 指す (どちらも 0 → 1 → 2 ... の順番)。実際の filename / manifest
> での index は `seedspread.01.png` 〜 `seedspread.06.png` (Rust は
> **1-based**) と `seedspread.00.png` 〜 `seedspread.05.png` (C は
> **0-based**) で、`scripts/golden_map.tsv` の `c_index` / `rust_index`
> 欄はその filename index を保持する。下表 1 行目 (4-cc dense) は
> Rust check 0 = filename `seedspread.01.png` / manifest idx 1、C
> check 0 = filename `seedspread.00.png` / manifest idx 0、という
> 対応を意味する。

| Test                              | Rust check (0-based) | C check (0-based) | Rust file (1-based) | C file (0-based) |
| --------------------------------- | -------------------: | ----------------: | ------------------: | ---------------: |
| 4-cc moderately dense (100 seeds) |                    0 |                 0 |                  01 |               00 |
| 8-cc moderately dense (100 seeds) |                    1 |                 1 |                  02 |               01 |
| 4-cc lattice (400 seeds)          |                    2 |                 2 |                  03 |               02 |
| 8-cc lattice (400 seeds)          |                    3 |                 3 |                  04 |               03 |
| 4-cc sparse (4 seeds)             |                    4 |                 4 |                  05 |               04 |
| 8-cc sparse (4 seeds)             |                    5 |                 5 |                  06 |               05 |

C 側にはこれに加えて check 6 (`pixSelectMinInConnComp` から得た
`pixd` の出力) があるが、Rust 側はその checkpoint を実装していない
ため、本 finding の対象外。

`compare_golden` (pixel-level diff) や FNV-1a の content hash
(`pixel_content_hash`) の値そのものは PNG payload の hash ではなく
pixel 配列 (RGBA 32-bit) の hash なので、PNG エンコーダの差では
なく **画像の pixel 値** が C と Rust で異なる。

## Root cause 仮説 (未確定、要追加調査)

`seedspread` は seed pixel から Voronoi-ish に値を広げる
algorithm で、tie-break の挙動が結果に影響する。

仮説:

1. **seedspread のアルゴリズム差**: Rust `region::seedfill::

   seedspread` と C `pixSeedspread` の内部実装で:

   - tie-break の優先順序 (同距離の seed が複数あるときどちらを

     選ぶか) が異なる

   - スキャン方向 (raster 順 vs Hilbert curve など) が異なる
   - 距離関数 (L1 / L∞ / 8-cc/4-cc の subtle 差) の実装差

2. **`paint_marker_3x3` の RGBA 配置差**: Rust 側コメントで

   "`0x00ff0000` を C は green として書き込む" と記載されている
   通り、C 32bpp PIX の byte order (R/G/B/A の bit offset) と Rust
   `core::pixel::compose_rgba` の解釈が一致しているかは要再確認。
   検証手段: 同じ seed image を seedspread せず直接 32bpp に変換
   して marker のみ描画 → C と Rust で hash 一致するか確認。

3. **8bpp → 32bpp 変換 (`convert_to_32`)**: Rust と C で grayscale

   → RGBA 変換の係数 / channel 配置 が違う可能性。

## Next step (別 PR)

優先度順:

1. **仮説 2 の切り分け**: seedspread を呼ばずに、同じ seed image

   から `convert_to_32` + `paint_marker_3x3` のみで hash 取得 →
   C `regTestSetup` + 同等の copy で hash 取得 → 一致するか比較。

2. 仮説 2 が原因なら `paint_marker_3x3` または `compose_rgba` の

   修正で全 6 件解消可能。

3. 仮説 1 (アルゴリズム差) なら個別解消が必要。Rust と C の

   `seedspread` 内部を読み合わせて差分を特定する。

## 関連

- 本 finding 開始経緯: Phase 3 第二弾 PR (region/seedspread の

  6 件 semantic マッピング追加)

- C 実装: `reference/leptonica/src/seedfill.c`, `pixSeedspread`
- Rust 実装: `src/region/seedfill.rs`, `seedspread`
