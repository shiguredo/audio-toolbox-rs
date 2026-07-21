# Encoder::encode_impl のエラー時にコールバックで drain 済みの PCM がサイレントに消える不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-encoder-encode-impl-silent-drain-on-error
- Polished:

## 目的

`Encoder::encode_impl` が非 `K_NO_MORE_INPUT` エラーで抜けたとき、コールバックが先に `pcm_buf` を `drain` している都合上、消費された PCM サンプルが呼び出し側から観測できない形で失われる不具合を修正する (もしくは契約として明文化して呼び出し側が対処できるようにする)。

## 優先度根拠

High とする。ブリック (issue 0014) までは至らないが、Apple 側 API が突発的にエラーを返したときに「何フレーム消費されたか呼び出し側が知る手段がない」状態になる。音声・映像の同期を PCM 側で行っている呼び出し側 (Hisui 等) では A/V ズレの原因となり、原因特定も困難。過去に issues/0007 (整数オーバーフロー) を修正した文脈と同じく、エラーパスの状態一貫性は堅牢化の対象となる。

## 現状

`src/lib.rs:438-481` の `Encoder::encode_impl` は以下の順に実行される。

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

一方、コールバック側 (`src/lib.rs:559-586`) は **AudioConverter 呼び出し中に無条件で `pcm_buf.drain(0..drain_end)` を実行する**:

```rust
this.pcm_buf.drain(0..drain_end);
```

このため、`AudioConverterFillComplexBuffer` が非 0 ステータスを返した場合や `size > ENCODE_BUF_SIZE` で `Err` を返す場合、drain 済みの PCM は破棄されており、対応する `EncodedFrame` は生成されない。呼び出し側は Err だけを受け取り、内部的にどれだけ消費されたかを観測できない。

Decoder 側 (issue 0014) と同じ「エラー後の状態一貫性」の問題だが、Encoder は復帰不能 (ブリック) までは至らない。

## 設計方針

以下のいずれかで対応する。

### 案 A: rustdoc とスキルで契約を明示

- `Encoder::encode` / `Encoder::finish` の rustdoc に「`Err` を返した場合、直前の呼び出しで一部の PCM が消費済みで復元不可、以降 `Encoder` は再利用してはならない」旨を明記する。
- 実装は変更しない。呼び出し側にエラー後の破棄を強制する。

### 案 B: 実装で永久 Err 化

- Err を返したら内部フラグを立て、以降の `encode` / `finish` / `next_frame` は同じ Err (もしくは専用のエラー) を返し続けるようにする。
- `pcm_buf.clear()`, `encoded_frames.clear()` も同時に行う。
- 呼び出し側は Err を検知したら Encoder を drop するしかなくなり、状態の曖昧さが消える。

案 B のほうが Decoder ブリック (issue 0014) との対称性が良い一方、既存 API 契約に対する変更幅が大きい。案 A のみで完了とするか、案 B に進むかは issue 内で判断する。

## 完了条件

- 案 A: `Encoder::encode` / `Encoder::finish` の rustdoc と `skills/shiguredo-audio-toolbox/SKILL.md` に「Err 後は再利用禁止」の契約が明記される。
- 案 B: 実装が Err 後の永久 Err 化を実装し、対応する回帰テストが追加される。
- どちらの案でも、エラー後の状態の曖昧さが解消されたことをレビューで確認できる。

## 解決方法

まず案 A の rustdoc 更新から着手する。案 B に進むかは Hisui 等の呼び出し側のユースケースを踏まえて別途判断する。案 A では以下を書き加える:

- `Encoder::encode` の rustdoc:
  - 「エラーが返った場合、コールバック内部で PCM の一部が既に消費されている可能性があり、消費バイト数は観測できない。以降この `Encoder` インスタンスを利用しないこと」
- `Encoder::finish` の rustdoc:
  - 同上
- `skills/shiguredo-audio-toolbox/SKILL.md` の Encoder フロー説明:
  - 同上の 1 行を追加

案 B に進む場合は追加で以下を実装する:

- `Encoder` に `poisoned: Option<Error>` (もしくは類似のフラグ) を追加
- `encode` / `finish` / `next_frame` の冒頭で poisoned を検査し、既にポイズンなら同じエラーを返す
- `encode_impl` の Err を返す前に `poisoned = Some(...)` を設定
- 回帰テストとして「1 回 Err → 次の encode も同じ Err」ケースを追加
