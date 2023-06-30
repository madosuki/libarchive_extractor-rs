use libarchive3_sys_by_madosuki as libarchive3_sys;
use libarchive3_sys::{ArchiveStruct, ArchiveEntryStruct};

use std::io::prelude::Write;
use libc::{ c_char, c_void, size_t};
mod error;
use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

fn entry_free(entry: *mut ArchiveEntryStruct) {
    unsafe { libarchive3_sys::archive_entry_free(entry); }
}

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

fn get_pathname_from_entry(entry: *mut ArchiveEntryStruct) -> LibArchiveResult<String> {
    let _pathname = unsafe { libarchive3_sys::archive_entry_pathname(entry) };
    if _pathname.is_null() {
        return Err(LibArchiveError::FailedGetPathNameFromEntry);
    }

    match convert_c_char_to_string(_pathname) {
        Some(_name) => {
            Ok(_name)
        },
        _ => {
            Err(LibArchiveError::FailedGetPathNameFromEntry)
        },
    }
}

fn load_and_write_datum_from_entry(archive: *mut ArchiveStruct, archive_write: *mut ArchiveStruct) {
    let mut offset = 0 as i64;

    loop {
        let buf: *mut c_void = std::ptr::null_mut();
        
        let mut _readed_size = 0 as size_t;
        let _r = unsafe { libarchive3_sys::archive_read_data_block(archive, &buf, &mut _readed_size, &mut offset) };
        println!("_f_size: {}", _readed_size);

        if _r == 1 {
            break;
        }

        let buf_const = buf.cast_const();
        let _result = unsafe { libarchive3_sys::archive_write_data_blocK(archive_write, buf_const, _readed_size, offset) };
        if _result != (libarchive3_sys::ARCHIVE_OK as isize) {
            break;
        }
    }
}

fn load_datum_from_entry(archive: *mut ArchiveStruct, entry_size: usize) -> LibArchiveResult<Vec<u8>> {
    let mut offset = 0 as i64;
    let mut result: Vec<u8> = vec!();

    loop {
        let tmp: *mut c_void = std::ptr::null_mut();
        let mut _readed_size = 0 as usize;
        let _r = unsafe { libarchive3_sys::archive_read_data_block(archive, &tmp, &mut _readed_size, &mut offset) };

        if _r == 1 {
            break;
        }

        let for_safe: &[u8] = unsafe { std::slice::from_raw_parts(tmp as *mut u8, _readed_size) };
        result.append(&mut for_safe.to_vec());

        if _r == 0 {
            continue;
        }
    }

    Ok(result)
}

fn write_file_to_dir(dir_path: &std::path::Path, file_name: &str, data: &[u8]) -> LibArchiveResult<()> {
    let _f_path_buf = dir_path.join(file_name);
    let Ok(mut fp) = std::fs::File::create(&_f_path_buf) else {
        return Err(LibArchiveError::FailedCreateFile);
    };
    
    match fp.write_all(&data) {
        Ok(_) => {
            match fp.flush() {
                Ok(_) => Ok(()),
                _ => {
                    return Err(LibArchiveError::FailedFlushWhenWrite);
                }
            }
        },
        Err(_) => {
            return Err(LibArchiveError::FailedWriteFile);
        }
    }

}


#[derive(Debug)]
pub struct FileInfo {
    file_name: String,
    size: usize,
}

#[derive(Debug)]
pub struct DecompressedData {
    file_info: FileInfo,
    data: Vec<u8>,
}

pub struct Archive {
    archive: *mut libarchive3_sys_by_madosuki::ArchiveStruct,
}

pub trait ArchiveExt {
    fn new() -> LibArchiveResult<Archive>;
    fn init(&self) -> LibArchiveResult<()>;
    fn extract_compressed_file_to_memory(&self, file_path: &str) -> LibArchiveResult<Vec<DecompressedData>>;
    fn get_errno(&self) -> Option<i32>;
    fn read_and_write_to_specific_dir(&self, file_path: &str, target_dir_path: &str) -> LibArchiveResult<Vec<FileInfo>>;
    fn read_close_and_free(&self) -> LibArchiveResult<()>;
    fn free(&mut self) -> LibArchiveResult<()>;
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

