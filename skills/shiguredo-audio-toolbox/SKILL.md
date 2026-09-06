---
name: shiguredo-audio-toolbox
description: 時雨堂の Audio Toolbox バインディングライブラリ shiguredo_audio_toolbox の機能・API リファレンス。PCM 音声データの AAC-LC エンコード、AAC-LC / MP3 / Opus デコード、ビットレート制御、コーデック品質、コーデック情報照会に関する質問時に使用。
---

# shiguredo_audio_toolbox

Apple の [Audio Toolbox] を利用した音声エンコーダー / デコーダーの Rust バインディングライブラリ。

[Audio Toolbox]: https://developer.apple.com/documentation/audiotoolbox/

## 特徴

- **macOS 専用**: Apple の [AudioConverter] API を直接利用するため macOS でしか動作しない
- **ビルド時バインディング生成**: ビルド時に Xcode の SDK ヘッダーを参照して `bindgen` でバインディングを自動生成する
- **AAC-LC エンコード**: 入力 PCM (i16 インターリーブ) を AAC-LC にエンコード
- **マルチコーデックデコード**: AAC-LC / MP3 / Opus に対応
- **出力 PCM 固定**: デコーダー出力は常にステレオ (2ch) 16bit PCM
- **コーデック情報照会**: [`supported_codecs()`] でシステムが対応するコーデックとビットレート制御モードを取得

[AudioConverter]: https://developer.apple.com/documentation/audiotoolbox/audio_converter
[`supported_codecs()`]: #コーデック情報照会

## バージョン情報

- crate 名: `shiguredo_audio_toolbox`
- バージョン: 2026.1.0
- Rust Edition: 2024
- 最小 Rust バージョン: 1.88
- ライセンス: Apache-2.0
- 対象 OS: macOS のみ (arm64)

`docs.rs` 向けには `DOCS_RS=1 cargo doc --no-deps` で macOS 以外でもドキュメントのみ生成可能。

## アーキテクチャ概要

- AudioConverter は低レベル API であり、コンテナ (MP4 等) の解析機能を持たない。入力フォーマット (サンプルレート、チャンネル数) は呼び出し側が指定する。
- エンコーダー / デコーダーともに「`encode` / `decode` で入力を渡す → `next_frame` で結果を取り出す」「最後に `finish()` を呼んでフラッシュ」のフローで使う。
- 内部では `AudioConverterFillComplexBuffer` を 1 パケット単位で呼び出し、コールバックから入力を供給する。コールバックは呼び出し元と同じスレッドで同期実行される。
- 入力データが不足している場合、コールバックは独自エラーコード `K_NO_MORE_INPUT` (12345) を返して処理を中断する。
- `Encoder` / `Decoder` ともに `Drop` で `AudioConverterDispose` を呼んでリソースを解放する。
- `AudioConverter` のスレッド間移動可否が Apple 公式に明記されていないため、`Encoder` / `Decoder` は `Send` を実装しない (`!Send`)。

## コア API

### エンコーダー側

| 型 | 説明 | 主要メソッド / フィールド |
|----|------|---------------------------|
| `Encoder` | AudioConverter による PCM → 圧縮データのエンコーダー (`!Send`) | `new(EncoderConfig)` (Result), `encode(&[i16])` (Result), `finish()` (Result), `next_frame() -> Option<EncodedFrame>` |
| `EncoderCodec` | エンコード対象コーデック (`#[non_exhaustive]` ではない) | `AacLc` |
| `EncoderConfig` | エンコーダー設定 (`Debug + Clone`) | `codec`, `sample_rate`, `channels`, `bitrate`, `bitrate_control_mode`, `codec_quality`, `vbr_quality` |
| `BitRateControlMode` | ビットレート制御モード | `Constant`, `LongTermAverage`, `VariableConstrained`, `Variable` |
| `CodecQuality` | コーデック品質 (速度との二律背反) | `Min`, `Low`, `Medium`, `High`, `Max` |
| `EncodedFrame` | エンコード済みフレーム | `data: Vec<u8>` (圧縮バイト列), `samples: usize` (チャンネルあたりのサンプル数。AAC-LC は通常 1024) |

`Encoder::new` は `sample_rate == 0` / `channels == 0` を `Error { status: -50, function: "Encoder::new(...)" }` で拒否する。`bitrate` / `bitrate_control_mode` / `codec_quality` / `vbr_quality` は `Option` であり、未指定時は AudioConverter のデフォルトに任せる。無効なビットレート (例: 1_000 bps) を指定すると `AudioConverterSetProperty(EncodeBitRate)` がエラーを返す。

