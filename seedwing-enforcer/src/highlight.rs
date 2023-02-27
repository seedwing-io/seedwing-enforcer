use ropey::Rope;
use roxmltree::{Document, Node};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Position {
    pub line: usize,
    pub position: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.position)
    }
}

impl From<Position> for tower_lsp::lsp_types::Position {
    fn from(value: Position) -> Self {
        Self {
            line: value.line as _,
            character: value.position as _,
        }
    }
}

impl From<tower_lsp::lsp_types::Position> for Position {
    fn from(value: tower_lsp::lsp_types::Position) -> Self {
        Self {
            line: value.line as _,
            position: value.character as _,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Range(pub std::ops::Range<Position>);

impl Deref for Range {
    type Target = std::ops::Range<Position>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Range {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Range> for tower_lsp::lsp_types::Range {
    fn from(value: Range) -> Self {
        Self {
            start: value.0.start.into(),
            end: value.0.end.into(),
        }
    }
}

impl From<tower_lsp::lsp_types::Range> for Range {
    fn from(value: tower_lsp::lsp_types::Range) -> Self {
        Self(std::ops::Range {
            start: value.start.into(),
            end: value.end.into(),
        })
    }
}

pub struct Highlighter<'a> {
    rope: Rope,
    doc: Document<'a>,
}

impl<'a> Highlighter<'a> {
    pub fn new(content: &'a str) -> anyhow::Result<Self> {
        let rope = Rope::from_str(&content);
        let doc = Document::parse(content)?;

        Ok(Self { rope, doc })
    }

    pub fn find<P>(&self, predicate: P) -> anyhow::Result<Option<Range>>
    where
        P: Fn(&Node) -> bool,
    {
        Ok(match self.doc.descendants().find(predicate) {
            Some(node) => Some(Range(self.make_range(node.range())?)),
            None => None,
        })
    }

    fn make_range(
        &self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<std::ops::Range<Position>> {
        Ok(std::ops::Range {
            start: self.make_position(range.start)?,
            end: self.make_position(range.end)?,
        })
    }

    fn make_position(&self, position: usize) -> anyhow::Result<Position> {
        let line = self.rope.try_byte_to_line(position)?;
        let position = position - self.rope.try_line_to_byte(line)?;
        Ok(Position { line, position })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn find(content: &str) -> anyhow::Result<()> {
        let h = Highlighter::new(content)?;

        let dependencies = h.find(|e| e.tag_name().name() == "dependencies")?;

        log::info!("Found: {dependencies:?}");

        if let Some(range) = dependencies {
            log::info!("Found: {range:?}");
        }

        Ok(())
    }

    #[test]
    fn test() {
        env_logger::init();

        let pom1 = include_str!("../test-data/pom1.xml");

        find(&pom1).unwrap();
    }
}
