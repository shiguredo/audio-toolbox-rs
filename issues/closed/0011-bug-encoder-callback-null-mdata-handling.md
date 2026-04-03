# エンコーダーコールバックで mData == NULL 時に自前バッファを提供する

Created: 2026-04-03
Model: Opus 4.6

## 概要

エンコーダーの FFI コールバックで `ioData->mBuffers[0].mData` が NULL の場合、`kAudio_ParamError` を返して処理を中断している。Apple の AudioConverter.h では、コールバック側が自前バッファを提供することが期待されている。

## 根拠

AudioConverter.h (L785 付近) には、コールバックが以前に自前バッファを返した場合、次回呼び出し時に `mData` が NULL で渡されることがあると明記されている。現在のエンコーダーは常にコピーで渡しているため、自ら NULL パスに入ることはないが、OS 実装差分や将来の macOS バージョンで挙動が変わるリスクがある。

## 該当箇所

- `src/lib.rs` L539: `io_data.mBuffers[0].mData.is_null()` の場合に `kAudio_ParamError` を返す

## 修正方針

Encoder 構造体に `scratch_buf: Vec<i16>` フィールドを追加し、`mData` が NULL の場合は `scratch_buf` にデータをコピーしてそのポインタを `mData` にセットする。Decoder 側は既に `encoded_buf` のポインタを直接渡しているため対応不要。

## 解決方法

Encoder 構造体に `scratch_buf: Vec<i16>` フィールドを追加した。コールバック内で `mData` が NULL の場合、`scratch_buf` に PCM データをコピーしてそのポインタを `mData` にセットするようにした。

Completed: 2026-04-03
