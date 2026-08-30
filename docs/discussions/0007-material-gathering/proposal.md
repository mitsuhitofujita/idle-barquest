# 素材獲得と報酬抽選の提案

- Status: Proposed
- Date: 2026-08-30

## 目的

現在の `Target → Location → Action` とタスク完了イベントに素材報酬を接続し、採取、釣り、狩りの結果をインベントリと Information Log へ反映する。

今回は素材を得る基礎と初期バランスの体感を作ることに限定し、戦闘、素材の消費、合計表示、永続化は要求しない。

## 素材

素材には安定した英語 id と英語表示名を与える。

| Resource id | Display name |
| --- | --- |
| `pebble` | `Pebble` |
| `twig` | `Twig` |
| `grass` | `Grass` |
| `vine` | `Vine` |
| `small_fish` | `Small Fish` |
| `seaweed_fragment` | `Seaweed Fragment` |
| `small_fang` | `Small Fang` |
| `awful_meat` | `Awful Meat` |

`Tiny Magic Stone` は入手経路が決まっていないため、今回は Resource として定義しない。

## 所要時間と報酬表

以下の全 Action の所要時間を10秒とする。報酬量はすべて1個とする。

| Location | Action | Outcome | Probability |
| --- | --- | --- | ---: |
| `Nearby Hill` | `Gather` | `Grass` x1 | 50% |
| `Nearby Hill` | `Gather` | `Pebble` x1 | 50% |
| `Nearby Hill` | `Hunt` | `Awful Meat` x1 | 100% |
| `Nearby Woods` | `Gather` | `Vine` x1 | 20% |
| `Nearby Woods` | `Gather` | `Twig` x1 | 80% |
| `Nearby Woods` | `Hunt` | `Awful Meat` x1 | 90% |
| `Nearby Woods` | `Hunt` | `Small Fang` x1 | 10% |
| `First Shore` | `Fish` | `Small Fish` x1 | 10% |
| `First Shore` | `Fish` | `Nothing` | 90% |
| `First Shore` | `Gather` | `Seaweed Fragment` x1 | 20% |
| `First Shore` | `Gather` | `Pebble` x1 | 80% |

各 Location と Action の組み合わせで、報酬表からちょうど1つの Outcome を抽選する。確率は排他的で、各1回の抽選表の合計を100%とする。`Nothing` は未定義の余りではなく、インベントリを変更しない明示的な Outcome とする。

## データモデル

Catalog に Resource テンプレートと、Location と Action の組み合わせごとの報酬表を保持する。ライブ状態とコンテンツ定義を分離する既存方針に従い、Resource の表示名や報酬確率を `GameState` へ重複して保持しない。

`GameState` には Resource id ごとの所持数をインベントリとして保持する。タスク完了時の処理は core で次の順に行う。

1. 完了した Location と Action に対応する報酬表を解決する。
2. 制御可能な乱数源を使って Outcome を1つ抽選する。
3. Resource が得られた場合は、その所持数を1増やす。
4. Target、Location、Action、抽選 Outcome を含む完了イベントを返す。
5. タスクを削除し、Target を再び選択可能にする。

抽選は core が責任を持ち、TUI は抽選やインベントリ更新を再実装しない。乱数源はテストおよびヘッドレスツールから結果を再現できる構造とし、確率に依存する不安定なテストを作らない。

## Information Log

完了通知と報酬通知は分けず、1回の Action 完了につき1行の英文ログを追加する。

```text
Hero completed Gather at Nearby Hill: Pebble x1
Hero completed Fish at First Shore: Nothing
```

従来の完了ログが持つ Target、Location、Action の情報を維持した上で、末尾に報酬結果を追加する。釣りの空振りもログに `Nothing` と明示し、完了したことと報酬がなかったことを区別できるようにする。

## 検証方針

core では少なくとも次を検証する。

- 出荷コンテンツの Resource id が一意で、全報酬表の Resource id を解決できる。
- 各報酬表の確率合計が100%である。
- 抽選境界の前後で想定した Outcome が選ばれる。
- Resource 獲得時に所持数が1増え、同じ Resource の獲得で累積する。
- `Nothing` でインベントリが変更されない。
- 完了イベントが Target、Location、Action、Outcome を保持する。
- 報酬抽選後にタスクが削除され、Target が再び選択可能になる。
- 各 Action が10秒で完了する。

TUI では Resource 獲得と `Nothing` の両方のログ文面を検証し、既存のログ容量と下揃え表示を維持する。

## 今回扱わないこと

- 戦闘、敵、勝敗、装備による成功率変化
- Action の自動反復、中断、置換
- 所持素材の合計を確認する UI
- インベントリのセーブ・ロード
- 素材の消費、クラフト、技術解放、食料加工
- 経験値、レベル、ステータス成長
- `Tiny Magic Stone` とその入手経路
- discussion 0002 の技術・クラフト表の更新

discussion 0002 の初期素材表は本提案の内容で置き換えるが、同文書のその他の序盤進行はここで代替仕様を決めない。開発が進み、素材の用途を実装する前に棚卸しする。
