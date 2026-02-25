//! 正弦波 PCM を AAC エンコードして MP4 ファイルに保存するサンプル
//!
//! ```bash
//! cargo run --example sine_to_mp4
//! cargo run --example sine_to_mp4 -- --bitrate 256000 --duration 10 --freq 880 --output tone.mp4
//! ```

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::num::NonZeroU32;

use shiguredo_audio_toolbox::{Encoder, EncoderCodec, EncoderConfig};
use shiguredo_mp4::boxes::{AudioSampleEntryFields, EsdsBox, Mp4aBox, SampleEntry};
use shiguredo_mp4::descriptors::{
    DecoderConfigDescriptor, DecoderSpecificInfo, EsDescriptor, SlConfigDescriptor,
};
use shiguredo_mp4::mux::{Mp4FileMuxer, Sample};
use shiguredo_mp4::{FixedPointNumber, TrackKind, Uint};

const SAMPLE_RATE: u32 = 48000;
const CHANNELS: usize = 2;
const FRAMES_PER_PACKET: u32 = 1024;

const DEFAULT_BITRATE: u32 = 128_000;
const DEFAULT_DURATION_SECS: f64 = 5.0;
const DEFAULT_FREQ: f64 = 440.0;

/// コマンドライン引数からオプション値を取得する
fn get_arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|pos| args.get(pos + 1).cloned())
}

/// 正弦波 PCM データを生成する (インターリーブドステレオ i16)
fn generate_sine_pcm(freq: f64, sample_offset: usize, num_samples: usize) -> Vec<i16> {
    let mut pcm = Vec::with_capacity(num_samples * CHANNELS);
    for i in 0..num_samples {
        let t = (sample_offset + i) as f64 / SAMPLE_RATE as f64;
        let value = (2.0 * std::f64::consts::PI * freq * t).sin();
        let sample = (value * i16::MAX as f64) as i16;
        // ステレオ: 左右同じ値
        pcm.push(sample);
        pcm.push(sample);
    }
    pcm
}

