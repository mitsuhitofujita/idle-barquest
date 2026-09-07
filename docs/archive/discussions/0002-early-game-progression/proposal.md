# 序盤進行の提案

- Status: Proposed
- Date: 2026-06-22

この文書は、序盤進行のメモを実装エージェント向けの参照資料として整理したものである。
現時点で示されている序盤の流れだけを扱い、進行、バランス、コンテンツの完全な仕様にはしない。

## 命名規則

ゲーム内の名前には ASCII で扱いやすい英語名を使う。元のメモで日本語の設計名を使っていても、
実装ではここに挙げる英語名か、それに近い名前を使う。

品質段階、アイテム品質、採集時の希少品は将来の拡張点とし、序盤進行の初回実装には要求しない。

| 基本アイテム | 将来の希少アイテム例 | 備考 |
| --- | --- | --- |
| `Stone` | `Sharp Stone` | 今後のバランス調整やクラフト拡張のために保留する。 |

## 開始状態

プレイヤーは海岸で目覚めた1人の Hero として始まる。
この状況をゲーム内で明示的に説明せず、利用できるエリア、Action、報酬から開始状態を伝える。

| Actor | 役割 |
| --- | --- |
| `Hero` | ゲーム開始時に利用できる唯一の Actor。 |

最初に探索できるエリアは次のとおり。

| Area id | 表示名 | 目的 |
| --- | --- | --- |
| `first_shore` | `First Shore` | 食料に近い素材と Stone を得る海岸の採集場所。 |
| `nearby_woods` | `Nearby Woods` | Berry、Mushroom、Twig、モンスター素材を得る林。 |
| `nearby_hill` | `Nearby Hill` | Herb とモンスター素材を得る丘。 |

## Stage 1: Hero による採集

Stage 1 の開始時、`Hero` は `First Shore`、`Nearby Woods`、`Nearby Hill` を探索できる。

### 採集報酬

| Area | Resource id | 表示名 | 入手元ラベル | 序盤での用途 |
| --- | --- | --- | --- | --- |
| `First Shore` | `small_fish` | `Small Fish` | Small fish | 保留。食料に近い序盤資源。 |
| `First Shore` | `seaweed` | `Seaweed` | Seaweed | `Campfire` 解放後に `Food` へ加工できる。 |
| `First Shore` | `stone` | `Stone` | Stone | `Stone Workbench` を解放する。 |
| `Nearby Woods` | `berry` | `Berry` | Tree nut / berry | `Campfire` 解放後に `Food` へ加工できる。 |
| `Nearby Woods` | `mushroom` | `Mushroom` | Mushroom | 用途を保留する序盤資源。 |
| `Nearby Woods` | `twig` | `Twig` | Small branch | `Bark` の材料となり、序盤技術を解放する。 |
| `Nearby Hill` | `herb` | `Herb` | Medicinal herb | 用途を保留する序盤資源。 |

### モンスター報酬

現段階では、モンスターは探索の成功率、所要時間、エリアへのアクセスに影響しない。
戦闘ルールができるまでは、探索後に資源を得る供給源として扱う。

| Area | Monster id | 表示名 | 報酬 |
| --- | --- | --- | --- |
| `Nearby Woods` | `biter` | `Biter` | `Small Fang` |
| `Nearby Hill` | `crawler` | `Crawler` | `Small Fang` |

戦闘、敗北、成功の厳密なルールはこの段階では決めない。
実装では、戦闘ルールができるまで単純な完了報酬として扱ってよい。

## Stage 1.1: 最初の技術とクラフト

序盤技術は採集した資源によって解放する。それぞれが小さな能力を追加し、
次の進行段階へ到達しやすくなるようにする。

### 技術の解放

| Technology id | 表示名 | 解放コスト | 効果 |
| --- | --- | --- | --- |
| `stone_workbench` | `Stone Workbench` | `Stone` x40 | `Twig` から `Bark` を作れるようにする。 |
| `campfire` | `Campfire` | `Twig` x10、`Bark` x10 | 基本的な `Food` の加工を解放する。 |
| `simple_bed` | `Simple Bed` | `Twig` x20、`Bark` x20 | `Rest` Action を追加する。 |

### クラフトレシピ

| Recipe id | 出力 | 入力 | 必要な技術 |
| --- | --- | --- | --- |
| `craft_bark_from_twig` | `Bark` x5 | `Twig` x20 | `Stone Workbench` |
| `craft_food_from_berry` | `Food` x1 | `Berry` x20 | `Campfire` |
| `craft_food_from_seaweed` | `Food` x1 | `Seaweed` x20 | `Campfire` |

### 進行上の意図

`Stone Workbench` は最初のクラフト基盤である。採集した `Twig` を `Bark` に変換し、
`Campfire` と `Simple Bed` の解放につなげる。

`Campfire` は基本的な食料生産を始める。`Berry` と `Seaweed` の両方を `Food` に加工できるため、
`Nearby Woods` と `First Shore` のどちらを探索してもサバイバル進行に寄与する。

`Simple Bed` は `Rest` Action を追加する。効果、所要時間、コスト、条件は保留する。

## 保留する決定

次の不足は既知のものとして受け入れ、周辺システムが必要とするまで決めない。

| トピック | 保留する問い |
| --- | --- |
| `Mushroom` | 序盤で何に使うか。 |
| `Herb` | 序盤で何に使うか。 |
| `Small Fish` | `Food` に加工するか、生の食料にするか、別の用途を持たせるか。 |
| `Small Fang` | クラフト、強化、取引のどれに使うか。 |
| `Rest` | 効果、所要時間、資源コスト、必要条件をどうするか。 |
| Monsters | 敗北、成功、報酬発生の条件をどうするか。 |

## 実装時の指針

この文書の表は、最終的なバランスデータではなくコンテンツの初期値として使う。

最初の実装範囲は次のとおり。

1. 3つの初期エリアを追加する。
2. 記載した資源とモンスター報酬を追加する。
3. 3つの序盤技術を追加する。
4. 3つのクラフトレシピを追加する。
5. `Simple Bed` の解放後にだけ `Rest` を追加し、Rest システムの設計までは最小限の挙動にする。

アイテム品質、希少資源のドロップ、完全な戦闘挙動を Stage 1 または Stage 1.1 の前提にしない。
これらは将来の拡張点である。
