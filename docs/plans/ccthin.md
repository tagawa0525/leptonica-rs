# 細線化（Thinning）実装計画

## 概要

連結成分を保持しながら二値画像を1ピクセル幅の骨格（スケルトン）に縮小する細線化機能を実装する。

## 参照

- C版ソース: `reference/leptonica/src/ccthin.c`
- 細線化用SEL定義: `reference/leptonica/src/sel2.c` (sela4ccThin, sela8ccThin, sela4and8ccThin)

## 実装する機能

### 1. 細線化用構造化要素（SEL）

C版の `sel2.c` で定義されている細線化用のSELパターンを実装する。

#### 4連結細線化用SEL (sel_4_1 ~ sel_4_9)

```text
sel_4_1:    sel_4_2:    sel_4_3:    sel_4_4:
  x          x         o          o
oCx        oCx        oCx        oCx
  x         o          x         o

sel_4_5:    sel_4_6:    sel_4_7:    sel_4_8:    sel_4_9:
 ox         o          xx          x         o x
oCx        oCx        oCx        oCx        oCx
 o          ox         o         o x          x
```

#### 8連結細線化用SEL (sel_8_1 ~ sel_8_9)

```text
sel_8_1:    sel_8_2:    sel_8_3:    sel_8_4:
 x          x         o          o
oCx        oCx        oCx        oCx
 x         o          x         o

sel_8_5:    sel_8_6:    sel_8_7:    sel_8_8:    sel_8_9:
o x        o          x          x         ox
oCx        oCx        oCx        oCx        oCx
o          o x        oo         ox          x
```

#### 4/8連結共用SEL (sel_48_1, sel_48_2)

```text
sel_48_1:   sel_48_2:
 xx        o x
oCx        oCx
oo         o x
```

### 2. 細線化タイプ

```rust
/// 細線化の対象（前景/背景）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinType {
    /// 前景を細線化（通常の細線化）
    Foreground,
    /// 背景を細線化（前景を太らせる効果）
    Background,
}
```

### 3. 細線化SELセット

C版の `selaMakeThinSets()` に対応する機能。11種類のSELセットを提供。

```rust
/// 細線化用SELセットのインデックス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinSelSet {
    /// Set 1: sel_4_1, sel_4_2, sel_4_3 (4連結、推奨)
    Set4cc1 = 1,
    /// Set 2: sel_4_1, sel_4_5, sel_4_6
    Set4cc2 = 2,
    /// Set 3: sel_4_1, sel_4_7, sel_4_7_rot
    Set4cc3 = 3,
    /// Set 4: sel_48_1, sel_48_1_rot, sel_48_2
    Set48 = 4,
    /// Set 5: sel_8_2, sel_8_3, sel_8_5, sel_8_6 (8連結、推奨)
    Set8cc1 = 5,
    /// Set 6: sel_8_2, sel_8_3, sel_48_2
    Set8cc2 = 6,
    /// Set 7: sel_8_1, sel_8_5, sel_8_6
    Set8cc3 = 7,
    /// Set 8: sel_8_2, sel_8_3, sel_8_8, sel_8_9
    Set8cc4 = 8,
    /// Set 9: sel_8_5, sel_8_6, sel_8_7, sel_8_7_rot
    Set8cc5 = 9,
    /// Set 10: sel_4_2, sel_4_3 (太らせ用)
    Thicken4cc = 10,
    /// Set 11: sel_8_4 (太らせ用)
    Thicken8cc = 11,
}
```

### 4. メインAPI

```rust
/// 連結成分を保持する細線化（シンプルAPI）
pub fn thin_connected(
    pix: &Pix,
    thin_type: ThinType,
    connectivity: Connectivity,
    max_iters: u32,
) -> MorphResult<Pix>;

/// 連結成分を保持する細線化（SELセット指定版）
pub fn thin_connected_by_set(
    pix: &Pix,
    thin_type: ThinType,
    sels: &[Sel],
    max_iters: u32,
) -> MorphResult<Pix>;

/// 細線化用SELセットを作成
pub fn make_thin_sels(set: ThinSelSet) -> Vec<Sel>;
```

## アルゴリズム

### 細線化の基本手順（pixThinConnectedBySet準拠）

1. 入力画像の準備
   - 前景細線化: 入力をそのまま使用
   - 背景細線化: 入力を反転

2. 反復処理（収束まで）

   ```python
   for iteration in 0..max_iters:
       for rotation in [0, 90, 180, 270]:  # 4方向の回転
           for sel in sels:
               rotated_sel = rotate_sel(sel, rotation)
               hmt_result = hit_miss_transform(pixd, rotated_sel)
               accumulated_result = OR(accumulated_result, hmt_result)
           pixd = pixd - accumulated_result  # 集約結果を減算

       if pixd == previous_pixd:
           break  # 収束
   ```

3. 後処理（背景細線化の場合）
   - 画像を再反転
   - 境界に接続された成分を除去

### SELの90度回転

既存の `Sel::reflect()` を参考に `Sel::rotate_orth()` を実装する必要がある。

```rust
impl Sel {
    /// 90度単位で直交回転
    /// rotation: 0=0度, 1=90度, 2=180度, 3=270度
    pub fn rotate_orth(&self, rotation: u32) -> Self;
}
```

## ファイル構成

```text
crates/leptonica-morph/src/
├── lib.rs          # thin モジュール追加
├── sel.rs          # rotate_orth() 追加
├── thin.rs         # 新規作成（細線化実装）
└── thin_sels.rs    # 新規作成（細線化用SEL定義）
```

## 実装手順

1. `sel.rs` に `rotate_orth()` メソッドを追加
2. `thin_sels.rs` を作成（細線化用SELパターン定義）
3. `thin.rs` を作成（細線化アルゴリズム実装）
4. `lib.rs` を更新してモジュールとAPIをエクスポート
5. ユニットテスト作成
6. `cargo fmt && cargo clippy` で品質チェック
7. 全テスト実行

## 依存関係

- 既存の `hit_miss_transform()` (binary.rs)
- 既存の `Sel` 構造体
- `Pix::clone()`, ピクセル操作

## テスト計画

1. SEL回転の正確性テスト
2. 細線化用SELパターンの検証
3. 単純な図形（正方形、線分）の細線化
4. 4連結/8連結での結果比較
5. 前景/背景細線化の双対性テスト
6. 収束テスト（無限ループ防止）

## 質問

なし

## ステータス

- [x] 計画作成
- [x] sel.rs に rotate_orth() 追加
- [x] thin_sels.rs 作成（細線化用SEL定義）
- [x] thin.rs 作成（細線化アルゴリズム）
- [x] lib.rs 更新
- [x] cargo fmt && cargo clippy 成功
- [x] 全テスト通過（65テスト）
- [x] 実装完了
