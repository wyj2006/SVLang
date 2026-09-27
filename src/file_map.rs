use crate::files;
use codespan::Span;
use codespan_reporting::files::{Error, Files, SimpleFile};
use indexmap::IndexMap;
use std::ops::Range;

pub type FileId = usize;

pub struct FileMap {
    pub files: Vec<SimpleFile<String, String>>,
    pub mappings: IndexMap<(usize, Span), (usize, Span)>,
}

impl FileMap {
    pub fn new() -> FileMap {
        FileMap {
            files: Vec::new(),
            mappings: IndexMap::new(),
        }
    }

    pub fn add(&mut self, name: String, source: String) -> usize {
        let file_id = self.files.len();
        let source = source.replace("\r\n", "\n");
        self.files.push(SimpleFile::new(name, source));
        file_id
    }

    pub fn get(&self, file_id: usize) -> Result<&SimpleFile<String, String>, Error> {
        self.files.get(file_id).ok_or(Error::FileMissing)
    }
}

impl Files<'_> for FileMap {
    type FileId = FileId;
    type Name = String;
    type Source = String;

    fn name(&self, file_id: usize) -> Result<String, Error> {
        Ok(self.get(file_id)?.name().clone())
    }

    fn source(&self, file_id: usize) -> Result<String, Error> {
        Ok(self.get(file_id)?.source().clone())
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Result<usize, Error> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Result<Range<usize>, Error> {
        self.get(file_id)?.line_range((), line_index)
    }
}

pub fn source_lookup(mut file_id: usize, mut span: Span) -> (usize, Span) {
    let file_map = &files.lock().unwrap();

    loop {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        let mut new_start = None;
        let mut new_end = None;

        for ((key_file_id, key_span), (val_file_id, val_span)) in &file_map.mappings {
            if *key_file_id != file_id {
                continue;
            }
            if !(key_span.start().to_usize() <= start && start < key_span.end().to_usize()) {
                continue;
            }
            if let None = new_start {
                new_start = Some(val_span.end().to_usize() - (key_span.end().to_usize() - start));
            }
            if !(key_span.start().to_usize() < end && end <= key_span.end().to_usize()) {
                continue;
            }
            if let None = new_end {
                new_end = Some(val_span.start().to_usize() + end - key_span.start().to_usize());
                file_id = *val_file_id;
                break;
            }
        }

        let (new_file_id, new_span) = if let Some(start) = new_start
            && let Some(end) = new_end
        {
            (file_id, Span::new(start as u32, end as u32))
        } else {
            (file_id, span)
        };

        if (file_id, span) == (new_file_id, new_span) {
            break;
        }
        (file_id, span) = (new_file_id, new_span);
    }

    (file_id, span)
}
