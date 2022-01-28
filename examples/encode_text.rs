use huffman::huffman;

fn main() {
    let s = String::from("Hello my name is Sam!");
    let tree = huffman(s.bytes().collect::<Vec<_>>()).unwrap();
    let (e, d) = tree.into_encoder_decoder_pair();

    let out = e.encode(s.bytes());
    let dec = String::from_utf8(d.decode(&out));

    println!("{:?}", dec);
}
