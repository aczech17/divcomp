use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use crate::file_reader::FileReader;

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

    fn join(self, node2: Node) -> Node
    {
        let mut new_node = Node::new(0, self.frequency + node2.frequency);
        new_node.left = Some(Box::new(self));
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
    }
}


struct HuffmanTree
{
    head: Option<Node>,
}

impl HuffmanTree
{
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

            let joined_node = node1.join(node2);
            nodes.push(joined_node);
        }

        let head = nodes.pop()
            .unwrap();

        HuffmanTree
        {
            head: Some(head),
        }
    }
}
