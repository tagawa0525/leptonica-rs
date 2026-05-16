# Core: Pixaa ヘルパー 3 関数 (plan 032 カテゴリ A-3 の続き)

Status: IMPLEMENTED
作成日: 2026-05-12
親計画: docs/plans/032_gap-fill-roadmap-v2.md カテゴリ A-3

## 対象 C 関数 (3)

Pixaa (Pixa of Pixa) の補助 3 関数。残り 10 件 (Pixaa の他、
sort/render 系) は plan 108b で扱う。

### Pixaa flatten / range

- `pixaaFlattenToPixa(paa) -> (Pixa, Option<Numa>)` — 全 Pixa を連結した Pixa、optional に index Numa
- `pixaaSelectRange(paas, first, last) -> Pixaa` — inner Pixa の範囲スライス
- `pixaaSizeRange(paa) -> (minw, minh, maxw, maxh)` — 全 Pix の寸法範囲

## API 設計

```rust
impl Pixaa {
    /// C: `pixaaFlattenToPixa` (with_index = true で Numa を返す)
    pub fn flatten_to_pixa(&self, with_index: bool) -> (Pixa, Option<Numa>);

    /// C: `pixaaSelectRange` (last < 0 は末尾を意味する)
    pub fn select_range(&self, first: i32, last: i32) -> Result<Pixaa>;

    /// C: `pixaaSizeRange` (空なら None)
    pub fn size_range(&self) -> Option<(u32, u32, u32, u32)>;
}
```

## 依存

- 既存 `Pixaa::get`, `Pixaa::len`, `Pixaa::with_capacity`, `Pixaa::push`
- 既存 `Pixa::size_range` (plan 108)
- 既存 `Pixa::pix_slice`, `boxa`, `with_capacity`, `push_with_box`
- 既存 `Numa::push`

## テスト方針

- flatten_to_pixa:
  - 3 個の inner Pixa から flatten すると合計 Pix 数 = 各 inner の合計
  - index Numa は inner index を反復記録
- select_range:
  - first..=last の inner Pixa を抽出
  - last < 0 は末尾まで
  - first >= len で Err
- size_range: 複数 inner で min/max が aggregate される

## 完了条件

- [x] cargo test/clippy/fmt 通過 (10 件パス)
- [x] core.md 3 件 ❌ -> ✅
- [x] plan 032 で 122 を新規 IMPLEMENTED 行として追加
- [ ] PR + Copilot レビュー対応 + マージ

## 実装メモ

- `flatten_to_pixa`: 各 inner Pixa を順に走査して deep_clone した Pix を append。with_index=true で inner index を Numa に push
- `select_range`: `first..=last` 範囲を inner Pixa の clone() で 詰める (Pixa clone は Vec<Arc<PixData>> なので shallow だが、C `L_CLONE` 相当の挙動)
- `size_range`: 各 inner で `Pixa::size_range` (plan 108) を呼び、 min/max を集計