/// AAC-LC 48kHz ステレオ用の SampleEntry を構築する
fn build_mp4a_sample_entry(bitrate: u32) -> SampleEntry {
    // AudioSpecificConfig (AAC-LC 48kHz ステレオ)
    // audioObjectType = 2 (AAC-LC): 5 bits -> 00010
    // samplingFrequencyIndex = 3 (48000Hz): 4 bits -> 0011
    // channelConfiguration = 2 (stereo): 4 bits -> 0010
    // 00010_0011_0010_000 = 0x11, 0x90
    let audio_specific_config = vec![0x11, 0x90];

    SampleEntry::Mp4a(Mp4aBox {
        audio: AudioSampleEntryFields {
            data_reference_index: AudioSampleEntryFields::DEFAULT_DATA_REFERENCE_INDEX,
            channelcount: CHANNELS as u16,
            samplesize: AudioSampleEntryFields::DEFAULT_SAMPLESIZE,
            samplerate: FixedPointNumber {
                integer: SAMPLE_RATE as u16,
                fraction: 0,
            },
        },
        esds_box: EsdsBox {
            es: EsDescriptor {
                es_id: EsDescriptor::MIN_ES_ID,
                stream_priority: EsDescriptor::LOWEST_STREAM_PRIORITY,
                depends_on_es_id: None,
                url_string: None,
                ocr_es_id: None,
                dec_config_descr: DecoderConfigDescriptor {
                    object_type_indication:
                        DecoderConfigDescriptor::OBJECT_TYPE_INDICATION_AUDIO_ISO_IEC_14496_3,
                    stream_type: DecoderConfigDescriptor::STREAM_TYPE_AUDIO,
                    up_stream: DecoderConfigDescriptor::UP_STREAM_FALSE,
                    buffer_size_db: Uint::new(0),
                    max_bitrate: bitrate,
                    avg_bitrate: bitrate,
                    dec_specific_info: Some(DecoderSpecificInfo {
                        payload: audio_specific_config,
                    }),
                },
                sl_config_descr: SlConfigDescriptor,
            },
        },
        unknown_boxes: vec![],
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let bitrate: u32 = get_arg(&args, "--bitrate")
        .map(|v| v.parse().map_err(|e| format!("invalid --bitrate: {e}")))
        .transpose()?
        .unwrap_or(DEFAULT_BITRATE);
    let duration_secs: f64 = get_arg(&args, "--duration")
        .map(|v| v.parse().map_err(|e| format!("invalid --duration: {e}")))
        .transpose()?
        .unwrap_or(DEFAULT_DURATION_SECS);
    let freq: f64 = get_arg(&args, "--freq")
        .map(|v| v.parse().map_err(|e| format!("invalid --freq: {e}")))
        .transpose()?
        .unwrap_or(DEFAULT_FREQ);
    let output_path = get_arg(&args, "--output").unwrap_or_else(|| "output.mp4".to_string());

    let total_samples = (SAMPLE_RATE as f64 * duration_secs) as usize;

    println!(
        "Encoding AAC {SAMPLE_RATE}Hz stereo {bitrate}bps {duration_secs:.1}s ({freq}Hz sine) -> {output_path}"
    );

    // AAC エンコーダーの初期化
    let mut encoder = Encoder::new(EncoderConfig {
        codec: EncoderCodec::AacLc,
        sample_rate: SAMPLE_RATE,
        channels: CHANNELS as u8,
        bitrate: Some(bitrate),
        bitrate_control_mode: None,
        codec_quality: None,
        vbr_quality: None,
    })?;

    // MP4 マルチプレクサーの初期化
    let mut muxer = Mp4FileMuxer::new()?;
    let initial_bytes = muxer.initial_boxes_bytes();
    let mut file = File::create(&output_path)?;
    file.write_all(initial_bytes)?;
    let mut data_offset = initial_bytes.len() as u64;

    let timescale = NonZeroU32::new(SAMPLE_RATE).unwrap();
    let mut first_frame = true;
    let mut sample_offset: usize = 0;
    let mut encoded_samples: usize = 0;

    // 1024 サンプルずつ正弦波を生成してエンコードする
    while sample_offset < total_samples {
        let chunk_samples = FRAMES_PER_PACKET as usize;
        let pcm = generate_sine_pcm(freq, sample_offset, chunk_samples);
        sample_offset += chunk_samples;

        encoder.encode(&pcm)?;
        while let Some(frame) = encoder.next_frame() {
            encoded_samples += frame.samples;

            let sample_entry = if first_frame {
                first_frame = false;
                Some(build_mp4a_sample_entry(bitrate))
            } else {
                None
            };

            file.write_all(&frame.data)?;
            let sample = Sample {
                track_kind: TrackKind::Audio,
                sample_entry,
                keyframe: false,
                timescale,
                duration: FRAMES_PER_PACKET,
                data_offset,
                data_size: frame.data.len(),
            };
            muxer.append_sample(&sample)?;
            data_offset += frame.data.len() as u64;
        }

        // 1 秒ごとに進捗を表示する
        if sample_offset.is_multiple_of(SAMPLE_RATE as usize) {
            let sec = sample_offset / SAMPLE_RATE as usize;
            println!("  {sec}/{duration_secs:.0}s encoded");
        }
    }

    // 残りのフレームをフラッシュする
    encoder.finish()?;
    while let Some(frame) = encoder.next_frame() {
        encoded_samples += frame.samples;

        let sample_entry = if first_frame {
            first_frame = false;
            Some(build_mp4a_sample_entry(bitrate))
        } else {
            None
        };

        file.write_all(&frame.data)?;
        let sample = Sample {
            track_kind: TrackKind::Audio,
            sample_entry,
            keyframe: false,
            timescale,
            duration: FRAMES_PER_PACKET,
            data_offset,
            data_size: frame.data.len(),
        };
        muxer.append_sample(&sample)?;
        data_offset += frame.data.len() as u64;
    }

    // MP4 ファイナライズ
    let finalized = muxer.finalize()?;
    for (offset, bytes) in finalized.offset_and_bytes_pairs() {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)?;
    }

    println!("Done: {output_path} ({encoded_samples} samples encoded)");

    Ok(())
}
