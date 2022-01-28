# huffman-rs

An implementation of huffman coding from first principles in rust.

## Why?
I saw [this video from Reducible](https://www.youtube.com/watch?v=B3y0RsVCyrw) on youtube and thought it sounded fun to write it in rust :)

## Does it work?
Yeah somehow lol, it can create a huffman encoding / decoding table from any generic stream of symbols.
The encoding / decoding table are also serialisable with `serde` so a compressed message and encoding / decoding table can all be serialised into one file.
