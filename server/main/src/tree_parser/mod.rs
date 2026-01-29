use logging::error;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Query, QueryCursor, Tree, TreeCursor};
use url::Url;

use crate::file::byte_index;

mod definition;
mod reference;
mod simple_lint;
mod symbols;

trait ToLspTypes {
    fn to_location(&self, url: &Url, content: &str, line_mapping: &[usize]) -> Location;
    fn to_range(&self, content: &str, line_mapping: &[usize]) -> Range;
}

impl ToLspTypes for Node<'_> {
    fn to_location(&self, url: &Url, content: &str, line_mapping: &[usize]) -> Location {
        Location {
            uri: url.clone(),
            range: self.to_range(content, line_mapping),
        }
    }

    fn to_range(&self, content: &str, line_mapping: &[usize]) -> Range {
        let start_position = self.start_position();
        let end_position = self.end_position();

        let start_line_index = line_mapping[start_position.row];
        let end_line_index = line_mapping[end_position.row];

        let start_column = content[start_line_index..(start_line_index + start_position.column)]
            .chars()
            .count();
        let end_column = content[end_line_index..(end_line_index + end_position.column)].chars().count();
        Range {
            start: Position {
                line: start_position.row as u32,
                character: start_column as u32,
            },
            end: Position {
                line: end_position.row as u32,
                character: end_column as u32,
            },
        }
    }
}

pub struct TreeParser;

impl TreeParser {
    fn current_node_fetch<'a>(position: Position, tree: &'a Tree, content: &str, line_mapping: &[usize]) -> Option<Node<'a>> {
        let position_offset = byte_index(content, position, line_mapping).0;

        let (start, end) = match content.as_bytes()[position_offset] {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' => (position_offset, position_offset + 1),
            _ => (position_offset - 1, position_offset),
        };
        tree.root_node().named_descendant_for_byte_range(start, end)
    }

    fn simple_global_search(url: &Url, tree: &Tree, content: &str, query_str: &str, line_mapping: &[usize]) -> Vec<Location> {
        let mut locations = vec![];

        let query = match Query::new(&tree_sitter_glsl::LANGUAGE_GLSL.into(), query_str) {
            Ok(query) => query,
            Err(err) => {
                error!("Query in `simple_global_search` error: {err}");
                return locations;
            }
        };

        let mut query_cursor = QueryCursor::new();

        query_cursor
            .matches(&query, tree.root_node(), content.as_bytes())
            .for_each(|query_match| {
                locations.extend(
                    query_match
                        .captures
                        .iter()
                        .map(|capture| capture.node.to_location(url, content, line_mapping)),
                );
            });

        locations
    }
}
