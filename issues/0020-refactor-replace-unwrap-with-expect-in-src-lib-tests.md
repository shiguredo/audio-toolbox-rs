# src/lib.rs::tests の .unwrap() を .expect("MESSAGE") に置き換える

- Priority: Medium
- Created: 2026-07-21
- Completed: 2026-09-05
- Model: Opus 4.7
- Branch: feature/refactor-replace-unwrap-with-expect
- Polished: 2026-07-31

## 目的

shiguredo-rust 規約「`.unwrap()` ではなく `.expect("MESSAGE")` を使用すること」に反し、`src/lib.rs::tests` に `.unwrap()` 呼び出しが残っている状態を解消する。

## 優先度根拠

Medium とする。テストコードとはいえ、規約は「lib / tests / examples 全て」を対象とし、失敗時の情報量を担保するために `.expect("MESSAGE")` を使う方針を明確にしている。テスト重複解消 (issue 0018) で `src/lib.rs::tests` を削除する場合は本 issue は自動的に消化される可能性があるが、独立した規約整合の観点で立てておく。

## 現状

`src/lib.rs:1167, 1177, 1185` に以下の `.unwrap()` が 3 箇所ある。

```rust
let aac_lc = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::AacLc)
    .unwrap();
...
let mp3 = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::Mp3)
    .unwrap();
...
let opus = codecs
    .iter()
    .find(|c| c.codec == AudioCodecType::Opus)
    .unwrap();
```

find が None を返した場合、`.unwrap()` は情報量のないパニックメッセージ (`called Option::unwrap() on a None value`) を出すのみで、どのコーデックのエントリが取れなかったかがログに残らない。

## 完了条件

- `src/lib.rs::tests` から `.unwrap()` が消える。
- 置換後の `.expect(...)` メッセージは日本語 (issue 0019 と整合)。
- issue 0018 で `src/lib.rs::tests` 全体を削除する場合は本 issue は不要になる旨をコメントで残す。

## 解決方法

本 issue は作業不要と判断し、closed とする。本 issue が対象とするコードは、open issue である issues/0018-refactor-consolidate-duplicate-tests-into-integration-tests.md の作業で削除されるためである。

根拠:

- 実ファイル照合で、`src/lib.rs` の `#[cfg(test)] mod tests` 内の `test_supported_codecs` に、本 issue が列挙する `.unwrap()` 3 箇所（1167 / 1177 / 1185 行目）が実在することを確認した。
- issues/0018 の完了条件は「`src/lib.rs::tests` モジュールが削除される」、解決方法 5 は「`mod tests` 全体を削除する」であり、`test_supported_codecs` は削除対象に含まれる。
- issues/0018 の解決方法 7 には「issue 0020 は、対象コードが本 issue で削除される `test_supported_codecs` 内にあり、0020 側も「0018 を先に片付ければ本 issue はクローズできる」と明記しているため、本 issue では作業を実施しない。本 issue の完了をもって 0020 は不要となりクローズする」と明記されている。
- 本 issue 側も優先度根拠・解決方法に「0018 で `src/lib.rs::tests` 全体を削除する場合は本 issue は不要になる」「0018 を先に片付ければ本 issue はクローズできる」と記述しており、両 issue で方針が一致している。
- `tests/` 配下（tests/test_*.rs）に `.unwrap()` は存在せず（`unwrap_err` / `unwrap_or_else` は本 issue の対象外）、issues/0018 の完了後、本 issue が対象とする `.unwrap()` が残る箇所はない。

したがって、`.expect("MESSAGE")` への置換作業は対象コードの削除により実施空間が消えるため、issues/0018 の実装をもって本 issue は事実上対応済みとなる。万一 issues/0018 の方針が変更され `src/lib.rs::tests` が存続する場合のみ、本 issue を再検討すること。
