use libarchive3_sys_by_madosuki as libarchive3_sys;
pub use libarchive3_sys::{ArchiveStruct, ArchiveEntryStruct};

use libc::{ c_int, c_char, c_void, size_t};
pub mod error;
pub use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

// fn entry_free(entry: *mut ArchiveEntryStruct) {
//     unsafe { libarchive3_sys::archive_entry_free(entry); }
// }

fn set_all_filter_and_format(_archive: *mut ArchiveStruct) -> LibArchiveResult<()> {
    let _read_support_filter_all_result = unsafe { libarchive3_sys::archive_read_support_filter_all(_archive) };
    if _read_support_filter_all_result != 0 {
        return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_read_support_filter_all_result)));
    }

    let _read_support_format_all_result = unsafe { libarchive3_sys::archive_read_support_format_all(_archive) };
    if _read_support_format_all_result != 0 {
        return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_read_support_format_all_result)));
    }
    
    Ok(())
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

fn read_and_write_data(archive: *mut ArchiveStruct, archive_write: *mut ArchiveStruct) -> LibArchiveResult<()> {
    let mut offset = 0 as i64;

    loop {
        let buf: *mut c_void = std::ptr::null_mut();
        
        let mut _readed_size = 0 as size_t;
        let _r = unsafe { libarchive3_sys::archive_read_data_block(archive, &buf, &mut _readed_size, &mut offset) };
        if _r == 1 {
            break;
        }

        let _write_dta_block_result = unsafe { libarchive3_sys::archive_write_data_block(archive_write, buf as *const c_void, _readed_size, offset) };
        if _write_dta_block_result == -1 {
            return Err(LibArchiveError::FailedWriteFile);
        }
    }

    Ok(())
}

fn read_data(archive: *mut ArchiveStruct) -> LibArchiveResult<Vec<u8>> {
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

fn read_free(mut _archive: *mut ArchiveStruct) -> LibArchiveResult<()> {
    let status_code = unsafe { libarchive3_sys::archive_free(_archive) };
    if status_code != 0 {
        return Err(LibArchiveError::FailedFreeArchive);
    }
    _archive = std::ptr::null_mut();
    Ok(())
}

fn read_close_and_free(mut _read_archive: *mut ArchiveStruct) -> LibArchiveResult<()> {
    if _read_archive.is_null() {
        return Ok(());
    }
        
    let close_status_code = unsafe { libarchive3_sys::archive_read_close(_read_archive) };
    if close_status_code != 0 {
        return Err(LibArchiveError::FailedCloseReadArchive);
    }

    read_free(_read_archive)?;

    Ok(())
}



#[derive(Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub size: usize,
    pub is_success: bool,
    pub error: Option<LibArchiveError>,
}

#[derive(Debug)]
pub struct DecompressedData {
    pub value: Vec<u8>,
    pub file_info: FileInfo,
}

pub struct Archive;

pub trait ArchiveExt {
    fn new() -> LibArchiveResult<Archive>;
    fn extract_to_memory(&self, file_path: &str) -> LibArchiveResult<Vec<DecompressedData>>;
    fn get_errno(&self, archive: *mut ArchiveStruct) -> Option<i32>;
    fn get_error_string(archive: *mut ArchiveStruct) -> Option<String>;
    fn extract_to_dir(&self, file_path: &str, target_dir_path: &str, flags: Option<i32>) -> LibArchiveResult<Vec<FileInfo>>;
}

impl ArchiveExt for Archive {
    fn new() -> LibArchiveResult<Archive> {
        let archive = unsafe { libarchive3_sys_by_madosuki::archive_read_new() };
        if archive.is_null() {
            Err(LibArchiveError::FailedCreateArchive)
        } else {
            Ok(Archive)
        }
    }


    fn get_errno(&self, archive: *mut ArchiveStruct) -> Option<i32> {
        if archive.is_null() {
            return None;
        }
        
        unsafe {
            Some(libarchive3_sys::archive_errno(archive))
        }
    }

    fn get_error_string(archive: *mut ArchiveStruct) -> Option<String> {
        if archive.is_null () {
            return None;
        }
        
        let bytes = unsafe { libarchive3_sys::archive_error_string(archive) };
        convert_c_char_to_string(bytes)
    }


