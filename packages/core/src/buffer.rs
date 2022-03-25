use bevy::ecs::prelude::*;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::{rbtree::AtomicLink, KeyAdapter, RBTree};
use std::{convert::From, fs};

#[derive(Component, Default, Debug)]
pub struct TextBuffer {
    file_path: Option<&'static str>,
    pub tree: RBTree<PieceAdapter>,
    original: Vec<u8>,
    info: TextBufferInfo,
}

impl From<&'static str> for TextBuffer {
    fn from(file_path: &'static str) -> TextBuffer {
        let original = fs::read(file_path.clone()).expect("Failed to read file");
        if original.is_empty() {
            return TextBuffer::default();
        }

        let bytes = original.as_slice();
        let info = TextBufferInfo::new(bytes);

        let mut tree = RBTree::new(PieceAdapter::new());
        let piece = Piece::new(
            0,
            BufferCursor::new(0, 0),
            BufferCursor::new(
                info.line_starts.len() as i32 - 1,
                match info.line_starts.last() {
                    Some(x) => original.len() as i32 - x,
                    None => 0,
                },
            ),
            bytes.len() as i32,
            info.line_starts.len() as i32 - 1,
        );
        tree.insert(Box::new(piece));

        TextBuffer {
            file_path: Some(file_path),
            tree,
            original,
            info,
        }
    }
}

impl TextBuffer {
    pub fn insert(&self, offset: i32, text: &str) {
        if self.tree.is_empty() {
            println!("tree is empty");
        } else {
            println!("tree is not empty");
        }

        todo!("insert");
    }

    pub fn delete(&self, offset: i32, count: i32) {
        todo!("delete");
    }

    pub fn as_str(&self) -> &'static str {
        todo!("as_str");
    }
}

const UTF8_BOM: &str = "\u{feff}";

#[derive(Debug)]
enum CharacterEncoding {
    Utf8,
    Utf8WithBom,
}

impl Default for CharacterEncoding {
    fn default() -> CharacterEncoding {
        CharacterEncoding::Utf8
    }
}

impl From<&[u8]> for CharacterEncoding {
    fn from(s: &[u8]) -> Self {
        if s.starts_with(UTF8_BOM.as_bytes()) {
            CharacterEncoding::Utf8WithBom
        } else {
            CharacterEncoding::Utf8
        }
    }
}

#[derive(Debug, Default)]
pub struct TextBufferInfo {
    encoding: CharacterEncoding,
    line_starts: Vec<i32>,
    line_break_count: LineBreakCount,
    // is_basic_ascii: bool,
    // contains_rtl: bool,
    // contains_unusual_line_terminators: bool,
    // is_basic_ascii: bool,
    // normalize_eol: bool,
}

impl TextBufferInfo {
    fn new(bytes: &[u8]) -> TextBufferInfo {
        let mut info = TextBufferInfo::default();
        info.encoding = CharacterEncoding::from(bytes);

        let mut enumerate = bytes.iter().enumerate();
        while let Some((i, c)) = enumerate.next() {
            match *c as char {
                '\r' => match enumerate.nth(i + 1) {
                    Some((_, c)) => match *c as char {
                        '\r' => {
                            info.line_starts.push(i as i32 + 2);
                            info.line_break_count.crlf += 1;
                        }
                        _ => {
                            info.line_starts.push(i as i32 + 1);
                            info.line_break_count.cr += 1;
                        }
                    },
                    None => {}
                },
                '\n' => {
                    info.line_starts.push(i as i32 + 1);
                    info.line_break_count.lf += 1;
                }
                _ => {}
            }
        }

        info
    }
}

#[derive(Debug, Default)]
struct LineBreakCount {
    cr: i32,
    lf: i32,
    crlf: i32,
}

#[derive(Default, Debug)]
pub struct Piece {
    link: AtomicLink,
    offset: i32,
    start: BufferCursor,
    end: BufferCursor,
    length: i32,
    line_feed_count: i32,
}

intrusive_adapter!(pub PieceAdapter = Box<Piece>: Piece { link: AtomicLink });
impl<'a> KeyAdapter<'a> for PieceAdapter {
    type Key = i32;
    fn get_key(&self, e: &'a Piece) -> i32 {
        e.offset
    }
}

impl Piece {
    pub fn new(
        offset: i32,
        start: BufferCursor,
        end: BufferCursor,
        length: i32,
        line_feed_count: i32,
    ) -> Self {
        Self {
            offset,
            start,
            end,
            length,
            line_feed_count,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct BufferCursor {
    line: i32,
    column: i32,
}

impl BufferCursor {
    fn new(line: i32, column: i32) -> Self {
        Self { line, column }
    }
}

#[cfg(test)]
mod inserts_and_deletes {
    use crate::buffer::TextBuffer;
    #[test]
    fn basic_insert_and_delete() {
        let buffer = TextBuffer::default();
        buffer.insert(0, "This is a document with some text.");
        assert_eq!(
            buffer.as_str(),
            "This is a document with some text."
        );

        buffer.insert(34, "This is some more text to insert at offset 34.");
        assert_eq!(
            buffer.as_str(),
            "This is a document with some text.This is some more text to insert at offset 34."
        );

        buffer.delete(42, 5);
        assert_eq!(
            buffer.as_str(),
            "This is a document with some text.This is more text to insert at offset 34."
        );
    }
}
