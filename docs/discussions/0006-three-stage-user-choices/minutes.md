# 3段階 User Choices の議事録

- 日付: 2026-08-30
- 結果: 提案を作成する

## 議論した内容

User Choices を、現在の `Target → Action` から `Target → Location → Action` の3段階へ変更する。
未到達の列は常時表示せず、選択に応じて右側へ次の列を生成する。これは先行するコンテキスト選択の方針を維持し、Location を新しい中間段階として加えるものである。

Target は人物または組織を表す。Team や Party のような組織も分割せず、一つの Target として扱う。従来 Target に含まれていた Farm、Livestock、Campfire などの施設は Location に移し、施設だけで自律稼働させず、必ず Target と Action を組み合わせて使用する。

実行中タスクは `Target + Location + Action` の組で識別する。各 Target が同時に保持できるタスクは1件だけとし、人物でも組織でも複数 Location や複数 Action へ分散させない。実行中 Target のタスクを中断または置換する操作は設けない。

Location は発見または解放済みのものだけを表示する。Action は、ゲーム状態で解放済みであり、選択した Target が実行でき、かつ選択した Location で実行できるものに限定する。この制約は表示だけでなく、core の割当処理でも検証する。

実行中 Target は一覧から消さず、選択キーの代わりに `--` を表示する。選択キーは現在選択可能な項目を詰め直した連番ではなく、Target の一覧上の固定スロットに対応させる。たとえば Hero が実行中なら `-- Hero` と表示し、その次の Adventurer は `a)` へ繰り上げず `b) Adventurer` のままとする。この状態で `a` は無効入力になる。

Location または Action の選択を取り消すため、Backspace で一段前へ戻る。Action 選択中は Location 選択へ、Location 選択中は Target 選択へ戻る。Target 選択中の Backspace は何もしない。`Esc` は従来どおりゲーム終了に使用する。

初期 Location は既存の序盤進行に合わせて `First Shore`、`Nearby Woods`、`Nearby Hill` とする。`First Shore` には `Gather` と `Fish`、`Nearby Woods` と `Nearby Hill` にはそれぞれ `Gather` と `Hunt` を用意する。海岸での `Gather` を残すことで、序盤技術に必要な Stone と Seaweed の入手経路を維持する。

Progress は Location を含む `Target | Location | Action | Progress Bar` の4列とし、幅をそれぞれ20%、20%、20%、40%とする。完了ログにも Location を含める。未実装機能のために予約されていた `Times`、`Sub Action`、`Sub Progress Bar` は撤去する。

Action 完了時はタスクを削除し、Target を再び選択可能にする。Action の自動反復は導入せず、再実行には再度3段階を選択する。

## 確認した質問

1. 3段階の列を常時表示するか、選択に応じて生成するか。
2. Target を人物または組織に限定した場合、従来の施設 Target をどこへ移すか。
3. `First Shore` の Action を `Fish` だけにすると序盤の Stone と Seaweed の入手経路が失われるが、`Gather` も用意するか。
4. Action の候補を、解放状態、Target の適性、Location の適性の積集合にするか。
5. 一つの Target が複数 Location または複数 Action で同時に活動できるか。
6. 実行中タスクと完了ログに Location を含めるか。
7. 施設が Target なしで自律稼働するか。
8. Location や Action の選択から一段戻る操作を追加するか。
9. 実行中 Target を再選択してタスクを中断または置換できるか。
10. Progress の列構成をどう変更するか。
11. 実行中 Target の表示と、後続 Target の選択キーをどう扱うか。
12. Action 完了後に自動反復するか。

## 採用しなかった案

- Target、Location、Action の全列を常時表示する案は採用しなかった。先行議論で、未到達の列を予約すると選択に応じて次の候補が生まれる関係が弱くなるとして退けられており、今回も段階的な列生成を維持する。
- Farm や Livestock などの施設を Target に残す案は採用しなかった。Target は人物または組織に限定し、施設は作業場所である Location として扱う。
- 施設が Target なしで自律的に Action を実行する案は採用しなかった。施設を使用する場合も、担当する Target を必要とする。
- 海岸の Action を `Fish` だけにする案は採用しなかった。既存の序盤進行で必要な Stone と Seaweed を得られなくなるため、`First Shore` に `Gather` も用意する。
- 一つの Target が複数タスクを同時実行する現行モデルは維持しなかった。組織を含め、Target は分散せず一つの Location で一つの Action だけを実行する。
- 実行中 Target を再選択し、現在のタスクを中断または置換する案は採用しなかった。完了するまで選択不可とする。
- 実行中 Target を一覧から除外する案と、残った選択可能 Target のキーを `a)` から詰め直す案は採用しなかった。`-- Hero` のようにスロットを残し、後続 Target のキーも固定する。
- `Esc` を一段戻る操作へ変更する案は採用しなかった。終了操作との役割を混ぜず、Backspace を一段戻る操作にする。
- `Times`、`Sub Action`、`Sub Progress Bar` の予約列を維持する案は採用しなかった。Location と現在の進捗を80列内で明確に表示するため、4列へ整理する。
- Action の自動反復は採用しなかった。完了後は Target を選択可能へ戻し、再実行には明示的な再選択を必要とする。

## 保留事項

- 実行中 Target の表示に関する追加の QOL 改善は、実際の操作感を確認した後で別途検討する。
- Team や Party など組織 Target の内部構造、編成、能力値は今回実装せず、将来のゲームプレイ設計に委ねる。
- Action の所要時間、報酬、レベリング速度などのバランスは今回の選択モデルでは決めない。
