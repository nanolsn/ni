use std::{any::Any, collections::vec_deque::VecDeque};

use crate::common::UWord;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileError {
    ReadingNotAvailable,
    WritingNotAvailable,
}

pub trait File: std::fmt::Debug {
    fn read(&mut self) -> Result<Option<u8>, FileError>;

    fn write(&mut self, val: u8) -> Result<(), FileError>;

    fn flush(&mut self) -> Result<(), FileError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
}

impl File for Vec<u8> {
    fn read(&mut self) -> Result<Option<u8>, FileError> {
        Err(FileError::ReadingNotAvailable)
    }

    fn write(&mut self, val: u8) -> Result<(), FileError> {
        self.push(val);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl File for VecDeque<u8> {
    fn read(&mut self) -> Result<Option<u8>, FileError> {
        let val = self.pop_front();
        Ok(val)
    }

    fn write(&mut self, val: u8) -> Result<(), FileError> {
        self.push_back(val);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilesError {
    FileError(FileError),
    CurrentIsNotSet,
    LimitExceeded,
    NotFound,
}

impl From<FileError> for FilesError {
    fn from(e: FileError) -> Self {
        FilesError::FileError(e)
    }
}

#[derive(Debug)]
pub struct Files {
    files: Vec<Option<Box<dyn File>>>,
    count: usize,
    current: Option<(usize, Box<dyn File>)>,
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

        let mut idx = None;
        let current = self.current.as_ref().map(|(idx, _)| *idx);

        // Find a free cell
        for i in 0..self.files.len() {
            if self.files[i].is_none() {
                match current {
                    // The cell can be current
                    Some(c) if c == i => (),
                    _ => {
                        idx = Some(i);
                        break;
                    }
                }
            }
        }

        // If the free cell not was found, it needs to expand `files` vector
        let idx = idx.unwrap_or_else(|| {
            let len = self.files.len();
            self.files.push(None);
            len
        });

        self.files[idx] = Some(Box::new(file));
        self.count += 1;

        Ok(idx as UWord)
    }

    pub fn close(&mut self, idx: UWord) -> Result<Box<dyn File>, FilesError> {
        let file = match self.current {
            Some((current, _)) if current == idx as usize => {
                self.current.take().map(|(_, file)| file).unwrap()
            }
            _ => self
                .files
                .get_mut(idx as usize)
                .ok_or(FilesError::NotFound)?
                .take()
                .ok_or(FilesError::NotFound)?,
        };

        self.count -= 1;
        Ok(file)
    }

    pub fn set_current(&mut self, idx: UWord) -> Result<(), FilesError> {
        let idx = idx as usize;
        let file = self
            .files
            .get_mut(idx)
            .ok_or(FilesError::NotFound)?
            .take()
            .ok_or(FilesError::NotFound)?;

        self.current = Some((idx, file));
        Ok(())
    }

    pub fn current(&self) -> Result<UWord, FilesError> {
        let (current, _) = self.current.as_ref().ok_or(FilesError::CurrentIsNotSet)?;

        Ok(*current as UWord)
    }

    fn get_mut(&mut self) -> Result<&mut dyn File, FilesError> {
        let (_, file) = self.current.as_mut().ok_or(FilesError::CurrentIsNotSet)?;

        Ok(Box::as_mut(file) as &mut dyn File)
    }

    pub fn read(&mut self) -> Result<Option<u8>, FilesError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_open() {
        let mut files = Files::new();
        assert_eq!(files.open(Vec::new()), Ok(0));
        assert_eq!(files.open(Vec::new()), Ok(1));

        files.set_current(0).unwrap();
        assert_eq!(files.current(), Ok(0));
        assert_eq!(files.open(Vec::new()), Ok(2));

        let _ = files.close(0).unwrap();
        assert_eq!(files.current(), Err(FilesError::CurrentIsNotSet));
        assert_eq!(files.open(Vec::new()), Ok(0));
    }
}
