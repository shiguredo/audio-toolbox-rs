# CI の cargo test --test-threads=1 の根拠を明文化する (または撤廃する)

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-ci-test-threads-rationale
- Polished:

## 目的

CI (`.github/workflows/ci.yml:37`) が `cargo test --workspace -- --test-threads=1` で直列実行している根拠が不明。撤廃可能なら並列に戻す、必要ならコメントで理由を明記する。prek / Makefile と実行方法を揃える。

## 優先度根拠

Medium とする。動作に直接影響しないが、CI と prek / Makefile で `cargo test` の引数が三様に食い違っており、根拠が不明のまま「壊れているかもしれない」引数が残り続けるのは不衛生。並列化を諦めているなら理由を残すか、テスト時間を削減できるなら並列に戻すべき。

## 現状

- CI (`.github/workflows/ci.yml:37`): `cargo test --workspace -- --test-threads=1`
- prek (`prek.toml:92`): `cargo test`
- Makefile (`Makefile:5`): `cargo test --workspace`

`Encoder` / `Decoder` は `!Send` (`src/lib.rs:601-602, 1036-1037`) だが、`!Send` は「1 スレッド内での作成・使用」しか要求しない。`cargo test` は「1 テスト = 1 スレッド」なので、`!Send` 型を並列テストしても問題ないはず。並列不可の根拠が Apple ドキュメントにも issue にもコメントにも無い。

## 完了条件

- `--test-threads=1` の要否が明確になる。
- 撤廃する場合: `cargo test --workspace` (もしくは `cargo test --workspace --locked`) に統一される。
- 保持する場合: `.github/workflows/ci.yml` にコメントで具体的な理由 (AudioToolbox のグローバル状態、self-hosted runner の CPU 制約、Apple の特定 API の非スレッドセーフ挙動 等) を残す。
- prek / Makefile / CI の cargo test 引数が同一になる。

## 解決方法

1. CI で `--test-threads=1` を外し、self-hosted runner でテストが安定してパスするか複数回確認する (少なくとも 5〜10 回)。
2. 安定するなら CI の引数を `cargo test --workspace --locked` に変え、prek / Makefile も同じに揃える。
3. 落ちるなら落ちるパターン (どのテストが並列で落ちるか) を特定し、`.github/workflows/ci.yml` にコメントで「AudioToolbox の内部状態 / self-hosted runner のリソース制約により並列不可」の旨を残す。この場合 prek / Makefile も `--test-threads=1` に揃える。
4. `--locked` は issue 0029 の対応と歩調を合わせる。

前提として `!Send` は並列テスト阻害の根拠にならない点をコメントで補足しておく。
