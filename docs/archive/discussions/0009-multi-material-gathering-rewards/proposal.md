# 複数素材報酬の提案

- Status: Proposed
- Date: 2026-09-06

## 目的

Location と Action ごとの報酬を排他的な1結果の抽選から、複数の独立した報酬判定へ変更する。1回の Action 完了で確定素材と追加素材を同時に獲得できるようにし、将来のクラフトを含む他の Action にも同じ仕組みを利用できるようにする。

本提案は discussion 0007 で定めた報酬の抽選方式と初期報酬表を置き換える。同文書で定めたインベントリへの累積、core が抽選を担当する責務、完了後に Target を解放する動作、および1回の完了を Information Log の1行に収める方針は維持する。

## 素材

既存の8素材に Tiny Magic Stone を追加する。

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
| `tiny_magic_stone` | `Tiny Magic Stone` |

Catalog の登録順はこの表の順序とし、既存素材の順序は変更せず Tiny Magic Stone を末尾へ追加する。

## 所要時間と報酬表

以下の全 Action の所要時間を10秒とする。各行は他の行と排他的な Outcome ではなく、Action 完了時に独立して1回判定する報酬エントリである。

| Location | Action | Resource | Amount | Chance |
| --- | --- | --- | ---: | ---: |
| `Nearby Hill` | `Gather` | `Grass` | 1 | 100% |
| `Nearby Hill` | `Gather` | `Pebble` | 1 | 50% |
| `Nearby Hill` | `Hunt` | `Awful Meat` | 1 | 100% |
| `Nearby Hill` | `Hunt` | `Small Fang` | 1 | 60% |
| `Nearby Woods` | `Gather` | `Vine` | 1 | 60% |
| `Nearby Woods` | `Gather` | `Twig` | 1 | 100% |
| `Nearby Woods` | `Hunt` | `Awful Meat` | 1 | 100% |
| `Nearby Woods` | `Hunt` | `Tiny Magic Stone` | 1 | 10% |
| `First Shore` | `Fish` | `Small Fish` | 1 | 30% |
| `First Shore` | `Fish` | `Seaweed Fragment` | 1 | 100% |
| `First Shore` | `Gather` | `Seaweed Fragment` | 1 | 60% |
| `First Shore` | `Gather` | `Pebble` | 1 | 100% |

各 Action の1回あたりの期待獲得数は次のとおりとなる。

| Location | Action | Expected units |
| --- | --- | ---: |
| `Nearby Hill` | `Gather` | 1.5 |
| `Nearby Hill` | `Hunt` | 1.6 |
| `Nearby Woods` | `Gather` | 1.6 |
| `Nearby Woods` | `Hunt` | 1.1 |
| `First Shore` | `Fish` | 1.3 |
| `First Shore` | `Gather` | 1.6 |

## 報酬モデル

RewardTable は Location と Action、および順序を持つ報酬エントリ列からなる。各エントリは Resource id、獲得量、1から100までの整数百分率を持つ。従来のように確率の合計を100%へ制限せず、明示的な `Nothing` エントリも保持しない。

同じ Resource id を一つの表へ複数回登録できる。各エントリは別々に成功判定を行い、成功した同一 Resource の数量を合算する。異なる Resource の順序は、その Resource が成功したエントリとして表内に最初に現れた順序とする。

表は一つ以上のエントリを持つものとするが、100%のエントリは必須としない。全エントリの判定に失敗した場合は空の報酬結果を返し、それを Action は完了したが素材を獲得しなかった `Nothing` と解釈する。

## 完了処理

タスク完了時の処理は core で次の順に行う。

1. 完了した Location と Action に対応する RewardTable を解決する。
2. 報酬エントリを定義順に処理する。
3. chance が100%なら乱数を消費せず成功とし、100%未満ならエントリごとに1回抽選する。
4. 成功した報酬を Resource id ごとに合算し、初出順の報酬結果を作る。
5. 合算後の数量をインベントリへ加算する。
6. Target、Location、Action、および0件以上の報酬結果を含む完了イベントを返す。
7. タスクを削除し、Target を再び選択可能にする。

抽選と集約は core の責務とする。TUI と tools は抽選や集約を再実装せず、core が返した同じ完了イベントを使用する。制御可能な乱数源を維持し、同じ初期状態、入力、seed から同じ結果を再現できるようにする。

## Information Log

完了通知と報酬通知は分けず、1回の Action 完了につき1行の英文ログを追加する。複数素材はカンマと空白で区切る。

```text
Hero completed Gather at Nearby Hill: Grass x1, Pebble x1
Hero completed Hunt at Nearby Woods: Awful Meat x1, Tiny Magic Stone x1
Hero completed Craft at Workshop: Nothing
```

同一素材の複数エントリが成功した場合は、別々に並べず合算した数量を表示する。

```text
Hero completed Gather at Nearby Hill: Pebble x2
```

報酬結果が空の場合だけ `Nothing` と表示する。既存の Target、Location、Action を含むログ形式と、最小端末高で利用できる3行の Information Log を維持する。

## 検証方針

Catalog の出荷コンテンツでは少なくとも次を検証する。

- Resource id が一意であり、全報酬エントリの Resource id を解決できる。
- RewardTable の Location と Action の組が重複せず、対応する全組に表が存在する。
- 各表が一つ以上のエントリを持つ。
- 各エントリの amount が1以上、chance が1以上100以下である。
- 同じ Resource id の複数エントリを許容する。
- 報酬エントリの確率合計を100%へ制限しない。

core では少なくとも次を決定的な乱数源で検証する。

- 複数の異なる Resource が同時に成功し、定義上の初出順で返る。
- 一部のエントリだけが成功する。
- 全エントリが失敗した場合に空の報酬結果が返り、インベントリが変化しない。
- 同じ Resource の複数エントリが成功した場合に数量が一つへ合算される。
- 100%のエントリが乱数を消費せず、必ず成功する。
- 成功した報酬が既存のインベントリへ累積する。
- 完了イベントが Target、Location、Action、および報酬結果の列を保持する。
- 報酬処理後にタスクが削除され、Target が再び選択可能になる。
- 出荷する6組の各 Action が10秒で完了する。

TUI では単一素材、複数素材、同一素材の合算、および `Nothing` のログ文面を検証する。Materials 行は既存どおりインベントリの ResourceStack と Catalog 順を使用するため、複数報酬に固有の表示状態を別に保持しない。

tools は core の報酬結果をそのまま利用し、seed を固定したシミュレーションが再現可能であることを維持する。

## 今回扱わないこと

- クラフトの Action、入力素材、素材消費、成功条件、および UI
- ステータス、道具、Location などによる chance の補正
- 戦闘、敵、勝敗、経験値、およびレベル
- Action の自動反復、中断、および置換
- インベントリのセーブ・ロード
- 素材の用途に基づく供給量と所要時間の再調整

クラフトは今回実装しないが、将来クラフトの全報酬判定が失敗する場合も空の報酬結果として表現できる。クラフト固有の素材消費や失敗時の扱いは、クラフトを提案する時点で別途決める。