### デコーダー側

| 型 | 説明 | 主要メソッド / フィールド |
|----|------|---------------------------|
| `Decoder` | AudioConverter による圧縮データ → PCM のデコーダー (`!Send`) | `new(DecoderConfig)` (Result), `decode(&[u8])` (Result), `finish()` (Result), `next_frame() -> Result<Option<Vec<i16>>, Error>` |
| `DecoderCodec` | デコード対象コーデック | `AacLc`, `Mp3`, `Opus` |
| `DecoderConfig` | デコーダー設定 (`Debug + Clone`) | `codec`, `input_sample_rate`, `input_channels` |

`Decoder::new` は `input_sample_rate == 0` / `input_channels == 0` を `Error { status: -50, function: "Decoder::new(...)" }` で拒否する。出力サンプルレートは入力と同じ値 (リサンプリングなし)。出力チャンネル数はステレオ固定で、入力がモノラルの場合は AudioConverter が自動でアップミックスする。

`Decoder::decode` は **1 回につき 1 パケット**を渡す API。前のパケットが `next_frame()` で消費されていない状態で再度 `decode` を呼ぶと `Error { status: -50, function: "Decoder::decode(previous packet not consumed)" }` を返す (連結による誤デコードを防ぐ)。ただし**空スライス `&[]` の `decode` は内部バッファを変更しないため**、続けて別の `decode` が可能。`next_frame()` がエラーを返した場合でも入力パケットは破棄され、以降は新しいパケットの `decode` から再開できる。

### エラー型

| 型 | 説明 | フィールド |
|----|------|------------|
| `Error` | Audio Toolbox API のエラー (`Debug + Display + std::error::Error`) | `status: i32` (OSStatus), `function: &'static str` (失敗した API 関数名) |

`Display` 実装は `"[shiguredo_audio_toolbox] <function>() failed: status=<status>"` の形で出力する。`status == 0` (`noErr`) は成功を表すため、`Error::check(status, function)` 経由でしか構築されない。

### コーデック情報照会

