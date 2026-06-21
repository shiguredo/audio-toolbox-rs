# Decoder のコールバックが 1 回の AudioConverterFillComplexBuffer 呼び出しで同じパケットを複数回返す

- Priority: High
- Created: 2026-06-21
- Completed: {YYYY-MM-DD}
- Model: Kimi K2.7 Code
- Branch: feature/fix-decoder-callback-feeds-same-packet-multiple-times
- Polished: 2026-06-21
- Reporter: @voluntas

## 目的

shiguredo_audio_toolbox::Decoder の next_frame() 内部で、AudioConverterFillComplexBuffer のコールバックが複数回呼ばれた際に同じ入力パケットを複数回返してしまう問題を修正する。結果として 1 回の decode() → next_frame() サイクルで、投入した 1 パケットが 1 回だけデコードされ、コーデック固有の本来のフレーム数のみが返されるようにする。

## 優先度根拠

High とする。本バグは 1 回の next_frame() で本来のフレーム数を大きく超えた PCM を返し、トランスコード後の音声が重複・無音を含む不正な内容になる。mp4dropxpd 等の呼び出し側では長さと内容の両方が狂うため、製品として致命的な品質劣化につながる。

## 現状

Decoder::decode_impl （src/lib.rs 852-908 行目）は次の流れになっている：

1. io_packets = DECODE_BUF_FRAMES （5760）を設定して AudioConverterFillComplexBuffer を呼ぶ
2. 呼び出しが戻ってから self.encoded_buf.clear() （885 行目）を実行する

一方、Decoder::callback （919-982 行目）は encoded_buf が空でなければ常に 1 パケットを返す （950 行目、964-965 行目）。そのため、AudioConverterFillComplexBuffer がコールバックを複数回呼んだ場合、2 回目以降もまだクリアされていない encoded_buf を再び返す。

AAC-LC の場合、1 パケット = 1024 フレームなので、5760 フレーム分の出力を作ろうとするとコールバックが複数回呼ばれる可能性がある。2 回目以降のコールバックでも同じ encoded_buf が返されるため、同じパケットが重複してデコードされる。

なお、Decoder::decode （818-834 行目）は未消費のパケットがある状態で再度 decode されるとエラーを返すようになっており、複数回の decode 呼び出しによるパケット連結は防止されている （issues/closed/0001-bug-decoder-decode-concatenates-packets.md で対応済み）。本 issue は、1 回の decode 投入に対する 1 回の next_frame() 呼び出しの中で発生する、同じパケットの複数回提供という別の問題である。

### 再現手順（概念）

1. Decoder::new で AAC-LC / 48000 Hz / 2ch のデコーダーを作成する。
2. 有効な非無音の AAC-LC パケット A （1024 フレーム分）を decode(&packet_a) で投入する。
3. next_frame() を呼ぶ。内部では io_packets = 5760 で AudioConverterFillComplexBuffer が呼ばれる。
4. AudioConverterFillComplexBuffer がコールバックを複数回呼ぶ場合、2 回目以降も encoded_buf に A が残っているため、A を再度返す。
5. 結果として、next_frame() から返される PCM に重複フレームが含まれる。

非無音のパケットを使うのは、無音パケットの場合でも同じパケットが複数回返されればゼロデータの重複となり検出しにくいためである。

## 影響範囲

修正後、1 回の next_frame() が返すフレーム数はコーデックの 1 パケットあたりのフレーム数を大きく超えなくなる （AAC-LC: 1024、MP3: 1152、Opus: 最大 5760）。呼び出し側は decode() → next_frame() ループを継続して使うため、API 互換性は維持される。ただし、AAC 等のコーデックはプライミングサンプルを含むため、1 回の next_frame() で返るフレーム数が必ずしも 1024 等の理論値と一致しない点に注意が必要である。

## 設計方針

Decoder に「現在の decode_impl 呼び出しですでにパケットを提供したか」を追跡するフラグを追加する。フラグの有効期間は 1 回の AudioConverterFillComplexBuffer 呼び出し内のみとし、decode_impl 開始時にリセットする。

