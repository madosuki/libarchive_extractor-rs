use libarchive3_sys_by_madosuki as libarchive3_sys;

mod error;
use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

pub struct Archive {
    archive: *mut libarchive3_sys_by_madosuki::ArchiveStruct,
}

pub trait ArchiveExt {
    fn new() -> LibArchiveResult<Archive>;
    fn load_compressed_file(&self, file: &str) -> LibArchiveResult<()>;
}

impl ArchiveExt for Archive {
    fn new() -> LibArchiveResult<Archive> {
        let archive = unsafe { libarchive3_sys_by_madosuki::archive_read_new() };
        if archive.is_null() {
            Err(LibArchiveError::Null)
        } else {
            Ok(Archive { archive })
        }
    }

    fn load_compressed_file(&self, file_path: &str) -> LibArchiveResult<()> {
        let Ok(_meta) = std::fs::metadata(file_path) else {
            return Err(LibArchiveError::FailedGetMetaDataFromFile);
        };
        
        if !_meta.is_file() {
            return Err(LibArchiveError::IsNotFile);
        }

        let Ok(_file_path_cstr) = std::ffi::CString::new(file_path) else {
            return Err(LibArchiveError::NulError);
        };

        let _f_size = _meta.len() as usize;
        
        unsafe {
            let _status_code = libarchive3_sys::archive_read_open_filename(self.archive, _file_path_cstr.as_ptr(), _f_size);
            if _status_code != 0 {
                let errno = libarchive3_sys::archive_errno(self.archive);
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(errno)));
            }
        };

        // let mut entry_count = 0;
        // let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };

        Ok(())
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
