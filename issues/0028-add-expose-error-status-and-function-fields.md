# Error 型の status / function フィールドを公開して呼び出し側が OSStatus を握れるようにする

- Priority: Medium
- Created: 2026-07-21
- Completed:
- Model: Opus 4.7
- Branch: feature/add-expose-error-fields
- Polished: 2026-07-31

## 目的

`Error` 型の `status: i32` / `function: &'static str` フィールドは現状 private で、crate 外からアクセサ経由でも取得できない。README / SKILL.md のフィールド一覧テーブルが公開 API の表記と区別できないため、実装とドキュメントが齟齬している。フィールドを `pub` にすることで実装とドキュメントを整合させ、呼び出し側が OSStatus を握って分岐する典型ユースケースに対応する。

## 優先度根拠

Medium とする。動作影響は無いが、実装とドキュメントの齟齬は利用者にとって「書いてあるのに使えない」状態で信頼を損なう。呼び出し側が特定 OSStatus をハンドリングする際、現状は `Display` 文字列のパースという壊れやすい方式しか取れない。本変更は後方互換のある公開 API の追加 ([ADD]) に該当する (0034 の書式変更は [CHANGE] となり得るが、本 issue 単体では追加のみ)。

## 現状

`src/lib.rs` の `Error` 構造体:

```rust
#[derive(Debug)]
pub struct Error {
    /// AudioConverter API が返したステータスコード (OSStatus)
    status: i32,
    /// エラーが発生した API 関数名
    function: &'static str,
}
```

- フィールドに `pub` が無い。
- `impl Error` にはアクセサ (`pub fn status(&self) -> i32` 等) も無い。
- `Error::check` も `pub` ではないため crate 外から `Error` を組み立てる手段は無い (これは意図的で正しい)。

一方 README.md の `### Error` 節は「フィールド / 型 / 説明」のテーブルで `status` / `function` を列挙し、`skills/shiguredo-audio-toolbox/SKILL.md` の「エラー型」節も「フィールド」列に `status: i32` (OSStatus) / `function: &'static str` を併記しており、他型の public フィールドと区別が付かない書き方になっている。なお SKILL.md の「`Error::check(status, function)` 経由でしか構築されない」という記述は、`src/lib.rs` 内で `Error { ... }` を直接構築している箇所 (9 箇所) があり実態と食い違っている。

## 完了条件

1. `Error` 構造体の `status` / `function` フィールドが `pub` になる (`grep -n "pub status: i32" src/lib.rs` が 1 件、`grep -n "pub function: &'static str" src/lib.rs` が 1 件)。
2. crate 外 (integration test) から `Error::status` / `Error::function` を直接読んで値を検証するテストが追加され、`cargo test` が成功する (例: `tests/test_encoder.rs` に `Encoder::new` を `sample_rate: 0` で呼び、返った `Error` の `status == -50` と `function == "Encoder::new(sample_rate)"` を assert するテストを追加する)。
3. README.md の `### Error` 節 / SKILL.md の「エラー型」節の記述が実装 (pub フィールド) と整合し、SKILL.md の「`Error::check` 経由でしか構築されない」の記述が実態 (crate 内で `Error::check` または直接リテラル構築により生成される。`status == 0` の Error はライブラリ内では構築されない。フィールドは pub のため crate 外からも構造体リテラルで構築可能だが、通常はライブラリが返した値をそのまま使う) と整合するよう修正される。
4. `CHANGES.md` の develop 直下に [ADD] として追記され (公開 API の追加のため `### misc` ではなく通常エントリ。種別順に従い既存 [FIX] エントリの前に挿入する)、追記エントリに issue 番号・issue ファイル名が含まれない。エントリは shiguredo-changelog スキルのフォーマット (担当者行 `- @ユーザー名` を含む) に従う。

## 解決方法

`Error` 構造体のフィールドを `pub` にする (案 A に確定)。

```rust
#[derive(Debug)]
pub struct Error {
    /// AudioConverter API が返したステータスコード (OSStatus)
    pub status: i32,
    /// エラーが発生した API 関数名
    pub function: &'static str,
}
```

`#[non_exhaustive]` は付けない (shiguredo-rust 規約「`#[non_exhaustive]` を使わないこと」「将来 variant や field を追加するときは素直に破壊的変更として扱うこと」に従う)。

案 A を採用する根拠: 既存ドキュメントの「フィールド」表記および他型 (`EncodedFrame` / `EncoderConfig` 等) の pub フィールド方針と整合するため。フィールドが `pub` になることで crate 外から `Error { ... }` を構築できるようになる (現状の「crate 外から構築できない」という設計は変更される。エラー値はライブラリから返されるのが通常で、利用者が独自構築しても実害はないため許容する)。

アクセサ追加 (案 B) は採らない: crate 外からの構築を禁止したままにできる利点はあるが、ドキュメントの「フィールド」表記を「メソッド」に書き直す必要があり、他型の pub フィールド方針とも揃わないため。

crate 外から `status` / `function` を読むテストを `tests/test_encoder.rs` に追加する (`src/error.rs` が存在しないため、`tests/test_error.rs` は新設しない。shiguredo-rust 規約「単体テストのファイル名は `tests/test_<module>.rs` とし、`src/<module>.rs` に対応させること」に従う)。

README.md / SKILL.md の記述を実装と整合させ、SKILL.md の「`Error::check` 経由でしか構築されない」を実態に合わせて修正する。

`CHANGES.md` の追記エントリは shiguredo-changelog スキルを参照して書く (例: [ADD] `Error` の `status` / `function` フィールドを公開する)。

issue 0034 (`Error::function` の命名一貫性) は本 issue の後に実施する (本 issue で公開方式が確定する)。0034 内の案 B 前提の記述 (優先度根拠の「アクセサ化される想定」や解決方法の「issue 0028 で `pub fn function()` を出す場合は」) と完了条件の「同一ブランチで扱うか判断する」は、本 issue の順序決定 (別ブランチ・後に実施) で確定したため成立しない。0034 の該当記述は 0034 の磨きで修正される。0034 実施時には、本 issue で追加したテストの `function` の assert (`== "Encoder::new(sample_rate)"` 等) も書式変更に合わせて更新対象となる。
