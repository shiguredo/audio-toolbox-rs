# audio-toolbox-rs

[![crates.io](https://img.shields.io/crates/v/shiguredo_audio_toolbox.svg)](https://crates.io/crates/shiguredo_audio_toolbox)
[![docs.rs](https://docs.rs/shiguredo_audio_toolbox/badge.svg)](https://docs.rs/shiguredo_audio_toolbox)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GitHub Actions](https://github.com/shiguredo/audio-toolbox-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/audio-toolbox-rs/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?logo=discord&logoColor=white)](https://discord.gg/shiguredo)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## shiguredo_audio_toolbox について

Apple の [Audio Toolbox](https://developer.apple.com/documentation/audiotoolbox/) を利用した音声エンコーダーおよびデコーダーの Rust バインディングです。

macOS 専用で、ビルド時に Xcode の SDK ヘッダーを参照して bindgen でバインディングを自動生成します。

## 特徴

- Audio Toolbox による AAC-LC エンコード
- Audio Toolbox によるデコード (AAC-LC / MP3 / Opus)
- エンコーダーはサンプルレート / チャンネル数 / ビットレート / 品質を指定可能
- デコーダーは入力サンプルレートとチャンネル数を指定可能
- 出力は 16 bit ステレオ PCM 固定

## 対応コーデック

### エンコード

| コーデック | `EncoderCodec` |
| --- | --- |
| AAC-LC | `EncoderCodec::AacLc` |

### デコード

| コーデック | `DecoderCodec` |
| --- | --- |
| AAC-LC | `DecoderCodec::AacLc` |
| MP3 | `DecoderCodec::Mp3` |
| Opus | `DecoderCodec::Opus` |

## 動作要件

- macOS (arm64)
- Rust 1.88 以降
- Xcode Command Line Tools (ビルド時に Audio Toolbox のヘッダーファイルが必要)

## ビルド

```bash
cargo build
```

### docs.rs 向けビルド

Xcode がない環境 (Linux 等) では、docs.rs 向けのドキュメント生成のみ可能です。

```bash
DOCS_RS=1 cargo doc --no-deps
```

## 使い方

### エンコード

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

### デコード

```rust
use shiguredo_audio_toolbox::{Decoder, DecoderCodec, DecoderConfig};

let mut decoder = Decoder::new(DecoderConfig {
    codec: DecoderCodec::AacLc,
    input_sample_rate: 48000,
    input_channels: 2,
})?;

// 1 パケットずつ decode する（前のパケットを next_frame で消費するまで再度 decode しない）
// `packets` は各パケットの圧縮バイト列のイテラブル（例: `Vec<Vec<u8>>`）
for packet in packets {
    decoder.decode(&packet)?;
    while let Some(pcm) = decoder.next_frame()? {
        println!("decoded samples: {}", pcm.len() / 2);
    }
}
decoder.finish()?;

while let Some(pcm) = decoder.next_frame()? {
    println!("decoded samples (flush): {}", pcm.len() / 2);
}
```

## コーデック情報取得

`supported_codecs()` 関数でシステムがサポートするコーデックの情報を取得できます。

```rust
use shiguredo_audio_toolbox::supported_codecs;

for info in supported_codecs() {
    println!("{:?}: decode={}, encode={}",
        info.codec,
        info.decoding.supported,
        info.encoding.supported,
    );
    if info.encoding.supported {
        println!("  bitrate control modes: {:?}", info.encoding.bitrate_control_modes);
    }
}
```

### `AudioCodecType`

| コーデック | `AudioCodecType` |
| --- | --- |
| AAC-LC | `AudioCodecType::AacLc` |
| AAC-HE | `AudioCodecType::AacHe` |
| AAC-HE v2 | `AudioCodecType::AacHeV2` |
| AAC-LD | `AudioCodecType::AacLd` |
| AAC-ELD | `AudioCodecType::AacEld` |
| MP3 | `AudioCodecType::Mp3` |
| Opus | `AudioCodecType::Opus` |
| FLAC | `AudioCodecType::Flac` |
| ALAC | `AudioCodecType::Alac` |

`supported_codecs()` が返す対象は `EncoderCodec` / `DecoderCodec` に対応するコーデック (AAC-LC, MP3, Opus) のみです。

### `AudioCodecInfo`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `codec` | `AudioCodecType` | コーデック種別 |
| `decoding` | `AudioDecodingInfo` | デコード対応情報 |
| `encoding` | `AudioEncodingInfo` | エンコード対応情報 |

### `AudioDecodingInfo`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `supported` | `bool` | デコードが利用可能かどうか |

### `AudioEncodingInfo`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `supported` | `bool` | エンコードが利用可能かどうか |
| `bitrate_control_modes` | `Vec<BitRateControlMode>` | 利用可能なビットレート制御モード |

## 設定

### `EncoderConfig`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `codec` | `EncoderCodec` | コーデック種別 |
| `sample_rate` | `u32` | サンプルレート (Hz)、0 不可 |
| `channels` | `u8` | チャンネル数、0 不可 |
| `bitrate` | `Option<u32>` | ビットレート (bps)、未指定時はバックエンド依存 |
| `bitrate_control_mode` | `Option<BitRateControlMode>` | ビットレート制御モード、未指定時はバックエンド依存 |
| `codec_quality` | `Option<CodecQuality>` | コーデック品質、未指定時はバックエンド依存 |
| `vbr_quality` | `Option<u32>` | VBR 品質 (0-127)、未指定時はバックエンド依存 |

### `DecoderConfig`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `codec` | `DecoderCodec` | コーデック種別 |
| `input_sample_rate` | `u32` | 入力のサンプルレート |
| `input_channels` | `u8` | 入力のチャンネル数、0 不可 |

デコーダー出力フォーマット:

| パラメータ | 値 | 説明 |
| --- | --- | --- |
| サンプルレート | 入力と同じ | 入力に合わせる |
| チャンネル数 | 2 (ステレオ) | 固定 |
| ビット深度 | 16 bit (i16) | 固定 |

### `EncodedFrame`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `data` | `Vec<u8>` | 圧縮データ |
| `samples` | `usize` | フレームに含まれるサンプル数 |

### `Error`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `status` | `i32` | AudioConverter API のステータスコード (OSStatus) |
| `function` | `&'static str` | エラーが発生した API 関数名 |

## サンプル

### sine_to_mp4

正弦波 PCM を AAC-LC エンコードして MP4 ファイルに保存するサンプルです。

```bash
cargo run --example sine_to_mp4 -- --bitrate 256000 --duration 10 --freq 880 --output tone.mp4
```

## スレッド安全性

`Encoder` / `Decoder` は `Send` を実装していません。Apple の AudioConverter API にスレッド間移動の規範的根拠がないためです。

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
