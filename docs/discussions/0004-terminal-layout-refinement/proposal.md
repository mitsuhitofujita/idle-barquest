# ターミナルレイアウト見直しの提案

- Status: Proposed
- Date: 2026-08-29

## 目的

タイトルとゲーム情報の間に視覚的な余白を設けつつ、情報ログ、選択肢、実行中プログレスの順に上から確認できるよう、画面の垂直レイアウトを再構成する。

従来の最小対応端末 `80x24` は維持し、その範囲内で各領域の最小表示量を明確にする。

## 領域の順序

画面を上から次の順序で配置する。

1. Title
2. Information Log
3. User Choices
4. Progress
5. Global Menu

Title と Information Log の間に全幅の区切り線は追加しない。Information Log の先頭1行を常に空白とし、Title 直下の余白に使う。

Information Log と User Choices、User Choices と Progress、Progress と Global Menu の間には、それぞれ従来どおり1行の全幅区切り線を配置する。

## 垂直方向の配分

最小端末高で24行の配分は次のとおりとする。

| 領域 | 高さ | 内容 |
| --- | ---: | --- |
| Title | 3行 | 現行の3行ASCIIアート |
| Information Log | 4行 | 先頭1行は空白、ログ表示は3行 |
| 区切り線 | 1行 | Information Log / User Choices |
| User Choices | 7行 | 見出し1行と選択肢6件 |
| 区切り線 | 1行 | User Choices / Progress |
| Progress | 6行 | 実行中 Action を最大6件表示 |
| 区切り線 | 1行 | Progress / Global Menu |
| Global Menu | 1行 | 常時表示コマンド |
| **合計** | **24行** | |

User Choices と Progress は当面、それぞれ7行と6行の固定高とする。端末が24行より高い場合、追加分はすべて Information Log へ割り当てる。Title 直下の空白は、Information Log が高くなっても1行のまま維持する。

## Information Log

Information Log の最小高は4行とする。先頭1行にはログを描画せず、Title とログの間の余白とする。残りの領域にログを下揃えで描画する。

新しいログは下端に追加し、表示可能な行数を超えた古いログは上側から表示範囲外へ送る。Action 完了など、現在 Information Log が担っている通知の役割は維持する。

## User Choices

User Choices は7行固定とする。1行目に現行の列見出しを表示し、2行目から7行目までに最大6件の選択肢を表示する。

現行コンテンツはこの上限内に収まる。6件を超える Target または Action を表示する必要が生じる前に、領域高の計算またはページングを別途設計する。

## Progress

Progress は6行固定とし、実行中 Action を1行に1件、最大6件表示する。

現行コンテンツでは同時に表示される Progress は最大3件のため、この固定高で欠落は生じない。同時実行数が6件を超えるコンテンツを追加する前に、端末の高さと表示件数に応じた領域配分を別途設計する。

## 最小端末サイズ

最小対応端末サイズは `80x24` を維持する。幅80列または高24行を下回る端末では、従来どおり警告を表示し、通常のゲームUIは描画しない。

## 将来の拡張

今回は領域配分の微調整に限定し、User Choices と Progress の高さは固定とする。選択肢や同時実行 Action が増えた段階で、端末高、必要表示行数、Information Log の最小高を考慮した高度な高さ計算を追加する。
