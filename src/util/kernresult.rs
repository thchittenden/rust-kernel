use core::prelude::*;
use core::fmt;

#[must_use]
pub type KernResultEx<T, E> = Result<T, KernErrorEx<E>>;

#[must_use]
pub type KernResult<T> = Result<T, KernError>;

#[macro_export]
macro_rules! err_ex {
    ($exp:expr, $ex:expr) => ({
        match $exp {
            Ok(v) => Ok(v),
            Err(e) => Err(KernErrorEx { err: e, ex: $ex }),
        }
    })
}

#[derive(Debug)]
pub enum KernError {
    OutOfMemory,
    NoSuchFile,
    NoSuchObject,
    NoSuchDirectory,
    FileExists,
    ObjectExists,
    DirectoryExists,
    DirectoryUnlinked,
    DirectoryNotEmpty,
    WrongType,
    FormatError,
    Unsupported,
}

pub struct KernErrorEx<E> {
    pub err: KernError,
    pub ex: E
}

impl From<fmt::Error> for KernError {
    fn from(_: fmt::Error) -> KernError {
        KernError::FormatError
    }
}

impl From<KernError> for fmt::Error {
    fn from(_: KernError) -> fmt::Error {
        fmt::Error
    }
}

impl<E> From<KernErrorEx<E>> for KernError {
    fn from(e: KernErrorEx<E>) -> KernError {
        e.err
    }
}

impl<E> From<KernErrorEx<E>> for fmt::Error {
    fn from(_: KernErrorEx<E>) -> fmt::Error {
        fmt::Error
    }
}

impl<E: fmt::Debug> fmt::Debug for KernErrorEx<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernErrorEx {{ err: {:?}, ex: {:?} }}", self.err, self.ex)
    }
}
