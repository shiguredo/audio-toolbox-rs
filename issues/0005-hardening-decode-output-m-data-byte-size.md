# デコード経路で `mDataByteSize` を検証し `pcm_buf.truncate(size)` のパニックと奇数バイトを防ぐ

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`Decoder::decode_impl` は `AudioConverterFillComplexBuffer` 成功後に `output_buffer_list.mBuffers[0].mDataByteSize` を `byte_size` とし、`size = byte_size / size_of::<i16>()` として `pcm_buf.truncate(size)` を呼ぶ。**`size` が事前に確保した `pcm_buf.len()` を超える**と `Vec::truncate` は **パニック**する（`new_len` が現在の長さより大きい場合）。

また **`byte_size` が奇数**のとき、i16 境界と整合しない出力として **データ不整合**が生じうる。フレームワーク異常値に対し **防御的に検証**する必要がある。

## 既存 issue との関係

- `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` は **理論上限（RFC・定数）**の妥当性調査が中心。**本 issue は「返却された `mDataByteSize` が実バッファと矛盾しないか」の実行時検証**にフォーカスする。

## 受け入れ条件の目安

- `size` が `pcm_buf` の要素数を超える場合は **エラー**（パニック禁止）。
- 奇数 `byte_size` の扱いを **エラー**または **明示仕様**のいずれかに決め、実装と rustdoc を一致させる。
- `0002` の調査結果と矛盾しない **定数・バッファサイズ**の選び方と整合すること。

## 参考（該当コード）

- `src/lib.rs`: `decode_impl`、`DECODE_BUF_FRAMES`、`OUTPUT_CHANNELS`