    fn extract_to_memory(&self, file_path: &str) -> LibArchiveResult<Vec<DecompressedData>> {
        let Ok(_meta) = std::fs::metadata(file_path) else {
            return Err(LibArchiveError::FailedGetMetaDataFromFile);
        };
        
        if !_meta.is_file() {
            return Err(LibArchiveError::IsNotFile);
        }

        let Ok(_file_path_cstr) = std::ffi::CString::new(file_path) else {
            return Err(LibArchiveError::NulError);
        };


        let mut _archive: *mut ArchiveStruct = unsafe { libarchive3_sys::archive_read_new() };
        if _archive.is_null() {
            return Err(LibArchiveError::FailedCreateArchive);
        }

        set_all_filter_and_format(_archive)?;

        let _f_size = _meta.len() as usize;
        unsafe {
            let _status_code = libarchive3_sys::archive_read_open_filename(_archive, _file_path_cstr.as_ptr(), _f_size);
            if _status_code != 0 {
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_status_code)));
            }
        };
        
        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            match read_close_and_free(_archive) {
                Ok(_) => {
                    return Err(LibArchiveError::FailedCreateArchiveEntry);
                },
                Err(_) => {
                    return Err(LibArchiveError::FailedCreateArchiveEntryAndFailedCloseRead);
                }
            }
        }

        let mut _result: Vec<DecompressedData> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(_archive, &mut entry) != 1 {
                let _pathname = libarchive3_sys::archive_entry_pathname(entry);
                if _pathname.is_null() {
                    let file_info = FileInfo {
                        file_name: "".to_owned(),
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGetPathNameFromEntry),
                    };

                    let _decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    _result.push(_decompress_data);
                    
                    continue;
                }

                let mut _f_name = std::string::String::new();
                match convert_c_char_to_string(_pathname) {
                    Some(_n) => {
                        _f_name = _n;
                    },
                    _ => {
                        let file_info = FileInfo {
                            file_name: "".to_owned(),
                            size: 0,
                            is_success: false,
                            error: Some(LibArchiveError::FailedGetPathNameFromEntry),
                        };

                        let _decompress_data = DecompressedData {
                            file_info,
                            value: vec!(),
                        };
                        _result.push(_decompress_data);

                        continue;
                    },
                }
                
                let mut _entry_size = libarchive3_sys::archive_entry_size(entry);
                if _entry_size < 1 {
                    let file_info = FileInfo {
                        file_name: _f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::EntrySizeLessThanOne),
                    };

                    let _decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    _result.push(_decompress_data);
                    
                    continue;
                }

                let Ok(readed_data) = read_data(_archive) else {
                    let file_info = FileInfo {
                        file_name: _f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedUncompress),
                    };

                    let _decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    _result.push(_decompress_data);
                    
                    continue;
                };

                let file_info = FileInfo {
                    file_name: _f_name,
                    size: _entry_size as usize,
                    is_success: true,
                    error: None,
                };

                let tmp = DecompressedData {
                    file_info,
                    value: readed_data,
                };
                _result.push(tmp);
            }
        }

        match read_close_and_free(_archive) {
            Ok(_) => {
                return Ok(_result);
            },
            Err(e) => {
                _result.clear();
                return Err(e);
            }
        }
    }

    fn extract_to_dir(&self, file_path: &str, target_dir_path: &str, flags: Option<i32>) -> LibArchiveResult<Vec<FileInfo>> {
        let _f_p = std::path::Path::new(file_path);
        if !_f_p.exists() {
            return Err(LibArchiveError::IsNotExists);
        }
        if !_f_p.is_file() {
            return Err(LibArchiveError::IsNotFile);
        }

        let _dir_path = std::path::Path::new(target_dir_path);
        if !_dir_path.exists() {
            let _r = std::fs::create_dir(_dir_path);
            if _r.is_err() {
               return Err(LibArchiveError::FailedCreateDirectory); 
            }
        }
        if !_dir_path.is_dir() {
            return Err(LibArchiveError::IsNotDir);
        }
        
        let Ok(_meta) = std::fs::metadata(_f_p) else {
            return Err(LibArchiveError::FailedGetMetaDataFromFile);
        };

        let Ok(_file_path_cstr) = std::ffi::CString::new(file_path) else {
            return Err(LibArchiveError::NulError);
        };

        let _f_size = _meta.len() as usize;
        
        let mut _archive = unsafe { libarchive3_sys::archive_read_new() };
        if _archive.is_null() {
            return Err(LibArchiveError::FailedCreateArchive);
        }
        set_all_filter_and_format(_archive)?;
        
        unsafe {
            let _status_code = libarchive3_sys::archive_read_open_filename(_archive, _file_path_cstr.as_ptr(), _f_size);
            if _status_code != 0 {
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(_status_code)));
            }
        };

        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            match read_close_and_free(_archive) {
                Ok(_) => return Err(LibArchiveError::FailedCreateArchiveEntry),
                Err(_) => return Err(LibArchiveError::FailedCreateArchiveEntryAndFailedCloseRead)
            }

        }

        let write_disk = unsafe { libarchive3_sys::archive_write_disk_new() };
        let _flags: c_int = match flags {
            Some(v) => v,
            _ => {
                libarchive3_sys::ARCHIVE_EXTRACT_TIME
                    | libarchive3_sys::ARCHIVE_EXTRACT_PERM
                    | libarchive3_sys::ARCHIVE_EXTRACT_ACL
                    | libarchive3_sys::ARCHIVE_EXTRACT_FFLAGS
            }
        };
        
        unsafe {
            libarchive3_sys::archive_write_disk_set_options(write_disk, _flags);
            libarchive3_sys::archive_write_disk_set_standard_lookup(write_disk);
        }

        let mut _result: Vec<FileInfo> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(_archive, &mut entry) != 1 {
                let mut _status_code = 0;
                let mut _f_name = std::string::String::new();
                match get_pathname_from_entry(entry) {
                    Ok(_name) => {
                        _f_name = _name;
                    },
                    Err(e) => {
                        let _file_info = FileInfo {
                            file_name: "".to_owned(),
                            size: 0,
                            is_success: false,
                            error: Some(e),
                        };
                        _result.push(_file_info);

                        continue;
                    }
                }

                let _out_path = _dir_path.join(&_f_name);
                let Some(_p) = _out_path.as_path().to_str() else {
                    let _file_info = FileInfo {
                        file_name: _f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGeneratePath),
                    };
                    _result.push(_file_info);
                    
                    continue;
                };
                let Ok(_path_with_terminate) = std::ffi::CString::new(_p) else {
                    let _file_info = FileInfo {
                        file_name: _f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGeneratePath),
                    };
                    _result.push(_file_info);
                    
                    continue;
                };

                libarchive3_sys::archive_entry_set_pathname_utf8(entry, _path_with_terminate.as_ptr());
                let _status_code = libarchive3_sys::archive_write_header(write_disk, entry);
                if _status_code != 0 {
                    let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                    
                    let _file_info = FileInfo {
                        file_name: _f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedWriteHeader),
                    };
                    _result.push(_file_info);

                    continue;
                }
                
                let mut _entry_size = libarchive3_sys::archive_entry_size(entry);
                if _entry_size < 1 {
                    let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                    
                    let _file_info = FileInfo {
                        file_name: _f_name,
                        size: _entry_size as usize,
                        is_success: false,
                        error: Some(LibArchiveError::EntrySizeLessThanOne),
                    };
                    _result.push(_file_info);

                    continue;
                }

                let _write_error = match read_and_write_data(_archive, write_disk) {
                    Ok(_) => {
                        None
                    },
                    Err(e) => {
                        Some(e)
                    }
                };
                
                let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                
                let _file_info = FileInfo {
                    file_name: _f_name,
                    size: _entry_size as usize,
                    is_success: true,
                    error: _write_error,
                };
                        
                _result.push(_file_info);

            }
        }

        let mut status = unsafe { libarchive3_sys::archive_write_close(write_disk) };
        if status != 0 {
            return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(status)));
        }
        status = unsafe { libarchive3_sys::archive_write_free(write_disk) };
        if status != 0 {
            return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(status)));
        }

        match read_close_and_free(_archive) {
            Ok(_) => {
                return Ok(_result);
            },
            Err(e) => {
                _result.clear();
                return Err(e);
            }
        }
    }

}


