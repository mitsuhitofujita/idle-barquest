# 拠点・素材表示の提案

- Status: Proposed
- Date: 2026-09-01

## 目的

最小端末サイズ `80x24` と画面全体の高さを維持しながら、現在の拠点と拾得済み素材を常時確認できるようにする。

拠点は将来のクラフトや技術開発へ接続できるゲーム状態として導入し、素材表示は個数やラベルの長さが変化しても安定して操作できる可変幅のビューポートとする。

## 画面構成

最小端末高24行では、上から次の順に配置する。

| 領域 | 高さ | 内容 |
| --- | ---: | --- |
| Title | 3行 | 現行の3行ASCIIアート |
| Information Log | 4行 | 先頭1行はTitle下の空白、ログ表示は3行 |
| 区切り線 | 1行 | Information Log / Settlement・User Choices |
| Settlement | 1行 | 現在の拠点 |
| User Choices | 6行 | 見出し1行と選択肢最大5件 |
| 区切り線 | 1行 | User Choices / Progress |
| Progress | 5行 | 実行中Actionを最大5件表示 |
| Materials | 1行 | 拾得済み素材の所持数 |
| 区切り線 | 1行 | Progress・Materials / Global Menu |
| Global Menu | 1行 | 常時表示コマンド |
| **合計** | **24行** | |

Settlement と User Choices の間、および Progress と Materials の間には区切り線を追加しない。既存の3本の区切り線を維持する。

端末が24行より高い場合、追加分は従来どおりすべて Information Log へ割り当てる。User Choices、Progress、Settlement、Materials の高さは固定する。

User Choices は見出しを含めて6行となるため、選択肢を最大5件表示する。Progress も最大5件とする。これらを超えるコンテンツを追加する前に、ページングまたは高さ計算を別途設計する。

## Settlement

Settlement は、Target が Action を行う場所である Location とは別の概念とする。プレイヤーの発展拠点を表し、将来はクラフトや技術開発の選択を現在の Settlement に結び付ける。

core に次の情報を持たせる。

- Settlement の安定した id。
- ASCII の英語表示ラベル。
- Catalog が保持する Settlement のコンテンツ定義。
- GameState が保持する現在の Settlement id。

初期 Settlement は次のとおりとする。

| Settlement id | Display name |
| --- | --- |
| `awakening_shore` | `Awakening Shore` |

Settlement 行は次の形式で表示する。

```text
 Settlement: Awakening Shore
```

今回は Settlement の切替、発見、解放、クラフト、技術開発は実装しない。現在値を表示し、後続機能が参照できるデータモデルの土台までを扱う。

## 表示対象となる素材

Materials 行には、Resource Catalog に存在し、一度でも拾得した素材だけを表示する。未拾得素材は0個であっても表示しない。

GameState の inventory に ResourceStack が存在することを拾得済みの印とする。将来素材の消費処理を追加した際も、数量が0になった ResourceStack は削除しない。これにより一度拾得した素材は0個でも表示され、拾得済み状態を別の集合へ重複して保持せずに済む。

表示順は Resource Catalog の登録順とする。inventory の獲得順には依存しない。

表示対象が1件もない場合、Materials 行は空行とする。`No materials` などの案内文や移動矢印は表示しない。

## 素材ビューポート

素材1件を次の形式で表示し、複数件を ` | ` で区切る。

```text
Pebble: 34 | Twig: 10
```

1画面の表示件数は固定しない。左右の矢印用の幅を除いた領域へ、現在の先頭素材から `Label: amount` をカタログ順に追加し、区切りを含めて次の素材が収まらなくなるまで表示する。

通常はラベルと個数を切り詰めず、収まる件数を減らす。1件だけでも領域に収まらない場合に限り、個数の `: amount` を残してラベル側を切り詰める。個数は素材表示の主要情報なので切り捨てない。

表示中の先頭素材は、数値のページ番号や配列位置ではなく ResourceId でTUIの表示状態に保持する。個数の桁数が変わって一度に収まる件数が増減しても、先頭素材は維持する。新しい素材を拾得して表示対象が増えた場合も、現在の先頭素材が存在する限り位置を維持する。

## 素材の移動

`,` は表示中の先頭素材をカタログ順で1件前へ、`.` は1件後ろへ移動する。ページ全体を送る操作にはしない。

`.` は、現在表示されている最後の素材より右側に未表示の素材がある場合だけ有効とする。残りの全素材が現在の領域に収まっている場合は何もしない。`,` は先頭素材より前に拾得済み素材がある場合だけ有効とする。

左側に戻れる場合だけ `<`、右側に未表示素材がある場合だけ `>` を表示する。

```text
< Pebble: 34 | Twig: 10 >
```

矢印を表示しない境界でも、その表示幅は空白として予約する。矢印の出現や消失によって素材本文の開始位置を変えない。

`,` と `.` は User Choices の選択段階に関係なく受け付けるグローバル操作とする。端で利用できない場合や素材がない場合は何もしない。日本語キーボード以外のキー配置は今回考慮しない。

## Global Menu

Global Menu には、素材移動、選択段階を戻る操作、終了操作を常時表示する。

```text
 ,) Previous Materials  .) Next Materials  BACKSPACE) Back  ESC) Quit
```

Backspace の既存動作は維持する。Action 選択から Location 選択へ、Location 選択から Target 選択へ一段戻り、Target 選択では何もしない。Global Menu の項目はその時点で利用できなくても隠さず、操作を何もしない入力として扱う。

## 検証方針

core では少なくとも次を検証する。

- 初期 Settlement が `awakening_shore` であり、Catalog から `Awakening Shore` を解決できる。
- 素材を初めて拾得すると ResourceStack が作られる。
- 数量が0になっても ResourceStack を保持するという、将来の消費処理が従うべき不変条件を維持する。

TUI では少なくとも次を検証する。

- `80x24` で提案どおりの行順と高さになり、24行を超える追加分は Information Log に入る。
- Settlement 行に `Settlement: Awakening Shore` が表示される。
- 未拾得時の Materials 行が空である。
- 拾得済み素材だけをカタログ順で表示し、0個のスタックも表示する。
- 端末幅、ラベル長、個数の桁数に応じて表示件数が変わる。
- 1件も収まらない場合はラベルだけを切り、個数を残す。
- `,` と `.` が先頭素材を1件ずつ移動し、利用不能時は何もしない。
- 個数の桁数が変わっても、保持している先頭ResourceIdが変わらない。
- 移動可能な側だけ矢印を表示し、非表示時も本文位置が変わらない。
- Global Menu の全項目が常時表示され、Backspace の既存状態遷移を維持する。

## 今回扱わないこと

- Settlement の発見、解放、切替UI
- Settlement ごとのクラフト、技術開発、レシピ、解放条件
- 素材の消費処理
- インベントリとSettlementのセーブ・ロード
- User Choices と Progress の5件を超える表示
- 日本語キーボード以外の`,`と`.`のキー配置
