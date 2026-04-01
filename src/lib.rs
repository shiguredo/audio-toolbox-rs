//! [Hisui] 用の [Audio Toolbox] エンコーダー / デコーダー
//!
//! Apple の Audio Toolbox フレームワークの [AudioConverter] API を使用して、
//! PCM 音声データのエンコード / デコードを行う。
//!
//! AudioConverter は低レベル API であり、コンテナフォーマット（MP4 等）の解析機能を持たない。
//! そのため、入力フォーマット（サンプルレート、チャンネル数等）は呼び出し側が指定する必要がある。
//!
//! [Hisui]: https://github.com/shiguredo/hisui
//! [Audio Toolbox]: https://developer.apple.com/documentation/audiotoolbox/
//! [AudioConverter]: https://developer.apple.com/documentation/audiotoolbox/audio_converter
#![warn(missing_docs)]

// macOS 以外ではビルドを許可しない (cargo doc 時は除外)
#[cfg(all(not(target_os = "macos"), not(doc)))]
compile_error!("this crate only supports macOS");

use std::{collections::VecDeque, ffi::c_void, mem::MaybeUninit};

mod codec_info;
mod sys;

pub use codec_info::*;

/// Audio Toolbox API のエラー
///
/// AudioConverter 等の API が返すステータスコード（OSStatus）をラップする。
/// ステータスコード 0 は成功を意味し、それ以外はエラーとなる。
#[derive(Debug)]
pub struct Error {
    /// AudioConverter API が返したステータスコード (OSStatus)
    status: i32,
    /// エラーが発生した API 関数名
    function: &'static str,
}

impl Error {
    /// ステータスコードを検査し、0 以外ならエラーを返す
    fn check(status: i32, function: &'static str) -> Result<(), Self> {
        if status == 0 {
            return Ok(());
        }
        Err(Self { status, function })
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}() failed: status={}",
            env!("CARGO_PKG_NAME"),
            self.function,
            self.status
        )
    }
}

impl std::error::Error for Error {}

// デコーダー出力のチャンネル数（ステレオ固定）
//
// AudioConverter のデコード出力は常にステレオ（2ch）とする。
// 入力がモノラルの場合でも AudioConverter が自動的にアップミックスする。
const OUTPUT_CHANNELS: usize = 2;

// エンコーダー / デコーダーのコールバック内で「入力データが不足している」ことを示す独自エラーコード。
//
// AudioConverterFillComplexBuffer のコールバックから返すことで、
// AudioConverter に「これ以上入力がない」ことを通知する。
// 理想的にはフレームワーク側と確実に衝突しない値を選択するべきだが、
// それを記載したドキュメントが見つからなかったので、実際に動かしてみて安全そうな値を使っている。
const K_NO_MORE_INPUT: i32 = 12345;

// エンコード結果を格納するための一時バッファのサイズ（バイト数）
//
// AudioConverterFillComplexBuffer の出力バッファとして使用する。
// AAC-LC の場合、1 パケットあたりの最大サイズは通常数百バイト程度なので、
// 4096 バイトあれば十分である。
const ENCODE_BUF_SIZE: usize = 4096;

// デコード時の出力バッファサイズ（フレーム数）
//
// AudioConverterFillComplexBuffer の出力バッファとして使用する。
// AAC-LC / MP3 の 1 入力パケットあたりのフレーム数はそれぞれ 1024 / 1152 であり、5760 で十分に覆える。
//
// Opus のパケットあたり最大長（最大 120 ms までの結合）は RFC 6716 §2.1.4 に規定される。
// RFC 8251 は主に参照デコーダ実装（Appendix A）等の修正であり、同節の英語本文は変更しないと明記している。
// 48 kHz では理論上 48000 × 0.12 = 5760 フレームまであり得るため、少なくともその値を確保する。
const DECODE_BUF_FRAMES: usize = 5760;

/// エンコーダーのコーデック種別
///
/// AudioConverter の出力フォーマット（AudioStreamBasicDescription の mFormatID）を決定する。
/// 現在は AAC-LC のみ対応しているが、将来的に他のコーデックを追加可能。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderCodec {
    /// AAC-LC (Low Complexity)
    ///
    /// mFormatID = kAudioFormatMPEG4AAC, mFormatFlags = kMPEG4Object_AAC_LC を使用する。
    /// 1 パケットあたり 1024 フレーム固定。
    AacLc,
}

/// ビットレート制御モード
///
/// AudioConverter の kAudioCodecPropertyBitRateControlMode プロパティに対応する。
/// エンコーダーがビットレートをどのように制御するかを指定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitRateControlMode {
    /// 固定ビットレート (CBR)
    ///
    /// kAudioCodecBitRateControlMode_Constant に対応する。
    /// 出力ビットレートを一定に保つ。
    Constant,
    /// 長期平均ビットレート
    ///
    /// kAudioCodecBitRateControlMode_LongTermAverage に対応する。
    /// 長期的な平均ビットレートを指定値に近づける。
    LongTermAverage,
    /// 制約付き可変ビットレート
    ///
    /// kAudioCodecBitRateControlMode_VariableConstrained に対応する。
    /// VBR だがビットレートの上限に制約を設ける。
    VariableConstrained,
    /// 可変ビットレート (VBR)
    ///
    /// kAudioCodecBitRateControlMode_Variable に対応する。
    /// 品質を一定に保ちつつビットレートを変動させる。
    Variable,
}

