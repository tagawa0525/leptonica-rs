# 未実装機能 順次実装オーケストレーション戦略

## 目的

feature-comparison-c-vs-rust.mdの未実装機能を、サブエージェントに委託して順次実装する。
メインエージェントは進捗管理・オーケストレーションに徹し、コンテキストを節約する。

---

## 役割分担

### メインエージェント（オーケストレーター）

- **実装順序の決定**: feature-comparison-c-vs-rust.mdに基づき優先順位を決定
- **並行実装の防止**: 1機能完了後に次のサブエージェントを起動
- **サブエージェント起動・監視**: Taskツールで起動、完了を待機
- **質問への回答**: サブエージェントからの質問に回答（必要に応じ調査エージェント委託）
- **進捗管理**: このファイルの進捗状況を更新
- **ドキュメント更新**: feature-comparison-c-vs-rust.mdを実装完了時に更新

### サブエージェント（実装担当）

- **ブランチ作成**: mainから機能ブランチを作成
- **計画立案**: **プランモード**で実装計画を立案
- **実装**: 計画に基づき実装
- **テスト作成**: 新機能には適切なテストを追加
- **品質チェック**: `cargo fmt` と `cargo clippy` を実行
- **コミット**: `/git-commit` でコミットを作成
- **質問の記録**: 不明点は計画ファイルに記録

### 調査エージェント（必要時）

時間のかかる調査をメインエージェントまたはサブエージェントから委託される。

- C版ソースコードの詳細調査
- 既存実装パターンの調査

---

## ワークフロー

```text
メインエージェント
│
├─ 1. 次の実装対象機能を選択
├─ 2. サブエージェントを起動（Taskツール、プランモード指示）
├─ 3. サブエージェント完了を待機
├─ 4. 質問があれば回答
├─ 5. 実装完了確認、進捗更新
└─ 6. 次の機能へ（手順1に戻る）
```

---

## サブエージェント起動テンプレート

```markdown
あなたは [機能名] の実装を担当するサブエージェントです。

## 手順
1. mainブランチから新規ブランチを作成
   git checkout -b [ブランチ名] main
2. プランモードで実装計画を立案（docs/plans/配下に計画ファイル作成）
3. 計画承認後、実装を実行
4. cargo fmt && cargo clippy で品質チェック
5. テストが全てパスすることを確認
6. /git-commit でコミット作成
7. 実装完了を報告

## 参照情報
- C版ソース: reference/leptonica/src/[ファイル名]
- 実装先クレート: crates/[クレート名]/
- 既存パターン参照: [関連する既存実装ファイル]

## 質問がある場合
計画ファイルの「## 質問」セクションに記録してください。
```

---

## 実装キュー

feature-comparison-c-vs-rust.mdに基づく優先順位：

### Phase 1（完了）

| # | 機能 | クレート | C版ソース | 状態 |
| --- | ------ | --------- | ---------- | ------ |
| 1 | GIF I/O | leptonica-io | gifio.c | ✅ 完了 |
| 2 | WebP I/O | leptonica-io | webpio.c | ✅ 完了 |
| 3 | グレースケール形態学 | leptonica-morph | graymorph.c | ✅ 完了 |
| 4 | Pixa | leptonica-core | pixabasic.c | ✅ 完了 |
| 5 | Numa | leptonica-core | numabasic.c | ✅ 完了 |
| 6 | 任意角度回転 | leptonica-transform | rotate.c, rotateam.c | ✅ 完了 |
| 7 | アフィン変換 | leptonica-transform | affine.c | ✅ 完了 |
| 8 | シアー変換 | leptonica-transform | shear.c | ✅ 完了 |
| 9 | バイラテラルフィルタ | leptonica-filter | bilateral.c | ✅ 完了 |
| 10 | ランクフィルタ | leptonica-filter | rank.c | ✅ 完了 |
| 11 | 色セグメンテーション | leptonica-color | colorseg.c | ✅ 完了 |
| 12 | 画像比較 | leptonica-core | compare.c | ✅ 完了 |
| 13 | 画像合成/ブレンド | leptonica-core | blend.c | ✅ 完了 |
| 14 | 論理演算 | leptonica-core | rop.c | ✅ 完了 |

### Phase 2（高優先度 - 基本機能の補完）

