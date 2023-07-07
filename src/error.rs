use std::ffi::c_int;
use libarchive3_sys_by_madosuki as libarchive3_sys;
use thiserror::Error;

#[derive(Debug)]
#[repr(i32)]
pub enum LibArchiveInternalStatus {
    ArchiveOk = libarchive3_sys::ARCHIVE_OK,
    ArchiveFailed = libarchive3_sys::ARCHIVE_FAILED,
    ArchiveEof = libarchive3_sys::ARCHIVE_EOF,
    ArchiveFatal = libarchive3_sys::ARCHIVE_FATAL,
    ArchiveRetry = libarchive3_sys::ARCHIVE_RETRY,
    ArchiveWarn = libarchive3_sys::ARCHIVE_WARN,
    Unknown,
}

impl std::fmt::Display for LibArchiveInternalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = generate_message(self);
        writeln!(f, "{}", msg)
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
    #[error("Failed free to archive")]
    FailedFreeArchive,
    #[error("Failed generate path")]
    FailedGeneratePath,
    #[error("Failed close to read archive")]
    FailedCloseReadArchive,
    #[error("Failed free to read archive")]
    FailedFreeReadArchive,
    #[error("Failed create archive entry")]
    FailedCreateArchiveEntry,
    #[error("Failed create archive")]
    FailedCreateArchive,
    #[error("Failed create directory")]
    FailedCreateDirectory,
    #[error("Failed create file")]
    FailedCreateFile,
    #[error("Failed write file")]
    FailedWriteFile,
    #[error("Failed flush when write")]
    FailedFlushWhenWrite,
    #[error("Failed get pathname from entry")]
    FailedGetPathNameFromEntry,
    #[error("Entry size less than one")]
    EntrySizeLessThanOne,
    #[error("NulError from ffi")]
    NulError,
    #[error("Failed get metadata from file")]
    FailedGetMetaDataFromFile,
    #[error("Failed get metadata from dir")]
    FailedGetMetaDataFromDir,
    #[error("Failed write header")]
    FailedWriteHeader,
    #[error("is not file")]
    IsNotFile,
    #[error("is not dir")]
    IsNotDir,
    #[error("is not exists")]
    IsNotExists,
    #[error("Failed uncompress")]
    FailedUncompress,
    #[error("libarchive internal error: {0}")]
    LibArchiveInternalError(LibArchiveInternalStatus),
}


pub type LibArchiveResult<T> = Result<T, LibArchiveError>;