/// コーデック品質
///
/// AudioConverter の kAudioConverterCodecQuality プロパティに対応する。
/// エンコード品質と処理速度のトレードオフを指定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecQuality {
    /// 最低品質 / 最高速度 (kAudioConverterQuality_Min = 0)
    Min,
    /// 低品質 (kAudioConverterQuality_Low = 32)
    Low,
    /// 中品質 (kAudioConverterQuality_Medium = 64)
    Medium,
    /// 高品質 (kAudioConverterQuality_High = 96)
    High,
    /// 最高品質 / 最低速度 (kAudioConverterQuality_Max = 127)
    Max,
}

/// エンコーダーの設定
///
/// AudioConverter によるエンコード処理に必要な全パラメーターを保持する。
/// `Option` のフィールドは未指定時に AudioConverter のデフォルト値が使用される。
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// コーデック種別
    ///
    /// AudioConverter の出力フォーマット（AudioStreamBasicDescription）を決定する。
    pub codec: EncoderCodec,

    /// 入力 PCM のサンプルレート (Hz)
    ///
    /// AudioStreamBasicDescription の mSampleRate として入力 / 出力の両方に設定される。
    /// 0 を指定するとエラーが返る。
    pub sample_rate: u32,

    /// 入力 PCM のチャンネル数
    ///
    /// AudioStreamBasicDescription の mChannelsPerFrame として入力 / 出力の両方に設定される。
    /// 0 を指定するとエラーが返る。
    pub channels: u8,

    /// ターゲットビットレート (bps)
    ///
    /// kAudioConverterEncodeBitRate プロパティに対応する。
    /// 未指定時は AudioConverter のデフォルト値が使用される。
    /// 指定可能な値はコーデック / サンプルレート / チャンネル数に依存する。
    /// 無効な値を指定するとエラーが返る。
    pub bitrate: Option<u32>,

    /// ビットレート制御モード
    ///
    /// kAudioCodecPropertyBitRateControlMode プロパティに対応する。
    /// 未指定時は AudioConverter のデフォルト値が使用される。
    pub bitrate_control_mode: Option<BitRateControlMode>,

    /// コーデック品質
    ///
    /// kAudioConverterCodecQuality プロパティに対応する。
    /// 未指定時は AudioConverter のデフォルト値が使用される。
    pub codec_quality: Option<CodecQuality>,

    /// VBR 品質 (0-127)
    ///
    /// kAudioCodecPropertySoundQualityForVBR プロパティに対応する。
    /// 0 が最低品質、127 が最高品質。
    /// `bitrate_control_mode` が `Variable` の場合に有効。
    /// 未指定時は AudioConverter のデフォルト値が使用される。
    pub vbr_quality: Option<u32>,
}

/// エンコーダー
///
/// AudioConverter を使用して PCM 音声データを圧縮フォーマットにエンコードする。
///
/// # 使用フロー
///
/// 1. [`Encoder::new()`] でインスタンスを生成する
/// 2. [`Encoder::encode()`] で PCM データを入力する（複数回呼び出し可能）
/// 3. [`Encoder::next_frame()`] でエンコード済みフレームを取り出す
/// 4. 全データの入力が完了したら [`Encoder::finish()`] を呼び出す
/// 5. 残りのフレームを [`Encoder::next_frame()`] で取り出す
#[derive(Debug)]
pub struct Encoder {
    /// AudioConverter のインスタンス（AudioConverterRef）
    converter: sys::AudioConverterRef,
    /// エンコーダーに設定されたチャンネル数
    channels: usize,
    /// まだエンコードされていない PCM サンプルのバッファ（インターリーブ形式）
    pcm_buf: Vec<i16>,
    /// エンコード済みフレームのキュー（next_frame() で取り出される）
    encoded_frames: VecDeque<EncodedFrame>,
    /// finish() が呼ばれたかどうか
    eos: bool,
}

