# `unsafe impl Send` の soundness を検証し不適切ならやめる

Created: 2026-04-01
Model: Claude Opus 4.5

**スコープ:** 本ブランチのミッションは **パニック・セグメンテーションフォルト（および FFI による未定義動作）** の防止に限定する。

## なぜこの対応が必要か

`unsafe impl Send for Encoder/Decoder` が **sound** でないと、**別スレッドへ移動したあと**に **データ競合**が起き **未定義動作**（**セグフォ** を含む）になりうる。

[`AudioConverterNew`](https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)) のページには、**スレッド安全性**の明記は **見当たらない**（取得時点）。**`Send` を付けてよい根拠**を別途確認する必要がある。

## 受け入れ条件の目安

- **いずれか:**
  - **A:** `AudioConverterRef` を **別スレッドへ移動したうえで単一スレッドのみ操作する**ことが API 上許容される根拠を、**Apple 公式・Technical Note、または検証可能な実測**に基づき記録し、`Send` **維持**でよいと結論する。
  - **B:** 根拠が取れない、または **スレッド親和性等で `Send` が不適切**と結論した場合は **`unsafe impl Send` を削除**（`!Send` とする）など **API 破壊を伴う変更**を実装する。

## 参考（該当コード）

- `src/lib.rs`: `unsafe impl Send`

## 参考（外部）

- `AudioConverterNew`: <https://developer.apple.com/documentation/audiotoolbox/audioconverternew(_:_:_:)>
