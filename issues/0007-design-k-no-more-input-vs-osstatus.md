# `K_NO_MORE_INPUT`（12345）と実 `OSStatus` の衝突リスクを解消する

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

入力不足を表すために **`K_NO_MORE_INPUT = 12345`** を `AudioConverterFillComplexBuffer` の戻り値として使い、`encode_impl` / `decode_impl` で **`status == K_NO_MORE_INPUT`** と分岐している。この値が **将来・環境により実在の `OSStatus` と一致**すると、成功・失敗の解釈が崩れる。

コードコメントでも「フレームワーク側と衝突しない値」の根拠が文献で固定されていない旨がある。**設計として衝突不能に近づける**か、**ステータスとユーザー定義の不足シグナルを分離**する必要がある。

## 受け入れ条件の目安

- Apple 公式または実測に基づき、**予約済みでない**ことが説明できる値にする、または **`OSStatus` 以外の経路**（例: 内部フラグと組み合わせた二段判定）で「入力不足」を表す。
- 変更後も **入力不足時の制御フロー**（`Ok(None)` 等）が現状と同等に保たれること。
- rustdoc または設計メモに **選定理由**を残す。

## 参考（該当コード）

- `src/lib.rs`: `K_NO_MORE_INPUT`、`encode_impl`、`decode_impl` のステータス分岐