impl Encoder {
    /// エンコーダーインスタンスを生成する
    ///
    /// AudioConverterNew で入力（PCM）→ 出力（圧縮フォーマット）のコンバーターを作成し、
    /// 各種プロパティを設定する。
    pub fn new(config: EncoderConfig) -> Result<Self, Error> {
        if config.sample_rate == 0 {
            return Err(Error {
                status: -50, // paramErr
                function: "Encoder::new(sample_rate)",
            });
        }
        if config.channels == 0 {
            return Err(Error {
                status: -50, // paramErr
                function: "Encoder::new(channels)",
            });
        }

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate as sys::Float64;
        // PCM の 1 フレームあたりのバイト数 = チャンネル数 × サンプルサイズ (i16 = 2 bytes)
        let bytes_per_frame = (channels * size_of::<i16>()) as sys::UInt32;

        unsafe {
            // 入力フォーマット: リニア PCM (i16, インターリーブ)
            let mut input_format =
                MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();
            // 出力フォーマット: 圧縮コーデック（コーデック種別に応じて設定）
            let mut output_format =
                MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();

            input_format.mSampleRate = sample_rate;
            input_format.mFormatID = sys::kAudioFormatLinearPCM;
            input_format.mFormatFlags =
                sys::kAudioFormatFlagIsSignedInteger | sys::kAudioFormatFlagIsPacked;
            // PCM では mBytesPerPacket = mBytesPerFrame（1 パケット = 1 フレーム）
            input_format.mBytesPerPacket = bytes_per_frame;
            input_format.mFramesPerPacket = 1;
            input_format.mBytesPerFrame = bytes_per_frame;
            input_format.mChannelsPerFrame = channels as sys::UInt32;
            input_format.mBitsPerChannel = 16; // i16 = 16 bits per sample

            // コーデック固有の出力フォーマット設定
            match config.codec {
                EncoderCodec::AacLc => {
                    // 以下の Table 2-6 が参考になる:
                    // https://developer.apple.com/library/archive/documentation/MusicAudio/Reference/CAFSpec/CAF_spec/CAF_spec.html
                    output_format.mSampleRate = sample_rate;
                    output_format.mFormatID = sys::kAudioFormatMPEG4AAC;
                    output_format.mFormatFlags = sys::kMPEG4Object_AAC_LC;
                    output_format.mChannelsPerFrame = channels as sys::UInt32;
                    // 圧縮フォーマットではビット深度は不定
                    output_format.mBitsPerChannel = 0;
                    // AAC-LC は 1 パケットあたり 1024 フレーム固定
                    output_format.mFramesPerPacket = 1024;
                    // 圧縮フォーマットではパケットサイズは可変
                    output_format.mBytesPerPacket = 0;
                }
            }

            let mut converter = std::ptr::null_mut();
            let status = sys::AudioConverterNew(&input_format, &output_format, &mut converter);
            Error::check(status, "AudioConverterNew")?;

            // ビットレート制御モード設定
            //
            // ビットレート設定より先に制御モードを設定する必要がある。
            // 制御モードによって有効なビットレート範囲が変わるため。
            if let Some(mode) = config.bitrate_control_mode {
                let value: u32 = match mode {
                    BitRateControlMode::Constant => sys::kAudioCodecBitRateControlMode_Constant,
                    BitRateControlMode::LongTermAverage => {
                        sys::kAudioCodecBitRateControlMode_LongTermAverage
                    }
                    BitRateControlMode::VariableConstrained => {
                        sys::kAudioCodecBitRateControlMode_VariableConstrained
                    }
                    BitRateControlMode::Variable => sys::kAudioCodecBitRateControlMode_Variable,
                };
                let status = sys::AudioConverterSetProperty(
                    converter,
                    sys::kAudioCodecPropertyBitRateControlMode,
                    size_of::<u32>() as sys::UInt32,
                    (&value as *const u32).cast(),
                );
                Error::check(status, "AudioConverterSetProperty(BitRateControlMode)")?;
            }

            // ビットレート設定
            if let Some(bitrate) = config.bitrate {
                let status = sys::AudioConverterSetProperty(
                    converter,
                    sys::kAudioConverterEncodeBitRate,
                    size_of::<u32>() as sys::UInt32,
                    (&bitrate as *const u32).cast(),
                );
                Error::check(status, "AudioConverterSetProperty(EncodeBitRate)")?;
            }

            // コーデック品質設定
            if let Some(quality) = config.codec_quality {
                let value: u32 = match quality {
                    CodecQuality::Min => sys::kAudioConverterQuality_Min,
                    CodecQuality::Low => sys::kAudioConverterQuality_Low,
                    CodecQuality::Medium => sys::kAudioConverterQuality_Medium,
                    CodecQuality::High => sys::kAudioConverterQuality_High,
                    CodecQuality::Max => sys::kAudioConverterQuality_Max,
                };
                let status = sys::AudioConverterSetProperty(
                    converter,
                    sys::kAudioConverterCodecQuality,
                    size_of::<u32>() as sys::UInt32,
                    (&value as *const u32).cast(),
                );
                Error::check(status, "AudioConverterSetProperty(CodecQuality)")?;
            }

            // VBR 品質設定
            if let Some(vbr_quality) = config.vbr_quality {
                let status = sys::AudioConverterSetProperty(
                    converter,
                    sys::kAudioCodecPropertySoundQualityForVBR,
                    size_of::<u32>() as sys::UInt32,
                    (&vbr_quality as *const u32).cast(),
                );
                Error::check(status, "AudioConverterSetProperty(SoundQualityForVBR)")?;
            }

            Ok(Self {
                converter,
                channels,
                pcm_buf: Vec::new(),
                encoded_frames: VecDeque::new(),
                eos: false,
            })
        }
    }

    /// PCM 音声データをエンコードする
    ///
    /// インターリーブ形式の i16 PCM データを受け取り、内部バッファに蓄積する。
    /// 十分なサンプルが蓄積されると AudioConverter がエンコードを実行する。
    /// エンコード結果は [`Encoder::next_frame()`] で取得できる。
    pub fn encode(&mut self, pcm: &[i16]) -> Result<(), Error> {
        self.pcm_buf.extend_from_slice(pcm);
        while let Some(frame) = self.encode_impl()? {
            self.encoded_frames.push_back(frame);
        }
        Ok(())
    }

