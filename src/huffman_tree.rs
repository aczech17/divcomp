use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use crate::file_reader::FileReader;
use crate::bit_vector::BitVector;

#[derive(Clone, Eq, PartialEq)]
struct Node
{
    data: u8,
    frequency: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node
{
    fn new(data: u8, frequency: usize) -> Node
    {
        Node
        {
            data,
            frequency,
            left: None,
            right: None,
        }
    }

    fn join(node1: Node, node2: Node) -> Node
    {
        let mut new_node = Node::new(0, node1.frequency + node2.frequency);
        new_node.left = Some(Box::new(node1));
        new_node.right = Some(Box::new(node2));

        new_node
    }
}

impl PartialOrd<Self> for Node
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

impl Ord for Node
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        other.frequency.cmp(&self.frequency)
            .then_with(|| self.data.cmp(&other.data))
    }
}


pub struct HuffmanTree
{
    head: Option<Node>,
}

impl HuffmanTree
{
    pub fn new(input: File) -> HuffmanTree
    {
        let node_vector = Self::get_flat_node_vector(input);
        if node_vector.is_empty()
        {
            return HuffmanTree {head: None};
        }

        let mut nodes = BinaryHeap::from(node_vector);
        while nodes.len() > 1
        {
            let node1 = nodes.pop()
                .unwrap();
            let node2 = nodes.pop()
                .unwrap();

            let joined_node = Node::join(node1, node2);
            nodes.push(joined_node);
        }

        let head = nodes.pop()
            .unwrap();

        HuffmanTree
        {
            head: Some(head),
        }
    }

    pub fn from_code(file_reader: &mut FileReader) -> HuffmanTree
    {
        let mut head = Node::new(0, 0);
        Self::recreate_from_code_recursive(file_reader, &mut head);

        HuffmanTree{head: Some(head)}
    }

    fn recreate_from_code_recursive(file_reader: &mut FileReader, node: &mut Node)
    {
        let bit = file_reader.read_bit()
            .expect(&format!("{} bits read so far.", file_reader.bits_read()));

        if bit == 0
        {
            let left_son = Node::new(0, 0);
            node.left = Some(Box::new(left_son));

            if let Some(left) = &mut node.left
            {
                Self::recreate_from_code_recursive(file_reader, left);
            }

            let right_son = Node::new(0, 0);
            node.right = Some(Box::new(right_son));

            if let Some(right) = &mut node.right
            {
                Self::recreate_from_code_recursive(file_reader, right);
            }
        }
        else // bit == 1
        {
            let mut value = 0;
            for shift in (0..8).rev()
            {
                let next_bit = file_reader.read_bit().unwrap();
                value |= next_bit << shift;
            }

            node.data = value;
        }
    }

    fn get_flat_node_vector(input: File) -> Vec<Node>
    {
        let file_reader = FileReader::new(input);
        let mut frequency_map = HashMap::new();

        for byte in file_reader
        {
            match frequency_map.get(&byte)
            {
                Some(freq) => frequency_map.insert(byte, freq + 1),
                None => frequency_map.insert(byte, 1),
            };
        }

        let flat_node_vector: Vec<Node> = frequency_map
            .iter().map(|(&byte, &freq)|
            Node::new(byte, freq))
            .collect();

        flat_node_vector
    }

    pub fn get_tree_encoding(&self) -> BitVector
    {
        let mut encoding = BitVector::new();
        if let Some(tree_head) = &self.head
        {
            self.make_tree_encoding_recursive(tree_head, &mut encoding);
        }

        encoding
    }

    fn make_tree_encoding_recursive(&self, node: &Node, encoding: &mut BitVector)
    {
        if let (Some(left_node), Some(right_node)) =
            (&node.left, &node.right)
        {
            encoding.push_bit(0);
            self.make_tree_encoding_recursive(left_node, encoding);
            self.make_tree_encoding_recursive(right_node, encoding);
        }
        else // is a leaf
        {
            encoding.push_bit(1);
            encoding.push_byte(node.data);
        }
    }

    pub fn get_bytes_encoding(&self) -> HashMap<u8, BitVector>
    {
        let mut map = HashMap::new();
        let mut code = BitVector::new();

        if let Some(tree_head) = &self.head
        {
            if tree_head.left.is_none() // only one node
            {
                let mut zero_bit = BitVector::new();
                zero_bit.push_bit(0);

                map.insert(tree_head.data, zero_bit);
                return map;
            }

            self.make_bytes_encoding_recursive(tree_head, &mut code, &mut map);
        }

        map
    }

    fn make_bytes_encoding_recursive
        (&self, node: &Node, code: &mut BitVector, codes: &mut HashMap<u8, BitVector>)
    {
        if let (Some(left_node), Some(right_node)) =
            (&node.left, &node.right)
        {
            code.push_bit(0);
            self.make_bytes_encoding_recursive(left_node, code, codes);
            code.pop_bit();

            code.push_bit(1);
            self.make_bytes_encoding_recursive(right_node, code, codes);
            code.pop_bit();
        }
        else // is a leaf
        {
            codes.insert(node.data, code.clone());
        }
    }

    pub fn empty(&self) -> bool
    {
        self.head.is_none()
    }
}
