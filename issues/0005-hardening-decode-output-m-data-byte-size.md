# デコード経路で `mDataByteSize` を検証し `pcm_buf` の長さと矛盾しない出力を返す

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`Decoder::decode_impl` は `AudioConverterFillComplexBuffer` 成功後に `output_buffer_list.mBuffers[0].mDataByteSize` を `byte_size` とし、`size = byte_size / size_of::<i16>()` として `pcm_buf.truncate(size)` を呼ぶ。

### 本 issue の主眼

**論点は `Vec::truncate` のパニックではない。** 主眼は **`mDataByteSize`（および導出した `size`）が、確保した出力バッファや i16 境界と矛盾する異常値でないかの検証**である。

### 調査結果（`Vec::truncate`）

[`Vec::truncate`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.truncate) は、**`len` が現在の `len()` 以上なら何もしない**（パニックしない）。**ローカル検証:** 長さ 4 の `Vec` に対し `truncate(100)` 後も `len() == 4`。

したがって **`size` が `pcm_buf.len()` を超える**と、`truncate` は **縮めない**ままとなり、**返却 `Vec` の `.len()` が「有効な PCM サンプル数」と一致しない**可能性がある（末尾に **初期化ゼロのままの余剰要素**などが残り、呼び出し側が「`len()` ＝有効サンプル数」と解釈すると **誤った長さ**になる）。

また **`byte_size` が奇数**のとき、i16 境界と整合しない出力として **データ不整合**が生じうる。フレームワーク異常値に対し **防御的に検証**する必要がある。

## 既存 issue との関係

- `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` は **理論上限（RFC・定数）**の妥当性調査が中心。**本 issue は「返却された `mDataByteSize` が実バッファと矛盾しないか」の実行時検証**にフォーカスする。
- **0002 で定数・バッファ方針が決まったあと**に本 issue の実装（閾値・エラー条件）を最終合わせすると、**設計と実行時検証が食い違わない**。

## 受け入れ条件の目安

- `size` が `pcm_buf` の要素数を超える場合は **`Result::Err`**（**無音の長さ不整合返却を避ける**）。`truncate` に頼らず **明示的に検証**する。
- 奇数 `byte_size` の扱いを **エラー**または **明示仕様**のいずれかに決め、実装と rustdoc を一致させる。
- `0002` の調査結果と矛盾しない **定数・バッファサイズ**の選び方と整合すること。

## 参考（該当コード）

- `src/lib.rs`: `decode_impl`、`DECODE_BUF_FRAMES`、`OUTPUT_CHANNELS`

## 参考（外部）

- `std::vec::Vec::truncate`: <https://doc.rust-lang.org/std/vec/struct.Vec.html#method.truncate>
