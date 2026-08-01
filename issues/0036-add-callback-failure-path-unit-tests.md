# Encoder / Decoder コールバックの失敗パスに対する単体テストを追加する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/add-callback-failure-path-tests
- Polished: 2026-07-31

## 目的

`Encoder::callback` / `Decoder::callback` の null チェック・`mData == NULL` scratch_buf・`packet_provided_in_this_fill`・`out_data_packet_description` 非 null 経路などの失敗パスに対する直接的な回帰テストを追加する。issues/0006〜0012 のうちコールバック内部の失敗パス (0006 / 0010 / 0011 / 0012。0007 のオーバーフロー経路は 64-bit macOS では到達不能のため対象外。0010 は先頭 null チェック経由で部分カバー) で潰した問題の再発を回帰的に検知できるようにする。

## 優先度根拠

Medium とする。現状のテストは正常系の encode / decode 経由でしかコールバックの分岐に到達せず、issues/0011 (`mData == NULL`) のような環境依存の問題を回帰的に検知できない。過去に複数の hardening を積んできた経緯からも、回帰試験の空白は放置すべきでない。

## 現状

`Encoder::callback` / `Decoder::callback` には以下の失敗パスがある (シンボル名で特定):

- `Encoder::callback` の先頭の null チェック (`in_user_data` / `io_number_data_packets` / `io_data` の null で param_err 返却)
- `Encoder::callback` の `mData == NULL` 経路 (scratch_buf 提供)
- `Decoder::callback` の null チェック
- `Decoder::callback` の `packet_provided_in_this_fill` 分岐 (2 回目以降は `K_NO_MORE_INPUT`)
- `Decoder::callback` の `out_data_packet_description` 非 null 経路 (`packet_desc` 設定と `*out_data_packet_description` の書き込み。null 経路は `decode_impl` が常に `null_mut()` を渡すため既に間接カバー済み)

- `Encoder::callback` / `Decoder::callback` のデータ不足経路 (`K_NO_MORE_INPUT` 返却): 正常系の encode / decode 経由で間接カバー済みのため対象外

なお、以下のパスは 64-bit macOS ではテスト不可能または静的到達不能のため対象外:

- `(packets as usize).checked_mul(channels)`: usize 演算で 64-bit ではオーバーフローしない
- `packets.checked_mul(ch_u32).checked_mul(bps)`: u32 演算だが、`packets` は `io_number_data_packets` へのクランプで `max_packets` 以下に制限されるため、オーバーフローには 4 GiB 超の `pcm_buf` が必要
- `u32::try_from(max_packets)`: `pcm_buf.len()` が 8 GiB 超 (2^32 サンプル) のときのみ失敗
- `u32::try_from(channels)`: `channels ≤ 255` で常に成功する完全なデッドコード (削除検討は別 issue)
- `drain_end` 境界検査: `packets` がクランプ後は `pcm_buf.len() / channels` 以下なので常に成立 (eos は無関係)
- `u32::try_from(encoded_buf.len())`: 4 GB 超の単一パケットが必要

`callback` は `unsafe extern "C" fn` の関連関数だが、`src/lib.rs` 内の `#[cfg(test)]` モジュールからは private のまま直接呼べる。

## 完了条件

1. 上記の対象パス (Encoder / Decoder の先頭 null チェック / `mData == NULL` scratch_buf / `packet_provided_in_this_fill` / `out_data_packet_description` 非 null 経路) に対応する回帰テストが `src/lib.rs` 内の新規 `#[cfg(test)] mod callback_tests` に追加される (issue 0018 実施後。`mod tests` を前提としない)。
2. `cargo test --workspace -- --test-threads=1` で新規テストがパスし、既存テストも引き続きパスする (`--test-threads=1` は ci.yml と同一の直列実行で検証するため。要否は issue 0038 の結論を反映する)。`cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` が成功する (`--all-targets` はテストターゲットも検証するため意図的に使用する。ci.yml の clippy はテストターゲットを含まない)。
3. テストは日本語コメント + 日本語 assert メッセージ (AGENTS.md 準拠)。英語の assert メッセージが残っていないことを grep で確認する (issue 0019 と同様。0019 の例外規定は callback が `Error` を返さず `i32` を返すため本 issue では発生しない)。テストコメント・テスト名に issue 番号や issue への言及を含めないこと (shiguredo-rust 規約)。
4. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない (issue 0022 の管轄)。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`callback` を `src/lib.rs` 内の `#[cfg(test)]` モジュールから private のまま直接呼ぶ (案 A で確定。`pub(crate)` 化や本番コードの変更は不要)。テスト対象の代わりに偽の実装を差し込むモックではなく、実コードに FFI レベルで加工した引数 (null ポインタ等) を渡す通常の単体テストのため、AGENTS.md のモック・スタブ禁止に抵触しない。

1. `src/lib.rs` 内に新規 `#[cfg(test)] mod callback_tests` を追加する (issue 0018 実施後。`mod tests` を前提としない)。
2. `Encoder::callback` を直接呼び、null チェック / `mData == NULL` scratch_buf の各パスを確認する (unsafe 前提でテストを書く)。
3. `Decoder::callback` を直接呼び、null チェック / `packet_provided_in_this_fill` / `out_data_packet_description` 非 null 経路を確認する。
4. 直接呼び出しは失敗分岐のロジック検証のみで、AudioConverter 経由の ABI 呼び出し規約そのものは検証しない (ABI 経路は `Some(Self::callback)` のシグネチャ一致をコンパイル時に強制し、既存の encode / decode 統合テストが間接カバーする)。
5. 最後に `cargo test --workspace -- --test-threads=1` / `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` で確認する。

issue 0018 を先に実施し、その後に着手する (0018 で `mod tests` が削除されるため。0018 の解決方法「0036 のテスト追加先は `mod tests` を前提としない」)。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] Encoder / Decoder コールバックの失敗パスに対する単体テストを追加する`)。