    /// エンコーダーに、これ以上データが来ないことを伝える
    ///
    /// 内部バッファに残っている PCM データをすべてエンコードする。
    /// 残りのエンコード結果は [`Encoder::next_frame()`] で取得できる。
    pub fn finish(&mut self) -> Result<(), Error> {
        self.eos = true;
        while let Some(frame) = self.encode_impl()? {
            self.encoded_frames.push_back(frame);
        }
        Ok(())
    }

    /// エンコード済みのフレームを取り出す
    ///
    /// エンコード結果がない場合は `None` を返す。
    pub fn next_frame(&mut self) -> Option<EncodedFrame> {
        self.encoded_frames.pop_front()
    }

    /// AudioConverterFillComplexBuffer を呼び出して 1 パケット分のエンコードを試みる
    ///
    /// エンコードに十分なサンプルがない場合（かつ finish() が呼ばれていない場合）は
    /// コールバックから K_NO_MORE_INPUT を返すことで処理を中断し、None を返す。
    fn encode_impl(&mut self) -> Result<Option<EncodedFrame>, Error> {
        let mut encoded_data = [0; ENCODE_BUF_SIZE];
        // 1 回の呼び出しで 1 パケットのみ要求する
        let mut io_packets = 1;
        let mut output_buffer_list =
            unsafe { MaybeUninit::<sys::AudioBufferList>::zeroed().assume_init() };
        output_buffer_list.mNumberBuffers = 1;
        output_buffer_list.mBuffers[0].mNumberChannels = self.channels as sys::UInt32;
        output_buffer_list.mBuffers[0].mData = encoded_data.as_mut_ptr().cast();
        output_buffer_list.mBuffers[0].mDataByteSize = encoded_data.len() as u32;

        // エンコード前のサンプル数を記録（エンコード後に消費されたサンプル数を計算するため）
        let old_samples = self.pcm_buf.len() / self.channels;
        let status = unsafe {
            sys::AudioConverterFillComplexBuffer(
                self.converter,
                Some(Self::callback),
                (self as *mut Self).cast(),
                &mut io_packets,
                &mut output_buffer_list,
                std::ptr::null_mut(),
            )
        };
        // コールバックが K_NO_MORE_INPUT を返した場合、入力データが不足している
        if status == K_NO_MORE_INPUT {
            return Ok(None);
        }
        Error::check(status, "AudioConverterFillComplexBuffer")?;

        let size = output_buffer_list.mBuffers[0].mDataByteSize as usize;
        if size == 0 {
            return Ok(None);
        }
        Ok(Some(EncodedFrame {
            data: encoded_data[..size].to_vec(),
            // 消費されたサンプル数 = エンコード前のサンプル数 - エンコード後の残りサンプル数
            samples: old_samples - (self.pcm_buf.len() / self.channels),
        }))
    }

    /// AudioConverterFillComplexBuffer のコールバック（エンコーダー用）
    ///
    /// AudioConverter がエンコード用の入力 PCM データを要求する際に呼び出される。
    /// pcm_buf から要求されたサンプル数分のデータを AudioConverter に渡す。
    ///
    /// # 注意
    ///
    /// Video Toolbox とは異なり、Audio Toolbox ではコールバックが
    /// AudioConverterFillComplexBuffer と同じスレッド内で同期的に実行される。
    unsafe extern "C" fn callback(
        _in_audio_converter: sys::AudioConverterRef,
        io_number_data_packets: *mut u32,
        io_data: *mut sys::AudioBufferList,
        _out_data_packet_description: *mut *mut sys::AudioStreamPacketDescription,
        in_user_data: *mut c_void,
    ) -> i32 {
        unsafe {
            let this: &mut Encoder = &mut *(in_user_data as *mut Encoder);
            let channels = this.channels;
            let packets = *io_number_data_packets;

            // 要求されたパケット数分の PCM データがバッファにあるか確認する。
            // finish() が呼ばれていない場合はデータ不足として K_NO_MORE_INPUT を返す。
            // finish() 後は残りのデータで処理を続ける。
            if !this.eos && this.pcm_buf.len() < packets as usize * channels {
                return K_NO_MORE_INPUT;
            }

            // 実際に提供可能なパケット数に調整する
            *io_number_data_packets = packets.min((this.pcm_buf.len() / channels) as u32);

            let packets = *io_number_data_packets;
            let io_data = &mut *io_data;
            // バイト数 = パケット数 × チャンネル数 × サンプルサイズ
            let size = packets * channels as u32 * size_of::<i16>() as u32;
            io_data.mNumberBuffers = 1;
            io_data.mBuffers[0].mNumberChannels = channels as sys::UInt32;

            // pcm_buf の先頭から必要なサンプル数分を AudioConverter のバッファにコピーする
            let num_samples = (size / size_of::<i16>() as u32) as usize;
            std::slice::from_raw_parts_mut(io_data.mBuffers[0].mData.cast(), num_samples)
                .copy_from_slice(&this.pcm_buf[..num_samples]);

            io_data.mBuffers[0].mDataByteSize = size;
            // コピー済みのデータをバッファから削除する
            this.pcm_buf.drain(0..packets as usize * channels);
        }
        sys::noErr as i32
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // AudioConverter のリソースを解放する
        unsafe {
            sys::AudioConverterDispose(self.converter);
        }
    }
}

