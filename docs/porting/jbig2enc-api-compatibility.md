# jbig2enc移植に向けたleptonica-rs API互換性調査

## 調査方針

jbig2encが実際に呼び出すleptonica関数について、C版の呼び出しパターンとleptonica-rsのAPIを突き合わせ、**挙動・シグネチャ上の差異**を特定した。
従来のgap analysis（`docs/plans/jbig2enc-gap-analysis.md`）は「関数が存在するか」のみを確認していたが、本文書では**使い方のレベルで差異がないか**を検証する。

調査対象ソース:
- `reference/jbig2enc/src/jbig2.cc`
- `reference/jbig2enc/src/jbig2enc.cc`
- `reference/jbig2enc/src/jbig2sym.cc`
- `reference/jbig2enc/src/jbig2comparator.cc`

---

## 差異一覧（リスク順）

### HIGH: 移植をブロックする差異

#### 1. `morph_sequence` — `r`・`x` 演算子が未実装

jbig2.ccで使用されるmorphシーケンス文字列に、leptonica-rsが未実装の演算子が含まれる。

```c
// jbig2.cc:119-121
static const char *segment_mask_sequence  = "r11";
static const char *segment_seed_sequence  = "r1143 + o4.4 + x4";
static const char *segment_dilation_sequence = "d3.3";
```

- `r<n>` = ランク縮小（rank reduction, 各桁1〜4がランク閾値; 桁数分だけ2倍縮小を繰り返す）
- `x<n>` = バイナリ拡張（binary expansion）

leptonica-rsの`sequence.rs:213-218`で明示的にエラーを返す:
```rust
'r' | 'x' | 'b' => Err(MorphError::UnsupportedOperation(...))
```

**影響**: テキストセグメンテーション処理全体が動作しない。jbig2enc の中核機能に直結する。

**対処**: `expand_replicate()`（transform crate）を活用した `x` 演算子と、`pixReduceRankBinaryCascade` 相当の `r` 演算子を morph crate に実装する必要がある。

---

#### 2. `pixRasterop` — 領域指定（オフセット付き）が未実装

jbig2sym.ccでは目標画像の特定オフセット位置にラスタ演算を適用している。

```c
// jbig2sym.cc:405-406
pixRasterop(targetcopy, deltax, deltay, symbol->w, symbol->h,
            PIX_SRC ^ PIX_DST,
            symbol, 0, 0);
```

引数の意味: `pixRasterop(dst, dx, dy, w, h, op, src, sx, sy)` — dstの(dx,dy)からw×hの領域にsrcの(sx,sy)からw×hを演算する。

leptonica-rsの`rop_inplace`は**画像全体同士の演算のみ**に対応しており、オフセット・領域指定がない:
```rust
// rop.rs:432
pub fn rop_inplace(&mut self, other: &Pix, op: RopOp) -> Result<()>
// → 同一サイズの画像全体にしか適用できない
```

**影響**: refinementコーディング（jbig2sym.cc）の実装が不可能。

**対処**: `PixMut`に以下のAPIを追加する必要がある:
```rust
pub fn rop_region_inplace(
    &mut self,
    dst_x: i32, dst_y: i32,
    width: u32, height: u32,
    op: RopOp,
    src: &Pix,
    src_x: i32, src_y: i32,
) -> Result<()>
```

---

### MEDIUM: 回避策があるが注意が必要な差異

#### 3. `pixSubtract` — 意味論の確認

```c
// jbig2.cc:158
pixSubtract(pixb, pixb, pixd);  // pixb = pixb AND NOT pixd
```

C版`pixSubtract(d, a, b)`は `d = a AND NOT b`（ビット演算）。

leptonica-rsの対応:
- `Pix::subtract()`（compare.rs）は**算術減算** `(a-b).clamp(0)`。1bppでは一致するが、他深度では意味が異なる。
- `Pix::rop(&other, RopOp::AndNotSrc)`は `!s & d` = `d AND NOT s`。

jbig2enc呼び出しの`pixb = pixb AND NOT pixd`は、`pixb`が dst かつ `pixd`が src として `RopOp::AndNotSrc` で対応できる。
ただし jbig2enc は1bpp画像での使用なので `Pix::subtract()` でも動作する。

**対処**: `Pix::rop(&pixd, RopOp::AndNotSrc)` を使う。`subtract()`は使わない（深度依存のため）。

---

#### 4. `pixRasteropFullImage` — ROP定数のマッピング

```c
// jbig2.cc:204
pixRasteropFullImage(pixd1, piximg1, PIX_SRC | PIX_DST);  // = OR演算

// jbig2sym.cc:405-406
pixRasterop(..., PIX_SRC ^ PIX_DST, ...);  // = XOR演算
```

C版の定数値（`reference/leptonica/src/pix.h`より）:
```
PIX_SRC = 0xc, PIX_DST = 0xa
PIX_SRC | PIX_DST = 0xe → OR
PIX_SRC ^ PIX_DST = 0x6 → XOR
```

leptonica-rsの対応:
- `RopOp::Or` = `s | d` ✓
- `RopOp::Xor` = `s ^ d` ✓
- 全画像への`rop_inplace` → `PixMut::rop_inplace(&other, op)` で対応可能

**対処**: 定数の数値からRopOpへの対応表を把握して使う。全画像適用なら問題なし。

---

#### 5. `naclass->array[i]` — Numa内部配列への直接アクセス

```cpp
// jbig2enc.cc:623, 641
const int sym = (int) ctx->classer->naclass->array[*i];
```

C版`NUMA`の内部フィールド`array`への生ポインタアクセス。

