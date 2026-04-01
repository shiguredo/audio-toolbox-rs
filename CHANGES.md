# 変更履歴

- UPDATE
  - 後方互換がある変更
- ADD
  - 後方互換がある追加
- CHANGE
  - 後方互換のない変更
- FIX
  - バグ修正

## develop

- [ADD] コーデック情報取得 API `supported_codecs()` を追加する
  - `AudioCodecType`, `AudioCodecInfo`, `AudioDecodingInfo`, `AudioEncodingInfo` 型を追加する
  - デコード判定に `AudioFormatGetPropertyInfo(kAudioFormatProperty_Decoders)` を使用する
  - エンコード判定に `AudioFormatGetPropertyInfo(kAudioFormatProperty_Encoders)` を使用する
  - ビットレート制御モード取得に `AudioConverter` のプロパティ照会を使用する
  - 照会対象は `EncoderCodec` / `DecoderCodec` に対応する `AudioCodecType` のみとし、`Encoder` / `Decoder` の表面積と返却一覧の意味を揃える（HE-AAC / FLAC / ALAC 等の列挙子は型として残し照会からは除外する）
  - @voluntas
- [CHANGE] `Decoder::next_decoded_data()` を `Decoder::next_frame()` にリネームする
  - @voluntas
- [CHANGE] `Encoder::encode()` の戻り値を `Result<(), Error>` に変更する
  - エンコード結果は `Encoder::next_frame()` で取得する
  - @voluntas
- [CHANGE] `Encoder::finish()` の戻り値を `Result<(), Error>` に変更する
  - 残りのエンコード結果は `Encoder::next_frame()` で取得する
  - @voluntas
- [CHANGE] `Encoder::new()` の引数を `EncoderConfig` に変更する
  - @voluntas
- [ADD] `BitRateControlMode` enum を追加する
  - @voluntas
- [ADD] `CodecQuality` enum を追加する
  - @voluntas
- [ADD] `EncoderConfig` 構造体を追加する
  - `sample_rate` / `channels` / `bitrate` / `bitrate_control_mode` / `codec_quality` / `vbr_quality` を設定可能にする
  - @voluntas
- [ADD] `EncoderCodec` enum を追加する
  - @voluntas
- [ADD] `Encoder::next_frame()` メソッドを追加する
  - @voluntas
- [ADD] `DecoderCodec` enum を追加し MP3 / Opus デコードに対応する
  - @voluntas
- [ADD] `DecoderConfig` 構造体を追加する
  - @voluntas
- [CHANGE] `Decoder::new()` の引数を `DecoderConfig` に変更する
  - @voluntas
- [ADD] AAC / MP3 / Opus デコーダーを追加する
  - @sile
- [FIX] `Decoder::decode` が `next_frame` より前に複数回呼ばれたとき圧縮データを連結してしまう不具合を修正する
  - 未消費のパケットがある状態で再度 `decode` した場合はエラーを返す
  - @voluntas

### misc

- [UPDATE] `tests/test_codec_info.rs` / `tests/test_encoder.rs` を追加し、`tests/test_decoder.rs` のカバレッジを拡張する
  - `tests/include/helpers.rs` を共有ヘルパとして追加する
  - @voluntas
- [UPDATE] `tests/test_decoder.rs` を単体テストのみとし、`proptest` を dev-dependencies から削除する
  - @voluntas
- [UPDATE] `DECODE_BUF_FRAMES` のコメントを RFC 6716 §2.1.4 に基づき、Opus の理論上の最大フレーム数と定数の関係を明示する（RFC 8251 は参照デコーダ等の更新であり §2.1.4 の本文は変更しない旨を注記する）
  - `issues/0002-investigate-decoder-output-buffer-vs-codec-limits.md` に調査内容を追記する
  - @voluntas
- [ADD] 正弦波 PCM を AAC エンコードして MP4 に保存するサンプルを追加する
  - @voluntas

## 2025.1.0

**リリース日**: 2025-09-26
