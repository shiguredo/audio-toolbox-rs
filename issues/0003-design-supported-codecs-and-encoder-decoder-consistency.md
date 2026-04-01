# `supported_codecs()` と `Encoder` / `Decoder` の意味・範囲を整合させる設計を決める

Created: 2026-04-01
Model: Claude Opus 4.5

## なぜこの対応が必要か

`supported_codecs()` は本クレートの公開 API として呼ばれる一方、返す `AudioCodecType` の集合は **`EncoderCodec` / `DecoderCodec` が表す「このクレートで実際にエンコード／デコードできる種別」と一致しない**。例として、照会結果に FLAC や ALAC が含まれても、`DecoderCodec` に対応するバリアントが無ければ **利用者はこのクレートの `Decoder` でそれを選べない**。

このずれは、**「supported」と命名された情報が、このライブラリの利用可能性を示すのか、macOS / AudioToolbox 全体の登録状況を示すのか**が曖昧になる原因になる。利用者が誤った前提で UI やフォールバックを組むリスクがあるため、**意図する意味を仕様として固定し、API またはドキュメントで一貫させる**必要がある。

本 issue は **実装の先行ではなく、方針とスコープの合意**を取るためのものである。

---

## 現状の整理（事実）

- `Encoder` は `EncoderCodec` により **AAC-LC のみ**を対象としている。
- `Decoder` は `DecoderCodec` により **AAC-LC / MP3 / Opus** を対象としている。
- `supported_codecs()` は `AudioCodecType` の広い集合（HE-AAC 系、FLAC、ALAC 等を含む）について、AudioToolbox にデコーダ／エンコーダが登録されているか等を照会する。
- `AudioCodecType` は上記以外のバリアントも持ち、`supported_codecs()` の照会対象と `Encoder` / `Decoder` の選択肢は **別集合**である。

---

## 検討すべき方針（例・排他的ではない）

次のいずれか、または組み合わせを決める必要がある。

1. **照会範囲の縮小**
   `supported_codecs()` が返す種別を、`EncoderCodec` / `DecoderCodec` と対応するものに限定する（ライブラリの「使えるか」と一致させる）。

2. **API の分割**
   「OS／AudioToolbox 上の登録情報」と「このクレートの `Encoder` / `Decoder` で利用可能」を **別関数または別型**で返し、命名で誤解を防ぐ。

3. **実装拡張**
   照会結果に合わせて `Encoder` / `Decoder` のコーデック種別を増やす（FLAC 等）。別 API（例: ExtAudioFile）が必要になる可能性があり、**スコープ・工数・保守**は別途見積りが必要。

4. **ドキュメントのみ**
   挙動は変えず、`supported_codecs()` の意味を README / rustdoc で厳密に定義する（「システム登録の有無であり、本クレートの全 API と 1 対 1 ではない」等）。

---

## テスト・CI に関する注意

`encoding.supported` 等は **OS バージョンや環境で変わりうる**。方針決定にあわせ、単体テストで **環境依存の強い前提**を固定しないか、どこまで保証するかを決める。

---

## 完了条件（この issue のスコープ）

- [ ] 上記方針のいずれか（またはハイブリッド）が文書化され、関係者が合意できる状態になっている。
- [ ] 後続の実装・ドキュメント作業は **別 issue またはタスク**に切り出せる粒度まで、決定事項が書かれている。

**実装完了をこの issue の完了条件には含めない**（設計・合意が先）。
