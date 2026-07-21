# Encoder::new の AudioConverterSetProperty 失敗時に AudioConverter がリークする不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-encoder-new-setproperty-error-leaks-audio-converter
- Polished:

## 目的

`Encoder::new` が `AudioConverterNew` で AudioConverter を取得した後、`AudioConverterSetProperty` のいずれかで失敗して早期 return したとき、`Self` が組み立てられていないため `Drop` が走らず AudioConverter がリークする不具合を修正する。同時に `codec_info::query_bitrate_control_modes` に潜む panic 経路のリークも同じガード型で塞ぐ。

## 優先度根拠

High とする。無効なビットレート値等で `Encoder::new` を試行する既存のテストパスで実際に踏まれており (`tests/test_encoder.rs::encoder_new_rejects_invalid_bitrate`)、長時間動作するプロセスで無効設定を試行し続けると AudioConverter リソースが蓄積してシステム負荷を高める。過去に issues/0004〜0012 で進めた FFI hardening の一環として塞ぐべき残存の穴。

## 現状

`src/lib.rs:323-388` の `Encoder::new` は以下の構造で書かれている。

```rust
let mut converter = std::ptr::null_mut();
let status = sys::AudioConverterNew(&input_format, &output_format, &mut converter);
Error::check(status, "AudioConverterNew")?;

// ここから下で SetProperty が失敗すると converter がリーク
if let Some(mode) = config.bitrate_control_mode {
    ...
    Error::check(status, "AudioConverterSetProperty(BitRateControlMode)")?;
}
if let Some(bitrate) = config.bitrate {
    ...
    Error::check(status, "AudioConverterSetProperty(EncodeBitRate)")?;
}
if let Some(quality) = config.codec_quality {
    ...
    Error::check(status, "AudioConverterSetProperty(CodecQuality)")?;
}
if let Some(vbr_quality) = config.vbr_quality {
    ...
    Error::check(status, "AudioConverterSetProperty(SoundQualityForVBR)")?;
}

Ok(Self { converter, ... })  // ここに到達して初めて Drop が有効になる
```

いずれかの `?` から早期 return した場合、`converter` は解放されずにプロセスに残り続ける。`impl Drop for Encoder` (`src/lib.rs:592-599`) は `Self` 構築後にしか働かない。

`tests/test_encoder.rs:43-45` の `encoder_new_rejects_invalid_bitrate` は `bitrate = Some(1_000)` を渡し、`AudioConverterSetProperty(EncodeBitRate)` の失敗を Err で確認しているが、その裏でリークが毎回起きている。

同じ問題は `src/codec_info.rs:194-240` の `query_bitrate_control_modes` にもある。`Vec::push` が allocator failure で panic した場合 (実用上は稀だが)、`AudioConverterDispose(converter)` に到達せず converter がリークする。

なお、Decoder 側 (`src/lib.rs:806-807`) は `AudioConverterNew` 直後に `Ok(Self { ... })` するため同種のリークはない。本 issue の対象は Encoder 側と `query_bitrate_control_modes` の 2 箇所。

## 設計方針

Rust の RAII に則り、`sys::AudioConverterRef` を保持する薄い drop-only なガード型を crate 内に導入する。`Encoder::new` と `query_bitrate_control_modes` の両方で使う。

```rust
struct ConverterGuard(sys::AudioConverterRef);
impl Drop for ConverterGuard {
    fn drop(&mut self) {
        unsafe { sys::AudioConverterDispose(self.0); }
    }
}
```

- `Encoder::new` では `AudioConverterNew` の直後にガードで包み、全 `SetProperty` を通過したら `mem::forget(guard)` で解放を抑止して `Self` を組み立てる。あるいはガード型で持たせたまま `Encoder` に取り込む API を検討する。
- `query_bitrate_control_modes` では `for` ループの前にガードで包み、正常時も panic 時もガードの `Drop` に任せる。明示の `AudioConverterDispose` 呼び出しは削除する。

## 完了条件

- `Encoder::new` の `AudioConverterSetProperty(...)` 失敗時に AudioConverter がリークしない (Instruments / leaks コマンド / valgrind 相当での確認は必須ではないが、コード上リークしないことをレビューで確認できる)。
- `query_bitrate_control_modes` の panic 経路でも同様にリークしない。
- 既存の `encoder_new_rejects_invalid_bitrate` 等のテストが引き続きパスする。
- 新しい失敗パス (SetProperty 各段) を明示的に踏むテストを最小限追加する。

## 解決方法

1. `src/lib.rs` (もしくは `src/sys.rs` の隣) にプライベートな `ConverterGuard` 型を追加する。`Drop` で `AudioConverterDispose` を呼ぶ。
2. `Encoder::new` を書き換える:
   - `AudioConverterNew` 直後に `let guard = ConverterGuard(converter);` を導入
   - 各 `AudioConverterSetProperty` は `guard.0` に対して行う (もしくはガードにアクセサを生やす)
   - 全プロパティ設定を通過したら `let converter = ManuallyDrop::new(guard); Ok(Self { converter: converter.0, ... })` のように所有権を移す
3. `codec_info::query_bitrate_control_modes` を書き換える:
   - `create_probe_converter` の戻り値をガードで包む
   - 明示的な `unsafe { sys::AudioConverterDispose(converter); }` を削除
4. 追加テスト:
   - `Encoder::new` に無効な `codec_quality` / `vbr_quality` を渡し、SetProperty のより後段で失敗する経路を最小限確認する (現状の `bitrate = Some(1_000)` は最初の SetProperty 段で失敗するため後段の分岐がカバーされていない可能性がある。ただし後段で必ず失敗する値の特定は Apple 仕様依存で難しい場合がある)。
