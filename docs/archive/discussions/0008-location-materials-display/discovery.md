---
date: 2026-09-01
tags: [tui, layout, interaction, materials]
status: decided
worklogs: [0007-location-materials-display]
decisions: [0015-settlement-and-materials-display]
---

「拠点」と「素材」表示行を追加する。`User Choices`と`Progress`をそれぞれ一行ずつ削る。

画面の情報配置を見直し、「拠点」と「素材」を常時確認できるようにしたい。
既存の二つの表示を一行ずつ圧縮し、全体の高さを保ったまま差し替える。
素材は、`< Pebble: 34 | Twig: 10 >`と表示し、`,`と`.`キーで左右にページ切り替えができる。日本語キーボード以外は今は考えない。
