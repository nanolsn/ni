use std::{rc::Rc, any::Any};

use common::UWord;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileError {
    ReadingNotAvailable,
    WritingNotAvailable,
}

#[derive(Copy, Clone, Debug)]
pub enum FileMode {
    Read,
    Write,
}

pub trait File: std::fmt::Debug {
    fn read(&mut self) -> Result<u8, FileError>;

    fn write(&mut self, val: u8) -> Result<(), FileError>;

    fn flush(&mut self) -> Result<(), FileError>;

    fn get_mode(&self) -> FileMode;

    fn as_any(&self) -> &dyn Any;
}

impl File for Vec<u8> {
    fn read(&mut self) -> Result<u8, FileError> {
        // TODO: impl
        Err(FileError::ReadingNotAvailable)
    }

    fn write(&mut self, val: u8) -> Result<(), FileError> {
        self.push(val);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), FileError> { Ok(()) }

    fn get_mode(&self) -> FileMode { FileMode::Write }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilesError {
    FileError(FileError),
    CurrentIsNotSet,
    LimitExceeded,
    NotFound,
}

impl From<FileError> for FilesError {
    fn from(e: FileError) -> Self { FilesError::FileError(e) }
}

#[derive(Debug)]
pub struct Files {
    files: Vec<Option<Rc<dyn File>>>,
    count: usize,
    current: Option<UWord>,
}

impl Files {
    pub const LIMIT: usize = 64;

    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            count: 0,
            current: None,
        }
    }

    pub fn open<F>(&mut self, file: F) -> Result<UWord, (FilesError, F)>
        where
            F: File + 'static,
    {
        if self.count == Self::LIMIT {
            return Err((FilesError::LimitExceeded, file));
        }

        let idx = self.files
            .iter()
            .position(|f| f.is_none());

        let idx = if let Some(idx) = idx {
            idx
        } else {
            let len = self.files.len();
            self.files.push(None);
            len
        };

        self.files[idx] = Some(Rc::new(file));
        self.count += 1;

        Ok(idx as UWord)
    }

    pub fn close(&mut self, idx: UWord) -> Result<Rc<dyn File>, FilesError> {
        let file = self.files
            .get_mut(idx as usize)
            .ok_or(FilesError::NotFound)?
            .take()
            .ok_or(FilesError::NotFound)?;

        if let Some(current) = self.current {
            if current == idx {
                self.current = None;
            }
        }

        self.count -= 1;
        Ok(file)
    }

    pub fn set_current(&mut self, idx: UWord) -> Result<(), FilesError> {
        let _ = self.files
            .get(idx as usize)
            .ok_or(FilesError::NotFound)?
            .as_ref()
            .ok_or(FilesError::NotFound)?;

        self.current = Some(idx);
        Ok(())
    }

    pub fn current(&self) -> Result<UWord, FilesError> {
        let current = self.current.ok_or(FilesError::CurrentIsNotSet)?;
        Ok(current)
    }

    fn get_mut(&mut self) -> Result<&mut dyn File, FilesError> {
        let idx = self.current
            .ok_or(FilesError::CurrentIsNotSet)?;

        let file = self.files[idx as usize].as_mut().unwrap();
        Ok(Rc::get_mut(file).unwrap() as &mut dyn File)
    }

    pub fn read(&mut self) -> Result<u8, FilesError> {
        let file = self.get_mut()?;
        let val = file.read()?;
        Ok(val)
    }

    pub fn write(&mut self, val: u8) -> Result<(), FilesError> {
        let file = self.get_mut()?;
        file.write(val)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), FilesError> {
        let file = self.get_mut()?;
        file.flush()?;
        Ok(())
    }
}