// AudioConverter 自体はスレッドセーフではないが、
// Encoder は &mut self を要求するため、同時アクセスは Rust の型システムで防がれる。
// Sync は実装しない: &Encoder から &mut self メソッドは呼べないが、
// AudioConverterRef が内部的にスレッドセーフでないため Sync を保証できない。
unsafe impl Send for Encoder {}

/// エンコードされた音声フレーム
///
/// 1 回のエンコードで生成される圧縮データとメタデータを保持する。
#[derive(Debug)]
pub struct EncodedFrame {
    /// 圧縮データ（コーデック固有のフォーマット）
    pub data: Vec<u8>,

    /// このフレームに含まれている PCM サンプル数（チャンネルあたり）
    ///
    /// AAC-LC の場合は通常 1024。
    pub samples: usize,
}

/// デコーダーのコーデック種別
///
/// AudioConverter の入力フォーマット（AudioStreamBasicDescription の mFormatID）を決定する。
/// AudioConverter API がサポートするコーデックのみ対応している。
/// FLAC は AudioConverter では非対応のため含まれていない（ExtAudioFile API が必要）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoderCodec {
    /// AAC-LC (Low Complexity)
    ///
    /// mFormatID = kAudioFormatMPEG4AAC, mFormatFlags = kMPEG4Object_AAC_LC を使用する。
    /// 1 パケットあたり 1024 フレーム固定。
    AacLc,
    /// MP3 (MPEG Audio Layer 3)
    ///
    /// mFormatID = kAudioFormatMPEGLayer3 を使用する。
    /// 1 パケットあたり 1152 フレーム固定。
    Mp3,
    /// Opus
    ///
    /// mFormatID = kAudioFormatOpus を使用する。
    /// パケットあたりのフレーム数は可変（mFramesPerPacket = 0）。
    Opus,
}

impl DecoderCodec {
    /// AudioStreamBasicDescription の mFormatID を返す
    fn format_id(self) -> u32 {
        match self {
            Self::AacLc => sys::kAudioFormatMPEG4AAC,
            Self::Mp3 => sys::kAudioFormatMPEGLayer3,
            Self::Opus => sys::kAudioFormatOpus,
        }
    }

    /// AudioStreamBasicDescription の mFormatFlags を返す
    ///
    /// AAC-LC のみプロファイル情報（kMPEG4Object_AAC_LC）を設定する。
    /// MP3 / Opus はフラグ不要のため 0 を返す。
    fn format_flags(self) -> u32 {
        match self {
            Self::AacLc => sys::kMPEG4Object_AAC_LC,
            _ => 0,
        }
    }

    /// AudioStreamBasicDescription の mFramesPerPacket を返す
    ///
    /// 圧縮フォーマットでは 1 パケットに含まれるフレーム数が固定されている場合がある。
    /// 可変長の場合は 0 を設定する。
    fn frames_per_packet(self) -> u32 {
        match self {
            Self::AacLc => 1024,
            Self::Mp3 => 1152,
            // Opus は可変長パケットなので 0 を設定する
            Self::Opus => 0,
        }
    }
}

/// デコーダーの設定
///
/// AudioConverter によるデコード処理に必要なパラメーターを保持する。
///
/// AudioConverter は低レベル API であり、コンテナフォーマット（MP4 等）の解析機能を持たない。
/// そのため、入力フォーマット（サンプルレート、チャンネル数）は呼び出し側が指定する必要がある。
///
/// AAC / MP3 / Opus いずれも AudioConverter のデコーダーにコーデック固有の設定項目はない。
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// コーデック種別
    ///
    /// AudioConverter の入力フォーマット（AudioStreamBasicDescription）を決定する。
    pub codec: DecoderCodec,

    /// 入力のサンプルレート (Hz)
    ///
    /// AudioStreamBasicDescription の mSampleRate として設定される。
    /// 出力のサンプルレートもこの値と同じになる（リサンプリングは行わない）。
    pub input_sample_rate: u32,

    /// 入力のチャンネル数
    ///
    /// AudioStreamBasicDescription の mChannelsPerFrame として設定される。
    /// 出力はステレオ（2ch）固定であり、入力チャンネル数と異なる場合は
    /// AudioConverter が自動的にチャンネルミックスを行う。
    /// 0 を指定するとエラーが返る。
    pub input_channels: u8,
}

