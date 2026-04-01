# AudioConverter コールバックで null ポインタと無効な `mData` を検査する

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

エンコーダー・デコーダーの `AudioConverterFillComplexBuffer` コールバックは、`io_data`、`io_number_data_packets`、`in_user_data` を **検査なしに逆参照**している。いずれかが **null** の場合、Rust の **未定義動作**となる。

また `from_raw_parts_mut(io_data.mBuffers[0].mData, num_samples)` は **`mData` が null** かつ **`num_samples > 0`** のとき **UB** になる。フレームワークが常に有効なポインタを渡す前提は、**防御的プログラミング**の観点では明示的チェックに置き換えたい。

## 早期 return と `io_data`（デコーダー）

デコーダーでは **`encoded_buf` が空かつ `!eos`** のとき **`K_NO_MORE_INPUT` を返して `io_data` を一切書かない**経路がある。`AudioConverter` がこの戻り値で **`io_data` を読まない**ことが仕様上保証されているかは、**Apple ドキュメントと照合**した上で、必要なら **ゼロクリアや `mNumberBuffers` の設定**などを検討する（実装時に判断）。

## 受け入れ条件の目安

- `in_user_data == null` のとき **エラーコードを返し**、`Encoder` / `Decoder` への参照を作らない。
- `io_number_data_packets` / `io_data` が null のとき同様（返却する `OSStatus` は Apple 慣習に合わせて調査の上決定）。
- `num_samples > 0` かつ `mData == null` のとき **`from_raw_parts_mut` を呼ばない**。
- ゼロ長スライス時の **`mData` の要件**は Rust のポインタ規則に合わせ、必要なら **ダミーアラインポインタ**等を検討する（実装時に公式ドキュメントと照合）。

## 参考（該当コード）

- `src/lib.rs`: `Encoder::callback`、`Decoder::callback`
