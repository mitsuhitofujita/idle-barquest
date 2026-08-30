# 3段階 User Choices の提案

- Status: Proposed
- Date: 2026-08-30

## 目的

User Choices を `Target → Location → Action` の3段階へ変更し、「誰が、どこで、何をするか」を明示的に選択できるようにする。

序盤のレベリングと資源獲得に必要なエリアと行動を自然に提示するとともに、人物・組織、作業場所、行動の責務をゲームモデル上でも分離する。

## 基本概念

### Target

Target は Action を実行する人物または組織である。Hero、Team、Party などを想定する。

組織も一つの Target として扱い、内部の構成員を複数の Location や Action へ分散させない。人物と組織のどちらも、同時に実行できるタスクは1件だけとする。

従来 Target として扱っていた Farm、Livestock などの施設は Target に含めない。

### Location

Location は Target が Action を実行する場所である。地理的なエリアだけでなく、Farm、Livestock、Campfire などの施設も含む。

施設 Location も単独では稼働せず、必ず Target と Action を割り当てて使用する。

Location はゲーム状態で発見または解放されたものだけを選択肢へ表示する。

### Action

Action は、選択した Target が選択した Location で実行する仕事である。

表示および割当可能な Action は、次の条件をすべて満たすものに限定する。

1. ゲーム状態で解放済みである。
2. 選択した Target が実行可能である。
3. 選択した Location で実行可能である。

core の割当処理でも同じ条件を検証し、未解放、非対応、または存在しない Target、Location、Action の組み合わせを拒否する。

## タスクモデル

実行中タスクは `Target + Location + Action` の組で表す。Location は表示上の補助情報ではなく、進行状態と完了イベントに含まれるゲームデータとする。

各 Target は実行中タスクを最大1件だけ保持する。実行中 Target への新しい割当は拒否し、現在のタスクを中断または置換する操作は提供しない。

Action が完了したらタスクを削除し、Target を再び選択可能にする。自動反復は行わないため、同じ仕事を再実行する場合も `Target → Location → Action` を選び直す。

## 選択フロー

User Choices は、現在到達している段階までの列だけを表示する。

1. 最初は Target 列だけを表示する。
2. Target を選択すると、その右に Location 列を生成する。
3. Location を選択すると、その右に Action 列を生成する。
4. Action を選択して割当が成功すると、Target 選択へ戻る。

選択済みの Target と Location は、それぞれ次の列との区切り直前に `<` を表示する。現在入力を受け付ける列だけに選択キーを表示する。

### 戻る操作

Backspace で一段前へ戻る。

- Action 選択中の Backspace: 選択した Location を解除し、Location 選択へ戻る。
- Location 選択中の Backspace: 選択した Target を解除し、Target 選択へ戻る。
- Target 選択中の Backspace: 何もしない。
- `Esc`: 従来どおりゲームを終了する。

### 実行中 Target と選択キー

実行中 Target は一覧から除外せず、選択キーの代わりに `--` を表示する。

```text
|> Target:
| -- Hero
| b) Adventurer
| c) Farmer
```

選択キーは選択可能な候補だけを詰め直した連番ではなく、Target 一覧の固定スロットに対応する。この例では `a` は実行中の Hero に対応するため無効であり、`b` は常に Adventurer を選択する。

Location と Action の候補は、発見・解放状態および互換性によって絞り込んだ一覧を表示する。入力時には画面に表示したスロットと同じ対応を使用し、core の検証を迂回しない。

## 初期コンテンツ

ゲーム開始時の Target は `Hero` とする。

初期 Location と Action の対応は次のとおりとする。ゲーム内の識別子と表示名は英語を使用する。

| Location | Actions |
| --- | --- |
| `First Shore` | `Gather`, `Fish` |
| `Nearby Woods` | `Gather`, `Hunt` |
| `Nearby Hill` | `Gather`, `Hunt` |

`First Shore` の `Gather` は、既存の序盤進行で Stone と Seaweed を入手する経路を維持する。`Fish` は海岸固有の食料獲得行動として分離する。

今回の提案では、Action の所要時間、報酬量、成功率、経験値、レベル上昇速度は定めない。

## Progress と完了ログ

Progress は次の4列へ整理する。

| Column | Width | Purpose |
| --- | ---: | --- |
| Target | 20% | Action を実行している人物または組織 |
| Location | 20% | Action を実行している場所または施設 |
| Action | 20% | 実行中の仕事 |
| Progress Bar | 40% | 進捗バーと進行率 |

未実装機能のために予約されていた `Times`、`Sub Action`、`Sub Progress Bar` は撤去する。

完了ログには Target、Location、Action を含め、同じ Action を異なる Location で実行した場合にも結果を識別できるようにする。具体的な英文は TUI の表示規約に合わせるが、たとえば `Hero completed Gather at Nearby Woods` の情報を欠かさない。

## データモデルへの要求

- Location の安定した識別子と英語ラベルを表すデータを追加する。
- Location ごとに実行可能な Action を定義する。
- ゲーム状態に発見・解放済み Location を保持する。
- 実行中タスクに Location の識別子を保持する。
- Target の実行中タスクを複数件のリストではなく最大1件として扱う。
- Action 割当時に Target の空き状態、Location の解放状態、Target と Action の互換性、Location と Action の互換性を検証する。
- 完了イベントに Location を含める。

Target、Location、Action はコンテンツデータとライブ状態を分離し、TUI 以外のフロントエンドから割り当てた場合にも同じ制約が適用される構造を維持する。

## 検証方針

core では次を検証する。

- 未発見または未解放 Location へ割り当てられない。
- Target または Location が対応しない Action を割り当てられない。
- 実行中 Target に二つ目のタスクを割り当てられない。
- 完了イベントが Target、Location、Action を保持し、完了後に Target が再び空き状態になる。

TUI の状態遷移では次を検証する。

- `Target → Location → Action` の順に列が生成される。
- Backspace が一段だけ戻り、Target 選択中には何もしない。
- `Esc` がすべての選択段階で終了として機能する。
- 実行中 Target の行が `--` となり、そのスロットの入力が無効になる。
- 実行中 Target より後ろの Target が元の選択キーを維持する。
- Action 割当後と Action 完了後の状態が仕様どおり遷移する。

Renderer では、3段階それぞれの代表表示、4列の Progress、Location を含む完了ログを、最低対応サイズ `80x24` を含む代表的な端末幅で検証する。

## 今回扱わないこと

- 実行中タスクの中断、置換、確認ダイアログ
- Action の自動反復
- 実行中 Target 表示の追加 QOL 改善
- 組織 Target の編成、構成員、内部並列処理
- Location または Action のページング
- 報酬、経験値、レベル、バランス値の具体化

これらは、3段階の選択モデルと最初期コンテンツを実装して操作感を確認した後に、必要に応じて別の議論で扱う。
