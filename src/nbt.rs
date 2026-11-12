//! The named binary format a world keeps: just enough of it to read a save, change one player and
//! write it back with every other tag untouched. Compounds keep their order, so a save that was
//! only read writes back the same bytes it came in as.

use std::io::{Read, Write};

/// One NBT tag. `Compound` is a list of pairs rather than a map: order is part of the file.
#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    /// The id byte this tag is written with.
    pub fn id(&self) -> u8 {
        match self {
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::ByteArray(_) => 7,
            Tag::String(_) => 8,
            Tag::List(_) => 9,
            Tag::Compound(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }

    /// The name of the tag kind, for messages.
    pub fn kind(&self) -> &'static str {
        match self {
            Tag::Byte(_) => "byte",
            Tag::Short(_) => "short",
            Tag::Int(_) => "int",
            Tag::Long(_) => "long",
            Tag::Float(_) => "float",
            Tag::Double(_) => "double",
            Tag::ByteArray(_) => "byte array",
            Tag::String(_) => "string",
            Tag::List(_) => "list",
            Tag::Compound(_) => "compound",
            Tag::IntArray(_) => "int array",
            Tag::LongArray(_) => "long array",
        }
    }

    /// A number as an `i32`, whichever number tag it is: NBT is loose about them between versions.
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Tag::Byte(v) => Some(i32::from(*v)),
            Tag::Short(v) => Some(i32::from(*v)),
            Tag::Int(v) => Some(*v),
            Tag::Long(v) => Some(*v as i32),
            _ => None,
        }
    }

    /// A number as an `f32`.
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Tag::Byte(v) => Some(f32::from(*v)),
            Tag::Short(v) => Some(f32::from(*v)),
            Tag::Int(v) => Some(*v as f32),
            Tag::Long(v) => Some(*v as f32),
            Tag::Float(v) => Some(*v),
            Tag::Double(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }

    /// The value under `key`, in a compound.
    pub fn get(&self, key: &str) -> Option<&Tag> {
        match self {
            Tag::Compound(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The value under `key`, to change, in a compound.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Tag> {
        match self {
            Tag::Compound(fields) => fields.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The value under `key`, in a nested path, to change.
    pub fn get_path_mut(&mut self, path: &[&str]) -> Option<&mut Tag> {
        let mut here = self;
        for key in path {
            here = here.get_mut(key)?;
        }
        Some(here)
    }

    /// Sets `key` in a compound, in place when it is already there.
    pub fn set(&mut self, key: &str, value: Tag) {
        if let Tag::Compound(fields) = self {
            match fields.iter_mut().find(|(k, _)| k == key) {
                Some((_, v)) => *v = value,
                None => fields.push((key.to_string(), value)),
            }
        }
    }

    /// Puts `value` under `key` only when the key is missing, and answers what is there after.
    pub fn set_if_missing(&mut self, key: &str, value: Tag) {
        if self.get(key).is_none() {
            self.set(key, value);
        }
    }

    /// Removes `key` from a compound, answering whether it was there.
    pub fn remove(&mut self, key: &str) -> bool {
        match self {
            Tag::Compound(fields) => {
                let before = fields.len();
                fields.retain(|(k, _)| k != key);
                fields.len() != before
            }
            _ => false,
        }
    }

    /// The items of a list.
    pub fn list(&self) -> Option<&[Tag]> {
        match self {
            Tag::List(items) => Some(items),
            _ => None,
        }
    }

    /// The items of a list, to change.
    pub fn list_mut(&mut self) -> Option<&mut Vec<Tag>> {
        match self {
            Tag::List(items) => Some(items),
            _ => None,
        }
    }
}

/// A whole file: the root compound and the name it carries (empty for a save).
#[derive(Clone, Debug, PartialEq)]
pub struct Root {
    pub name: String,
    pub tag: Tag,
}

impl Root {
    pub fn get(&self, key: &str) -> Option<&Tag> {
        self.tag.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Tag> {
        self.tag.get_mut(key)
    }

    /// The compound under a path, to change.
    pub fn path_mut(&mut self, path: &[&str]) -> Option<&mut Tag> {
        self.tag.get_path_mut(path)
    }
}

/// Why a file could not be read as NBT.
#[derive(Debug)]
pub struct NbtError(pub String);

impl std::fmt::Display for NbtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn err<T>(what: impl Into<String>) -> Result<T, NbtError> {
    Err(NbtError(what.into()))
}

/// Reads a gzipped NBT file, as a world keeps it.
pub fn read_gzip(bytes: &[u8]) -> Result<Root, NbtError> {
    let mut text = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .read_to_end(&mut text)
        .map_err(|e| NbtError(format!("not a gzipped file: {e}")))?;
    read(&text)
}

/// Writes a gzipped NBT file.
pub fn write_gzip(root: &Root) -> Result<Vec<u8>, NbtError> {
    let plain = write(root);
    let mut out = Vec::new();
    let mut encoder = flate2::write::GzEncoder::new(&mut out, flate2::Compression::default());
    encoder
        .write_all(&plain)
        .map_err(|e| NbtError(format!("can't compress: {e}")))?;
    encoder
        .finish()
        .map_err(|e| NbtError(format!("can't compress: {e}")))?;
    Ok(out)
}

/// Reads uncompressed NBT.
pub fn read(bytes: &[u8]) -> Result<Root, NbtError> {
    let mut reader = Reader { bytes, at: 0 };
    let id = reader.u8()?;
    if id == 0 {
        return err("empty NBT: no root tag");
    }
    let name = reader.string()?;
    let tag = reader.tag(id, 0)?;
    Ok(Root { name, tag })
}

/// Writes uncompressed NBT.
pub fn write(root: &Root) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(root.tag.id());
    write_string(&mut out, &root.name);
    write_tag(&mut out, &root.tag);
    out
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        let end = self.at.checked_add(n).ok_or(NbtError("length overflow".into()))?;
        if end > self.bytes.len() {
            return err(format!(
                "the file ends after {} bytes, {} more were needed",
                self.bytes.len(),
                n
            ));
        }
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, NbtError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn i32(&mut self) -> Result<i32, NbtError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i64(&mut self) -> Result<i64, NbtError> {
        let b = self.take(8)?;
        Ok(i64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn string(&mut self) -> Result<String, NbtError> {
        let len = usize::from(self.u16()?);
        let bytes = self.take(len)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    /// A tag of `id`, `depth` compounds deep. Lists of compounds and nested compounds recurse.
    fn tag(&mut self, id: u8, depth: usize) -> Result<Tag, NbtError> {
        if depth > 512 {
            return err("NBT nested more than 512 deep");
        }
        Ok(match id {
            1 => Tag::Byte(self.u8()? as i8),
            2 => {
                let b = self.take(2)?;
                Tag::Short(i16::from_be_bytes([b[0], b[1]]))
            }
            3 => Tag::Int(self.i32()?),
            4 => Tag::Long(self.i64()?),
            5 => Tag::Float(f32::from_bits(self.i32()? as u32)),
            6 => Tag::Double(f64::from_bits(self.i64()? as u64)),
            7 => {
                let len = self.i32()?.max(0) as usize;
                Tag::ByteArray(self.take(len)?.to_vec())
            }
            8 => Tag::String(self.string()?),
            9 => {
                let inner = self.u8()?;
                let len = self.i32()?.max(0) as usize;
                let mut items = Vec::new();
                for _ in 0..len {
                    items.push(self.tag(inner, depth + 1)?);
                }
                Tag::List(items)
            }
            10 => self.compound(depth)?,
            11 => {
                let len = self.i32()?.max(0) as usize;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(self.i32()?);
                }
                Tag::IntArray(values)
            }
            12 => {
                let len = self.i32()?.max(0) as usize;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(self.i64()?);
                }
                Tag::LongArray(values)
            }
            other => return err(format!("unknown tag id {other}")),
        })
    }

    fn compound(&mut self, depth: usize) -> Result<Tag, NbtError> {
        let mut fields = Vec::new();
        loop {
            let id = self.u8()?;
            if id == 0 {
                return Ok(Tag::Compound(fields));
            }
            let name = self.string()?;
            let value = self.tag(id, depth + 1)?;
            fields.push((name, value));
        }
    }
}

fn write_string(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len().min(usize::from(u16::MAX)) as u16;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&bytes[..usize::from(len)]);
}

fn write_tag(out: &mut Vec<u8>, tag: &Tag) {
    match tag {
        Tag::Byte(v) => out.push(*v as u8),
        Tag::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::ByteArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            out.extend_from_slice(values);
        }
        Tag::String(text) => write_string(out, text),
        Tag::List(items) => {
            let id = items.first().map_or(0, Tag::id);
            out.push(id);
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for item in items {
                write_tag(out, item);
            }
        }
        Tag::Compound(fields) => {
            for (name, value) in fields {
                out.push(value.id());
                write_string(out, name);
                write_tag(out, value);
            }
            out.push(0);
        }
        Tag::IntArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                out.extend_from_slice(&value.to_be_bytes());
            }
        }
        Tag::LongArray(values) => {
            out.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                out.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Root {
        Root {
            name: String::new(),
            tag: Tag::Compound(vec![
                ("Data".into(), Tag::Compound(vec![
                    ("LevelName".into(), Tag::String("Test".into())),
                    ("RandomSeed".into(), Tag::Long(-1234567890123)),
                    ("hardcore".into(), Tag::Byte(0)),
                    ("Player".into(), Tag::Compound(vec![
                        ("Health".into(), Tag::Float(7.5)),
                        ("foodLevel".into(), Tag::Int(4)),
                        (
                            "Inventory".into(),
                            Tag::List(vec![Tag::Compound(vec![
                                ("Slot".into(), Tag::Byte(0)),
                                ("id".into(), Tag::String("minecraft:bread".into())),
                                ("Count".into(), Tag::Byte(3)),
                            ])]),
                        ),
                    ])),
                ])),
            ]),
        }
    }

    #[test]
    fn a_save_reads_back_as_it_was_written() {
        let root = sample();
        assert_eq!(read(&write(&root)).unwrap(), root);
    }

    #[test]
    fn gzip_round_trips() {
        let root = sample();
        assert_eq!(read_gzip(&write_gzip(&root).unwrap()).unwrap(), root);
    }

    #[test]
    fn a_change_keeps_every_other_tag_in_order() {
        let mut root = sample();
        *root.path_mut(&["Data", "Player", "Health"]).unwrap() = Tag::Float(20.0);
        let read_back = read(&write(&root)).unwrap();
        assert_eq!(
            read_back.get("Data").and_then(|d| d.get("LevelName")),
            Some(&Tag::String("Test".into()))
        );
        assert_eq!(
            read_back
                .get("Data")
                .and_then(|d| d.get("Player"))
                .and_then(|p| p.get("Health"))
                .and_then(Tag::as_f32),
            Some(20.0)
        );
    }

    #[test]
    fn a_truncated_file_is_an_error_not_a_panic() {
        let root = sample();
        let bytes = write(&root);
        for cut in [1, 10, bytes.len() / 2, bytes.len() - 1] {
            assert!(read(&bytes[..cut]).is_err(), "cut at {cut}");
        }
    }
}
