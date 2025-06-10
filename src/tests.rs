use crate::fs_utils::find_file;
use crate::hashes::{cache_hash, hash};

#[test]
fn test_hash(){
    let cwd = std::env::current_dir().unwrap();
    println!("{:?}", cwd);
    let path = find_file("test-c-project/src/main.c").unwrap();
    println!("{:?}", &path);

}