| # | 機能 | クレート | C版ソース | 状態 |
| --- | ------ | --------- | ---------- | ------ |
| 15 | 算術演算 | leptonica-core | pixarith.c | ✅ 完了 |
| 16 | FPix（浮動小数点画像） | leptonica-core | fpix1-2.c | ✅ 完了 |
| 17 | 双線形変換 | leptonica-transform | bilinear.c | ✅ 完了 |
| 18 | 射影変換 | leptonica-transform | projective.c | ✅ 完了 |
| 19 | 適応マッピング | leptonica-filter | adaptmap.c | ✅ 完了 |
| 20 | 境界追跡 | leptonica-region | ccbord.c | 待機中 |

### Phase 3（中優先度 - よく使われる機能）

| # | 機能 | クレート | C版ソース | 状態 |
| --- | ------ | --------- | ---------- | ------ |
| 21 | カラー形態学 | leptonica-morph | colormorph.c | 待機中 |
| 22 | 細線化 | leptonica-morph | ccthin.c | 待機中 |
| 23 | シーケンス操作 | leptonica-morph | morphseq.c | 待機中 |
| 24 | 色塗りつぶし | leptonica-color | colorfill.c | 待機中 |
| 25 | 着色 | leptonica-color | coloring.c | 待機中 |
| 26 | ヒストグラム | leptonica-core | numafunc1.c | 待機中 |
| 27 | グラフィックス | leptonica-core | graphics.c | 待機中 |
| 28 | 四分木 | leptonica-region | quadtree.c | 待機中 |

### Phase 4（低優先度 - 専門的機能）

| # | 機能 | クレート | C版ソース | 状態 |
| --- | ------ | --------- | ---------- | ------ |
| 29 | DWA（高速形態学） | leptonica-morph | morphdwa.c | 待機中 |
| 30 | デワーピング | leptonica-recog | dewarp1-4.c | 待機中 |
| 31 | Sarray（文字列配列） | leptonica-core | sarray1-2.c | 待機中 |
| 32 | 迷路生成/解法 | leptonica-* | maze.c | 待機中 |
| 33 | ワーパー | leptonica-transform | warper.c | 待機中 |
| 34 | バーコード | leptonica-recog | bardecode.c | 待機中 |

### Phase 5（特殊用途 - 外部依存が重い）

| # | 機能 | クレート | C版ソース | 状態 |
| --- | ------ | --------- | ---------- | ------ |
| 35 | JP2K (JPEG2000) | leptonica-io | jp2kio.c | 待機中 |
| 36 | PDF出力 | leptonica-io | pdfio1-2.c | 待機中 |
| 37 | PostScript出力 | leptonica-io | psio1-2.c | 待機中 |

---

## 質問・回答ログ

サブエージェントからの質問と回答を記録する。

```markdown
### [日付] - [機能名] - 質問#N
**質問**: ...
**回答**: ...
**調査委託**: あり/なし
```

（まだ記録なし）

---

## 進捗チェックリスト

### Phase 1（完了: 14/14）

- [x] GIF I/O（feat/io-giff）
- [x] WebP I/O（feat/io-webp）
- [x] グレースケール形態学（feat/morph-grayscale）
- [x] Pixa（feat/core-pixa）
- [x] Numa（feat/core-numa）
- [x] 任意角度回転（feat/transform-rotate）
- [x] アフィン変換（feat/transform-affine）
- [x] シアー変換（feat/transform-shear）
- [x] バイラテラルフィルタ（feat/filter-bilateral）
- [x] ランクフィルタ（feat/filter-rank）
- [x] 色セグメンテーション（feat/color-segmentation）
- [x] 画像比較（feat/compare）
- [x] 画像合成/ブレンド（feat/blend）
- [x] 論理演算（feat/rop）

### Phase 2（高優先度: 5/6）

- [x] 算術演算（feat/arith）
- [x] FPix（feat/fpix）
- [x] 双線形変換（feat/bilinear）
- [x] 射影変換（feat/projective）
- [x] 適応マッピング（feat/adaptmap）
- [ ] 境界追跡

### Phase 3（中優先度: 0/8）

- [ ] カラー形態学
- [ ] 細線化
- [ ] シーケンス操作
- [ ] 色塗りつぶし
- [ ] 着色
- [ ] ヒストグラム
- [ ] グラフィックス
- [ ] 四分木

### Phase 4（低優先度: 0/6）

- [ ] DWA（高速形態学）
- [ ] デワーピング
- [ ] Sarray（文字列配列）
- [ ] 迷路生成/解法
- [ ] ワーパー
- [ ] バーコード

### Phase 5（特殊用途: 0/3）

- [ ] JP2K (JPEG2000)
- [ ] PDF出力
- [ ] PostScript出力
