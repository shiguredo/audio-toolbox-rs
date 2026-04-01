//! コーデック情報の照会

use std::mem::MaybeUninit;

use crate::{BitRateControlMode, sys};

/// オーディオコーデック種別
///
/// AudioToolbox では各コーデックに個別の format ID (`kAudioFormat*`) が割り当てられている。
/// AAC のバリアント（LC, HE, HE v2, LD, ELD）も API 上は別のコーデックとして扱われる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodecType {
    /// AAC-LC (Low Complexity)
    AacLc,
    /// AAC-HE (High Efficiency / SBR)
    AacHe,
    /// AAC-HE v2 (High Efficiency v2 / SBR + PS)
    AacHeV2,
    /// AAC-LD (Low Delay)
    AacLd,
    /// AAC-ELD (Enhanced Low Delay)
    AacEld,
    /// MP3 (MPEG Audio Layer 3)
    Mp3,
    /// Opus
    Opus,
    /// FLAC (Free Lossless Audio Codec)
    Flac,
    /// ALAC (Apple Lossless Audio Codec)
    Alac,
}

impl AudioCodecType {
    /// 列挙に含まれるすべてのコーデック種別を返す（順序固定）
    pub fn all() -> &'static [Self] {
        &[
            Self::AacLc,
            Self::AacHe,
            Self::AacHeV2,
            Self::AacLd,
            Self::AacEld,
            Self::Mp3,
            Self::Opus,
            Self::Flac,
            Self::Alac,
        ]
    }

    /// AudioStreamBasicDescription の mFormatID を返す
    fn format_id(self) -> u32 {
        match self {
            Self::AacLc => sys::kAudioFormatMPEG4AAC,
            Self::AacHe => sys::kAudioFormatMPEG4AAC_HE,
            Self::AacHeV2 => sys::kAudioFormatMPEG4AAC_HE_V2,
            Self::AacLd => sys::kAudioFormatMPEG4AAC_LD,
            Self::AacEld => sys::kAudioFormatMPEG4AAC_ELD,
            Self::Mp3 => sys::kAudioFormatMPEGLayer3,
            Self::Opus => sys::kAudioFormatOpus,
            Self::Flac => sys::kAudioFormatFLAC,
            Self::Alac => sys::kAudioFormatAppleLossless,
        }
    }

    /// AudioStreamBasicDescription の mFormatFlags を返す
    ///
    /// AAC-LC のみ MPEG-4 Audio Object Type を設定する。
    /// 他のコーデックは format ID で種別が決まるためフラグ不要。
    fn format_flags(self) -> u32 {
        match self {
            Self::AacLc => sys::kMPEG4Object_AAC_LC,
            _ => 0,
        }
    }

    /// AudioStreamBasicDescription の mFramesPerPacket を返す
    ///
    /// 圧縮フォーマットの 1 パケットあたりのフレーム数。
    /// 可変長の場合は 0 を返す。
    fn frames_per_packet(self) -> u32 {
        match self {
            Self::AacLc => 1024,
            Self::AacHe | Self::AacHeV2 => 2048,
            Self::AacLd | Self::AacEld => 480,
            Self::Mp3 => 1152,
            // Opus / FLAC / ALAC は可変長
            Self::Opus | Self::Flac | Self::Alac => 0,
        }
    }

    /// ロスレスコーデックかどうかを返す
    ///
    /// ロスレスコーデックではエンコード出力の mBitsPerChannel に
    /// 入力のビット深度を設定する必要がある。
    fn is_lossless(self) -> bool {
        matches!(self, Self::Flac | Self::Alac)
    }
}

/// コーデックごとの情報
#[derive(Debug, Clone, PartialEq)]
pub struct AudioCodecInfo {
    /// コーデック種別
    pub codec: AudioCodecType,
    /// デコード情報
    pub decoding: AudioDecodingInfo,
    /// エンコード情報
    pub encoding: AudioEncodingInfo,
}

/// デコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDecodingInfo {
    /// デコードが可能か
    pub supported: bool,
}

/// エンコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioEncodingInfo {
    /// エンコードが可能か
    pub supported: bool,
    /// 対応するビットレート制御モード（エンコード非対応の場合は空）
    pub bitrate_control_modes: Vec<BitRateControlMode>,
}

/// このバックエンドで利用可能なオーディオコーデック情報の一覧を返す
#[cfg(target_os = "macos")]
pub fn supported_codecs() -> Vec<AudioCodecInfo> {
    AudioCodecType::all()
        .iter()
        .map(|&codec| AudioCodecInfo {
            codec,
            decoding: probe_decoding(codec),
            encoding: probe_encoding(codec),
        })
        .collect()
}

