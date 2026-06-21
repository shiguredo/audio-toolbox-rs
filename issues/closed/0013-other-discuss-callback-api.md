# Encoder / Decoder のコールバック API 化を検討する

- Priority: Low
- Created: 2026-06-21
- Completed: 2026-06-21
- Model: Kimi K2.7 Code
- Branch: feature/doc-discuss-callback-api
- Polished:
- Reporter: @voluntas

## 目的

`video-toolbox-rs` と同様に `audio-toolbox-rs` の `Encoder` / `Decoder` をコールバック API に変更するかどうかを検討し、判断を記録する。

## 優先度根拠

Low とする。本 issue は設計判断の記録であり、現時点では実装を行わないことが決定している。今後 Hisui 等の呼び出し側で video-toolbox-rs との API 統一が強く求められた場合に再検討すればよい。

## 現状

現在の `audio-toolbox-rs` は「入力を push し、出力を pull する」ハイブリッド API になっている。

- `Encoder::encode(&[i16])` で PCM を内部バッファに蓄積する
- `Encoder::next_frame()` でエンコード済みフレームを 1 つずつ取り出す
- `Decoder::decode(&[u8])` で 1 パケットを内部バッファに蓄積する
- `Decoder::next_frame()` でデコード済み PCM を取り出す

Audio Toolbox の `AudioConverter` API は、`AudioConverterFillComplexBuffer` の呼び出しスレッド上で同期的に入力コールバックを実行する。これは Video Toolbox（Apple 内部スレッドで非同期コールバックを発火）とは根本的に異なる。

## 設計方針

検討した選択肢は以下の 3 つである。

1. **出力コールバック方式に全面移行する**
   - `video-toolbox-rs` と同じ `EncodeHandler` / `DecodeHandler` trait を導入し、結果をコールバックで通知する
   - 入力は従来通り `encode()` / `decode()` で受け取る
2. **現状維持 + オプションのコールバックラッパーを追加する**
   - 既存 API はそのままに、`CallbackEncoder` / `CallbackDecoder` 等の薄いラッパーを別途提供する
3. **現状維持**
   - Audio Converter が同期 API であることを理由に、現在の push-pull API を維持する

## 完了条件

- コールバック化のメリット・デメリットを整理すること
- Audio Converter の同期 API という特性を考慮した上で、実施しない判断を記録すること
- 本 issue を closed に移動すること

## 解決方法

**現状維持（選択肢 3）を採用し、コールバック化は実施しない。**

判断の理由は以下の通り。

- Audio Converter は本質的に同期 API である。コールバック化しても「Video Toolbox 式の非同期通知」にはならず、`encode()` / `decode()` 呼び出し中に同期的にハンドラーが発火するだけである。
- 現在の `encode()` → `next_frame()` の流れは、同期コンバーターの動作を素直に表現しており、誤解を生みにくい。
- コールバック化によるメリット（video-toolbox-rs との API 一貫性、呼び出し側のループ削減）は、破壊的変更と再入制御の複雑さを上回らない。
- 入出力の相関が必要な場合は、呼び出し側が自分で紐付けるか、ラッパーを書けば済む。

もし Hisui 側で「どうしても video-toolbox-rs と同じトレイトベース API にしたい」という強い要求が出た場合は、選択肢 2（ラッパー追加）で対応し、既存 API はそのままにするのが現実的である。
