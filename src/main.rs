
extern crate memmap;

mod keys;

use std::env::args_os;
use std::path::Path;

fn main() {
    let args: Vec<_> = args_os().skip(1).collect();
    for arg in args {
        let path = Path::new(&arg);
        convert(path, &path.with_extension("nx"));
    }
}
fn convert(inpath: &Path, outpath:&Path) {
    println!("{:?} -> {:?}", inpath, outpath);
}
