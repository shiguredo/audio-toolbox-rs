# コールバックのエラー返却時に *io_number_data_packets = 0 を保証する

Created: 2026-04-03
Model: Opus 4.6

## 概要

Encoder / Decoder の FFI コールバック (`AudioConverterComplexInputDataProc`) で、エラーを返却する際に `*io_number_data_packets = 0` をセットしていない経路がある。

## 根拠

Apple の AudioConverter.h (L800 付近) には、コールバックがエラーを返す場合は `*ioNumberDataPackets` を 0 にセットすべきと明記されている。現状は要求時のパケット数がそのまま残る経路があり、AudioConverter フレームワーク側に stale なパケット数を見せる実装になっている。

## 該当箇所

### Encoder callback (`src/lib.rs`)

- L496: `in_user_data` / `io_data` が null で `io_number_data_packets` が非 null の場合、ゼロ化せずに `kAudio_ParamError` を返す
- L504: `checked_mul` オーバーフロー時、ゼロ化せずに `kAudio_ParamError` を返す
- L510: データ不足時、ゼロ化せずに `K_NO_MORE_INPUT` を返す
- L516: `u32::try_from` 失敗時、ゼロ化せずに `kAudio_ParamError` を返す

### Decoder callback (`src/lib.rs`)

- L893: 同上のポインタ null 検査パス
- L905: データ不足時、ゼロ化せずに `K_NO_MORE_INPUT` を返す
- L913: `u32::try_from` 失敗時、L909 で書き込んだ値が残ったまま `kAudio_ParamError` を返す

## 修正方針

各エラー return の前に `*io_number_data_packets = 0` を挿入する。ポインタ null チェックの分岐では `io_number_data_packets` 自体が null の可能性があるため、非 null 確認後にゼロ化する。

## 解決方法

Encoder / Decoder の FFI コールバック内の全エラー返却パスで `*io_number_data_packets = 0` をセットするようにした。ポインタ null チェックの複合条件では `io_number_data_packets` が非 null の場合のみゼロ化する。

Completed: 2026-04-03
