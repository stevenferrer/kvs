use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{Error, Result};

// TODO: Add notes about merging/compaction process
const MERGE_THRESHOLD: u64 = 1024 * 1024;

/// The `DB` stores string key/value pairs.
///
/// Key/value pairs are persisted on disk.
pub struct DB {
    path: PathBuf,
    writer: BufWriterWithPos<File>,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    key_dir: BTreeMap<String, CommandPos>,
    current_seg: u64,
    unmerged: u64,
}

impl DB {
    /// Creates a `KvStore`.
    pub fn open(path_buf: impl Into<PathBuf>) -> Result<Self> {
        let path = path_buf.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        // list old segments (1.log, 2.log, ...)
        let segments = sorted_segment_list(&path)?;
        let mut unmerged = 0;

        // load (in-memory) index from old segments
        for &segment in &segments {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, segment))?)?;
            unmerged += load_segment(segment, &mut reader, &mut index)?;

            // store old logs as reader
            readers.insert(segment, reader);
        }

        // create new log file for writing
        let current_seg = segments.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_seg, &mut readers)?;

        Ok(DB {
            path,
            writer,
            readers,
            key_dir: index,
            current_seg,
            unmerged,
        })
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key.clone(), value);

        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        // SELF-NOTE: why we need to flush?
        self.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self
                .key_dir
                .insert(key, (self.current_seg, pos..self.writer.pos).into())
            {
                // save length of new command
                self.unmerged += old_cmd.len;
            }
        }

        if self.unmerged > MERGE_THRESHOLD {
            self.merge_segments()?;
        }

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.key_dir.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_pos.seg)
                .expect("cannot find log reader");

            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);

            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(Error::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.key_dir.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;

            if let Command::Remove { key } = cmd {
                let old_cmd = self.key_dir.remove(&key).expect("key not found");
                // use for compaction
                self.unmerged += old_cmd.len;
            }

            Ok(())
        } else {
            Err(Error::KeyNotFound)
        }
    }

    /// merge immutable segments
    pub fn merge_segments(&mut self) -> Result<()> {
        let merge_seg = self.current_seg + 1;
        self.current_seg += 2;

        self.writer = new_log_file(&self.path, self.current_seg, &mut self.readers)?;

        let mut merge_writer = new_log_file(&self.path, self.current_seg, &mut self.readers)?;

        let mut new_pos = 0;
        for cmd_pos in &mut self.key_dir.values_mut() {
            let reader = self
                .readers
                .get_mut(&cmd_pos.seg)
                .expect("cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut merge_writer)?;
            *cmd_pos = (merge_seg, new_pos..new_pos + len).into();
            new_pos += len;
        }
        merge_writer.flush()?;

        // remove stale log files.
        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < merge_seg)
            .cloned()
            .collect();
        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }
        self.unmerged = 0;

        Ok(())
    }
}

/// Struct representing a command.
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// Represents the position and length of a json-serialized command in the log.
struct CommandPos {
    seg: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((seg, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            seg,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

fn new_log_file(
    path: &Path,
    gen: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    readers.insert(gen, BufReaderWithPos::new(File::open(&path)?)?);
    Ok(writer)
}

fn log_path(dir: &Path, segment: u64) -> PathBuf {
    dir.join(format!("{}.log", segment))
}

fn sorted_segment_list(path: &Path) -> Result<Vec<u64>> {
    // list and sort old segments
    let mut segment_list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                // why we need OsStr??
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    segment_list.sort_unstable();

    Ok(segment_list)
}

fn load_segment(
    segment: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    // SELF-NOTE: How does Deserializer work?
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();

    let mut unmerged = 0;
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, (segment, pos..new_pos).into()) {
                    // count uncompacted log
                    unmerged += old_cmd.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    // count uncompacted log
                    unmerged += old_cmd.len;
                }
                // the "remove" command can be deleted in the next compaction
                unmerged += new_pos - pos;
            }
        }

        pos = new_pos;
    }

    Ok(unmerged)
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
