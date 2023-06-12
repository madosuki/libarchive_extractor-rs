use std::io::prelude::Write;

use libarchive3_sys_by_madosuki as libarchive3_sys;
use thiserror::Error;

#[derive(Debug)]
pub enum LibArchiveInternalStatus {
    ArchiveOk = libarchive3_sys::ARCHIVE_OK as isize,
    ArchiveFailed = libarchive3_sys::ARCHIVE_FAILED as isize,
    ArchiveEof = libarchive3_sys::ARCHIVE_EOF as isize,
    ArchiveFatal = libarchive3_sys::ARCHIVE_FATAL as isize,
    ArchiveRetry = libarchive3_sys::ARCHIVE_RETRY as isize,
    ArchiveWarn = libarchive3_sys::ARCHIVE_WARN as isize
}

impl std::fmt::Display for LibArchiveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = generate_message(self);
        writeln!(f, "Status: {}", msg)
    }
}

fn generate_message(status: &LibArchiveStatus) -> String {
    match status {
        LibArchiveInternalStatus::ArchiveOk => "Ok".to_owned(),
        LibArchiveInternalStatus::ArchiveFailed => "Failed".to_owned(),
        LibArchiveInternalStatus::ArchiveFatal => "Fatal".to_owned(),
        LibArchiveInternalStatus::ArchiveEof => "Eof".to_owned(),
        LibArchiveInternalStatus::ArchiveWarn => "Warn".to_owned(),
        LibArchiveInternalStatus::ArchiveRetry => "Retry".to_owned()
    }
}

#[derive(Error, Debug)]
pub enum LibArchiveError {
    #[error("Null")]
    Null,
    #[error("is not file")]
    IsNotFile,
    #[error("is not exists")]
    IsNotExists,
    #[error("libarchive internal error: {0}")]
    LibArchiveInternalError(LibArchiveInternalStatus),
}


pub type LibArchiveResult<T> = Result<T, LibArchiveError>;
