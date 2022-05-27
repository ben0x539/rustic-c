// - runtime-c
//
// "Does God stay in Heaven out of fear of what we have created?"
//
// A very, very, VERY horrible thing made by Rin
// please never do this
//
//      lostkagamine@outlook.com
//      @lostkagamine@twitter.com
//      https://kagamine-r.in

use std::{process::Command, ptr, ffi::CString, fs};
use std::alloc::Layout;

// One meg oughta be enough for everyone
const ONE_MEGABYTE: usize = 1_048_576usize;

extern "C" {
    // Yes, we need this.
    fn mprotect(addr: *mut (), len: usize, prot: i32);

    fn dlsym(handle: *const(), symbol: *const i8) -> *const ();
}

/// Compiles C code found in `code' and returns a raw page-aligned pointer containing its data.
pub unsafe fn compile_c(code: &str) -> (*mut u8, usize) {
    fs::write("temp_file.c", code).unwrap();

    let gcc = Command::new("gcc")
        .args(["-Wl,--format=binary", "temp_file.c", "-o", "tmp.bin", "-c", "-m64"])
        .output()
        .expect("gcc error");

    if !gcc.status.success() {
        panic!("gcc exited unhappily: {}", String::from_utf8_lossy(&gcc.stderr));
    }

    let objdump = Command::new("objdump")
        .args(["--reloc", "tmp.bin"])
        .output()
        .expect("should have executed gcc");

    if !objdump.status.success() {
        panic!("objdump exited unhappily: {}", String::from_utf8_lossy(&gcc.stderr));
    }

    let syms = String::from_utf8_lossy(&objdump.stdout).lines().filter_map(|line| {
        if line.len() <= 35 || !line.as_bytes()[0].is_ascii_digit() || !line.as_bytes()[35].is_ascii_alphabetic() {
            return None;
        }

        Some(line[35..].split('-').next()?.to_string())
    }).collect::<Vec<_>>();

    let mut new_code = code.to_string();

    for sym in &syms {
        let c_sym = CString::new(sym.as_bytes()).unwrap();
        let p = dlsym(ptr::null(), c_sym.as_ptr());
        if p.is_null() { continue; }

        new_code = new_code.replace(sym, &format!("((void(*)()) {})", p as usize));
    }

    fs::write("temp_file.c", &new_code).unwrap();

    let gcc = Command::new("gcc")
        .args(["-Wl,--format=binary", "temp_file.c", "-o", "tmp.bin", "-c", "-m64"])
        .output()
        .expect("gcc error");

    if !gcc.status.success() {
        panic!("gcc exited unhappily: {}", String::from_utf8_lossy(&gcc.stderr));
    }

    // invoke `objcopy' to ensure we have a flat bin
    let objcopy = Command::new("objcopy")
        .args(["-O", "binary", "-j", ".text", "tmp.bin", "flat.bin"])
        .output()
        .expect("objcopy error");

    if !objcopy.status.success() {
        panic!("objcopy exited unhappily: {}", String::from_utf8_lossy(&objcopy.stderr));
    }
    
    let alignment = page_size::get();
    let layout = Layout::from_size_align(ONE_MEGABYTE, alignment).expect("could not construct page-aligned 1mb layout");

    let memory = std::alloc::alloc(layout);
    let byte_file = std::fs::read("flat.bin").expect("could not read temporary file");
    let count = byte_file.len();
    std::ptr::copy(byte_file.as_ptr(), memory, count);

    std::fs::remove_file("temp_file.c").unwrap();
    std::fs::remove_file("tmp.bin").unwrap();
    std::fs::remove_file("flat.bin").unwrap();

    return (memory, count);
}

/// Does horrible, horrible things. (executes code pointed to by `ptr' where it is assumed to point to
/// a memory region containing valid executable code for your current architecture. `ptr' must also
/// be page-aligned or else `mprotect' will be very, very sad)
pub unsafe fn do_horrible_crimes<T>(ptr: *mut u8, size: usize) -> T {
    //                                PROT_EXEC | PROT_WRITE | PROT_READ
    mprotect(ptr as *mut (), size, 0x01 | 0x02 | 0x04);
    let x = ptr as *const ();
    let func = std::mem::transmute::<*const (), fn() -> T>(x);
    func()
}
