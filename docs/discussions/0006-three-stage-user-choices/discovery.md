---
date: 2026-08-30
tags: [tui, interaction, gameplay, leveling]
status: worked
worklogs: [0005-three-stage-user-choices]
decisions: []
---

User Choicesを変更して。
3段階にする。
1. Targetは人物または組織(チーム、パーティ)
2. Locationを新設、`近くの林`や`火事場`など
3. Actionは`採取`、`狩り`など何をするか

今後最初期のレベリングを実装するので選択肢をそれっぽくしたい。
指示は日本語でけど、このゲームは全て英語を使用して。

Target:
- Hero

Location:
- 近場の丘
- 近場の林
- 近場の沿岸

Action:
- 近場の丘
  - 採取
  - 狩り
- 近場の林
  - 採取
  - 狩り
- 近場の沿岸
  - 釣り
