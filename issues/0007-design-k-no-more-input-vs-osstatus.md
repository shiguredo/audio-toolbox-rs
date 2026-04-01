# `K_NO_MORE_INPUT`（12345）と実 `OSStatus` の衝突リスクを解消する

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

入力不足を表すために **`K_NO_MORE_INPUT = 12345`** を `AudioConverterFillComplexBuffer` のコールバックから返し、`encode_impl` / `decode_impl` では **`AudioConverterFillComplexBuffer` の戻り値 `status`** に対して **`status == K_NO_MORE_INPUT`** と分岐している。この値が **将来・環境により実在の `OSStatus` と一致**すると、成功・失敗の解釈が崩れる。

**補足:** コールバックが返す `i32` は、**外側の `AudioConverterFillComplexBuffer` の戻り値として伝播**する想定で実装されている。調査・設計変更時は **「コールバック戻り値 = 外側に見える `status`」**の関係を前提に、衝突回避の値選定または二段判定を行う。

コードコメントでも「フレームワーク側と衝突しない値」の根拠が文献で固定されていない旨がある。**設計として衝突不能に近づける**か、**ステータスとユーザー定義の不足シグナルを分離**する必要がある。

## 受け入れ条件の目安

- **コールバックの戻り値と `AudioConverterFillComplexBuffer` の戻り値 `status` の関係**を、Apple 公式ドキュメントまたは実測で確認し、**引用または要約を issue・設計メモに残す**（「想定」で終わらせない）。
- Apple 公式または実測に基づき、**予約済みでない**ことが説明できる値にする、または **`OSStatus` 以外の経路**（例: 内部フラグと組み合わせた二段判定）で「入力不足」を表す。
- 変更後も **入力不足時の制御フロー**（`Ok(None)` 等）が現状と同等に保たれること。
- rustdoc または設計メモに **選定理由**を残す。

## 参考（該当コード）

- `src/lib.rs`: `K_NO_MORE_INPUT`、`encode_impl`、`decode_impl` のステータス分岐
