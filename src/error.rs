use std::ffi::c_int;
use libarchive3_sys_by_madosuki as libarchive3_sys;
use thiserror::Error;

#[derive(Debug)]
pub enum LibArchiveInternalStatus {
    ArchiveOk = libarchive3_sys::ARCHIVE_OK as isize,
    ArchiveFailed = libarchive3_sys::ARCHIVE_FAILED as isize,
    ArchiveEof = libarchive3_sys::ARCHIVE_EOF as isize,
    ArchiveFatal = libarchive3_sys::ARCHIVE_FATAL as isize,
    ArchiveRetry = libarchive3_sys::ARCHIVE_RETRY as isize,
    ArchiveWarn = libarchive3_sys::ARCHIVE_WARN as isize,
    Unknown,
}

impl std::fmt::Display for LibArchiveInternalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = generate_message(self);
        writeln!(f, "Status: {}", msg)
    }
}

impl std::convert::From<c_int> for LibArchiveInternalStatus {
    fn from(v: c_int) -> Self {
        match v {
            libarchive3_sys::ARCHIVE_OK => Self::ArchiveOk,
            libarchive3_sys::ARCHIVE_EOF => Self::ArchiveEof,
            libarchive3_sys::ARCHIVE_WARN => Self::ArchiveWarn,
            libarchive3_sys::ARCHIVE_FATAL => Self::ArchiveFatal,
            libarchive3_sys::ARCHIVE_RETRY => Self::ArchiveRetry,
            libarchive3_sys::ARCHIVE_FAILED => Self::ArchiveFailed,
            _ => Self::Unknown,
        }
    }
}

fn generate_message(status: &LibArchiveInternalStatus) -> String {
    match status {
        LibArchiveInternalStatus::ArchiveOk => "Ok".to_owned(),
        LibArchiveInternalStatus::ArchiveFailed => "Failed".to_owned(),
        LibArchiveInternalStatus::ArchiveFatal => "Fatal".to_owned(),
        LibArchiveInternalStatus::ArchiveEof => "Eof".to_owned(),
        LibArchiveInternalStatus::ArchiveWarn => "Warn".to_owned(),
        LibArchiveInternalStatus::ArchiveRetry => "Retry".to_owned(),
        _ => "Unknown".to_owned(),
    }
}

#[derive(Error, Debug)]
pub enum LibArchiveError {
    #[error("Null")]
    Null,
    #[error("NulError from ffi")]
    NulError,
    #[error("Failed get metadata from file")]
    FailedGetMetaDataFromFile,
    #[error("is not file")]
    IsNotFile,
    #[error("is not exists")]
    IsNotExists,
    #[error("libarchive internal error: {0}")]
    LibArchiveInternalError(LibArchiveInternalStatus),
}


pub type LibArchiveResult<T> = Result<T, LibArchiveError>;
