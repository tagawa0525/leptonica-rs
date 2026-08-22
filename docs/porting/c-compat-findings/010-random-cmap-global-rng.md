# C互換性調査 #010: pixaDisplayRandomCmap のグローバル乱数系列 (未解消)

plan 902 PR 42 で `watershed_reg` の 22 ペアを張った際、`wshedRenderColors`
とそれをタイル表示する check (C 10/11 と 22/23 の計 4 件) が Mismatch に
なった。原因を特定済みで、PR 43 で解消する。

## 症状

| C check | 内容 | 結果 |
| --: | --- | --- |
| 7 / 19 | `pixaDisplayRandomCmap` の出力そのもの | Ok |
| 10 / 22 | `wshedRenderColors` (内部で同関数を再度呼ぶ) | Mismatch |
| 11 / 23 | 全段のタイル表示 (10 を含む) | Mismatch |

差分画素はすべて「C も Rust も彩色されているが色が違う」。マスク
(`pixaDisplay`) と盆地の切り出しは一致しており、**カラーマップの中身だけ**
が異なる。

## check 7 が一致した理由

pixel hash はカラーマップではなくインデックスを対象にする。
`pixaDisplayRandomCmap` のインデックス割り当ては `1 + (i % 254)` で決定的
なので、色が違っても hash は一致する。`wshedRenderColors` は
`pixConvertTo32` で色を 32bpp に焼き込むため、そこで初めて差が出る。

## 根本原因

C `pixcmapCreateRandom()` は glibc の `rand()` を呼ぶ:

```c
for (i = hasblack; i < ncolors - haswhite; i++) {
    red[i] = (l_uint32)rand() & 0xff;
    green[i] = (l_uint32)rand() & 0xff;
    blue[i] = (l_uint32)rand() & 0xff;
    ...
}
```

`rand()` は**プロセス全体で共有される 1 本の系列**で、`srand()` が呼ばれ
なければ種は 1。`watershed_reg.c` は `DoWatershed()` を 2 回呼び、各回で
`pixaDisplayRandomCmap` が 2 回 (check 7 と `wshedRenderColors` の内部) 実行
される。1 回あたり `3 * 254 = 762` 個消費するので、4 回の呼び出しはそれぞれ
系列の異なる位置を使う。

Rust の `PixColormap::create_random()` は呼び出しごとに同じ LCG を depth
から初期化するため、何回呼んでも同じカラーマップを返す。したがって
1 回目 (check 7) は「たまたま」インデックス一致で通り、2 回目以降で
色が食い違う。

## 対応方針 (PR 43)

グローバル可変状態は入れない。ライブラリ内に glibc 互換の
`GlibcRand` (TYPE_3 additive-feedback generator。`tests/core/overlap_reg.rs`
に同等の実装が既にある) を置き、乱数源を引数で明示的に渡せる API を足す:

- `PixColormap::create_random_with(depth, has_black, has_white, &mut rng)`
- `Pixa::display_random_cmap_with(w, h, &mut rng)`
- `Wshed::render_colors_with(&mut rng)`

引数なしの既存 API は種 1 の新しい系列を使う (C で `srand()` を呼ばずに
最初に `rand()` を使った場合と一致する)。C の 1 本の系列を再現したい
テストは `GlibcRand` を 1 つ作って全ての呼び出しに渡す。

この仕組みは `warper_reg` (8 件、`srand(seed)` + `rand()` で歪みパラメータを
生成) にもそのまま使える。
