use std::fs::File;
use crate::compress::Compress;
use crate::io_utils::bit_vector::BitVector;
use crate::io_utils::bit_vector_writer::BitVectorWriter;
use crate::io_utils::HUFFMAN_SIGNATURE;
use crate::io_utils::universal_reader::UniversalReader;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::compress::DecompressError;

use crate::compress::byte_writer::ByteWriter;
use crate::compress::Decompress;

type Dictionary = HashMap<u8, BitVector>;

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


struct HuffmanTree
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

    pub fn from_code(file_reader: &mut UniversalReader) -> Result<HuffmanTree, ()>
    {
        let mut head = Node::new(0, 0);
        Self::recreate_from_code_recursive(file_reader, &mut head)?;

        let tree = HuffmanTree{head: Some(head)};
        Ok(tree)
    }

    fn recreate_from_code_recursive(file_reader: &mut UniversalReader, node: &mut Node)
                                    -> Result<(), ()>
    {
        let bit = match file_reader.read_bit()
        {
            Some(b) => b,
            None => return Err(())
        };

        if bit == 0
        {
            let left_son = Node::new(0, 0);
            node.left = Some(Box::new(left_son));

            if let Some(left) = &mut node.left
            {
                Self::recreate_from_code_recursive(file_reader, left)?;
            }

            let right_son = Node::new(0, 0);
            node.right = Some(Box::new(right_son));

            if let Some(right) = &mut node.right
            {
                Self::recreate_from_code_recursive(file_reader, right)?;
            }
        }
        else // bit == 1
        {
            let mut value = 0;
            for shift in (0..8).rev()
            {
                let next_bit = match file_reader.read_bit()
                {
                    Some(b) => b,
                    None => return Err(()),
                };

                value |= next_bit << shift;
            }

            node.data = value;
        }

        Ok(())
    }

    fn get_flat_node_vector(input: File) -> Vec<Node>
    {
        let file_reader = UniversalReader::new(input);
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


pub struct HuffmanCompressor;

impl Compress for HuffmanCompressor
{
    fn compress(&self, input_filename: &str, output_filename: &str) -> Result<(), String>
    {
        let input = match File::open(input_filename)
        {
            Ok(file) => file,
            Err(_) => return Err(format!("Could not open file {}.", input_filename)),
        };

        let huffman_tree = HuffmanTree::new(input);
        if huffman_tree.empty()
        {
            let _empty_file = File::create(output_filename)
                .map_err(|_| format!("Could not create empty file {}.", output_filename))?;

            return Ok(());
        }

        let tree_encoding = huffman_tree.get_tree_encoding();
        let bytes_encoding = huffman_tree.get_bytes_encoding();

        let mut file_writer = match BitVectorWriter::new(output_filename)
        {
            Some(fw) => fw,
            None => return Err(format!("Could not create file writer for {}.", output_filename)),
        };

        // Start writing to file.
        file_writer.write_bit_vector(&BitVector::from_u64(HUFFMAN_SIGNATURE));
        file_writer.write_bit_vector(&tree_encoding);


        // Reopen the file.
        let input = match File::open(input_filename)
        {
            Ok(file) => file,
            Err(_) => return Err(format!("Could not open file {} second time.", input_filename)),
        };
        let mut buffer = UniversalReader::new(input);


        // read byte by byte
        while let Some(byte) = buffer.read_byte()
        {
            let codeword = bytes_encoding.get(&byte)
                .ok_or(&format!("Could not find codeword for byte {:X}", byte))?;

            file_writer.write_bit_vector(codeword);
        }

        Ok(())
    }
}

pub struct HuffmanDecompressor
{
    file_reader: UniversalReader,
    dictionary: Dictionary,
}

impl HuffmanDecompressor
{
    pub fn new(input_file: File) -> Result<HuffmanDecompressor, DecompressError>
    {
        let input_file_size = input_file.metadata()
            .unwrap()
            .len() as usize;

        if input_file_size == 0
        {
            return Err(DecompressError::EmptyFile);
        }

        let mut file_reader = UniversalReader::new(input_file);


        let huffman_tree = HuffmanTree::from_code(&mut file_reader)
            .map_err(|_| DecompressError::BadFormat)?;
        let dictionary = huffman_tree.get_bytes_encoding();

        let decompressor = HuffmanDecompressor
        {
            file_reader,
            dictionary,
        };

        Ok(decompressor)
    }

    fn get_byte_from_codeword(&self, potential_codeword: &BitVector) -> Option<u8>
    {
        self.dictionary.iter()
            .find(|&(_, value)| value == potential_codeword)
            .map(|(&byte, _)| byte)
    }


    fn decompress_somewhere
    (
        &mut self,
        bytes_count: usize,
        output_filename: Option<String>,
        save_to_memory: bool
    )
        -> Result<Option<Vec<u8>>, DecompressError>
    {

        let mut bytes_decompressed = 0;
        let mut potential_result_vector: Option<Vec<u8>> = match save_to_memory
        {
            true => Some(Vec::with_capacity(bytes_count)),
            false => None,
        };

        let mut potential_file_writer = match output_filename
        {
            Some(filename) =>
                {
                    let writer = ByteWriter::new(&filename)
                        .map_err(|_| DecompressError::Other)?;

                    Some(writer)
                }

            None => None,
        };

        let mut potential_codeword = BitVector::new();
        while bytes_decompressed < bytes_count
        {
            let bit = self.file_reader.read_bit()
                .ok_or(DecompressError::FileTooShort)?;
            potential_codeword.push_bit(bit);

            if let Some(byte) = self.get_byte_from_codeword(&potential_codeword)
            {
                if let Some(vector) = &mut potential_result_vector
                {
                    vector.push(byte);
                }

                if let Some(writer) = &mut potential_file_writer
                {
                    writer.write_byte(byte);
                }

                bytes_decompressed += 1;
                potential_codeword.clear();
            }
        }

        Ok(potential_result_vector)
    }
}

impl Decompress for HuffmanDecompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize) -> Result<Vec<u8>, DecompressError>
    {
        let bytes =
            self.decompress_somewhere(bytes_to_get, None, true)?;

        Ok(bytes.unwrap())
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize) -> Result<(), DecompressError>
    {
        self.decompress_somewhere(count, Some(output_filename.to_owned()), false)?;

        Ok(())
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError>
    {
        self.decompress_somewhere(bytes_count, None, false)?;

        Ok(())
    }
}
