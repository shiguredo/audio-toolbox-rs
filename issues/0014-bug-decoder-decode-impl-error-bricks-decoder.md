# Decoder::decode_impl のエラーパスで encoded_buf が未クリアのため以降永久にブリックする不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-decoder-decode-impl-error-bricks-decoder
- Polished:

## 目的

`Decoder::decode_impl` が `AudioConverterFillComplexBuffer` の実エラーを受け取ったとき、`encoded_buf` を消費済みとしてクリアせずに `Err` を返してしまい、以降その `Decoder` インスタンスが恒久的に使えなくなる不具合を修正する。1 回の非致命的な API エラーで `Decoder` を作り直さないと復帰不能になる現状は堅牢性を損なっている。

## 優先度根拠

High とする。エラー経路とはいえ Apple 側の API が突発的に非 0 ステータスを返すケース (メモリ不足、コーデック内部エラー、SDK バグ、コールバック側 param_err のバブルアップ等) は本番稼働で起こり得る。回復手段が `Decoder` の drop 再生成に限られる現状は、長時間動作するプロセスでの信頼性を大きく下げる。過去に issues/0004〜0012 で FFI hardening を進めてきた方針とも整合しない。

## 現状

`src/lib.rs:881-905` の `Decoder::decode_impl` は以下の順に実行される。

```rust
let status = unsafe {
    sys::AudioConverterFillComplexBuffer(
        self.converter,
        Some(Self::callback),
        (self as *mut Self).cast(),
        &mut io_packets,
        &mut output_buffer_list,
        std::ptr::null_mut(),
    )
};
if status != 0 && status != K_NO_MORE_INPUT {
    return Err(Error {
        status,
        function: "AudioConverterFillComplexBuffer",
    });
}

// デコード処理が完了したら入力バッファをクリアする
self.encoded_buf.clear();
```

`AudioConverterFillComplexBuffer` が `0` / `K_NO_MORE_INPUT` 以外のエラーを返した瞬間に return してしまうため、`self.encoded_buf.clear()` (line 905) に到達しない。

以降ユーザーが `Decoder::decode(new_packet)` を呼ぶと、`src/lib.rs:832-838` の以下の分岐で永久に弾かれる。

```rust
pub fn decode(&mut self, encoded: &[u8]) -> Result<(), Error> {
    if !self.encoded_buf.is_empty() {
        return Err(Error {
            status: -50, // paramErr
            function: "Decoder::decode(previous packet not consumed)",
        });
    }
    ...
}
```

対称的な `Encoder::encode_impl` (`src/lib.rs:438-481`) はエラー時の `pcm_buf` 状態が異なる問題を抱えるが、`Decoder` はブリックまで至る点で挙動が非対称。この差異が仕様として意図された形跡もない。

## 完了条件

- `Decoder::decode_impl` が `AudioConverterFillComplexBuffer` の実エラーを受け取っても、次に `Decoder::decode(new_packet)` を呼んだときに `previous packet not consumed` エラーで弾かれずに受理できる。
- 回帰テストとして「壊れたパケットを渡して `next_frame()` が Err → 別の正常なパケットを `decode()` して成功する」ケースが `tests/test_decoder.rs` に追加され、パスする。
- 既存の正常系テスト (`decode_multiple_aac_packets_no_duplicate_feeds` 等) が引き続きパスする。

## 解決方法

`src/lib.rs:881-905` のステータス検査と `encoded_buf.clear()` の順序を入れ替える。

```rust
let status = unsafe {
    sys::AudioConverterFillComplexBuffer(...)
};

// デコード処理を試行した以上、入力バッファは消費済みとして扱う
// (Apple 側 API が実エラーを返した場合でも、パケットを再提供しても回復しないため)
self.encoded_buf.clear();

if status != 0 && status != K_NO_MORE_INPUT {
    return Err(Error {
        status,
        function: "AudioConverterFillComplexBuffer",
    });
}
```

コメントで「clear は status 検査より前」の理由を明記しておく。

回帰テストとして `tests/test_decoder.rs` に以下を追加する:

- 意図的に壊れたバイト列を `decode()` に渡す
- `next_frame()` が Err を返すことを確認する
- その直後に正常な AAC-LC パケットを `decode()` して成功することを確認する
- さらに `next_frame()` で PCM が取れることを確認する

なお、Apple の AudioConverter API 契約上、コールバック中の `pcm_buf` drain (Encoder 側) や `packet_provided_in_this_fill` フラグ (Decoder 側) は既に副作用として発生しているため、「encoded_buf を消費済みとして扱う」判断は API の実挙動と整合する。
