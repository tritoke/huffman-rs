# huffman-rs

An implementation of huffman coding from first principles in rust.

## Why?
I saw [this video from Reducible](https://www.youtube.com/watch?v=B3y0RsVCyrw) on youtube and thought it sounded fun to write it in rust :)

## Does it work?
Yeah somehow lol, it can create a huffman encoding / decoding table from any generic stream of symbols.
The encoding / decoding table are also serialisable with `serde` so a compressed message and encoding / decoding table can all be serialised into one file.

## Examples

`examples/encode_text.rs` shows an encoder and decoder being created for a given piece of text, they are then used to encode and decode that text.
You can run it with `cargo run --example encode_text`

`examples/encode_file.rs` encodes the file and then serialises the encoded data, the encoder and the decoder using `serde` + `rmp-serde` to encode it in the `messagepack` binary format.
It then reads this encoded data back from the file, de-serialises it and decodes the encoded symbols.
You can run it with `cargo run --example encode_file --release -- /path/to/file/to/encode.txt`
