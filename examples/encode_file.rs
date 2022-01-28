use bitvec::prelude::*;
use huffman::huffman;
use serde::{Serialize, Deserialize};
use std::hash::Hash;
use std::fs;
use std::env;

#[derive(Serialize, Deserialize)]
struct HuffmanSerialized<Symbol>
where
    Symbol: Clone + Eq + Hash + Serialize,
{
    data: Box<[usize]>,
    bit_len: usize,
    encoder: huffman::SerializableEncoder<Symbol>,
    decoder: huffman::SerializableDecoder<Symbol>,
}

impl<Symbol> HuffmanSerialized<Symbol>
where
    Symbol: Clone + Eq + Hash + Serialize,
{
    fn new(bv: BitVec, e: &huffman::Encoder<Symbol>, d: &huffman::Decoder<Symbol>) -> Self {
        let bl = bv.len();

        Self {
            data: bv.into_boxed_bitslice().into_boxed_slice(),
            bit_len: bl,
            encoder: e.into(),
            decoder: d.into(),
        }
    }

    fn into_parts(self) -> (BitVec, huffman::Encoder<Symbol>, huffman::Decoder<Symbol>) {
        let Self { data, bit_len, encoder, decoder } = self;

        let mut bv = BitBox::from_boxed_slice(data).into_bitvec();
        bv.resize(bit_len, false);

        (bv, encoder.into(), decoder.into())
    }
}

fn main() {
    let fp = env::args().nth(1).expect("Please provide path to input file as first argument.");

    let input_bytes = fs::read(fp).expect("First argument was not a valid filepath.");

    // encode scope - save to file
    {
        let tree = huffman(input_bytes.clone()).unwrap();
        let (e, d) = tree.into_encoder_decoder_pair();

        let encoded = e.encode(input_bytes.iter().copied());
        let packed = HuffmanSerialized::new(encoded, &e, &d);
        let data = rmp_serde::to_vec(&packed).unwrap();

        fs::write("encoded.mp", data).unwrap();
    }

    // decode scope - read from file
    {
        let file_data = fs::read("encoded.mp").unwrap();

        let packed: HuffmanSerialized<u8> = rmp_serde::from_read_ref(&file_data).unwrap();
        let (enc, _, d) = packed.into_parts();
        let decoded = d.decode(&enc);

        fs::write("decoded.txt", decoded).unwrap();
    }
}
