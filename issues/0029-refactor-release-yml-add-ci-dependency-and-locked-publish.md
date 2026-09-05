# release.yml の publish ジョブが publish 前にテストを走らせない問題を修正する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-release-yml-add-publish-verification
- Polished: 2026-09-05

## 目的

`.github/workflows/release.yml` の `publish` ジョブがジョブ内で `cargo test` も走らせずに `cargo publish` を実行している状態を修正する。CI はタグ push で発火しないため、CI 未実行のコードでタグを push すると、壊れたクレートが crates.io に公開されるリスクがある。

## 優先度根拠

Medium とする。オペレーションミスや CI 落ちに気付かずタグを打つ運用ミスで、crates.io に壊れた版が上がってしまう。crates.io は yank しか出来ず既に配布された版を差し戻せないため、事故ると影響が広い。修正の効果が高い。

## 現状

`.github/workflows/release.yml` の `publish` ジョブ:

```yaml
  publish:
    needs: github-release
    timeout-minutes: 10
    runs-on: macos-26
    environment: release
    permissions:
      id-token: write
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      - uses: rust-lang/crates-io-auth-action@bbd81622f20ce9e2dd9622e3218b975523e45bbe # v1.0.4
        id: auth
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

- `needs: github-release` のみで、`ci.yml` の成功に依存しない (GitHub Actions の workflow 間依存は `workflow_run` で明示する必要があるが、`ci.yml` はタグ push で発火しないため `workflow_run` での紐付けは成立しない)。
- ジョブ内で `cargo test` を走らせない。
- `cargo publish` に `--locked` が付いていない (Cargo.lock が古いと黙って更新され、コミットされたロックと異なる依存解決で検証が行われる可能性がある)。

GitHub Release 作成成功後に publish が走る。CI はタグ push で発火しないため、CI 未実行のまま publish される。

## 完了条件

1. `publish` ジョブ内で CI 相当の検証 (`cargo fmt --all --check` / `cargo check --workspace --locked` / `cargo clippy --workspace --locked -- -D warnings` / `cargo test --workspace --locked -- --test-threads=1`) が `cargo publish` の前に実行される (`grep -n "cargo test" .github/workflows/release.yml` が 1 件で `--locked` を含む、`grep -n "cargo check" .github/workflows/release.yml` が 1 件で `--locked` を含む、`grep -n "cargo clippy" .github/workflows/release.yml` が 1 件で `--locked` と `-D warnings` を含む、`grep -n "cargo fmt" .github/workflows/release.yml` が 1 件で `--all --check` を含む)。
2. `cargo publish` に `--locked` が付く (`grep -n "cargo publish" .github/workflows/release.yml` が 1 件で `--locked` を含む)。
3. `--test-threads=1` の要否は issue 0038 の結果を反映する (0038 を先に実施し、0038 の結論に従って引数を揃える)。
4. canary.py を実行して canary タグを push し、GitHub Actions の実行ログで `publish` ジョブが検証成功後に publish することを確認する。
5. `CHANGES.md` の develop / `### misc` に [UPDATE] として追記され、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`publish` ジョブに検証ステップを追加し、`cargo publish` を `--locked` で実行する。

```yaml
  publish:
    needs: github-release
    timeout-minutes: 20
    runs-on: macos-26
    environment: release
    permissions:
      id-token: write
    env:
      RUST_BACKTRACE: 1
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      - name: Install Rust
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      - name: Update PATH
        run: echo "$HOME/.cargo/bin" >> $GITHUB_PATH
      - run: cargo fmt --all --check
      - run: cargo check --workspace --locked
      - run: cargo clippy --workspace --locked -- -D warnings
      - run: cargo test --workspace --locked -- --test-threads=1
      - uses: rust-lang/crates-io-auth-action@bbd81622f20ce9e2dd9622e3218b975523e45bbe # v1.0.4
        id: auth
      - run: cargo publish --locked
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

- 検証コマンドの実行に備えて、ci.yml と同様に rustup で stable toolchain を導入する (現状の publish ジョブは GitHub-hosted のプリインストール Rust に依存している)。
- 依存解決を伴うコマンド (`cargo check` / `cargo clippy` / `cargo test`) には全て `--locked` を付け、依存解決を伴う最初のコマンド (`cargo check`) で stale な Cargo.lock を検知して失敗させる (最初のコマンドだけ `--locked` なしだと、そこでロックが黙って更新され後続のガードが機能しないため)。canary 運用は canary.py が `cargo update` 後に Cargo.lock をコミットしているため壊れない。
- 検証コマンドの引数は issue 0038 の完了後に ci.yml と同一になるよう揃える (現時点の ci.yml は `--locked` なしのため、0038 の結論で両者を一致させる)。`--test-threads=1` の要否は 0038 の結論を反映する。
- 検証ステップ追加のため `timeout-minutes` を 10 → 20 に引き上げる。
- CI は self-hosted (macOS ARM64)、`publish` ジョブは GitHub-hosted `macos-26` のため環境は同一ではないが、publish 前の最低限の検証として許容する。`macos-26` は現状の release.yml が使用中で実行実績があるため継続する。canary タグでも `publish` ジョブの検証が走る (検証時間の分だけ遅延する)。
- 案 B (`workflow_run` による CI 依存) は採らない: `ci.yml` はタグ push で発火しないため、`workflow_run` ではタグ push 時に publish が発火せず、リリース機能自体が壊れる。

issue 0038 は本 issue より先に実施する (`--test-threads=1` の要否が本 issue の YAML に影響するため)。issue 0030 (slack-notify ピン留め) と同じ release.yml / CHANGES.md を編集するため、並行作業時はコンフリクトに注意する (編集箇所は別ジョブのため直接の衝突は少ない)。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: `- [UPDATE] release.yml の publish ジョブに検証ステップを追加する`)。
