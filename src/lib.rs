use bitvec::prelude::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use total_float_wrap::TotalF64;

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<Symbol> {
    probability: TotalF64,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    #[derivative(Hash = "ignore")]
    symbol: Option<Symbol>,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    #[derivative(Hash = "ignore")]
    left: Option<Box<Node<Symbol>>>,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    #[derivative(Hash = "ignore")]
    right: Option<Box<Node<Symbol>>>,
}

impl<Symbol> Node<Symbol>
where
    Symbol: Hash + Eq + Clone + Serialize,
{
    fn new(s: Symbol, p: f64) -> Self {
        Self {
            probability: TotalF64(p),
            symbol: Some(s),
            left: None,
            right: None,
        }
    }

    fn from_children(left: Node<Symbol>, right: Node<Symbol>) -> Self {
        Self {
            probability: TotalF64(left.probability.0 + right.probability.0),
            symbol: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    pub fn into_encoder_decoder_pair(self) -> (Encoder<Symbol>, Decoder<Symbol>) {
        fn traverse<Symbol: Clone>(
            node: &Node<Symbol>,
            v: &mut BitVec,
            dec: &mut HashMap<BitVec, Symbol>,
        ) {
            // symbol nodes have no children
            if let Some(sym) = &node.symbol {
                dec.insert(v.clone(), sym.clone());
                return;
            }

            if let Some(left) = &node.left {
                v.push(true);
                traverse(&left, v, dec);
                v.pop();
            }

            if let Some(right) = &node.right {
                v.push(false);
                traverse(&right, v, dec);
                v.pop();
            }
        }

        let mut bv = BitVec::new();
        let mut dec = HashMap::new();
        traverse(&self, &mut bv, &mut dec);

        let enc = dec
            .iter()
            .map(|(k, v)| (v.clone(), k.clone().into_boxed_bitslice()))
            .collect();

        (Encoder { encode_table: enc }, Decoder { decode_table: dec })
    }
}

#[derive(Debug, Clone)]
pub struct Encoder<Symbol> {
    encode_table: HashMap<Symbol, BitBox>,
}

impl<Symbol> Encoder<Symbol>
where
    Symbol: Eq + Hash,
{
    pub fn encode(&self, stream: impl Iterator<Item = Symbol>) -> BitVec {
        let mut out = BitVec::new();
        for s in stream {
            out.extend_from_bitslice(self.encode_table.get(&s).unwrap());
        }

        out
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableEncoder<Symbol>
where
    Symbol: Hash + Eq,
{
    encode_table: HashMap<Symbol, (usize, Box<[usize]>)>,
}

impl<'a, Symbol> From<&'a Encoder<Symbol>> for SerializableEncoder<Symbol>
where
    Symbol: Clone + Eq + Hash + Serialize,
{
    fn from(other: &'a Encoder<Symbol>) -> Self {
        Self {
            encode_table: other.encode_table
                .iter()
                .map(|(k, v)| {
                    // serialize a BitBox as a pair of usize, Box<[usize]>
                    let len = v.len();
                    let slice = v.clone().into_boxed_slice();

                    (k.clone(), (len, slice))
                })
                .collect(),
        }
    }
}

impl<Symbol> From<SerializableEncoder<Symbol>> for Encoder<Symbol>
where
    Symbol: Hash + Eq,
{
    fn from(other: SerializableEncoder<Symbol>) -> Self {
        Self {
            encode_table:
                other.encode_table
                    .into_iter()
                    .map(|(k, (len, bs))| {
                        let mut bv = BitBox::from_boxed_slice(bs).into_bitvec();
                        bv.resize(len, false);
                        (k, bv.into_boxed_bitslice())
                    })
                    .collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decoder<Symbol> {
    decode_table: HashMap<BitVec, Symbol>,
}

impl<Symbol> Decoder<Symbol>
where
    Symbol: Clone,
{
    pub fn decode(&self, input: &BitSlice) -> Vec<Symbol> {
        let mut out = Vec::new();

        let mut cursor = BitVec::new();
        for b in input.iter().by_vals() {
            cursor.push(b);
            if let Some(sym) = self.decode_table.get(&cursor) {
                cursor.clear();
                out.push(sym.clone());
            }
        }

        out
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableDecoder<Symbol> {
    decode_table: HashMap<(usize, Box<[usize]>), Symbol>,
}

impl<'a, Symbol> From<&'a Decoder<Symbol>> for SerializableDecoder<Symbol>
where
    Symbol: Clone + Serialize,
{
    fn from(other: &'a Decoder<Symbol>) -> Self {
        Self {
            decode_table: other.decode_table
                .iter()
                .map(|(k, v)| {
                    // serialize a BitBox as a pair of usize, Box<[usize]>
                    let len = k.len();
                    let slice = k.clone().into_boxed_bitslice().into_boxed_slice();

                    ((len, slice), v.clone())
                })
                .collect(),
        }
    }
}

impl<Symbol> From<SerializableDecoder<Symbol>> for Decoder<Symbol> {
    fn from(other: SerializableDecoder<Symbol>) -> Self {
        Self {
            decode_table:
                other.decode_table
                    .into_iter()
                    .map(|((len, bs), v)| {
                        let mut bv = BitBox::from_boxed_slice(bs).into_bitvec();
                        bv.resize(len, false);
                        (bv, v)
                    })
                    .collect()
        }
    }
}

pub fn huffman<Symbol: Eq + Clone + Hash + Serialize>(
    symbols: Vec<Symbol>,
) -> Option<Node<Symbol>> {
    let num_symbols = symbols.len();
    let mut freq: HashMap<Symbol, usize> = HashMap::new();
    for s in symbols {
        *freq.entry(s).or_default() += 1;
    }

    let mut pq: BinaryHeap<_> = freq
        .into_iter()
        .map(|(s, count)| Reverse(Node::new(s, count as f64 / num_symbols as f64)))
        .collect();

    while pq.len() > 1 {
        let left = pq.pop().unwrap();
        let right = pq.pop().unwrap();
        pq.push(Reverse(Node::from_children(left.0, right.0)));
    }

    pq.pop().map(|r| r.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_new() {
        let n = Node::new(true, 0.1);
        assert_eq!(n.probability, TotalF64(0.1));
        assert_eq!(n.symbol, Some(true));
        assert_eq!(n.left, None);
        assert_eq!(n.right, None);
    }

    #[test]
    fn node_from_children() {
        let left = Node::new(true, 0.5);
        let right = Node::new(false, 0.25);

        let n = Node::from_children(left.clone(), right.clone());

        assert_eq!(n.probability, TotalF64(0.75));
        assert_eq!(n.symbol, None);
        assert_eq!(n.left, Some(Box::new(left)));
        assert_eq!(n.right, Some(Box::new(right)));
    }

    #[test]
    fn node_compare_prob_only() {
        let a: Node<bool> = Node::new(true, 0.1);
        let b: Node<bool> = Node::new(false, 0.1);

        assert_eq!(a, b);
    }

    #[test]
    fn node_compare_ordering() {
        for i in 1..=1000 {
            let a: Node<bool> = Node::new(true, 0.1 * i as f64);
            let b: Node<bool> = Node::new(false, 0.1 * i as f64 + 0.1);

            assert!(a < b);
        }
    }

    #[test]
    fn test_encode_decode() {
        let s = String::from(
            "This is a really long message, I sure do hope it encodes and decodes properly.",
        );
        let tree = huffman(s.bytes().collect::<Vec<_>>()).unwrap();
        let (e, d) = tree.into_encoder_decoder_pair();

        let out = e.encode(s.bytes());
        let dec = String::from_utf8(d.decode(&out)).unwrap();

        assert_eq!(dec, s);
    }
}
