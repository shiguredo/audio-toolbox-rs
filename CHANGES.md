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

### misc

- [ADD] 正弦波 PCM を AAC エンコードして MP4 に保存するサンプルを追加する
  - @voluntas

## 2025.1.0

**リリース日**: 2025-09-26