/// デコーダー
///
/// AudioConverter を使用して圧縮音声データを PCM にデコードする。
/// 出力はステレオ（2ch）16bit PCM 固定。
///
/// # 使用フロー
///
/// 1. [`Decoder::new()`] でインスタンスを生成する
/// 2. [`Decoder::decode()`] で **1 パケット分**の圧縮データを入力する（1 回につき 1 パケット。前のパケットを
///    [`Decoder::next_frame()`] で消費するまで、再度 `decode` してはならない）
/// 3. [`Decoder::next_frame()`] でデコード済み PCM を取り出す（`Ok(None)` になるまで繰り返す）
/// 4. 複数パケットがある場合は 2〜3 を繰り返す
/// 5. 全データの入力が完了したら [`Decoder::finish()`] を呼び出す
/// 6. 残りの PCM を [`Decoder::next_frame()`] で取り出す
#[derive(Debug)]
pub struct Decoder {
    /// AudioConverter のインスタンス（AudioConverterRef）
    converter: sys::AudioConverterRef,
    /// 次の `next_frame` までに渡す未処理パケット（高々 1 パケット分）
    encoded_buf: Vec<u8>,
    /// finish() が呼ばれたかどうか
    eos: bool,
    /// AudioConverterFillComplexBuffer のコールバックで使用するパケット記述情報
    ///
    /// 圧縮フォーマットでは各パケットのサイズが可変なため、
    /// AudioStreamPacketDescription でパケットの位置とサイズを通知する必要がある。
    /// Box で確保しているのは、コールバック内でポインタを返す必要があるため。
    packet_desc: Box<sys::AudioStreamPacketDescription>,
    /// 入力のチャンネル数（コールバック内で使用）
    input_channels: u8,
}

impl Decoder {
    /// デコーダーインスタンスを生成する
    ///
    /// AudioConverterNew で入力（圧縮フォーマット）→ 出力（PCM）のコンバーターを作成する。
    /// 出力チャネル数はステレオ固定で、サンプルレートは入力と同じとなる。
    pub fn new(config: DecoderConfig) -> Result<Self, Error> {
        if config.input_sample_rate == 0 {
            return Err(Error {
                status: -50, // paramErr
                function: "Decoder::new(input_sample_rate)",
            });
        }
        if config.input_channels == 0 {
            return Err(Error {
                status: -50, // paramErr
                function: "Decoder::new(input_channels)",
            });
        }

        unsafe {
            // 入力フォーマット: 圧縮コーデック
            let mut input_format =
                MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();
            // 出力フォーマット: リニア PCM (i16, インターリーブ, ステレオ)
            let mut output_format =
                MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();

            // 入力フォーマットの設定
            //
            // 圧縮フォーマットでは mBitsPerChannel = 0, mBytesPerPacket = 0 とする。
            // これらは可変であり、AudioConverter が内部で処理する。
            input_format.mSampleRate = config.input_sample_rate as f64;
            input_format.mFormatID = config.codec.format_id();
            input_format.mFormatFlags = config.codec.format_flags();
            input_format.mChannelsPerFrame = config.input_channels as sys::UInt32;
            input_format.mFramesPerPacket = config.codec.frames_per_packet();
            input_format.mBitsPerChannel = 0;
            input_format.mBytesPerPacket = 0;

            // 出力フォーマットの設定（ステレオ 16bit PCM 固定）
            let output_bytes_per_frame = (OUTPUT_CHANNELS * size_of::<i16>()) as sys::UInt32;
            output_format.mFormatID = sys::kAudioFormatLinearPCM;
            output_format.mFormatFlags =
                sys::kAudioFormatFlagIsSignedInteger | sys::kAudioFormatFlagIsPacked;
            output_format.mBytesPerPacket = output_bytes_per_frame;
            output_format.mFramesPerPacket = 1;
            output_format.mBytesPerFrame = output_bytes_per_frame;
            output_format.mChannelsPerFrame = OUTPUT_CHANNELS as sys::UInt32;
            output_format.mBitsPerChannel = 16; // i16 = 16 bits per sample

            // 出力サンプルレートは入力に合わせる
            //
            // 固定値にすると AudioConverter がリサンプリングを行い、
            // デコード後の音声が変になることがあるため。
            output_format.mSampleRate = config.input_sample_rate as f64;

            let mut converter = std::ptr::null_mut();
            let status = sys::AudioConverterNew(&input_format, &output_format, &mut converter);
            Error::check(status, "AudioConverterNew")?;

            Ok(Self {
                converter,
                encoded_buf: Vec::new(),
                eos: false,
                // コールバック内でポインタを返すため Box で確保する
                packet_desc: Box::new(sys::AudioStreamPacketDescription {
                    mStartOffset: 0,
                    mVariableFramesInPacket: 0,
                    mDataByteSize: 0,
                }),
                input_channels: config.input_channels,
            })
        }
    }

    /// 圧縮データをデコーダーに入力する
    ///
    /// 1 回の呼び出しで 1 パケット分のデータを渡す。
    /// 実際のデコード処理は [`Decoder::next_frame()`] 呼び出し時に行われる。
    ///
    /// 前回の `decode` で入力したパケットが [`Decoder::next_frame()`] により消費されるまで、
    /// 再度 `decode` すると [`Error`] を返す（連結による誤デコードを防ぐ）。
    pub fn decode(&mut self, encoded: &[u8]) -> Result<(), Error> {
        if !self.encoded_buf.is_empty() {
            return Err(Error {
                status: -50, // paramErr
                function: "Decoder::decode(previous packet not consumed)",
            });
        }
        self.encoded_buf.extend_from_slice(encoded);
        Ok(())
    }

