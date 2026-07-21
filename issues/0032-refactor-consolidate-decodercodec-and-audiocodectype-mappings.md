# DecoderCodec と AudioCodecType の format mapping の重複を解消する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-consolidate-codec-mappings
- Polished:

## 目的

`DecoderCodec::{format_id, format_flags, frames_per_packet}` (`src/lib.rs:642-676`) と `AudioCodecType::{format_id, format_flags, frames_per_packet}` (`src/codec_info.rs:44-82`) に同じ Audio Toolbox 定数への mapping が別実装で存在している状態を解消する。仕様変更時に片方だけ更新される事故を防ぎ、責務を対称化する。

## 優先度根拠

Medium とする。動作は正しく mapping も現時点で一致しているが、責務の重複は保守負担を増やすことが確実。`EncoderCodec` にはさらに mapping メソッドが無く `Encoder::new` の match に直書きされているため、責務の非対称も同時に是正できる余地がある。

## 現状

- `DecoderCodec::format_id` (`src/lib.rs:645-651`) は `kAudioFormatMPEG4AAC` / `kAudioFormatMPEGLayer3` / `kAudioFormatOpus` を返す。
- `AudioCodecType::format_id` (`src/codec_info.rs:44-56`) は同じ 3 種を含む全 9 種を返す。
- `format_flags` / `frames_per_packet` も同様の重複。
- `EncoderCodec` は `AacLc` のみを持ち、`Encoder::new` (`src/lib.rs:306-321`) の match アームで mapping が直書きされている。

## 完了条件

- mapping が 1 箇所 (`AudioCodecType` もしくは内部専用のヘルパー) に集約される。
- `DecoderCodec` / `EncoderCodec` から一次ソースを引くようになる。
- 既存の全テストが引き続きパスする。

## 解決方法

以下の 2 案を検討する。

### 案 A: AudioCodecType に一元化 (推奨)

- `DecoderCodec` から対応する `AudioCodecType` を返す変換関数を用意する (`fn to_audio_codec_type(self) -> AudioCodecType`)。
- `Decoder::new` の中で `DecoderCodec` から `AudioCodecType` に変換して mapping を引く。
- `EncoderCodec::AacLc` にも同様の変換を追加し、`Encoder::new` から mapping を引くように書き換える。
- `DecoderCodec::{format_id, format_flags, frames_per_packet}` を削除する。

### 案 B: 内部専用ヘルパー

- `src/codec_info.rs` (もしくは新規モジュール) に crate 内部専用の `pub(crate) fn format_id_for(codec: AudioCodecType) -> u32` を切り出す。
- `AudioCodecType::format_id` はこの内部ヘルパーを呼ぶだけの薄いラッパーになる。
- `DecoderCodec::format_id` も同じ内部ヘルパーを呼ぶ。

案 A のほうが「AudioCodecType が一次ソース」の関係が明確になる。

作業は以下の順で進める。

1. `DecoderCodec::to_audio_codec_type()` を追加する。
2. `Decoder::new` を書き換えて `AudioCodecType` 経由で mapping を引く。
3. `DecoderCodec::{format_id, format_flags, frames_per_packet}` を削除する。
4. `EncoderCodec::to_audio_codec_type()` を追加する。
5. `Encoder::new` を書き換えて mapping を `AudioCodecType` 経由で引く (AAC-LC の場合の `mBytesPerPacket` などのコーデック固有設定は別途 match で扱う)。
6. 既存テストが通ることを確認する。

なお、`AudioCodecType` の未使用バリアントは別 issue で扱う (関連する判断)。
