use libarchive3_sys_by_madosuki as libarchive3_sys;
use anyhow;

mod error;
use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

pub struct Archive {
    archive: *mut libarchive3_sys_by_madosuki::ArchiveStruct,
}

pub trait ArchiveExt {
    fn new() -> LibArchiveResult<Archive>;
    fn load_compressed_file(&self, file: &str) -> anyhow::Result<()>;
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

    fn load_compressed_file(&self, file_path: &str) -> anyhow::Result<()> {
        let _meta = std::fs::metadata(file_path)?;

        if !_meta.is_file() {
            return Err(LibArchiveError::IsNotFile.into());
        }

        let _file_size = _meta.len() as usize;
        
        let _file_path_cstr = std::ffi::CString::new(file_path)?;
        
        let mut _status_code = unsafe {
            libarchive3_sys::archive_read_open_filename(self.archive, _file_path_cstr.as_ptr(), _file_size)
        };

        if _status_code != 0 {
            let err_no = unsafe { libarchive3_sys::archive_errno(self.archive) };
            let status = LibArchiveInternalStatus::from(err_no);

            anyhow::bail!(LibArchiveError::LibArchiveInternalError(status));
        }

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