    /// デコーダーに、これ以上データが来ないことを伝える
    pub fn finish(&mut self) -> Result<(), Error> {
        self.eos = true;
        Ok(())
    }

    /// デコードされた PCM データを取得する
    ///
    /// `Ok(Some(pcm))` が返される間はこのメソッドをループして呼び出す。
    /// `Ok(None)` が返されたら、それ以上デコード結果がないことを意味する。
    ///
    /// 返される `Vec<i16>` はインターリーブ形式のステレオ PCM データ。
    pub fn next_frame(&mut self) -> Result<Option<Vec<i16>>, Error> {
        self.decode_impl()
    }

    /// AudioConverterFillComplexBuffer を呼び出してデコードを実行する
    fn decode_impl(&mut self) -> Result<Option<Vec<i16>>, Error> {
        // 出力バッファ: ステレオ PCM (DECODE_BUF_FRAMES フレーム分)
        let mut pcm_buf = vec![0i16; DECODE_BUF_FRAMES * OUTPUT_CHANNELS];

        let mut io_packets = DECODE_BUF_FRAMES as u32;

        let mut output_buffer_list =
            unsafe { MaybeUninit::<sys::AudioBufferList>::zeroed().assume_init() };
        output_buffer_list.mNumberBuffers = 1;
        output_buffer_list.mBuffers[0].mNumberChannels = OUTPUT_CHANNELS as sys::UInt32;
        output_buffer_list.mBuffers[0].mData = pcm_buf.as_mut_ptr().cast();
        output_buffer_list.mBuffers[0].mDataByteSize = (pcm_buf.len() * size_of::<i16>()) as u32;

        let status = unsafe {
            sys::AudioConverterFillComplexBuffer(
                self.converter,
                Some(Self::callback),
                (self as *mut Self).cast(),
                &mut io_packets,
                &mut output_buffer_list,
                std::ptr::null_mut(),
            )
        };
        // コールバックが K_NO_MORE_INPUT を返した場合、入力データが不足している
        if status == K_NO_MORE_INPUT {
            return Ok(None);
        }
        Error::check(status, "AudioConverterFillComplexBuffer")?;

        // デコード処理が完了したら入力バッファをクリアする
        //
        // AudioConverter は 1 回のコールバックで入力バッファの全データを消費するため、
        // 部分的に消費されることはない。
        self.encoded_buf.clear();

        let byte_size = output_buffer_list.mBuffers[0].mDataByteSize as usize;
        let size = byte_size / size_of::<i16>();
        if size == 0 {
            return Ok(None);
        }

        pcm_buf.truncate(size);
        Ok(Some(pcm_buf))
    }

    /// AudioConverterFillComplexBuffer のコールバック（デコーダー用）
    ///
    /// AudioConverter がデコード用の圧縮入力データを要求する際に呼び出される。
    /// encoded_buf のデータを AudioConverter に渡す。
    ///
    /// # 注意
    ///
    /// Video Toolbox とは異なり、Audio Toolbox ではコールバックが
    /// AudioConverterFillComplexBuffer と同じスレッド内で同期的に実行される。
    unsafe extern "C" fn callback(
        _in_audio_converter: sys::AudioConverterRef,
        io_number_data_packets: *mut u32,
        io_data: *mut sys::AudioBufferList,
        out_data_packet_description: *mut *mut sys::AudioStreamPacketDescription,
        in_user_data: *mut c_void,
    ) -> i32 {
        unsafe {
            let this: &mut Decoder = &mut *(in_user_data as *mut Decoder);
            let requested_packets = *io_number_data_packets;

            if this.encoded_buf.is_empty() {
                if this.eos {
                    // finish() 後かつデータなし → 正常終了を通知する
                    *io_number_data_packets = 0;
                    return sys::noErr as i32;
                }
                // データ不足 → K_NO_MORE_INPUT を返して処理を中断する
                return K_NO_MORE_INPUT;
            }

            // 入力は 1 パケット分のみ（decode が連結を許さないため encoded_buf は 1 パケット相当）
            *io_number_data_packets = requested_packets.min(1);

            let io_data = &mut *io_data;
            io_data.mNumberBuffers = 1;
            io_data.mBuffers[0].mNumberChannels = this.input_channels as sys::UInt32;
            // encoded_buf のポインタを直接渡す（コピーは発生しない）
            io_data.mBuffers[0].mData = this.encoded_buf.as_mut_ptr().cast();
            io_data.mBuffers[0].mDataByteSize = this.encoded_buf.len() as u32;

            // パケット記述情報の設定
            //
            // 圧縮フォーマットではパケットサイズが可変なため、
            // AudioStreamPacketDescription で各パケットの位置とサイズを通知する必要がある。
            // out_data_packet_description が null でない場合のみ設定する
            // （PCM 等の固定サイズフォーマットでは null が渡される）。
            if !out_data_packet_description.is_null() {
                this.packet_desc.mStartOffset = 0;
                this.packet_desc.mVariableFramesInPacket = 0;
                this.packet_desc.mDataByteSize = this.encoded_buf.len() as u32;

                *out_data_packet_description = &mut *this.packet_desc as *mut _;
            }
        }
        sys::noErr as i32
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // AudioConverter のリソースを解放する
        unsafe {
            sys::AudioConverterDispose(self.converter);
        }
    }
}