- コールバック 1 回目：パケットを提供し、フラグを true にする
- コールバック 2 回目以降：*io_number_data_packets = 0 を設定し、K_NO_MORE_INPUT を返す

K_NO_MORE_INPUT は「今はこれ以上入力がないが、今後の decode_impl 呼び出しでさらに入力がありうる」を示す独自エラーコードである。AudioConverter はこのステータスを受け取っても、既に供給された 1 パケットから生成可能な出力を output_buffer_list に書き戻して返すことを期待する。decode_impl 側でも K_NO_MORE_INPUT 受信時に output_buffer_list を確認し、生成済みの出力を破棄しないようにする。実装上はこの挙動を macOS 上で検証する必要がある。

コールバックで noErr + 0 パケットを返すと、AudioConverter にストリーム終端を誤通知し、後続のパケットがデコードされなくなる恐れがある。そのため、1 パケット提供後の追加入力不足は K_NO_MORE_INPUT で表す。

finish() 後かつ encoded_buf が空の場合は、これ以上入力がないことを示すため noErr + 0 パケットを返す（既存通り）。

encoded_buf をコールバック内で即座にクリアする案は採用しない。AudioConverter がコールバック戻り後も mData ポインタを非同期に参照する可能性を排除できないため、クリアは AudioConverterFillComplexBuffer 呼び出し戻り後 （885 行目）に維持する。

## 完了条件

- 1 回の decode → next_frame サイクルで、同じパケットが AudioConverter に複数回提供されないこと
- AAC-LC / MP3 / Opus いずれも、1 パケットあたりの出力がコーデックの仕様に沿った範囲に大きく超えないこと
  - AAC-LC: 1024 フレーム / パケット
  - MP3: 1152 フレーム / パケット
  - Opus: RFC 6716 §2.1.4 に基づき 120 ms = 最大 5760 フレーム / パケット（48 kHz 時）
- tests/test_decoder.rs に、Encoder で生成した非無音の AAC-LC パケットを使った回帰テストを追加し、以下を検証すること
  - 複数パケットを連続投入した場合、各パケットがデコードされ、ストリーム終端誤通知により後続パケットが無視されないこと
  - 1 回の next_frame() で返るフレーム数が、AAC-LC の 1 パケットあたりフレーム数（1024）の 2 倍（2048 フレーム）を超えないこと
  - 連続投入したパケット群の総返却フレーム数が、投入パケット数 × 1024 フレームの近傍（プライミングサンプル等を考慮した許容範囲内）であること
- CHANGES.md の develop に [FIX] で追記すること
  - エントリ例：「Decoder のコールバックが 1 回の AudioConverterFillComplexBuffer 呼び出しで同じパケットを複数回返していた不具合を修正する」

## 解決方法

Decoder 構造体に packet_provided_in_this_fill: bool フィールドを追加し、以下のように制御する：

1. decode_impl 開始時に packet_provided_in_this_fill = false にリセットする
2. callback の先頭で、以下の順序で判定する。packet_provided_in_this_fill のチェックを encoded_buf.is_empty() より先に置くのは、同じ fill 呼び出し内で既にパケットを提供済みの場合、残っている encoded_buf を誤って再提供するのを防ぐためである：
   - ポインタ null チェック
   - packet_provided_in_this_fill == true なら *io_number_data_packets = 0 を設定し、io_data.mBuffers[0].mDataByteSize も 0 に設定して K_NO_MORE_INPUT を返す。out_data_packet_description が null でなければ *out_data_packet_description = std::ptr::null_mut() とする
   - encoded_buf.is_empty() の既存チェック（空で eos なら *io_number_data_packets = 0 で noErr、空で eos == false なら *io_number_data_packets = 0 で K_NO_MORE_INPUT）
3. パケットを提供した直後に packet_provided_in_this_fill = true にする
4. decode_impl で AudioConverterFillComplexBuffer 呼び出し後、status が 0 でも K_NO_MORE_INPUT でもない場合のみエラーとする。status が 0 または K_NO_MORE_INPUT の場合は output_buffer_list を通常通り処理し、生成された PCM があれば返す
5. decode_impl 終了時に encoded_buf.clear() を実行する（既存通り）
