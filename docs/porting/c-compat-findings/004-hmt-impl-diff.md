# C互換性調査 #004: Hit-Miss Transform 実装差 (fhmtauto 計 8 件)

Phase 2 / Phase 1.5 で発生した `fhmtauto` 系 8 件の `Mismatch` を調査した
結果、`src/morph/binary.rs::hit_miss_transform` と C `pixHMT` の実装差が
原因と判明した。とくに **Identity 1x1 brick で 100% pixel diff** という
極端なケースが含まれており、Sel/HMT パスのどこかに bug があると推定される。

修正は本書のスコープ外で、別 PR (複数想定) で個別に詰める。

## 観測

`compare_golden --module fhmtauto`:

| Test                       |          Diff/Total |    %Diff | 備考          |
| -------------------------- | ------------------: | -------: | ------------- |
| HMT sel_4_1 (Set4cc1)      |      35,437/523,800 |    6.77% | DIFF(alg)     |
| HMT sel_4_2 (Set4cc1)      |      17,862/523,800 |    3.41% |               |
| HMT sel_4_3 (Set4cc1)      |      17,616/523,800 |    3.36% |               |
| HMT sel_8_2 (Set8cc1)      |      25,750/523,800 |    4.92% |               |
| HMT sel_8_3 (Set8cc1)      |      25,293/523,800 |    4.83% |               |
| HMT sel_8_5 (Set8cc1)      |      28,171/523,800 |    5.38% | DIFF(alg)     |
| HMT sel_8_6 (Set8cc1)      |      28,136/523,800 |    5.37% | DIFF(alg)     |
| **HMT identity 1x1 brick** | **523,800/523,800** | **100%** | **DIFF(alg)** |

7 件は 3-7% の局所差 (端近傍が疑わしい)。8 件目 Identity は **全 pixel が
違う** という極端な状態。

## Identity 1x1 brick が 100% diff になる仮説

`scripts/verify_fhmtauto.c:99-105` (Identity ブロック):

```c
/* Identity: 1x1 HIT sel */
printf("\n=== Identity (1x1 brick) ===\n");
SEL *sel_id = selCreateBrick(1, 1, 0, 0, SEL_HIT);
PIX *id_result = pixHMT(NULL, pixs, sel_id);
pixWriteTiff("/tmp/c_fhmtauto_id.tif", id_result, IFF_TIFF_G4, "w");
```

C `pixHMT` は 1x1 hit SEL に対して `pixRasterop(pixd, 0, 0, w, h, PIX_SRC,
pixt, 0, 0)` で **入力をそのままコピー**。「Clear near edges」も SEL の
translations が 0 で no-op。結果: `pixd = pixs`。

tests/morph/fhmtauto_reg.rs L77-78:

```rust
let sel = Sel::create_brick(1, 1).expect("create 1x1 sel");
let result = hit_miss_transform(&pix, &sel).expect("hit_miss_transform 1x1");
```

Rust 側を **コードレベルで読む限り** identity が出るはずだが:

- `Sel::create_brick(1, 1)` は `cx=0, cy=0, data=[Hit; 1]`
- `hit_offsets()` は `[(0, 0)]` を返す (idx=0 → x=0, y=0, x-cx=0, y-cy=0)
- `hit_miss_transform` は `out = all 1` → `shift_and_row(dst, src, 0)` で

  `dst[i] &= src[i]` → `out = src`

→ 出力 = 入力のはずだが、`tests/regout/fhmtauto_id.01.tif` の hex dump
を見ると `ffffffffffffffffffff...` が連続しており、**画像 data が all 1
(全黒)** になっている疑いがある (確証は要 debug)。

可能性 (要 debug):

1. `shift_and_row(dst, src, 0)` で意図と違う動作
2. `Pix::new(w, h, Bit1)` 直後の `data_mut()` が `0xFFFFFFFF` で初期化できていない
3. `try_into_mut().unwrap()` の挙動
4. `clear_unused_bits` が想定外の範囲をクリアしている
5. `hit_offsets()` が `enumerate` のクロージャキャプチャで想定外の動作

## 7 件 (sel_4_*, sel_8_*) が 3-7% diff になる仮説

C `pixHMT` (`reference/leptonica/src/morph.c:380-389`):

```c
/* Clear near edges */
selFindMaxTranslations(sel, &xp, &yp, &xn, &yn);
if (xp > 0) pixRasterop(pixd, 0, 0, xp, h, PIX_CLR, NULL, 0, 0);
if (xn > 0) pixRasterop(pixd, w - xn, 0, xn, h, PIX_CLR, NULL, 0, 0);
if (yp > 0) pixRasterop(pixd, 0, 0, w, yp, PIX_CLR, NULL, 0, 0);
if (yn > 0) pixRasterop(pixd, 0, h - yn, w, yn, PIX_CLR, NULL, 0, 0);
```

SEL の最大 translation 分だけ端を強制 CLEAR。Rust 側 `hit_miss_transform`
(`src/morph/binary.rs:153-224`) には **この処理が無い** (`clear_unused_bits`
は word boundary の padding bits を消すだけで、画像左右上下の数 pixel を
クリアしないため別物)。

これが 3-7% pixel diff の原因と推定。差分が「内部一様、端部集中」であるかは
要確認 (pixXor + count by row で boundary 集中を確認できる)。

## 修正方針 (別 PR)

### Step 1: Identity 1x1 の debug

最も極端なケースから着手:

1. Rust テスト `tests/morph/fhmtauto_reg.rs` に直接 print する debug 用 sub-test

   を追加 (`pix.count_pixels()` で全黒判定)

2. `shift_and_row(dst, src, 0)` をユニットテスト化し、`dst = all 1`,

   `src = arbitrary` で `dst == src` になることを確認

3. `Pix::new(w, h, Bit1)` 直後の `data_mut()` が all 0 で初期化されている

   ことを確認 (1bpp Pix のデフォルト値の規約)

4. `hit_miss_transform` 内で `0xFFFF_FFFF` 初期化が意図通り全 word に

   反映されることを確認

### Step 2: Clear near edges 実装

C `pixHMT:380-389` を Rust に移植:

```rust
let (xp, yp, xn, yn) = sel.find_max_translations();
clear_pixels_in_rect(&mut out_mut, 0, 0, xp, h);
clear_pixels_in_rect(&mut out_mut, w - xn, 0, xn, h);
clear_pixels_in_rect(&mut out_mut, 0, 0, w, yp);
clear_pixels_in_rect(&mut out_mut, 0, h - yn, w, yn);
```

`find_max_translations` は `src/morph/sel.rs` に既にあるので、helper を
追加するだけ。

### Step 3: 検証

修正後の Phase 2 レポートで:

- fhmtauto 8 件すべて Mismatch → Ok を確認
- `tests/morph/fhmtauto_reg.rs` の compare_values で内部 assertion も

  すべて通ることを確認 (regression が無い)

## 関連

- Phase 2.5 第一弾 ([001-jpeg-codec-diffs.md](001-jpeg-codec-diffs.md))
- Phase 2.5 第二弾 ([002-tiff-1bpp-write-limit.md](002-tiff-1bpp-write-limit.md))
- Phase 2.5 第三弾 ([003-morph-brick-comp-vs-plain.md](003-morph-brick-comp-vs-plain.md))
- C `pixHMT` 実装: `reference/leptonica/src/morph.c:338-393`
- Rust `hit_miss_transform`: `src/morph/binary.rs:153-224`
- Rust `Sel::hit_offsets`: `src/morph/sel.rs:442-459`
