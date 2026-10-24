#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    Category,
    Database,
    Schema,
    Table,
    View,
    Index,
    Trigger,
    Column,
    ForeignKey,
    Procedure,
    Other(String),
}

/// A node in the object tree a driver exposes.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub name: String,
    pub kind: ObjectKind,
    pub has_children: bool,
}

/// Location of an object in the tree, as the names of its ancestors and itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ObjectPath(Vec<String>);

impl ObjectPath {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn child(&self, seg: &str) -> Self {
        let mut segs = self.0.clone();
        segs.push(seg.to_owned());
        Self(segs)
    }

    pub fn parent(&self) -> Option<Self> {
        let (_, rest) = self.0.split_last()?;
        Some(Self(rest.to_vec()))
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Page {
    pub offset: u64,
    pub limit: u32,
}
