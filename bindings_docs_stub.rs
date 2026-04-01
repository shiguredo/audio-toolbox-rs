// docs.rs / Linux CI 向けの最小バインディングスタブ（include! 用。//! は使わない）。
// Audio Toolbox の実ヘッダは使わず、ドキュメント生成時の型チェックに必要な識別子だけを揃える。

pub type SInt32 = i32;
pub type SInt64 = i64;
pub type UInt32 = u32;
pub type Float64 = f64;
pub type OSStatus = SInt32;
pub type AudioFormatID = UInt32;
pub type AudioFormatFlags = UInt32;
pub type AudioConverterPropertyID = UInt32;
pub type AudioFormatPropertyID = UInt32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct OpaqueAudioConverter {
    _private: [u8; 0],
}
pub type AudioConverterRef = *mut OpaqueAudioConverter;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AudioBuffer {
    pub mNumberChannels: UInt32,
    pub mDataByteSize: UInt32,
    pub mData: *mut ::std::os::raw::c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AudioBufferList {
    pub mNumberBuffers: UInt32,
    pub mBuffers: [AudioBuffer; 1usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AudioStreamBasicDescription {
    pub mSampleRate: Float64,
    pub mFormatID: AudioFormatID,
    pub mFormatFlags: AudioFormatFlags,
    pub mBytesPerPacket: UInt32,
    pub mFramesPerPacket: UInt32,
    pub mBytesPerFrame: UInt32,
    pub mChannelsPerFrame: UInt32,
    pub mBitsPerChannel: UInt32,
    pub mReserved: UInt32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AudioStreamPacketDescription {
    pub mStartOffset: SInt64,
    pub mVariableFramesInPacket: UInt32,
    pub mDataByteSize: UInt32,
}

pub const noErr: OSStatus = 0;
pub const kAudio_ParamError: OSStatus = -50;

pub const kAudioFormatLinearPCM: AudioFormatID = 1_819_304_813;
pub const kAudioFormatFlagIsSignedInteger: AudioFormatFlags = 4;
pub const kAudioFormatFlagIsPacked: AudioFormatFlags = 8;
pub const kAudioFormatMPEG4AAC: AudioFormatID = 1_633_772_320;
pub const kMPEG4Object_AAC_LC: AudioFormatFlags = 2;
pub const kAudioFormatMPEG4AAC_HE: AudioFormatID = 1_633_772_392;
pub const kAudioFormatMPEG4AAC_HE_V2: AudioFormatID = 1_633_772_400;
pub const kAudioFormatMPEG4AAC_LD: AudioFormatID = 1_633_772_396;
pub const kAudioFormatMPEG4AAC_ELD: AudioFormatID = 1_633_772_389;
pub const kAudioFormatMPEGLayer3: AudioFormatID = 778_924_083;
pub const kAudioFormatOpus: AudioFormatID = 1_869_641_075;
pub const kAudioFormatFLAC: AudioFormatID = 1_718_378_851;
pub const kAudioFormatAppleLossless: AudioFormatID = 1_634_492_771;

pub const kAudioCodecPropertyBitRateControlMode: AudioConverterPropertyID = 1_633_903_206;
pub const kAudioCodecPropertySoundQualityForVBR: AudioConverterPropertyID = 1_986_163_313;
pub const kAudioCodecBitRateControlMode_Constant: UInt32 = 0;
pub const kAudioCodecBitRateControlMode_LongTermAverage: UInt32 = 1;
pub const kAudioCodecBitRateControlMode_VariableConstrained: UInt32 = 2;
pub const kAudioCodecBitRateControlMode_Variable: UInt32 = 3;

pub const kAudioConverterEncodeBitRate: AudioConverterPropertyID = 1_651_663_220;
pub const kAudioConverterCodecQuality: AudioConverterPropertyID = 1_667_527_029;
pub const kAudioConverterQuality_Min: UInt32 = 0;
pub const kAudioConverterQuality_Low: UInt32 = 32;
pub const kAudioConverterQuality_Medium: UInt32 = 64;
pub const kAudioConverterQuality_High: UInt32 = 96;
pub const kAudioConverterQuality_Max: UInt32 = 127;

pub const kAudioFormatProperty_Decoders: AudioFormatPropertyID = 1_635_148_901;
pub const kAudioFormatProperty_Encoders: AudioFormatPropertyID = 1_635_149_166;

pub type AudioConverterComplexInputDataProc = ::std::option::Option<
    unsafe extern "C" fn(
        inAudioConverter: AudioConverterRef,
        ioNumberDataPackets: *mut UInt32,
        ioData: *mut AudioBufferList,
        outDataPacketDescription: *mut *mut AudioStreamPacketDescription,
        inUserData: *mut ::std::os::raw::c_void,
    ) -> OSStatus,
>;

unsafe extern "C" {
    pub fn AudioConverterNew(
        inSourceFormat: *const AudioStreamBasicDescription,
        inDestinationFormat: *const AudioStreamBasicDescription,
        outAudioConverter: *mut AudioConverterRef,
    ) -> OSStatus;

    pub fn AudioConverterSetProperty(
        inAudioConverter: AudioConverterRef,
        inPropertyID: AudioConverterPropertyID,
        inPropertyDataSize: UInt32,
        inPropertyData: *const ::std::os::raw::c_void,
    ) -> OSStatus;

    pub fn AudioConverterFillComplexBuffer(
        inAudioConverter: AudioConverterRef,
        inInputDataProc: AudioConverterComplexInputDataProc,
        inInputDataProcUserData: *mut ::std::os::raw::c_void,
        ioOutputDataPacketSize: *mut UInt32,
        outOutputData: *mut AudioBufferList,
        outPacketDescription: *mut AudioStreamPacketDescription,
    ) -> OSStatus;

    pub fn AudioConverterDispose(inAudioConverter: AudioConverterRef) -> OSStatus;

    pub fn AudioFormatGetPropertyInfo(
        inPropertyID: AudioFormatPropertyID,
        inSpecifierSize: UInt32,
        inSpecifier: *const ::std::os::raw::c_void,
        outPropertyDataSize: *mut UInt32,
    ) -> OSStatus;
}
