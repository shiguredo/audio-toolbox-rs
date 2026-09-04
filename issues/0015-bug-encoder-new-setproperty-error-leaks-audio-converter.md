# Encoder::new の AudioConverterSetProperty 失敗時に AudioConverter がリークする不具合を修正する

- Priority: High
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/fix-encoder-new-setproperty-error-leaks-audio-converter
- Polished: 2026-09-04

## 目的

`Encoder::new` が `AudioConverterNew` で AudioConverter を取得した後、`AudioConverterSetProperty` のいずれかで失敗して早期 return したとき、`Self` が組み立てられていないため `Drop` が走らず AudioConverter がリークする不具合を修正する。

## 優先度根拠

High とする。無効なビットレート値で `Encoder::new` を試行する既存のテストパスで実際に踏まれており (`tests/test_encoder.rs` の `encoder_new_rejects_invalid_bitrate` 等)、FFI リソースのリークは繰り返し発生すると蓄積し得る。過去に issues/0004〜0008 の hardening と issues/0010〜0012 のコールバック契約修正を進めてきた堅牢性向上の方針とも整合しない。

## 現状

`src/lib.rs` の `Encoder::new` は以下の構造で書かれている。

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

いずれかの `?` から早期 return した場合、`converter` は解放されずにプロセスに残り続ける。`src/lib.rs` の `impl Drop for Encoder` は `Self` 構築後にしか働かない。

`tests/test_encoder.rs` の `encoder_new_rejects_invalid_bitrate` は `bitrate = Some(1_000)` を渡し、`AudioConverterSetProperty(EncodeBitRate)` の失敗を Err で確認しているが、その裏でリークが毎回起きている。同じ失敗パスは `src/lib.rs` の `#[cfg(test)]` モジュールのテストにもある。

なお、Decoder 側 (`src/lib.rs` の `Decoder::new`) は `AudioConverterNew` 以降に失敗し得るプロパティ設定が存在しないため、エラー返却経路のリークはない。

`src/codec_info.rs` の `query_bitrate_control_modes` は明示的に `AudioConverterDispose` を呼んでおりリークしない。`Vec::push` の allocator failure はデフォルトのアロケータではプロセス abort に至るため実質的なリークにならず、本 issue の対象としない。

## 設計方針

`Encoder::new` では `AudioConverterNew` の成功後、`converter` が null でないことを確認してから `Self` を先に組み立て、その後、組み立てた `Self` の `converter` フィールドに対して各 `AudioConverterSetProperty` を実行する。SetProperty の失敗時は `?` による早期 return で `Self` が drop され、既存の `impl Drop for Encoder` が `AudioConverterDispose` を呼ぶ。RAII によりリークしないことが自明になるため、専用のガード型や `mem::forget` / `ManuallyDrop` は導入しない。

## 完了条件

- `Encoder::new` の `AudioConverterSetProperty(...)` 失敗時に AudioConverter がリークしない (コード上リークしないことをレビューで確認できる)。
- 既存の `encoder_new_rejects_invalid_bitrate` 等のテストが引き続きパスする。
- `CHANGES.md` の develop に [FIX] として追記する。

## 解決方法

1. `src/lib.rs` の `Encoder::new` を書き換える:
   - `AudioConverterNew` の成功後、`converter` が null でないことを確認してから `Self` を組み立て、ローカル変数に保持する (`src/codec_info.rs` の `create_probe_converter` と同じ null 検査。`AudioConverterDispose` の null 入力時の挙動は Apple ドキュメントに明記されていないため)。null だった場合は `Error` を返して早期 return する (例: `Error { status: sys::kAudio_ParamError, function: "AudioConverterNew" }`。null のため dispose は不要)
   - 各 `AudioConverterSetProperty` はローカル変数の `Self` の `converter` フィールドに対して実行する
   - SetProperty の失敗時はそのまま `?` で早期 return する (ローカル変数の `Self` が drop され `AudioConverterDispose` が走る)
   - すべての SetProperty を通過したらローカル変数の `Self` を `Ok` で返す
2. 追加テストは不要。既存の `encoder_new_rejects_invalid_bitrate` (`bitrate = Some(1_000)` で EncodeBitRate 段を失敗させる) が失敗パスをカバーしており、修正後のコードではどの SetProperty 段で失敗してもローカルの `Self` の drop によって同一の解放経路を通るため、この 1 つの失敗パスで機構を検証できる。EncodeBitRate 段以外の SetProperty 段 (BitRateControlMode / CodecQuality / SoundQualityForVBR) は、`bitrate_control_mode` / `codec_quality` が enum のため無効値を構築できず、`vbr_quality` (`Option<u32>`) だけは範囲外値を渡せるが、SoundQualityForVBR の範囲外値に対する失敗挙動は Apple 側の実装依存で本 issue では検証しない。有効値で失敗しないことは既存テスト (`encoder_new_accepts_each_bitrate_control_mode` / `encoder_new_accepts_each_codec_quality`) で確認されている。
