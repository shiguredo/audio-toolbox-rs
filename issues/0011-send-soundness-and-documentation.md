# `unsafe impl Send` の soundness 検証とドキュメント化

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

### `Send` はドキュメントだけでは unsound を解消しない

`unsafe impl Send for Encoder/Decoder` が **sound** であるには、**`AudioConverterRef` を別スレッドへ所有権移動したあと、単一スレッドからのみ操作する**ことが、Apple API の契約上許される必要がある（**スレッド親和性**や**特定スレッド専用**といった制約がある場合、`Send` は誤りになりうる）。

**rustdoc で「誤用禁止」と書くだけでは `unsafe impl Send` そのものの正当性は証明されない。** 本 issue の完了条件には **根拠調査・soundness の判断**を含め、ドキュメント化は **そのあと**の利用者向け説明として位置づける。

### 調査結果（Apple 公開ドキュメント・一次）

[`AudioConverterNew`](https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)) のページには、**オブジェクトのスレッド安全性**や **スレッド間での共有可否**についての明記は **見当たらない**（2026-04-01 時点の取得内容）。**「Send 相当が安全」**と断定するには、別資料（Technical Note、他 API のスレッド規約、Apple への確認、または実測方針）が必要である。

### 既存コードとの関係

`src/lib.rs` には **`Sync` を実装しない理由**や **AudioConverter がスレッドセーフでない**旨、`Encoder` / `Decoder` の **コールバックが同一スレッドで同期的**である旨の **コメント**がある。本 issue は **それを rustdoc に昇格するだけ**に終わらせず、**`Send` の根拠**を先に整理する。

### コールバックとポインタ寿命（ドキュメント化の対象）

コールバックで **`encoded_buf.as_mut_ptr()`** を渡している。**コールバック終了後に Apple がポインタを保持しない**ことは、安全性の重要な前提である。soundness の判断**後**、公式で裏付けられる範囲で **「同期・同一スレッド・非保持」**を利用者向けに rustdoc に書く。

## 受け入れ条件の目安

1. **`unsafe impl Send` の扱い（いずれかを満たす）**
   - **A:** `AudioConverterRef` を **別スレッドへ移動したうえで単一スレッドからのみ操作する**ことが許容される根拠を、**Apple 公式ドキュメント・Technical Note、または検証可能な実測手順**に基づき issue または設計メモに記載し、**`Send` を維持してよい**と結論づける。
   - **B:** 上記根拠が **取れない**、または **スレッド親和性等により `Send` が不適切**と結論した場合は、**`unsafe impl Send` を削除する**（`!Send` とする）など、**API 破壊を伴う設計変更**を issue のスコープに含め、実装する。

2. **ドキュメント**（1 で `Send` を維持する場合、または代替設計に合わせて）
   `Encoder` / `Decoder` またはクレートルートの rustdoc に、**スレッド境界**と **コールバック内ポインタの寿命**を記載する。既存コメントと**重複しないよう整理**する。

3. 参照可能なら **Apple Developer Documentation** の URL を rustdoc に含める（リンク切れに注意）。

## 参考（該当コード）

- `src/lib.rs`: `unsafe impl Send`、`Decoder::callback`（`encoded_buf` の `mData`）、既存の `Sync` / スレッド関連コメント

## 参考（外部）

- `AudioConverterNew`: <https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)>
