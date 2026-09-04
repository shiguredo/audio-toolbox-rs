# Decoder::decode_impl のエラーパスで encoded_buf が未クリアのため以降永久にブリックする不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-decoder-decode-impl-error-bricks-decoder
- Polished: 2026-09-04

## 目的

`Decoder::decode_impl` が `AudioConverterFillComplexBuffer` の実エラーを受け取ったとき、`encoded_buf` を消費済みとしてクリアせずに `Err` を返してしまい、以降その `Decoder` インスタンスが恒久的に使えなくなる不具合を修正する。1 回の API エラーで `Decoder` を作り直さないと復帰不能になる現状は堅牢性を損なっている。

## 優先度根拠

High とする。エラー経路とはいえ Apple 側の API が突発的に非 0 ステータスを返すケース (メモリ不足、コーデック内部エラー、SDK バグ、コールバック側 param_err のバブルアップ等) は本番稼働で起こり得る。回復手段が `Decoder` の drop 再生成に限られる現状は、長時間動作するプロセスでの信頼性を大きく下げる。過去に issues/0004〜0008 の hardening と issues/0010〜0012 のコールバック契約修正を進めてきた堅牢性向上の方針とも整合しない。

## 現状

`src/lib.rs` の `Decoder::decode_impl` は、`AudioConverterFillComplexBuffer` 呼び出しのステータス検査でエラーが返ると `encoded_buf.clear()` に到達せずに `Err` を返す。

```rust
let status = unsafe {
    sys::AudioConverterFillComplexBuffer(...)
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

`AudioConverterFillComplexBuffer` が `0` / `K_NO_MORE_INPUT` 以外のエラーを返した瞬間に return してしまうため、`self.encoded_buf.clear()` に到達しない。

以降ユーザーが `Decoder::decode(new_packet)` を呼ぶと、`src/lib.rs` の `Decoder::decode` 冒頭のガード分岐で永久に弾かれる。

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

Encoder 側も `Encoder::encode_impl` はエラー時に `pcm_buf` をクリアしないが、`Encoder` はブリックには至らない。エラー後の状態一貫性の問題として issues/0016 で別途対応する。

## 完了条件

- `Decoder::decode_impl` が `AudioConverterFillComplexBuffer` の実エラーを受け取っても、次に `Decoder::decode(new_packet)` を呼んだときに `previous packet not consumed` エラーで弾かれずに受理できる。
- 壊したパケットが `AudioConverterFillComplexBuffer` の実エラーを確実に誘発すること、およびエラー後に正常パケットでデコードへ復帰することを macOS 実機で確認してからテスト内容を確定し (エラーが誘発しない・復帰しない場合はテスト要件や設計を再検討する)、「壊れたパケットを渡して `next_frame()` が Err → 別の正常なパケットを `decode()` して成功する」ケースを `tests/test_decoder.rs` に追加してパスさせる。
- 既存の正常系テスト (`decode_multiple_aac_packets_no_duplicate_feeds` 等) が引き続きパスする。
- `Decoder::decode` / `Decoder::next_frame` の rustdoc に「エラー時には当該パケットが消費されたか否かに関わらず入力バッファがクリアされ (消費状況は不明)、以降は新しいパケットの `decode` から再開できる」旨が明記される。
- `skills/shiguredo-audio-toolbox/SKILL.md` の `Decoder::decode` の説明 (1 パケット制約) にエラー時は入力パケットが破棄され再開できる旨が追記され、`Decoder::next_frame` フロー記述がエラー時にも `encoded_buf` がクリアされる順序に合わせて更新される (issue 参照の除去は行わない。issues/0022 の管轄)。
- `CHANGES.md` の develop に [FIX] として追記する。

## 解決方法

`src/lib.rs` の `Decoder::decode_impl` 内で、ステータス検査と `encoded_buf.clear()` の順序を入れ替える。ステータス検査の上にある「0 または K_NO_MORE_INPUT 以外のステータスだけをエラーとする」旨の既存コメントは保持し、clear の上にある既存コメント (「デコード処理が完了したら入力バッファをクリアする」とその下の「AudioConverter は 1 回のコールバックで入力バッファの全データを消費するため、部分的に消費されることはない。」を含む) は下記の新コメントに置き換える。

```rust
let status = unsafe {
    sys::AudioConverterFillComplexBuffer(...)
};

// デコード処理を試行した以上、入力バッファは消費済みとして扱う
//
// API はエラー時に入力パケットが消費されたか否かを通知しない。
// 未クリアのままだと以降の decode が previous packet not consumed で
// 永久に弾かれるため、消費済みとみなしてクリアするのが本実装の方針である。
self.encoded_buf.clear();

if status != 0 && status != K_NO_MORE_INPUT {
    return Err(Error {
        status,
        function: "AudioConverterFillComplexBuffer",
    });
}
```

回帰テストとして `tests/test_decoder.rs` に以下を追加する:

- 正常な AAC-LC パケットのデータを壊した非空のバイト列 (例: ペイロード部分を 0xFF で上書き、末尾を切り詰める等。テストヘルパー `encode_aac_packets` の出力が 2 パケット以上になることをテスト内で検証し、1 つは正常パケット、もう 1 つは壊したパケットに使う) を `decode()` に渡す
- `next_frame()` が Err を返すことを確認する
- その直後に正常な AAC-LC パケットを `decode()` して成功することを確認する
- さらに `next_frame()` を `Ok(None)` になるまでループし、PCM が取れることを確認する