/// AudioFormatGetPropertyInfo でデコード対応を判定する
///
/// kAudioFormatProperty_Decoders で指定した format ID のデコーダーが
/// 登録されているかどうかを確認する。
#[cfg(target_os = "macos")]
fn probe_decoding(codec: AudioCodecType) -> AudioDecodingInfo {
    let format_id = codec.format_id();
    let mut size: u32 = 0;

    let status = unsafe {
        sys::AudioFormatGetPropertyInfo(
            sys::kAudioFormatProperty_Decoders,
            size_of::<u32>() as u32,
            &format_id as *const u32 as *const _,
            &mut size,
        )
    };

    AudioDecodingInfo {
        supported: status == 0 && size > 0,
    }
}

/// AudioFormatGetPropertyInfo でエンコード対応を判定し、
/// 対応している場合はビットレート制御モードも照会する
#[cfg(target_os = "macos")]
fn probe_encoding(codec: AudioCodecType) -> AudioEncodingInfo {
    let format_id = codec.format_id();
    let mut size: u32 = 0;

    let status = unsafe {
        sys::AudioFormatGetPropertyInfo(
            sys::kAudioFormatProperty_Encoders,
            size_of::<u32>() as u32,
            &format_id as *const u32 as *const _,
            &mut size,
        )
    };

    let supported = status == 0 && size > 0;

    let bitrate_control_modes = if supported {
        query_bitrate_control_modes(codec)
    } else {
        Vec::new()
    };

    AudioEncodingInfo {
        supported,
        bitrate_control_modes,
    }
}

/// AudioConverter を作成してビットレート制御モードの対応状況を照会する
///
/// 代表的な設定（48kHz, ステレオ）で AudioConverter を作成し、
/// 各ビットレート制御モードの設定を試みることで対応状況を判定する。
/// AudioConverter の作成に失敗した場合は空のリストを返す。
#[cfg(target_os = "macos")]
fn query_bitrate_control_modes(codec: AudioCodecType) -> Vec<BitRateControlMode> {
    let converter = match create_probe_converter(codec) {
        Some(c) => c,
        None => return Vec::new(),
    };

    // 各ビットレート制御モードの設定を試みる
    let candidates = [
        (
            sys::kAudioCodecBitRateControlMode_Constant,
            BitRateControlMode::Constant,
        ),
        (
            sys::kAudioCodecBitRateControlMode_LongTermAverage,
            BitRateControlMode::LongTermAverage,
        ),
        (
            sys::kAudioCodecBitRateControlMode_VariableConstrained,
            BitRateControlMode::VariableConstrained,
        ),
        (
            sys::kAudioCodecBitRateControlMode_Variable,
            BitRateControlMode::Variable,
        ),
    ];

    let mut modes = Vec::new();
    for &(raw_value, mode) in &candidates {
        let status = unsafe {
            sys::AudioConverterSetProperty(
                converter,
                sys::kAudioCodecPropertyBitRateControlMode,
                size_of::<u32>() as u32,
                &raw_value as *const u32 as *const _,
            )
        };
        if status == 0 {
            modes.push(mode);
        }
    }

    unsafe {
        sys::AudioConverterDispose(converter);
    }

    modes
}

/// ビットレート制御モード照会用の AudioConverter を作成する
///
/// 48kHz ステレオの PCM → 指定コーデックのコンバーターを作成する。
/// 作成に失敗した場合は None を返す。
#[cfg(target_os = "macos")]
fn create_probe_converter(codec: AudioCodecType) -> Option<sys::AudioConverterRef> {
    let channels: u32 = 2;
    let sample_rate: f64 = 48000.0;
    let bytes_per_frame = channels * size_of::<i16>() as u32;

    unsafe {
        let mut input_format =
            MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();
        let mut output_format =
            MaybeUninit::<sys::AudioStreamBasicDescription>::zeroed().assume_init();

        // 入力: リニア PCM (i16, インターリーブ)
        input_format.mSampleRate = sample_rate;
        input_format.mFormatID = sys::kAudioFormatLinearPCM;
        input_format.mFormatFlags =
            sys::kAudioFormatFlagIsSignedInteger | sys::kAudioFormatFlagIsPacked;
        input_format.mBytesPerPacket = bytes_per_frame;
        input_format.mFramesPerPacket = 1;
        input_format.mBytesPerFrame = bytes_per_frame;
        input_format.mChannelsPerFrame = channels;
        input_format.mBitsPerChannel = 16;

        // 出力: 指定コーデック
        output_format.mSampleRate = sample_rate;
        output_format.mFormatID = codec.format_id();
        output_format.mFormatFlags = codec.format_flags();
        output_format.mChannelsPerFrame = channels;
        output_format.mFramesPerPacket = codec.frames_per_packet();
        output_format.mBytesPerPacket = 0;
        // ロスレスコーデックでは入力のビット深度を設定する
        output_format.mBitsPerChannel = if codec.is_lossless() { 16 } else { 0 };

        let mut converter = std::ptr::null_mut();
        let status = sys::AudioConverterNew(&input_format, &output_format, &mut converter);
        if status != 0 || converter.is_null() {
            return None;
        }

        Some(converter)
    }
}
