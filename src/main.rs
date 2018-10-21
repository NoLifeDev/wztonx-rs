
extern crate byteorder;
extern crate memmap;

mod keys;

use byteorder::{ByteOrder, LittleEndian};
use memmap::Mmap;
use std::{
    env::args_os,
    fs::File,
    io::Result,
    path::Path,
};

pub const KEYS: &[&[u8]; 3] = &[keys::KEY_BMS, keys::KEY_GMS, keys::KEY_KMS];

struct FileIn {
    index: usize,
    file: Mmap,
}
impl FileIn {
    fn open(path: &Path) -> Result<FileIn> {
        let file = File::open(path)?;
        // Yes I know this is unsafe but whatever
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(FileIn {
            index: 0,
            file: mmap,
        })
    }
    fn seek(&mut self, n: usize) {
        self.index = n;
    }
    fn skip(&mut self, n: usize) {
        self.index += n;
    }
    fn tell(&self) -> usize {
        self.index
    }
    fn read_u8(&mut self) -> u8 {
        let value = self.file[self.index];
        self.index += 1;
        value
    }
    fn read_i8(&mut self) -> i8 {
        let value = self.file[self.index];
        self.index += 1;
        value as i8
    }
    fn read_u32(&mut self) -> u32 {
        let slice = &self.file[self.index..];
        self.index += 4;
        LittleEndian::read_u32(slice)
    }
    fn read_i32(&mut self) -> i32 {
        let slice = &self.file[self.index..];
        self.index += 4;
        LittleEndian::read_i32(slice)
    }
    fn read_cint(&mut self) -> i32 {
        let n = self.read_i8();
        if n != -128 { n as i32 } else { self.read_i32() }
    }
    fn read_slice(&mut self, len: usize) -> &[u8] {
        let slice = &self.file[self.index..self.index + len];
        self.index += len;
        slice
    }
}
struct WzToNx {
    fin: FileIn,
    start: usize,
    key: &'static [u8],
}
impl WzToNx {
    fn new(inpath: &Path, outpath: &Path) -> WzToNx {
        let fin = FileIn::open(inpath).unwrap();
        WzToNx {
            fin: fin,
            start: 0,
            key: keys::KEY_BMS,
        }
    }
    fn parse_header(&mut self) {
        let magic = self.fin.read_u32();
        assert_eq!(magic, 0x31474B50, "not a valid WZ file");
        self.fin.skip(8);
        self.start = self.fin.read_u32() as usize;
        self.fin.seek(self.start + 2);
        self.fin.read_cint();
        self.fin.skip(1);
        self.deduce_key();
        self.fin.seek(self.start + 2);
    }
    fn deduce_key(&mut self) {
        let len = self.fin.read_i8();
        assert!(len < 0, "could not deduce key because length was not negative");
        let slen = if len == -128 { self.fin.read_u32() as usize } else { -len as usize };
        let os = self.fin.read_slice(slen);
        for key in KEYS {
            let mut mask: u8 = 0xAA;
            let mut valid = true;
            for i in 0..slen {
                let c = os[i] ^ key[i] ^ mask;
                if c < 0x20 || c >= 0x80 {
                    valid = false;
                }
                mask = mask.wrapping_add(1);
            }
            if valid {
                self.key = key;
                return;
            }
        }
        panic!("failed to deduce key");
    }
    fn convert_wz(inpath: &Path, outpath: &Path) {
        println!("Working on {:?}", inpath);
        let mut wztonx = WzToNx::new(inpath, outpath);
        wztonx.parse_header();
        println!("Done with {:?}", outpath);
    }
}

fn main() {
    let args: Vec<_> = args_os().skip(1).collect();
    for arg in args {
        let path = Path::new(&arg);
        WzToNx::convert_wz(path, &path.with_extension("nx"));
    }
}
