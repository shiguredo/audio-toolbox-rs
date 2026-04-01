# Decoder の 1 回あたり出力 PCM バッファと io_packets がコーデックの上限に足りるか検証し不足なら修正する

Created: 2026-04-01  
Model: Claude Opus 4.5

## なぜこの対応が必要か

`decode_impl` は `DECODE_BUF_FRAMES` と `io_packets` に基づき、**1 回の** `AudioConverterFillComplexBuffer` で受け取る出力 PCM の上限を決めている。ここが **実際に起こり得る最大出力フレーム数** より小さいと、バッファオーバーフロー、未初期化領域の参照、またはサイレントな切り詰めなどの **未定義動作またはデータ破損** のリスクがある。

---

## 調査結果（一次情報・算術）

### Opus（RFC 6716、RFC 8251 により更新）

**出典:** [RFC 6716](https://www.rfc-editor.org/rfc/rfc6716) §2.1.4 「Frame Duration」

[RFC 8251](https://www.rfc-editor.org/rfc/rfc8251) は RFC 6716 の参照デコーダ実装（Appendix A）の修正やセキュリティ修正が中心であり、**英語の仕様本文は変更しない**（RFC 8251 §1）。よって §2.1.4 のフレーム／パケット長の記述は **引き続き RFC 6716 の該当節を根拠とする**。

該当箇所（抜粋）:

> Opus can encode frames of 2.5, 5, 10, 20, 40, or 60 ms. **It can also combine multiple frames into packets of up to 120 ms.**

解釈:

- 1 つの Opus **パケット**に含められる音声は、**最大 120 ms** まで（複数フレームの結合）。
- デコーダ出力のサンプルレートが **48 kHz** のとき、1 チャンネルあたりのサンプル数の理論上限は  
  \( 48000 \times 0.12 = 5760 \) **フレーム**（ここでは Core Audio の「1 フレーム = 全チャンネル同一時刻のサンプル集合」と一致）。
- よって **5760 はリポジトリ内コメントからの推測ではなく、RFC の「最大 120 ms パケット」から直接導ける数値**である。

### 現行実装との比較（コード更新後）

- `src/lib.rs` の `DECODE_BUF_FRAMES` は **5760** に設定済み（RFC 6716 §2.1.4 に基づく 48 kHz 時の理論上限と一致）。
- **RFC 8251** 適用後も §2.1.4 の該当記述は変わらないため、上記算術の根拠は維持される。

### Apple AudioToolbox 層（未完了の検証）

RFC の上限と **実際に Core Audio が 1 呼び出しで返す PCM 量**は一致するとは限らない。少なくとも次は **別途** 確認する（コメントや推測に頼らない）。

1. **プロパティ照会**  
   `AudioConverterGetProperty` / `AudioConverterGetPropertyInfo` で、当該コンバータに対する出力側の最大パケットサイズ・フレーム数に相当する情報が取得できるか（利用可能な定数名は SDK / 生成バインディングで確認）。
2. **実測**  
   macOS 上で、RFC 上 120 ms に近い Opus パケット（または Apple が受け付ける最大長）を 1 パケット入力し、`mDataByteSize` の最大値・分割出力の有無を記録する。
3. **既知の制約**  
   コミュニティ報告では、Apple 実装が **RFC とは異なるフレーム長制約**を課す事例がある。これは RFC を否定しないが、**「Apple 上で実際に通る最大」**は実測または公式リリースノートで裏取りする。

---

## 現状の問題（更新）

- **Opus / バッファサイズ:** `DECODE_BUF_FRAMES == 5760` により、RFC 6716 §2.1.4 の **理論上の**最大（48 kHz 時 5760 フレーム）に対応する。
- **AAC-LC / MP3:** 1 パケットあたりのフレーム数は一般的に 1024 / 1152 であり、5760 で十分に覆える。
- **Apple 固有の上限**（1 呼び出しで実際に返る PCM 量が RFC 理論値と一致するか）は、上記「AudioToolbox 層」の検証が完了するまで **未確定**（残タスク）。

## 期待する動作

- RFC 上のワーストケースと、Apple 実測（または公式 API で取得できる上限）の **どちらか大きい方**（または安全側の単一定数）に、`pcm_buf` と `io_packets` を整合させる。
- 「5760 はコメントだから」ではなく、**RFC §2.1.4 と算術、および Apple 側の検証結果**を issue または設計メモに残す。

## 受け入れ条件の目安

- 採用した数値の根拠が **RFC 6716 §2.1.4（および必要なら Apple 側の根拠）** に紐づいていること。
- 境界に近い入力（可能なら 120 ms 相当の Opus パケット）を用いたテストが **macOS 上で**通ること、または「本クレートはその入力をサポートしない」と明示し拒否すること。

## 参考（該当コード）

- `src/lib.rs`: `DECODE_BUF_FRAMES`、`decode_impl`、`DecoderCodec::Opus`

## 参考（外部）

- RFC 6716 §2.1.4 Frame Duration: <https://www.rfc-editor.org/rfc/rfc6716#section-2.1.4>
- RFC 8251（RFC 6716 の更新）: <https://www.rfc-editor.org/rfc/rfc8251>
