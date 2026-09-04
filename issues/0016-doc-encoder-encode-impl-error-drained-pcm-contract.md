# Encoder::encode_impl のエラー時に drain 済み PCM の行方が観測不能になる契約を明文化する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/update-encoder-encode-impl-error-drained-pcm-contract
- Polished: 2026-09-04

## 目的

`Encoder::encode_impl` が非 `K_NO_MORE_INPUT` エラーで抜けたとき、コールバックが先に `pcm_buf` を `drain` している都合上、消費された PCM サンプルの行方が呼び出し側から観測不能になる。この挙動は実装では解消できない (AudioConverter の内部挙動であり、エラー時に入力が消費されたか否かは API から通知されない) ため、エラー後に `Encoder` を再利用すると音声が欠落している可能性があり、欠落の有無を知る手段がないことを契約として明文化し、呼び出し側がエラー後に再利用しない選択を確実にできるようにする。

## 優先度根拠

High とする。ブリック (issue 0014) までは至らないが、Apple 側 API が突発的にエラーを返したときに「何フレーム消費されたか呼び出し側が知る手段がない」状態になる。音声・映像の同期を PCM 側で行っている呼び出し側 (Hisui 等) では A/V ズレの原因となり得、原因特定も困難。過去に issues/0004〜0008 の hardening と issues/0010〜0012 のコールバック契約修正を進めてきた堅牢性向上の方針とも整合しない。

## 現状

`src/lib.rs` の `Encoder::encode_impl` は以下の順に実行される。

```rust
let status = unsafe {
    sys::AudioConverterFillComplexBuffer(...)
};
if status == K_NO_MORE_INPUT {
    return Ok(None);
}
Error::check(status, "AudioConverterFillComplexBuffer")?;

let size = output_buffer_list.mBuffers[0].mDataByteSize as usize;
if size == 0 {
    return Ok(None);
}
if size > ENCODE_BUF_SIZE {
    return Err(Error {
        status: sys::kAudio_ParamError,
        function: "Encoder::encode_impl(mDataByteSize)",
    });
}
```

一方、コールバック側 (`src/lib.rs` の `Encoder::callback`) は、AudioConverter にデータを提供した場合に `pcm_buf.drain(0..drain_end)` を実行する。

```rust
this.pcm_buf.drain(0..drain_end);
```

このため、`AudioConverterFillComplexBuffer` が非 0 ステータスを返した場合や `size > ENCODE_BUF_SIZE` で `Err` を返す場合、コールバックが既に drain を実行している可能性があり、その場合の drain 済み PCM の行方は呼び出し側から観測不能になる (コンバーター内部で保持されるのか破棄されるのかは Apple ドキュメントに明記されておらず、断定できない)。対応する `EncodedFrame` は生成されず、呼び出し側は Err だけを受け取り、内部的にどれだけ消費されたかを観測できない。

なお、`size == 0` で `Ok(None)` を返す経路と `K_NO_MORE_INPUT` 経路は正常経路であり、本契約 (Err 時のみ) の対象外とする。

`encode_impl` には Decoder の `decode` のようなガード分岐がなく、Err 後に `encode` 自体は再実行できる。ただしエラー後の AudioConverter の内部状態は未検証であり、再利用した場合の挙動は保証されない。

Decoder 側 (issue 0014) と同じ「エラー後の状態一貫性」の問題だが、0014 は「エラー後に次のパケットを受理できる」ように修正される方針である。Encoder 側は入力 PCM の消費量が観測不能なため、復帰可能にしても音声欠落のまま継続することになる。

## 設計方針

契約の明文化 (案 A) に確定する。実装は変更しない。

- `Encoder::encode` / `Encoder::finish` の rustdoc と `skills/shiguredo-audio-toolbox/SKILL.md` に「`Err` を返した場合、これまでの入力の一部がコールバック内で消費されている可能性があり、消費サンプル数は呼び出し側から観測できない。以降この `Encoder` インスタンスを利用しないこと」旨を明記する。
- 実装の変更 (案 B: poisoned フラグによる永久 Err 化) は採用しない。理由は以下のとおり:
  - `Encoder::next_frame` は `Result` ではなく `Option<EncodedFrame>` を返すため、`next_frame` ではエラーを通知できない。全メソッドでエラーを返す完全な永久 Err 化にはシグネチャの破壊的変更が必要になる
  - `encode_impl` の Err を公開 API から誘発する手段がなく、回帰テストを書けない
  - 0014 が「エラー後復帰可能に修正」した方向と逆方向の設計になる

## 完了条件

- `Encoder::encode` / `Encoder::finish` の rustdoc と `skills/shiguredo-audio-toolbox/SKILL.md` に「`Err` を返した場合、これまでの入力の一部がコールバック内で消費されている可能性があり、消費サンプル数は呼び出し側から観測できない。以降この `Encoder` インスタンスを利用しないこと」旨が明記される。
- 明記された内容が実装の挙動と一致していることをレビューで確認できる。
- `CHANGES.md` の develop / `### misc` に [UPDATE] として追記する (rustdoc の変更分のみ。SKILL.md の変更分は `.md` のため変更履歴に反映しない)。

## 解決方法

案 A として以下を書き加える。契約文言は完了条件に記載の文言をそのまま使用する。

- `Encoder::encode` の rustdoc:
  - 「`Err` を返した場合、これまでの入力の一部がコールバック内で消費されている可能性があり、消費サンプル数は呼び出し側から観測できない。以降この `Encoder` インスタンスを利用しないこと」
- `Encoder::finish` の rustdoc:
  - 同上
- `skills/shiguredo-audio-toolbox/SKILL.md`:
  - 「利用上の注意」セクションに新節「エラー時の `Encoder` の再利用禁止」を設け、同上の 1 行を追加する