    fn free(&mut self) -> LibArchiveResult<()> {
        let status_code = unsafe { libarchive3_sys::archive_free(self.archive) };
        if status_code != 0 {
            return Err(LibArchiveError::FailedFreeArchive);
        }
        self.archive = std::ptr::null_mut();
        Ok(())
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

    fn read_close_and_free(&self) -> LibArchiveResult<()> {
        if self.archive.is_null() {
            return Ok(());
        }
        
        let close_status_code = unsafe { libarchive3_sys::archive_read_close(self.archive) };
        if close_status_code != 0 {
            return Err(LibArchiveError::FailedCloseReadArchive);
        }

        let free_status_code = unsafe { libarchive3_sys::archive_read_free(self.archive) };
        if free_status_code != 0 {
            return Err(LibArchiveError::FailedFreeReadArchive);
        }

        Ok(())
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
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_status_code)));
            }
        };

        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            match self.read_close_and_free() {
                Ok(_) => {
                    return Err(LibArchiveError::FailedCreateArchiveEntry);
                },
                Err(e) => {
                    return Err(e);
                }
            }

        }

        let mut _result: Vec<DecompressedData> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(self.archive, &mut entry) != 1 {
                let _pathname = libarchive3_sys::archive_entry_pathname(entry);
                if _pathname.is_null() {
                    let _ = self.read_close_and_free();
                    return Err(LibArchiveError::FailedGetPathNameFromEntry);
                }

                let mut _f_name = std::string::String::new();
                match convert_c_char_to_string(_pathname) {
                    Some(_n) => {
                        _f_name = _n;
                    },
                    _ => {
                        let _ = self.read_close_and_free();
                        return Err(LibArchiveError::FailedGetPathNameFromEntry);
                    },
                }
                
                let mut _entry_size = libarchive3_sys::archive_entry_size(entry);
                if _entry_size < 1 {
                    let _ = self.read_close_and_free();
                    return Err(LibArchiveError::EntrySizeLessThanOne);
                }

                let Ok(readed_data) = load_datum_from_entry(self.archive, _entry_size as usize) else {
                    let _ = self.read_close_and_free();
                    return Err(LibArchiveError::FailedUncompress);
                };

                let file_info = FileInfo {
                    file_name: _f_name,
                    size: _entry_size as usize,
                };

                let tmp = DecompressedData {
                    file_info,
                    data: readed_data,
                };
                _result.push(tmp);
            }
        }

        self.read_close_and_free()?;
        Ok(_result)
    }

    fn read_and_write_to_specific_dir(&self, file_path: &str, target_dir_path: &str) -> LibArchiveResult<Vec<FileInfo>> {

        let _f_p = std::path::Path::new(file_path);
        if !_f_p.exists() {
            return Err(LibArchiveError::IsNotExists);
        }

        let _dir_path = std::path::Path::new(target_dir_path);
        if !_dir_path.exists() {
            let _r = std::fs::create_dir(_dir_path);
            if _r.is_err() {
               return Err(LibArchiveError::FailedCreateDirectory); 
            }
        }
        
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
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_status_code)));
            }
        };

        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            let _ = self.read_close_and_free();
            return Err(LibArchiveError::FailedCreateArchiveEntry);
        }

        let mut _result: Vec<FileInfo> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(self.archive, &mut entry) != 1 {
                let mut _f_name = std::string::String::new();
                match get_pathname_from_entry(entry) {
                    Ok(_name) => {
                        _f_name = _name;
                    },
                    Err(e) => {
                        let _ = self.read_close_and_free()?;
                        return Err(e);
                    }
                }
                
                let mut _entry_size = libarchive3_sys::archive_entry_size(entry);
                if _entry_size < 1 {
                    let _ = self.read_close_and_free()?;
                    return Err(LibArchiveError::EntrySizeLessThanOne);
                }

                let Ok(readed_data) = load_datum_from_entry(self.archive, _entry_size as usize) else {
                    let _ = self.read_close_and_free()?;
                    return Err(LibArchiveError::FailedUncompress);
                };

                let write_result = write_file_to_dir(_dir_path, &_f_name, &readed_data);
                match write_result {
                    Ok(_) => {
                        let tmp = FileInfo {
                            file_name: _f_name,
                            size: _entry_size as usize,
                        };

                        _result.push(tmp);
                    },
                    Err(e) => {
                        let _ = self.read_close_and_free()?;
                        return Err(e);
                    }
                }
            }
        }

        self.read_close_and_free()?;
        Ok(_result)
    }
}


