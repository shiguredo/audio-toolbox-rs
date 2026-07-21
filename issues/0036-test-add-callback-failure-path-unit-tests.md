# Encoder / Decoder コールバックの失敗パスに対する単体テストを追加する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/test-callback-failure-paths
- Polished:

## 目的

`Encoder::callback` / `Decoder::callback` の null チェック・`checked_mul` オーバーフロー・`mData == NULL` scratch_buf・`packet_provided_in_this_fill` などの失敗パスに対する直接的な回帰テストを追加する。過去 issues/0004〜0012 で潰したバグの再発を検知できるようにする。

## 優先度根拠

Medium とする。現状のテストは正常系の encode / decode 経由でしかコールバックの分岐に到達せず、issues/0011 (`mData == NULL`) のような環境依存のバグを regressional に検知できない。過去に複数の hardening を積んできた経緯からも、回帰試験の空白は放置すべきでない。

## 現状

`src/lib.rs:492-588` (Encoder callback) と `src/lib.rs:939-1023` (Decoder callback) には以下の失敗パスがある。

- Encoder callback (`src/lib.rs:499-506`): `in_user_data.is_null() || io_number_data_packets.is_null() || io_data.is_null()` で param_err 返却
- Encoder callback (`src/lib.rs:512-518`): `(packets * channels).checked_mul` オーバーフロー
- Encoder callback (`src/lib.rs:528-534`): `u32::try_from(max_packets)` 失敗
- Encoder callback (`src/lib.rs:540-546`): `u32::try_from(channels)` 失敗
- Encoder callback (`src/lib.rs:549-555`): `packets.checked_mul(ch_u32).checked_mul(bps)` 失敗
- Encoder callback (`src/lib.rs:562-573`): `mData == NULL` 経路 (scratch_buf 提供)
- Encoder callback (`src/lib.rs:578-584`): `drain_end` 境界検査失敗
- Decoder callback (`src/lib.rs:947-954`): null チェック
- Decoder callback (`src/lib.rs:957-984`): `packet_provided_in_this_fill` 分岐
- Decoder callback (`src/lib.rs:990-996`): `u32::try_from(this.encoded_buf.len())` 失敗
- Decoder callback (`src/lib.rs:1011`): `out_data_packet_description` null / 非 null の両方

これらは間接テストされていない、もしくはカバーが薄い。callback は `unsafe extern "C" fn` の関連関数のためテストから直接呼びづらい。

## 完了条件

- 各 callback の主要な失敗パスに対応する回帰テストが追加される (最低: null チェック、mData == NULL scratch_buf、packet_provided_in_this_fill、out_data_packet_description null 分岐)。
- テストはコンパイル可能かつパスする。
- テストは日本語コメント + 日本語 assert メッセージ (AGENTS.md 準拠)。

## 解決方法

以下のいずれか (もしくは組み合わせ) で対応する。

### 案 A: callback を pub(crate) に落として直接呼ぶ

- `Self::callback` を `pub(crate) unsafe extern "C" fn` にする。
- `src/lib.rs` の `#[cfg(test)] mod tests` から直接呼び、null / オーバーフロー / mData == NULL の各パスを確認する。
- 呼び出し側は unsafe 前提でテストを書く。

### 案 B: 内部状態を仕込めるファクトリを test-utility として提供

- テスト用に `Encoder` / `Decoder` の内部状態 (pcm_buf, encoded_buf, eos, converter=null 等) を仕込めるコンストラクタを `#[cfg(test)] pub(crate) fn new_for_test(...)` として用意する。
- そのインスタンスに対して encode / decode を叩き、間接的に失敗パスに到達させる。

案 A のほうが直接的だが、issue 0018 と競合する可能性 (`src/lib.rs::tests` を削除する方針との齟齬)。案 A で対応する場合は tests/ 側から `pub(crate)` を触れるようにするのは難しいため、`src/lib.rs::tests` 内で書くことになる。もしくは callback を `pub(crate)` にしつつ `pub mod` を通じて crate 外から見えないよう管理する。

推奨は案 A で、テストは `src/lib.rs` の `#[cfg(test)] mod tests` (issue 0018 で撤去するか判断中) もしくは `src/lib.rs` 内の新規 `#[cfg(test)] mod callback_tests` に置く。issue 0018 と対応順序を調整する必要がある。
