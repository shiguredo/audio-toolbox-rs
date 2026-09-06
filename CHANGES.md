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

- [FIX] Decoder のコールバックが 1 回の AudioConverterFillComplexBuffer 呼び出しで同じパケットを複数回返していた不具合を修正する
  - `Decoder` に `packet_provided_in_this_fill` フラグを追加し、1 回の `AudioConverterFillComplexBuffer` 呼び出し内で同じ入力パケットを 2 回目以降に提供しないようにする
  - `finish()` 後かつ入力が空の場合は `AudioConverterFillComplexBuffer` を呼ばずに `Ok(None)` を早期リターンし、ストリーム終端を誤って通知することによる `-50` エラーを防ぐ
  - `tests/test_decoder.rs` に非無音の AAC-LC パケットを使った回帰テストを追加する
  - @voluntas
- [FIX] Decoder のデコードエラー時に以降の decode が受け付けられなくなる不具合を修正する
  - `AudioConverterFillComplexBuffer` がエラーを返した場合でも入力バッファをクリアし、次のパケットから再開できるようにする
  - @voluntas

### misc

## 2026.1.0

**リリース日**: 2026-04-03

- [ADD] コーデック情報取得 API `supported_codecs()` を追加する
  - `AudioCodecType`, `AudioCodecInfo`, `AudioDecodingInfo`, `AudioEncodingInfo` 型を追加する
  - デコード判定に `AudioFormatGetPropertyInfo(kAudioFormatProperty_Decoders)` を使用する
  - エンコード判定に `AudioFormatGetPropertyInfo(kAudioFormatProperty_Encoders)` を使用する
  - ビットレート制御モード取得に `AudioConverter` のプロパティ照会を使用する
  - 照会対象は `EncoderCodec` / `DecoderCodec` に対応する `AudioCodecType` のみとし、`Encoder` / `Decoder` の表面積と返却一覧の意味を揃える（HE-AAC / FLAC / ALAC 等の列挙子は型として残し照会からは除外する）
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
- [ADD] AAC / MP3 / Opus デコーダーを追加する
  - @sile
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
- [CHANGE] `Decoder::new()` の引数を `DecoderConfig` に変更する
  - @voluntas
- [CHANGE] `Encoder` / `Decoder` の `unsafe impl Send` を削除する（Apple 公式にスレッド間移動の規範的根拠が取れないため sound と言い切れない）
  - @voluntas
- [FIX] `Decoder::decode` が `next_frame` より前に複数回呼ばれたとき圧縮データを連結してしまう不具合を修正する
  - 未消費のパケットがある状態で再度 `decode` した場合はエラーを返す
  - @voluntas
- [FIX] `mDataByteSize` の境界検証、FFI コールバックの null 検査・整数乗算の checked 化、`encoded_buf.len()` の `u32` 変換、`AudioStreamBasicDescription` / `AudioBufferList` の `assume_init` 依存除去を行う
  - @voluntas
- [FIX] Encoder / Decoder コールバックのエラー返却時に `*io_number_data_packets = 0` を保証する
  - Apple の `AudioConverterComplexInputDataProc` 契約に準拠する
  - @voluntas
- [FIX] エンコーダーコールバックで `mData == NULL` 時に自前バッファ (`scratch_buf`) を提供する
  - Apple の AudioConverter.h の規約に準拠し、OS 実装差分への堅牢性を確保する
  - @voluntas

### misc

- [UPDATE] docs.rs / Linux CI 向けに `bindings_docs_stub.rs` を追加し、`DOCS_RS` 時のスタブを bindgen 出力と整合する型・定数・ `extern "C"` 宣言で揃える
  - `build.rs` に `cargo::rerun-if-env-changed=DOCS_RS` を追加し、`DOCS_RS` の切り替えで bindgen とスタブが入れ替わるようにする
  - @voluntas
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
