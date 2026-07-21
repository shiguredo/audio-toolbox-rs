# release.yml が CI 依存を持たず publish 前にテストも走らせない問題を修正する

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/refactor-release-yml-ci-dependency
- Polished:

## 目的

`.github/workflows/release.yml` の `publish` ジョブが CI ワークフローの成功に依存せず、かつジョブ内で `cargo test` も走らせずに `cargo publish` を実行している状態を修正する。CI 失敗中のコードでタグを push すると、壊れたクレートが crates.io に公開されるリスクがある。

## 優先度根拠

Medium とする。オペレーションミスや CI 落ちに気付かずタグを打つ運用ミスで、crates.io に壊れた版が上がってしまう。crates.io は yank しか出来ず既に配布された版を差し戻せないため、事故ると影響が広い。修正の効果が高い。

## 現状

`.github/workflows/release.yml:38-51`:

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

- `needs: github-release` のみで、`ci.yml` の成功に依存しない (GitHub Actions の workflow 間依存は `workflow_run` で明示する必要がある)。
- ジョブ内で `cargo test` を走らせない。
- `cargo publish` に `--locked` が付いていない (Cargo.lock を無視した依存解決が起こりうる)。

タグ push だけで即 publish が走るため、CI が赤のままでもリリースが飛ぶ。

## 完了条件

- `release.yml` の `publish` ジョブが以下のいずれかで CI 成功と紐付く。
  - 案 A: `publish` ジョブ内で `cargo test --locked` を実行する。
  - 案 B: `on: workflow_run` で `ci.yml` の成功後に発火する。
- `cargo publish` に `--locked` が付く。
- 上記変更が CHANGES.md の `### misc` に記載される。

## 解決方法

推奨は案 A + `--locked` 化。

```yaml
publish:
  needs: github-release
  timeout-minutes: 20
  runs-on: macos-26
  environment: release
  permissions:
    id-token: write
  steps:
    - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
    - name: Install Rust
      run: |
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - name: Update PATH
      run: echo "$HOME/.cargo/bin" >> $GITHUB_PATH
    - run: cargo fmt --all -- --check
    - run: cargo clippy --all-targets -- -D warnings
    - run: cargo test --locked -- --test-threads=1
    - uses: rust-lang/crates-io-auth-action@bbd81622f20ce9e2dd9622e3218b975523e45bbe # v1.0.4
      id: auth
    - run: cargo publish --locked
      env:
        CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

macos-26 が GitHub-hosted で使えるかは確認する (現状の release.yml が macos-26 を指定している前提で継続)。tomllib / Xcode の availability も確認。

案 B (workflow_run) にする場合は release.yml 自体の起動条件を変更する。

なお、issue 0038 (CI の `--test-threads=1` の根拠明確化) と結びつくので、`--test-threads=1` を残すかは 0038 の結果を反映する。
