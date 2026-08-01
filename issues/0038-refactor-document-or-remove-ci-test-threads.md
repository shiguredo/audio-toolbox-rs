# CI の cargo test --test-threads=1 の根拠を明文化する (または撤廃する)

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-ci-test-threads-rationale
- Polished: 2026-07-31

## 目的

CI (`.github/workflows/ci.yml` の `test-audio-toolbox` ジョブ) が `cargo test --workspace -- --test-threads=1` で直列実行している根拠が不明。撤廃可能なら並列に戻す、必要ならコメントで理由を明記する。

あわせて、依存解決を伴うコマンド (`cargo check` / `cargo clippy` / `cargo test`) に `--locked` を付与して stale な Cargo.lock での検証を防ぎ、prek / Makefile と実行方法を揃える (issue 0029 の release.yml 検証コマンドと同一にする前提)。

## 優先度根拠

Medium とする。動作に直接影響しないが、CI と prek / Makefile で `cargo test` の引数が食い違っており、根拠が不明のまま「壊れているかもしれない」引数が残り続けるのは不衛生。並列化を諦めているなら理由を残すか、テスト時間を削減できるなら並列に戻すべき。

## 現状

- CI: `.github/workflows/ci.yml` の `test-audio-toolbox` ジョブが `cargo test --workspace -- --test-threads=1` を実行している。
- prek: `prek.toml` の `cargo-test` フック (pre-push で実行) が `cargo test` を実行している。
- Makefile: `test` ターゲットが `cargo test --workspace` を実行している。

単一パッケージ構成 (Cargo.toml に `[workspace]` セクションなし) のため `cargo test` と `cargo test --workspace` は挙動同等で、実質的な食い違いは `--test-threads=1` の有無のみ。

`--test-threads=1` の導入経緯は git 履歴に理由の記載がない (コミット a0c5a23「インポート」が初出で、コミットメッセージに本文なし)。

`Encoder` / `Decoder` は `!Send` (`src/lib.rs` で「`unsafe impl Send` を sound に正当化する根拠が取れないため実装しない」とコメント) だが、`!Send` は「1 スレッド内での作成・使用」しか要求しない。`cargo test` は「1 テスト = 1 スレッド」なので、`!Send` 型を並列テストしても問題ないはず。並列不可の根拠が Apple ドキュメントにも issue にもコメントにも無い。

## 完了条件

1. `--test-threads=1` の要否が確定する。撤廃する場合は `.github/workflows/ci.yml` / `prek.toml` / `Makefile` に `test-threads` の記述が残らない (`grep -rn "test-threads" .github/workflows/ci.yml prek.toml Makefile` が 0 件)。保持する場合は `.github/workflows/ci.yml` の `cargo test` ステップ直上のコメントに、保持の根拠 (並列実行で落ちたテスト名・失敗内容、または共有リソースの検出内容) が含まれる。
2. `.github/workflows/ci.yml` の `cargo check` / `cargo clippy` / `cargo test` が全て `--locked` を含む (`grep -n "run: cargo check\|run: cargo clippy\|run: cargo test" .github/workflows/ci.yml` の該当 3 行が全て `--locked` を含む)。これは issue 0029 が release.yml の検証コマンドを「ci.yml と同一になるよう揃える」前提としているため。
3. `prek.toml` の `cargo-test` フックと Makefile の `test` ターゲットが `cargo test --workspace --locked` になる (CI と同一の `--workspace` / `--locked` 指定)。`--test-threads=1` は保持する場合 CI のみに付与する (ローカル実行 (prek の pre-push フックと Makefile の `test` ターゲット) は現状並列実行でも問題が起きていないため)。
4. 撤廃する場合: `workflow_dispatch` による手動実行を連続 10 回行い、`cargo test` ステップが全てパスすることを self-hosted runner (macOS ARM64) で確認する。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない (issue 0022 の管轄)。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`--test-threads=1` を外した状態で実証し、結果によって撤廃 / 保持を確定する。

1. テスト間で共有されるリソースが無いことを確認する。対象は一時ファイルの作成・環境変数の読み書き・プロセス全体で共有されるグローバル状態 (AudioComponent の状態を含む) を参照するテスト (tests/ 配下と src/ の `#[cfg(test)]` モジュール) が存在しないこと。共有リソースが検出された場合はその内容を記録して「保持」に倒し、手順 6 へ進む。
2. `feature/refactor-ci-test-threads-rationale` ブランチで `.github/workflows/ci.yml` の `cargo test` ステップから `--test-threads=1` を外して push する。push トリガーによる自動実行は数えず、`workflow_dispatch` による手動実行を連続 10 回繰り返す。1 回でも `cargo test` ステップが失敗したら、失敗内容を記録して手順 6 へ進む。
3. 失敗の判定対象は `cargo test` ステップのみとする。ランナー障害・checkout 失敗・rustup インストール失敗などテストと無関係な失敗はカウントせず、時間帯を変えて再実行する。検証はスケジュール実行 (平日 10:00 JST) と重ならない時間帯に行う。
4. `cargo test` ステップの失敗が並走ジョブとのリソース競合と疑われる場合 (他ブランチの CI やスケジュール実行と重なった場合) は、同じテストを単独実行で再現して切り分ける。単独実行でパスした場合は並列テスト起因ではないため検証を続行する (失敗はカウントせず再実行)。単独実行でも落ちる場合は並列テスト起因として手順 6 へ進む。
5. 連続 10 回パスした場合、撤廃を確定する。`.github/workflows/ci.yml` の `cargo test` ステップ直上に「self-hosted runner で並列実行の安定を連続 10 回確認したため直列実行を撤廃した。`!Send` は並列テスト阻害の根拠にならない (1 テスト = 1 スレッドのため)」旨のコメントを残す (issue 番号と `test-threads` の文字列を含めない。完了条件 1 の grep 0 件と整合させるため)。
6. 保持に倒した場合、保持の根拠 (落ちたテスト名・失敗内容、または共有リソースの検出内容) を `.github/workflows/ci.yml` の `cargo test` ステップ直上にコメントで残す (issue 番号は含めない)。
7. 撤廃 / 保持のどちらの場合も、`.github/workflows/ci.yml` の `cargo check` / `cargo clippy` / `cargo test` に `--locked` を付与する (stale な Cargo.lock での検証を防ぐ。issue 0029 が release.yml の検証コマンドを ci.yml と同一に揃える前提のため)。
8. `prek.toml` の `cargo-test` フックを `cargo test --workspace --locked` に、Makefile の `test` ターゲットを `cargo test --workspace --locked` に変更する (依存を追加した直後は Cargo.lock を更新してから push する。stale な Cargo.lock の検知は意図的な動作)。`--test-threads=1` は保持する場合 CI のみに付与する。
9. `CHANGES.md` の develop / `### misc` に [UPDATE] エントリを追記する (担当者行付き、issue 番号・issue ファイル名を含めない)。

撤廃を確定した場合、issue 0029 の完了条件 3 に従い release.yml の検証コマンドからも `--test-threads=1` が外れ、0036 / 0037 が注記している「`--test-threads=1` は issue 0038 の結論に従う」に従い、0036 / 0037 の検証コマンドも並列実行に切り替わる。

issue 0023 (Makefile の不要ターゲット削除) と並行実施しても、`test` ターゲットは 0023 の変更対象外のため直接の衝突はない。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く。撤廃時: `- [UPDATE] CI の cargo test から --test-threads=1 を撤廃し --locked を導入する`。保持時: `- [UPDATE] CI の cargo test の --test-threads=1 の根拠をコメントで明記する`。