leptonica-rsでは`Numa::as_slice()`が`&[f32]`を返す:
```rust
// numa/mod.rs:198
pub fn as_slice(&self) -> &[f32]
```

**対処**: `naclass.as_slice()[i] as usize` で代替可能。ただし`JbClasser::naclass`は`Vec<usize>`なので直接インデックスアクセスで問題なし。

---

#### 6. `pixChangeRefcount` / `pixGetRefcount` — 参照カウント操作

```cpp
// jbig2enc.cc:214-216
ctx->classer->pixat->pix[new_representant]->refcount +=
    ctx->classer->pixat->pix[second_template]->refcount;
pixChangeRefcount(ctx->classer->pixat->pix[new_representant],
                  pixGetRefcount(ctx->classer->pixat->pix[second_template]));
```

この操作はC版のPIXAが「同一PIXポインタを複数スロットで共有する」仕組みに依存している。
シンボル統合時に参照カウントを手動で増やし、PIXAから削除してもPIXを解放させないためのハック。

leptonica-rsでは `Pix` は `Arc<PixData>` で管理されるため、この参照カウント操作は概念的に不要。
`Pixa`はシンボルのcloneを保持するので、`replace()`と`remove()`を使ったシンボル統合ロジックの実装で同等の挙動を実現できる。

**対処**: jbig2enc移植時に、このブロックを `Pixa::replace()` / `remove()` を使った所有権ベースのロジックに書き直す（leptonica-rs互換のffi呼び出しではなく、Rustネイティブな実装として）。

---

### LOW: 実質的に同等（問題なし）

| C側 | Rust側 | 備考 |
|-----|--------|------|
| `jbCorrelationInitWithoutComponents` | `JbClasser::correlation_init()` | シグネチャ互換 |
| `jbAddPage` | `JbClasser::add_page()` | 互換 |
| `classer->pixat` | `JbClasser::pixat: Vec<Pix>` | 公開フィールド、直接アクセス可 |
| `classer->naclass` | `JbClasser::naclass: Vec<usize>` | 公開フィールド |
| `classer->ptall` | `JbClasser::ptall: Vec<(i32,i32)>` | 公開フィールド |
| `classer->napage` | `JbClasser::napage: Vec<usize>` | 公開フィールド |
| `classer->nclass` | `JbClasser::nclass: usize` | 公開フィールド |
| `classer->npages` | `JbClasser::npages: usize` | 公開フィールド |
| `classer->baseindex` | `JbClasser::base_index: usize` | 公開フィールド（命名差異のみ）|
| `numaGetIValue` | `Numa::get_i32()` | `Option<i32>` 返却 |
| `numaSetValue` | `Numa::set()` | `Result<()>` 返却 |
| `pixaGetCount` | `Pixa::len()` | 互換 |
| `pixaReplacePix` | `Pixa::replace()` | 旧値を`Result<Pix>`で返す |
| `pixaRemovePix` | `Pixa::remove()` | 旧値を`Result<Pix>`で返す |
| `pixThresholdPixelSum` | `Pix::threshold_pixel_sum()` | `Result<bool>`返却 |
| `pixCountConnComp` | `find_connected_components().len()` | コンポーネント数はlen()で取得 |
| `pixSeedfillBinary` | `seedfill_binary()` + `ConnectivityType` | 互換 |
| `pixMorphSequence`（d/e/o/c のみ） | `morph_sequence()` | `r`/`x`を除いて互換 |
| `pixOr` | `Pix::or()` / `PixMut::rop_inplace(Or)` | 全画像OR |
| `pixGetWidth/Height/Depth` | `width()` / `height()` / `depth()` | 互換 |
| `pixGetPixel` / `pixSetPixel` | `get_pixel()` / `set_pixel()` | safe変種あり |
| `pixSetPadBits` | `set_pad_bits()` | 互換 |
| `pixCountPixels` | `count_pixels()` | 互換 |
| `tiffGetCount` | `tiff_page_count()` | 互換 |
| `pixReadTiff` (subpage) | `read_tiff_page()` | 互換 |

---

## 必要な実装作業のまとめ

jbig2enc-rsを実装する前に、leptonica-rs側で以下を追加する必要がある:

### leptonica-morph crate への追加

1. **`r<n>` ランク縮小演算子** (`morph_sequence`対応)
   - 参照: `reference/leptonica/src/binreduce.c`の`pixReduceRankBinary2()`等
   - 関連: `transform::expand_replicate()`（逆演算）がすでにある

2. **`x<n>` バイナリ拡張演算子** (`morph_sequence`対応)
   - 参照: `reference/leptonica/src/binexpand.c`の`pixExpandBinaryPower2()`
   - 注: `transform::expand_replicate()`として既実装の可能性あり（確認要）

### leptonica-core crate への追加

3. **`PixMut::rop_region_inplace()`** — オフセット・領域指定付きラスタ演算
   - 参照: `reference/leptonica/src/rop.c`の`pixRasterop()`
   - jbig2sym.ccのrefinementコーディングに必要

---

## 補足: 不要になったC関数

以下のC関数群はRustの言語仕様・型システムにより不要:

| C関数 | 不要な理由 |
|-------|-----------|
| `pixClone` / `pixDestroy` | `Arc<PixData>` による自動管理 |
| `pixChangeRefcount` / `pixGetRefcount` | `Arc` + 所有権モデルで不要 |
| `lept_fopen` / `lept_fclose` / `lept_free` | Rustの所有権システムで不要 |
| `pixCopy(NULL, src)` | `src.clone()` で代替 |