// AudioConverter 自体はスレッドセーフではないが、
// Decoder は &mut self を要求するため、同時アクセスは Rust の型システムで防がれる。
// Sync は実装しない: &Decoder から &mut self メソッドは呼べないが、
// AudioConverterRef が内部的にスレッドセーフでないため Sync を保証できない。
unsafe impl Send for Decoder {}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: u32 = 48000;
    const TEST_CHANNELS: u8 = 2;

    fn encoder_config(bitrate: Option<u32>) -> EncoderConfig {
        EncoderConfig {
            codec: EncoderCodec::AacLc,
            sample_rate: TEST_SAMPLE_RATE,
            channels: TEST_CHANNELS,
            bitrate,
            bitrate_control_mode: None,
            codec_quality: None,
            vbr_quality: None,
        }
    }

    #[test]
    fn init_encoder() {
        // ビットレート指定あり
        assert!(Encoder::new(encoder_config(Some(128_000))).is_ok());

        // ビットレート指定なし（バックエンドデフォルト）
        assert!(Encoder::new(encoder_config(None)).is_ok());

        // 無効なビットレート
        assert!(Encoder::new(encoder_config(Some(1_000))).is_err());
    }

    #[test]
    fn encode_silent() {
        let mut encoder =
            Encoder::new(encoder_config(Some(128_000))).expect("create encoder error");
        let mut sample_count = 0;

        for _ in 0..100 {
            encoder
                .encode(&[0; 100 * TEST_CHANNELS as usize])
                .expect("encode error");
            while let Some(encoded) = encoder.next_frame() {
                sample_count += encoded.samples;
            }
        }
        encoder.finish().expect("finish error");
        while let Some(encoded) = encoder.next_frame() {
            sample_count += encoded.samples;
        }

        assert_eq!(sample_count, 100 * 100);
    }

    #[test]
    fn decode_aac_silent() {
        // 有効な AAC データを取得するためにエンコーダーを使用する
        let mut encoder =
            Encoder::new(encoder_config(Some(128_000))).expect("create encoder error");
        let mut decoder = Decoder::new(DecoderConfig {
            codec: DecoderCodec::AacLc,
            input_sample_rate: TEST_SAMPLE_RATE,
            input_channels: TEST_CHANNELS,
        })
        .expect("create decoder error");

        // 無音のオーディオをエンコードする（パケット単位でリスト化する）
        let mut packets: Vec<Vec<u8>> = Vec::new();
        encoder
            .encode(&[0; 1024 * TEST_CHANNELS as usize])
            .expect("encode error");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data);
        }
        encoder.finish().expect("finish error");
        while let Some(encoded) = encoder.next_frame() {
            packets.push(encoded.data);
        }

        // 1 パケットずつ decode → next_frame（連結して 1 回の decode に渡さない）
        let mut total_decoded = Vec::new();
        for packet in packets {
            decoder.decode(&packet).expect("decode error");
            while let Some(data) = decoder.next_frame().expect("decode error") {
                total_decoded.extend(data);
            }
        }
        decoder.finish().expect("finish error");
        while let Some(data) = decoder.next_frame().expect("decode error") {
            total_decoded.extend(data);
        }

        // デコード結果を確認する
        // 出力にはエンコーダーのプライミングサンプルが含まれるため、入力の 1024 フレーム以上になりうる
        assert!(total_decoded.len() >= 1024 * TEST_CHANNELS as usize);
        assert!(total_decoded.iter().all(|v| *v == 0));
    }

    #[test]
    fn init_decoder_mp3() {
        let result = Decoder::new(DecoderConfig {
            codec: DecoderCodec::Mp3,
            input_sample_rate: 44100,
            input_channels: 2,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn init_decoder_opus() {
        let result = Decoder::new(DecoderConfig {
            codec: DecoderCodec::Opus,
            input_sample_rate: 48000,
            input_channels: 2,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_supported_codecs() {
        let codecs = supported_codecs();

        // 9 種類のコーデックが返る
        assert_eq!(codecs.len(), 9);

        // AAC-LC はエンコード・デコード両方サポートされている
        let aac_lc = codecs
            .iter()
            .find(|c| c.codec == AudioCodecType::AacLc)
            .unwrap();
        assert!(aac_lc.decoding.supported);
        assert!(aac_lc.encoding.supported);
        // AAC-LC はビットレート制御モードが少なくとも 1 つある
        assert!(!aac_lc.encoding.bitrate_control_modes.is_empty());

        // MP3 はデコードのみサポートされている
        let mp3 = codecs
            .iter()
            .find(|c| c.codec == AudioCodecType::Mp3)
            .unwrap();
        assert!(mp3.decoding.supported);
        assert!(!mp3.encoding.supported);

        // ALAC はエンコード・デコード両方サポートされている
        let alac = codecs
            .iter()
            .find(|c| c.codec == AudioCodecType::Alac)
            .unwrap();
        assert!(alac.decoding.supported);
        assert!(alac.encoding.supported);
    }
}