| 型 / 関数 | 説明 | 主要メソッド / フィールド |
|-----------|------|---------------------------|
| `supported_codecs() -> Vec<AudioCodecInfo>` | システムが本クレートで扱えるコーデック (`AAC-LC / MP3 / Opus`) のサポート状況を返す | 戻り値は順序固定: `AacLc`, `Mp3`, `Opus` |
| `AudioCodecType` | コーデック種別 (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`) | `AacLc`, `AacHe`, `AacHeV2`, `AacLd`, `AacEld`, `Mp3`, `Opus`, `Flac`, `Alac` |
| `AudioCodecInfo` | コーデックごとの情報 (`Debug + Clone + PartialEq`) | `codec: AudioCodecType`, `decoding: AudioDecodingInfo`, `encoding: AudioEncodingInfo` |
| `AudioDecodingInfo` | デコード情報 (`Debug + Clone + PartialEq + Eq`) | `supported: bool` |
| `AudioEncodingInfo` | エンコード情報 (`Debug + Clone + PartialEq + Eq`) | `supported: bool`, `bitrate_control_modes: Vec<BitRateControlMode>` |

`AudioCodecType` の列挙子には HE-AAC / FLAC / ALAC 等も含まれるが、`supported_codecs()` が返すのは `EncoderCodec` / `DecoderCodec` に対応する 3 種 (`AacLc`, `Mp3`, `Opus`) のみ。

`AudioEncodingInfo::bitrate_control_modes` は `AudioConverterSetProperty(kAudioCodecPropertyBitRateControlMode)` を実際に試して成功したモードだけが含まれる。エンコード非対応コーデックでは空。

## 入出力フォーマット仕様

### エンコーダー入力

- フォーマット: リニア PCM (`kAudioFormatLinearPCM`)
- フラグ: `kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked`
- サンプル型: `i16` インターリーブ
- ビット深度: 16
- サンプルレート: `EncoderConfig::sample_rate`
- チャンネル数: `EncoderConfig::channels`

### エンコーダー出力 (AAC-LC)

- フォーマット: `kAudioFormatMPEG4AAC`
- フラグ: `kMPEG4Object_AAC_LC`
- 1 パケット 1024 フレーム固定
- パケットサイズは可変 (`mBytesPerPacket = 0`)

### デコーダー入力

| コーデック | `mFormatID` | `mFormatFlags` | `mFramesPerPacket` |
|------------|-------------|----------------|--------------------|
| AAC-LC | `kAudioFormatMPEG4AAC` | `kMPEG4Object_AAC_LC` | 1024 |
| MP3 | `kAudioFormatMPEGLayer3` | 0 | 1152 |
| Opus | `kAudioFormatOpus` | 0 | 0 (可変長) |

### デコーダー出力 (固定)

- フォーマット: リニア PCM (`kAudioFormatLinearPCM`)
- フラグ: `kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked`
- サンプル型: `i16` インターリーブ
- ビット深度: 16
- サンプルレート: 入力と同じ (リサンプリングなし)
- チャンネル数: 2 (ステレオ固定)

## コード例

### AAC-LC エンコード

```rust
use shiguredo_audio_toolbox::{Encoder, EncoderCodec, EncoderConfig};

let mut encoder = Encoder::new(EncoderConfig {
    codec: EncoderCodec::AacLc,
    sample_rate: 48000,
    channels: 2,
    bitrate: Some(128_000),
    bitrate_control_mode: None,
    codec_quality: None,
    vbr_quality: None,
})?;

// PCM データ (i16, インターリーブ) をエンコード
let pcm: &[i16] = &[0; 1024 * 2];
encoder.encode(pcm)?;
while let Some(frame) = encoder.next_frame() {
    println!("encoded bytes: {}, samples: {}", frame.data.len(), frame.samples);
}

// 残りのフレームをフラッシュ
encoder.finish()?;
while let Some(frame) = encoder.next_frame() {
    println!("flushed bytes: {}", frame.data.len());
}
```

### AAC-LC / MP3 / Opus デコード

```rust
use shiguredo_audio_toolbox::{Decoder, DecoderCodec, DecoderConfig};

let mut decoder = Decoder::new(DecoderConfig {
    codec: DecoderCodec::AacLc,
    input_sample_rate: 48000,
    input_channels: 2,
})?;

// 1 パケットずつ decode する。
// 前のパケットを next_frame で `Ok(None)` まで消費してから次の decode に進む。
for packet in packets {
    decoder.decode(&packet)?;
    while let Some(pcm) = decoder.next_frame()? {
        // pcm は i16 インターリーブのステレオ PCM
        println!("decoded samples: {}", pcm.len() / 2);
    }
}

decoder.finish()?;
while let Some(pcm) = decoder.next_frame()? {
    println!("decoded samples (flush): {}", pcm.len() / 2);
}
```

### ビットレート制御モードとコーデック品質

```rust
use shiguredo_audio_toolbox::{
    BitRateControlMode, CodecQuality, Encoder, EncoderCodec, EncoderConfig,
};

let mut encoder = Encoder::new(EncoderConfig {
    codec: EncoderCodec::AacLc,
    sample_rate: 48000,
    channels: 2,
    bitrate: Some(128_000),
    bitrate_control_mode: Some(BitRateControlMode::Constant),
    codec_quality: Some(CodecQuality::High),
    vbr_quality: None, // bitrate_control_mode = Variable のときに利用 (0-127)
})?;
```

ビットレート制御モードは AudioConverter のプロパティ設定順序に依存する: 実装内部では「`bitrate_control_mode` → `bitrate` → `codec_quality` → `vbr_quality`」の順に `AudioConverterSetProperty` を呼ぶ。これは制御モードによって有効なビットレート範囲が変わるためである。

### コーデック情報の照会

```rust
use shiguredo_audio_toolbox::{AudioCodecType, supported_codecs};

for info in supported_codecs() {
    println!(
        "{:?}: decode={}, encode={}",
        info.codec, info.decoding.supported, info.encoding.supported,
    );
    if info.encoding.supported {
        println!("  bitrate control modes: {:?}", info.encoding.bitrate_control_modes);
    }
}
```

`supported_codecs()` は順序が固定 (AAC-LC → MP3 → Opus) のため、`find` で特定種別を引いてもよい。

```rust
let aac_lc = supported_codecs()
    .into_iter()
    .find(|info| info.codec == AudioCodecType::AacLc)
    .expect("AAC-LC は AudioToolbox に常に存在する");
assert!(aac_lc.decoding.supported && aac_lc.encoding.supported);
```

## 利用上の注意

### `Encoder::encode` のバッファリング

`encode` は呼び出されるたびに入力 PCM を内部バッファ (`pcm_buf`) に蓄積し、1 パケット分 (`channels * 1024` サンプル) たまるたびに `AudioConverterFillComplexBuffer` を呼ぶ。1 パケットに満たない端数は内部に残り、次の `encode` または `finish` まで保留される。`finish()` を呼ぶと内部バッファに残った端数も含めて全てフラッシュされる。

### `Decoder::decode` の 1 パケット制約

`Decoder` は「1 回の `decode` あたり 1 パケット」の前提で実装されている。同じパケットが 1 回の `AudioConverterFillComplexBuffer` 呼び出し内で複数回コールバックに渡されないよう、内部フラグ `packet_provided_in_this_fill` で制御している。これは過去にあったバグ ([0012 で修正済み](https://github.com/shiguredo/audio-toolbox-rs/pull/4)) への対策。

複数パケットを連結して 1 回の `decode` に渡すと、フレーム数が想定の倍以上になる誤動作を起こすため、呼び出し側で必ずパケット境界を保ったまま投入する。

`next_frame()` がエラーを返した場合でも入力パケットは破棄され、以降は新しいパケットの `decode` から再開できる。

### プライミングサンプル

AAC エンコーダーはエンコーダー内部のプライミング (先頭にゼロ詰めしたフレーム) を生成するため、`encode → decode` の往復で得られる PCM サンプル数は元の入力 PCM サンプル数より多くなる。テストでも `total_decoded.len() >= 入力フレーム数` と弱めに検証している。

### スレッド安全性

- `Encoder` / `Decoder` は `Send` を実装しない。Apple 公式に `AudioConverter` のスレッド間移動可否の規範的根拠がないため、`unsafe impl Send` を意図的に避けている。
- 同一スレッド内では問題なく利用可能。スレッド間で受け渡しが必要な場合は呼び出し側で適切に直列化する必要がある。

### `Drop` の挙動

`Encoder` / `Decoder` の `Drop` 実装は `AudioConverterDispose(self.converter)` を呼ぶ。コンバーター生成 (`AudioConverterNew`) が失敗した場合は `Self` が組み立てられないため、`Drop` は走らない。

### 内部定数 (lib.rs)

| 定数 | 値 | 用途 |
|------|----|------|
| `OUTPUT_CHANNELS` | 2 | デコーダー出力チャンネル数 (ステレオ固定) |
| `K_NO_MORE_INPUT` | 12345 | コールバック内で「入力データ不足」を AudioConverter に通知する独自エラーコード |
| `ENCODE_BUF_SIZE` | 4096 | エンコード結果を格納する一時バッファサイズ (バイト) |
| `DECODE_BUF_FRAMES` | 5760 | デコード時の出力バッファサイズ (フレーム)。Opus の最大パケット (48 kHz × 120 ms = 5760 フレーム、RFC 6716 §2.1.4) を覆える値 |

`K_NO_MORE_INPUT` は Apple のドキュメントに記載のない独自値であり、フレームワーク側と衝突しない値として実際に動かしてみて選択している (理論的根拠はない)。

## エンコード/デコード時の挙動詳細

### `Encoder::encode` フロー

1. 引数の PCM を `pcm_buf` に追加する。
2. `encode_impl` をループで呼び、`Ok(Some(frame))` の間は `encoded_frames` キューに積む。
3. `encode_impl` 内では `AudioConverterFillComplexBuffer` を 1 パケット分だけ要求して呼び出す。
4. コールバックは要求パケット数分の PCM (`packets * channels * 2 bytes`) を `pcm_buf` の先頭からコピーし、`drain` する。
5. データが不足している場合 (`!eos && pcm_buf.len() < need_samples`) はコールバックが `K_NO_MORE_INPUT` を返し、`encode_impl` は `Ok(None)` を返してループを終わる。

### `Encoder::finish` フロー

1. `eos = true` をセット。
2. `encode_impl` をループで呼び、内部に残っている PCM を全てフラッシュ。
3. コールバックは `eos = true` の状態では「使えるサンプル数だけ提供する」ように動作する。

### `Decoder::next_frame` (= `decode_impl`) フロー

1. `packet_provided_in_this_fill = false` にリセット。
2. `eos && encoded_buf.is_empty()` なら `Ok(None)` を返す。
3. 出力バッファ (`DECODE_BUF_FRAMES * OUTPUT_CHANNELS * 2 bytes`) を確保。
4. `AudioConverterFillComplexBuffer` を呼ぶ。
5. コールバックは初回呼び出し時のみ `encoded_buf` を 1 パケットとして提供し、`packet_provided_in_this_fill = true` をセット。同じ fill 呼び出し内の 2 回目以降は `K_NO_MORE_INPUT` を返す。
6. デコード処理を試行した以上、入力バッファは消費済みとして `encoded_buf` をクリアする (エラー時も同様で、消費状況は不明)。
7. 戻り値が `0` または `K_NO_MORE_INPUT` 以外ならエラー。
8. 出力バイト列をサンプル数で truncate して返す。

### Drop / リソース管理

`Encoder` / `Decoder` の `Drop` 実装は `unsafe { sys::AudioConverterDispose(self.converter); }` のみ。`AudioConverterNew` が成功している前提で安全。

## サンプル

リポジトリの `examples/sine_to_mp4.rs` は正弦波 PCM を AAC-LC エンコードし、`shiguredo_mp4` クレートで MP4 ファイルに保存するサンプル。

```bash
cargo run --example sine_to_mp4 -- --bitrate 256000 --duration 10 --freq 880 --output tone.mp4
```

オプション:

| オプション | 既定値 | 説明 |
|-----------|--------|------|
| `--bitrate` | 128000 | AAC-LC のビットレート (bps) |
| `--duration` | 5 | 出力長 (秒) |
| `--freq` | 440 | 正弦波の周波数 (Hz) |
| `--output` | `output.mp4` | 出力ファイル名 |

AAC-LC 48 kHz ステレオ用の `AudioSpecificConfig` (`0x11, 0x90`) を直接埋め込んで `mp4a` SampleEntry を構築している点が参考になる。

## 既知の制限事項

- **macOS 専用**: `compile_error!("this crate only supports macOS")` でビルドを禁止している (`cfg(doc)` 時は除外)。
- **AudioConverter スコープ**: AudioConverter API がサポートしないコーデックは扱えない。例として **FLAC は AudioConverter では非対応** のため `DecoderCodec` に含まれない (ExtAudioFile API が必要)。
- **エンコードは AAC-LC のみ**: `EncoderCodec` は `AacLc` の 1 種類のみ。HE-AAC / Opus / FLAC 等のエンコードは未対応。
- **出力は常にステレオ 16bit PCM**: モノラル出力やビット深度 24/32 の取得には対応していない。
- **コンテナ非対応**: AudioConverter は MP4 / ADTS / Ogg 等のコンテナを扱わない。コンテナの解析・生成は呼び出し側 (例: `shiguredo_mp4`) で行う。
- **`bitrate_control_modes` のプローブ前提**: `supported_codecs()` が返すビットレート制御モードは「48 kHz ステレオの代表的な設定で AudioConverter を作成して `AudioConverterSetProperty` を試した結果」であり、別のサンプルレート / チャンネル数で同じモードが使えるとは限らない。
- **Opus エンコード対応の OS 依存性**: `supported_codecs()` で得られる Opus のエンコード可否は OS バージョンに依存するため、本クレートのテストでも encode 側は検証していない。

## ソースファイル構成

| ファイル | 役割 |
|---------|------|
| `src/lib.rs` | `Error`, `Encoder`, `Decoder`, `EncoderConfig`, `DecoderConfig`, `EncoderCodec`, `DecoderCodec`, `BitRateControlMode`, `CodecQuality`, `EncodedFrame` 等のコア API |
| `src/codec_info.rs` | `supported_codecs()`, `AudioCodecType`, `AudioCodecInfo`, `AudioDecodingInfo`, `AudioEncodingInfo` のコーデック情報照会 API |
| `src/sys.rs` | `bindgen` で生成された Audio Toolbox の FFI バインディング (`OUT_DIR/bindings.rs` を `include!`) |
| `build.rs` | macOS SDK の `<AudioToolbox/AudioToolbox.h>` から `bindgen` でバインディングを生成 |
| `examples/sine_to_mp4.rs` | 正弦波 PCM → AAC-LC → MP4 のエンコードサンプル (`shiguredo_mp4` を利用) |
| `tests/test_encoder.rs` | `Encoder` の単体テスト (引数バリデーション、ビットレート制御モード、コーデック品質) |
| `tests/test_decoder.rs` | `Decoder` の単体テスト (引数バリデーション、1 パケット制約、複数パケット連続投入) |
| `tests/test_codec_info.rs` | `supported_codecs()` の単体テスト |
| `tests/include/helpers.rs` | テスト共有のヘルパー (`encoder_config`, `decoder_config_aac`, `sine_pcm`, `encode_aac_packets`) |
