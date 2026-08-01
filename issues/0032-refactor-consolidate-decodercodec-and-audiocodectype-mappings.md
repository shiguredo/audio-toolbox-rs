# DecoderCodec と AudioCodecType の format mapping の重複を解消する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-consolidate-codec-mappings
- Polished: 2026-07-31

## 目的

`DecoderCodec::{format_id, format_flags, frames_per_packet}` と `AudioCodecType::{format_id, format_flags, frames_per_packet}` に同じ Audio Toolbox 定数への mapping が別実装で存在している状態を解消する。あわせて `Encoder::new` の直書きも `AudioCodecType` 経由に統一する (責務の対称化)。仕様変更時に片方だけ更新される事故を防ぐ。

## 優先度根拠

Medium とする。動作は正しく mapping も現時点で一致しているが、責務の重複は保守負担を増やすことが確実。`EncoderCodec` には mapping メソッドが無く `Encoder::new` の match に直書きされているため、責務の非対称も本 issue で是正する。

## 現状

- `src/lib.rs` の `DecoderCodec::format_id` は `kAudioFormatMPEG4AAC` / `kAudioFormatMPEGLayer3` / `kAudioFormatOpus` を返す。
- `src/codec_info.rs` の `AudioCodecType::format_id` は 9 種すべてに値があり、うち 3 種 (`AacLc` / `Mp3` / `Opus`) が `DecoderCodec` と重複している。
- `format_flags` / `frames_per_packet` も同様の重複 (`AacLc => kMPEG4Object_AAC_LC` / `_ => 0`、`AacLc => 1024` / `Mp3 => 1152` / `Opus => 0`)。
- `EncoderCodec` は `AacLc` のみを持ち、`Encoder::new` の match アームで mapping (`kAudioFormatMPEG4AAC` / `kMPEG4Object_AAC_LC` / 1024) が直書きされている。

## 完了条件

1. mapping が `AudioCodecType` に一元化される (`grep -n "sys::kAudioFormatMPEG4AAC\|sys::kAudioFormatMPEGLayer3\|sys::kAudioFormatOpus\|sys::kMPEG4Object" src/lib.rs` が 0 件。`kAudioFormatLinearPCM` 等の PCM フォーマット定数は対象外で残る)。
2. `DecoderCodec::{format_id, format_flags, frames_per_packet}` が削除される。
3. `Decoder::new` / `Encoder::new` が `AudioCodecType` 経由で mapping を引く。
4. 既存の全テストが引き続きパスする (`cargo test --workspace -- --test-threads=1` が成功する)。
5. `src/lib.rs` の doc comment に Audio Toolbox 定数名 (`kAudioFormat*` / `kMPEG4Object*`) が残らない (手順 7 の完了確認)。
6. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`AudioCodecType` に一元化する (案 A で確定。内部ヘルパー案 (案 B) は `DecoderCodec` → `AudioCodecType` の変換が結局必要で実質差が小さく、`AudioCodecType` が一次ソースの関係が明確な案 A を採る)。

1. `AudioCodecType::{format_id, format_flags, frames_per_packet}` を `pub(crate) fn format_id(self) -> u32` / `pub(crate) fn format_flags(self) -> u32` / `pub(crate) fn frames_per_packet(self) -> u32` に変更する (現状は module 内 private のため `Decoder::new` / `Encoder::new` から呼べない)。
2. `DecoderCodec::to_audio_codec_type` を追加する (`pub(crate) fn to_audio_codec_type(self) -> AudioCodecType`。match で `AacLc => AudioCodecType::AacLc` / `Mp3 => AudioCodecType::Mp3` / `Opus => AudioCodecType::Opus` を対応させる)。
3. `Decoder::new` を書き換えて、`to_audio_codec_type()` 経由で `AudioCodecType` の mapping を引く (`mSampleRate` / `mChannelsPerFrame` は config から、`mBitsPerChannel` / `mBytesPerPacket` は固定値 0 のまま)。
4. `DecoderCodec::{format_id, format_flags, frames_per_packet}` を削除する。
5. `EncoderCodec::to_audio_codec_type` を新設の impl ブロックとして追加する (`#[cfg(target_os = "macos")]` を付ける。既存の `DecoderCodec` の impl と同様。`pub(crate) fn to_audio_codec_type(self) -> AudioCodecType` で `AacLc => AudioCodecType::AacLc` の 1 対 1 対応)。
6. `Encoder::new` を書き換える。`mFormatID` / `mFormatFlags` / `mFramesPerPacket` の 3 フィールドは `AudioCodecType` 経由で引く。`mSampleRate` / `mChannelsPerFrame` は config から、`mBitsPerChannel` / `mBytesPerPacket` は固定値 0 を設定する。match アーム内の Table 2-6 参照コメントは codec_info.rs 側 (mapping の一次ソース) へ移動する。
7. `DecoderCodec` / `EncoderCodec` のバリアント doc comment に記載された mapping 定数 (例: 「mFormatID = kAudioFormatMPEG4AAC ...」) を「mapping は `AudioCodecType` に定義される」旨の参照に書き換える (doc comment が 3 つ目の情報源になるのを防ぐ)。書き換え対象は mapping 定数のみで、「1 パケットあたり 1024 フレーム固定」等の仕様情報は残す (仕様情報を一元化する場合は codec_info.rs 側のバリアント doc にも同様に記載する)。
8. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --workspace -- -D warnings` で確認する (手順 2-4 / 5-6 は変換の追加・使用・旧 mapping の削除を同一コミットにし、中間状態で dead_code 警告が CI を落とさないようにする)。

`AudioCodecType` の probe 対象外バリアント (`AacHe` / `AacHeV2` / `AacLd` / `AacEld` / `Flac` / `Alac`。コード上は `format_id` 等の match で使用されるが probe では構築されない) の扱いは、issue 0033 の解決方法 4 と同様に別 issue で再確認する (両 issue とも同じ懸念を持つため、0032 / 0033 のいずれか先に実施された側で別 issue を立てる)。

issue 0033 と同ファイル (`src/codec_info.rs`) を編集するため、マージ順・コンフリクト対応に注意する。issue 0037 は `tests/test_codec_info.rs` のみを編集するため直接のコンフリクトはない。テストは公開 API のみを使用するため issue 0018 (テスト移設) との順序依存はない。`tests/include/helpers.rs` の `AAC_FRAMES_PER_PACKET` 等のフレーム数の複製はテスト用定数のため対象外。CHANGES.md の develop / `### misc` に同時期に追記する他 issue とは、マージ時にコンフリクトした場合は develop の最新を取り込んで解決する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] コーデックの format mapping を AudioCodecType に一元化する`)。
