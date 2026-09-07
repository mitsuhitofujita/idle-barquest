# コンテキストに応じた Action 選択の提案

- Status: Proposed
- Date: 2026-08-29

## 目的

User Choices を、Target、Action の全候補を常時並べる表示ではなく、現在の選択に応じて次の選択肢が現れる段階的な操作にする。

選択した Target と実行可能な Action の関係を画面上で明確にし、表示されない組み合わせをゲームルール上も割り当てられないようにする。

## 選択フロー

最初は Target だけを表示する。

```text
|> Target:
| a) Hero
| b) Adventurer
| c) Farmer
```

プレイヤーが `b` で Adventurer を選ぶと、Target の右側に Action の選択肢を生成する。

```text
|  Target:       |> Action:
|    Hero        | a) Forest Exploration
|    Adventurer <|
|    Farmer      |
```

Action を選択すると、その Action を Target に割り当て、Target 選択へ戻る。

## 入力方式

選択肢には表示順に `a)`、`b)`、`c)` とASCII小文字を割り当てる。入力時は対応する大文字も同じ選択として扱う。

数字入力は選択に使用しない。文字は項目名の頭文字ではなく、現在表示されている選択肢内での位置を示す。

選択キーは現在入力を受け付ける列だけに表示する。したがって Action 選択中の Target 列には `a)` などを表示しない。

## Target と Action の対応

Target の定義は、その種類が実行可能な Action の識別子を保持する。

ある Target に表示する Action は、次の両方を満たすものに限定する。

1. ゲーム状態で解放済みである。
2. 選択した Target の種類が実行可能である。

この制約は TUI の表示だけでなく core の Action 割当処理でも検証する。未解放の Action、非対応の Action、存在しない Target または Action の割当は拒否する。

## 文字配置

discovery のサンプルを表示契約とし、空白を含む文字位置を維持する。

- 各行の先頭に `|` を表示する。
- 選択中の見出しは `> Target:` または `> Action:` とし、`>` の後に1文字の空白を置く。
- Target 名は Target 選択中と Action 選択中のどちらでも6文字目から開始する。
- Action 選択中、選択済み Target の `<` は Target–Action 区切りの直前へ配置する。
- たとえば Hero を選択した行は `|    Hero       <|` とする。

## 列の生成と幅

Target 選択中は Action 用の列幅や区切りを予約しない。Target が選択されたときだけ Target–Action の区切りと Action 列を生成する。

Action 選択時の Target 列幅は、見出しと Target 名のうち最も長い内容から動的に計算する。選択済みマーカー用の位置を区切り直前に確保する。

極端に長い Target 名によって Action が表示できなくなる場合は、Action 側に最低20文字分を確保し、Target 側を切り詰める。固定比率による列分割は行わない。

Times は関連機能が実装されるまで表示しない。将来追加する場合も、Action 選択後に次の選択段階として生成する。

## 検証方針

Target 選択前後の代表行を文字列として検証し、空白、Target 名の開始位置、左端と列間の `|`、選択済みマーカーの `<|` が discovery と一致することを保証する。

加えて、Action の表示と割当について、解放状態と Target の対応関係の積集合になっていること、数字入力が無視されること、文字入力が表示順の項目を選択することを検証する。

## 今回変更しないこと

Action 選択を取り消して Target 選択へ戻る専用操作は追加しない。Global Menu の終了操作と、Action 選択後に Target 選択へ戻る既存フローを維持する。
