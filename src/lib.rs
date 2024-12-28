use libarchive3_sys_by_madosuki as libarchive3_sys;
pub use libarchive3_sys::{ArchiveStruct, ArchiveEntryStruct};

use libc::{ c_int, c_char, c_void, size_t};
pub mod error;
pub use error::{LibArchiveError, LibArchiveResult, LibArchiveInternalStatus};

// fn entry_free(entry: *mut ArchiveEntryStruct) {
//     unsafe { libarchive3_sys::archive_entry_free(entry); }
// }

fn set_all_filter_and_format(archive: *mut ArchiveStruct) -> LibArchiveResult<()> {
    let read_support_filter_all_result = unsafe { libarchive3_sys::archive_read_support_filter_all(archive) };
    if read_support_filter_all_result != 0 {
        return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(read_support_filter_all_result)));
    }

    let read_support_format_all_result = unsafe { libarchive3_sys::archive_read_support_format_all(archive) };
    if read_support_format_all_result != 0 {
        return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(read_support_format_all_result)));
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
    let pathname = unsafe { libarchive3_sys::archive_entry_pathname(entry) };
    if pathname.is_null() {
        return Err(LibArchiveError::FailedGetPathNameFromEntry);
    }

    match convert_c_char_to_string(pathname) {
        Some(name) => {
            Ok(name)
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
        
        let mut readed_size = 0 as size_t;
        let r = unsafe { libarchive3_sys::archive_read_data_block(archive, &buf, &mut readed_size, &mut offset) };
        if r == 1 {
            break;
        }

        let write_dta_block_result = unsafe { libarchive3_sys::archive_write_data_block(archive_write, buf as *const c_void, readed_size, offset) };
        if write_dta_block_result == -1 {
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
        let mut readed_size = 0 as usize;
        let r = unsafe { libarchive3_sys::archive_read_data_block(archive, &tmp, &mut readed_size, &mut offset) };

        if r == 1 {
            break;
        }

        let for_safe: &[u8] = unsafe { std::slice::from_raw_parts(tmp as *mut u8, readed_size) };
        result.append(&mut for_safe.to_vec());

        if r == 0 {
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

fn read_close_and_free(read_archive: *mut ArchiveStruct) -> LibArchiveResult<()> {
    if read_archive.is_null() {
        return Ok(());
    }
        
    let close_status_code = unsafe { libarchive3_sys::archive_read_close(read_archive) };
    if close_status_code != 0 {
        return Err(LibArchiveError::FailedCloseReadArchive);
    }

    read_free(read_archive)?;

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

        let Ok(file_path_cstr) = std::ffi::CString::new(file_path) else {
            return Err(LibArchiveError::NulError);
        };


        let archive: *mut ArchiveStruct = unsafe { libarchive3_sys::archive_read_new() };
        if archive.is_null() {
            return Err(LibArchiveError::FailedCreateArchive);
        }

        set_all_filter_and_format(archive)?;

        let f_size = _meta.len() as usize;
        unsafe {
            let status_code = libarchive3_sys::archive_read_open_filename(archive, file_path_cstr.as_ptr(), f_size);
            if status_code != 0 {
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(status_code)));
            }
        };
        
        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            match read_close_and_free(archive) {
                Ok(_) => {
                    return Err(LibArchiveError::FailedCreateArchiveEntry);
                },
                Err(_) => {
                    return Err(LibArchiveError::FailedCreateArchiveEntryAndFailedCloseRead);
                }
            }
        }

        let mut result: Vec<DecompressedData> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(archive, &mut entry) != 1 {
                let pathname = libarchive3_sys::archive_entry_pathname(entry);
                if pathname.is_null() {
                    let file_info = FileInfo {
                        file_name: "".to_owned(),
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGetPathNameFromEntry),
                    };

                    let decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    result.push(decompress_data);
                    
                    continue;
                }

                let f_name;
                match convert_c_char_to_string(pathname) {
                    Some(n) => {
                        f_name = n;
                    },
                    _ => {
                        let file_info = FileInfo {
                            file_name: "".to_owned(),
                            size: 0,
                            is_success: false,
                            error: Some(LibArchiveError::FailedGetPathNameFromEntry),
                        };

                        let decompress_data = DecompressedData {
                            file_info,
                            value: vec!(),
                        };
                        result.push(decompress_data);

                        continue;
                    },
                }
                
                let entry_size = libarchive3_sys::archive_entry_size(entry);
                if entry_size < 1 {
                    let file_info = FileInfo {
                        file_name: f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::EntrySizeLessThanOne),
                    };

                    let decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    result.push(decompress_data);
                    
                    continue;
                }

                let Ok(readed_data) = read_data(archive) else {
                    let file_info = FileInfo {
                        file_name: f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedUncompress),
                    };

                    let decompress_data = DecompressedData {
                        file_info,
                        value: vec!(),
                    };
                    result.push(decompress_data);
                    
                    continue;
                };

                let file_info = FileInfo {
                    file_name: f_name,
                    size: entry_size as usize,
                    is_success: true,
                    error: None,
                };

                let tmp = DecompressedData {
                    file_info,
                    value: readed_data,
                };
                result.push(tmp);
            }
        }

        match read_close_and_free(archive) {
            Ok(_) => {
                return Ok(result);
            },
            Err(e) => {
                result.clear();
                return Err(e);
            }
        }
    }

    fn extract_to_dir(&self, file_path: &str, target_dir_path: &str, flags: Option<i32>) -> LibArchiveResult<Vec<FileInfo>> {
        let f_p = std::path::Path::new(file_path);
        if !f_p.exists() {
            return Err(LibArchiveError::IsNotExists);
        }
        if !f_p.is_file() {
            return Err(LibArchiveError::IsNotFile);
        }

        let dir_path = std::path::Path::new(target_dir_path);
        if !dir_path.exists() {
            let r = std::fs::create_dir(dir_path);
            if r.is_err() {
               return Err(LibArchiveError::FailedCreateDirectory); 
            }
        }
        if !dir_path.is_dir() {
            return Err(LibArchiveError::IsNotDir);
        }
        
        let Ok(meta) = std::fs::metadata(f_p) else {
            return Err(LibArchiveError::FailedGetMetaDataFromFile);
        };

        let Ok(file_path_cstr) = std::ffi::CString::new(file_path) else {
            return Err(LibArchiveError::NulError);
        };

        let f_size = meta.len() as usize;
        
        let archive = unsafe { libarchive3_sys::archive_read_new() };
        if archive.is_null() {
            return Err(LibArchiveError::FailedCreateArchive);
        }
        set_all_filter_and_format(archive)?;
        
        unsafe {
            let status_code = libarchive3_sys::archive_read_open_filename(archive, file_path_cstr.as_ptr(), f_size);
            if status_code != 0 {
                return Err(LibArchiveError::LibArchiveInternalError(LibArchiveInternalStatus::from(status_code)));
            }
        };

        let mut entry: *mut ArchiveEntryStruct = unsafe { libarchive3_sys::archive_entry_new() };
        if entry.is_null() {
            match read_close_and_free(archive) {
                Ok(_) => return Err(LibArchiveError::FailedCreateArchiveEntry),
                Err(_) => return Err(LibArchiveError::FailedCreateArchiveEntryAndFailedCloseRead)
            }

        }

        let write_disk = unsafe { libarchive3_sys::archive_write_disk_new() };
        let flags: c_int = match flags {
            Some(v) => v,
            _ => {
                libarchive3_sys::ARCHIVE_EXTRACT_TIME
                    | libarchive3_sys::ARCHIVE_EXTRACT_PERM
                    | libarchive3_sys::ARCHIVE_EXTRACT_ACL
                    | libarchive3_sys::ARCHIVE_EXTRACT_FFLAGS
            }
        };
        
        unsafe {
            libarchive3_sys::archive_write_disk_set_options(write_disk, flags);
            libarchive3_sys::archive_write_disk_set_standard_lookup(write_disk);
        }

        let mut result: Vec<FileInfo> = vec!();
        unsafe {
            while libarchive3_sys::archive_read_next_header(archive, &mut entry) != 1 {
                let f_name;
                match get_pathname_from_entry(entry) {
                    Ok(name) => {
                        f_name = name;
                    },
                    Err(e) => {
                        let file_info = FileInfo {
                            file_name: "".to_owned(),
                            size: 0,
                            is_success: false,
                            error: Some(e),
                        };
                        result.push(file_info);

                        continue;
                    }
                }

                let out_path = dir_path.join(&f_name);
                let Some(path_str) = out_path.as_path().to_str() else {
                    let _file_info = FileInfo {
                        file_name: f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGeneratePath),
                    };
                    result.push(_file_info);
                    
                    continue;
                };
                let Ok(path_with_terminate) = std::ffi::CString::new(path_str) else {
                    let file_info = FileInfo {
                        file_name: f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedGeneratePath),
                    };
                    result.push(file_info);
                    
                    continue;
                };

                libarchive3_sys::archive_entry_set_pathname_utf8(entry, path_with_terminate.as_ptr());
                let status_code = libarchive3_sys::archive_write_header(write_disk, entry);
                if status_code != 0 {
                    let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                    
                    let file_info = FileInfo {
                        file_name: f_name,
                        size: 0,
                        is_success: false,
                        error: Some(LibArchiveError::FailedWriteHeader),
                    };
                    result.push(file_info);

                    continue;
                }
                
                let entry_size = libarchive3_sys::archive_entry_size(entry);
                if entry_size < 1 {
                    let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                    
                    let _file_info = FileInfo {
                        file_name: f_name,
                        size: entry_size as usize,
                        is_success: false,
                        error: Some(LibArchiveError::EntrySizeLessThanOne),
                    };
                    result.push(_file_info);

                    continue;
                }

                let _write_error = match read_and_write_data(archive, write_disk) {
                    Ok(_) => {
                        None
                    },
                    Err(e) => {
                        Some(e)
                    }
                };
                
                let _ = libarchive3_sys::archive_write_finish_entry(write_disk);
                
                let _file_info = FileInfo {
                    file_name: f_name,
                    size: entry_size as usize,
                    is_success: true,
                    error: _write_error,
                };
                        
                result.push(_file_info);

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

        match read_close_and_free(archive) {
            Ok(_) => {
                return Ok(result);
            },
            Err(e) => {
                result.clear();
                return Err(e);
            }
        }
    }

}


