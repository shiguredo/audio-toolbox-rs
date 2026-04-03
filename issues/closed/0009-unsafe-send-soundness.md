# `unsafe impl Send` の soundness を検証し不適切ならやめる

Created: 2026-04-01
Completed: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

`unsafe impl Send for Encoder/Decoder` が **sound** でないと、**別スレッドへ移動したあと**に **データ競合**が起き **未定義動作**（**セグフォ** を含む）になりうる。

[`AudioConverterNew`](https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)) のページには、**スレッド安全性**の明記は **見当たらない**（取得時点）。**`Send` を付けてよい根拠**を別途確認する必要がある。

## 受け入れ条件の目安

**注意（soundness と実測）:** `unsafe impl Send` が **sound** かどうかは、**再現実験が通ったことでは証明できない**。実測は「たまたま壊れなかった」以上の保証にならず、最適化や将来の OS 実装差分も排除できない。**実測のみを根拠に `Send` を維持する判断は採用しない。**

- **いずれか:**
  - **A（`Send` 維持）:** `AudioConverterRef` を **別スレッドへ移動したうえで単一スレッドのみ操作する**ことが許容されることを、次の **いずれか**で **規範的に**示すこと。**実測・ベンチ単体では不十分。**
    - **Apple 公式**（ドキュメント・リリースノート・Technical Q&A 等）に **スレッド制約または移動後の利用**が書かれていること。
    - 上記と **同等の仕様上の根拠**（例: 対象 API が **スレッドセーフ**と明記されている等）が取れること。
    - **crate 側**で `Send` を sound にする **追加不変条件**（例: 移動後も **一度に一スレッドからしか** `Encoder` / `Decoder` に触れないことを API ドキュメントと型で保証する等）を **明示し、その不変条件が `unsafe impl` と整合すること**を論じられること。
  - **B（`Send` 撤回）:** 上記 **A** の根拠が取れない、または **スレッド親和性等で `Send` が不適切**と結論した場合は **`unsafe impl Send` を削除**（`!Send` とする）など **API 破壊を伴う変更**を実装する。

## ミッション適合性の確認

- **適合する。** 根拠: **誤った `Send`** は **データ競合**を招き、Rust の **未定義動作**（**セグフォ** を含む）になる。パニック・セグフォ対策のミッションに含めるのは妥当。
- **注意:** 実運用で **別スレッドへ移動しない**場合でも、`unsafe impl` の **soundness** は言語上の義務であり、本ブランチで **検証または撤回**する対象とする。

## 参考（該当コード）

- `src/lib.rs`: `Encoder` / `Decoder`（`unsafe impl Send` は削除済み）

## 参考（外部）

- `AudioConverterNew`: <https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)>

## 解決方法

- 受け入れ条件 **B** を採用し、`Encoder` / `Decoder` の **`unsafe impl Send` を削除**した。Apple 公式にスレッド間移動を規範的に保証する根拠は取れないため、`Send` を sound に正当化できない。
- rustdoc に **`Send` を実装しない**旨を追記した。
