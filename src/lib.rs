use libarchive3_sys_by_madosuki as libarchive3_sys;
use libarchive3_sys::{ArchiveStruct, ArchiveEntryStruct};

use std::ffi::c_char;
mod error;
use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

fn convert_c_char_to_string(data: *const c_char) -> Option<String> {
    if data.is_null() {
        return None;
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(data) };
    match c_str.to_str() {
        Ok(v) => Some(v.to_string()),
        _ => None
    }
}

fn load_data_from_entry(archive: *mut ArchiveStruct, entry_size: usize) -> LibArchiveResult<Vec<u8>> {
    let mut offset = 0 as i64;
    let mut result: Vec<u8> = vec!();

    loop {
        let mut buf: *mut u8 = Vec::with_capacity(entry_size).as_mut_ptr();
        let mut _readed_size = 0 as usize;
        let _r = unsafe { libarchive3_sys::archive_read_data_block(archive, &mut buf, &mut _readed_size, &mut offset) };

        if _r == 1 {
            break;
        }

        let for_safe: &[u8] = unsafe { std::slice::from_raw_parts(buf, _readed_size) };
        result.append(&mut for_safe.to_vec());

        if _r == 0 {
            continue;
        }
    }

    Ok(result)
}

#[derive(Debug)]
pub struct DecompressedData {
    file_name: String,
    size: usize,
    data: Vec<u8>,
}

pub struct Archive {
    archive: *mut libarchive3_sys_by_madosuki::ArchiveStruct,
}

pub trait ArchiveExt {
    fn new() -> LibArchiveResult<Archive>;
    fn init(&self) -> LibArchiveResult<()>;
    fn extract_compressed_file_to_memory(&self, file: &str) -> LibArchiveResult<Vec<DecompressedData>>;
    fn get_errno(&self) -> Option<i32>;
}

impl ArchiveExt for Archive {
    fn new() -> LibArchiveResult<Archive> {
        let archive = unsafe { libarchive3_sys_by_madosuki::archive_read_new() };
        if archive.is_null() {
            Err(LibArchiveError::FailedCreateArchive)
        } else {
            Ok(Archive { archive })
        }
    }

    fn init(&self) -> LibArchiveResult<()> {
        let mut _r = unsafe { libarchive3_sys::archive_read_support_filter_all(self.archive) };
        if _r != 0 {
            return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_r)));
        }

        _r = unsafe { libarchive3_sys::archive_read_support_format_all(self.archive) };
        if _r != 0 {
            return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_r)));
        }

        Ok(())
    }

    fn get_errno(&self) -> Option<i32> {
        if self.archive.is_null() {
            return None;
        }
        
        unsafe {
            Some(libarchive3_sys::archive_errno(self.archive))
        }
    }

    fn extract_compressed_file_to_memory(&self, file_path: &str) -> LibArchiveResult<Vec<DecompressedData>> {
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

        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            return Err(LibArchiveError::FailedCreateArchiveEntry);
        }

        let mut _result: Vec<DecompressedData> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(self.archive, &mut entry) != 1 {
                let _pathname = libarchive3_sys::archive_entry_pathname(entry);
                if _pathname.is_null() {
                    return Err(LibArchiveError::FailedGetPathNameFromEntry);
                }

                let mut _f_nmae = std::string::String::new();
                match convert_c_char_to_string(_pathname) {
                    Some(_n) => {
                        _f_nmae = _n;
                    },
                    _ => {
                        return Err(LibArchiveError::FailedGetPathNameFromEntry);
                    },
                }
                
                let mut _entry_size = libarchive3_sys::archive_entry_size(entry);
                if _entry_size < 1 {
                    return Err(LibArchiveError::EntrySizeLessThanOne);
                }

                let Ok(readed_data) = load_data_from_entry(self.archive, _entry_size as usize) else {
                    return Err(LibArchiveError::FailedUncompress);
                };

                let tmp = DecompressedData {
                    file_name: _f_nmae,
                    size: _entry_size as usize,
                    data: readed_data,
                };
                _result.push(tmp);
            }
        }

        Ok(_result)
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
