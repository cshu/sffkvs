#![allow(clippy::print_literal)]
#![allow(clippy::needless_return)]
#![allow(dropping_references)]
#![allow(clippy::assertions_on_constants)]
mod common;
mod util;

use common::CustRes;
use common::CustomErr;

use log::*;
use std::path::PathBuf;
use std::process::*;
use std::*;

#[macro_use(defer)]
extern crate scopeguard;

fn main() -> ExitCode {
    env::set_var("RUST_BACKTRACE", "1"); //? not 100% sure this has 0 impact on performance? Maybe setting via command line instead of hardcoding is better?
                                         //env::set_var("RUST_LIB_BACKTRACE", "1");//? this line is useless?
                                         ////
    env::set_var("RUST_LOG", "trace"); //note this line must be above logger init.
    env_logger::init();

    let args: Vec<String> = env::args().collect(); //Note that std::env::args will panic if any argument contains invalid Unicode.
    defer! {
    if std::thread::panicking() {
            eprintln!("{}", "PANICKING");
    }
        //println!("{}", "ALL DONE");
    }
    if main_inner(args).is_err() {
        return ExitCode::from(1);
    }
    ExitCode::from(0)
}
fn main_inner(args: Vec<String>) -> CustRes<()> {
    use sha2::Digest;
    let mut ctx = Ctx {
        hasher: sha2::Sha256::new(),
        args,
        ..Default::default()
    };
    ctx.home_dir = dirs::home_dir().ok_or("Failed to get home directory.")?;
    if !util::real_dir_without_symlink(&ctx.home_dir) {
        return dummy_err("Failed to recognize the home dir as folder.");
    }
    ctx.everycom = ctx.home_dir.join(".everycom");
    ctx.app_support_dir = ctx.everycom.join(PKG_NAME);
    ctx.store_dir = ctx.app_support_dir.join("store");
    fs::create_dir_all(&ctx.store_dir)?;
    ctx.parse_args()?;
    ctx.key_folder_path = ctx.store_dir.join(ctx.key_hash);
    if ctx.v_provided {
        fs::create_dir_all(&ctx.key_folder_path)?; //create_dir is also fine but returns err when it already exists
        fs::write(ctx.key_folder_path.join("k0"), ctx.key)?;
        fs::write(ctx.key_folder_path.join("v0"), ctx.value)?;
    } else {
        if !ctx.key_folder_path.try_exists()? {
            return dummy_err("Key is not found");
        }
        //let k0 = ctx.key_folder_path.join("k0");
        let v0 = ctx.key_folder_path.join("v0");
        //if !util::real_reg_file_without_symlink(&k0) {
        //    return dummy_err("Key is corrupt");
        //}
        if !util::real_reg_file_without_symlink(&v0) {
            return dummy_err("Value is corrupt");
        }
        println!("{}", fs::read_to_string(v0)?);
    }
    Ok(())
}
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const _: () = assert!(!PKG_NAME.is_empty(), "Constraint on const");

//todo default `get` behavior is printing on stdout but you should add one option to just print full file path of value file on stdout. Say, `-filename-only`
//todo allow specifying a custom location instead of default location as store `sffkvs -s=/opt/special_store -k=KK -v=VV`
//todo support tuple key and tuple value (i.e. similar to Rust tuple. `sffkvs -k=K1 -k=K2 -v=V1 -v=V2`. In this case, folder name will be longer than 64. 2 keys mean folder name length of 65. One char '2' is appended to the folder name. Thus 10 keys will make folder name length 66)
//todo add option for deletion
//todo add option for deleting everything
//todo if 2 instances of this program run simultaneously then there might be problems. Try to do some locking

#[derive(Default)]
struct Ctx {
    hasher: sha2::Sha256,
    args: Vec<String>,
    home_dir: PathBuf,
    everycom: PathBuf,
    app_support_dir: PathBuf,
    store_dir: PathBuf,
    v_provided: bool,
    key: String,
    value: String,
    key_hash: String,
    key_folder_path: PathBuf,
}

impl Ctx {
    fn parse_args_as_kv(&mut self) -> CustRes<()> {
        self.key = self.args.get(2).ok_or("No key")?.clone();
        if let Some(vstr) = self.args.get(3) {
            self.v_provided = true;
            self.value = vstr.clone();
        }
        self.prepare_key_hash()
    }
    fn parse_args(&mut self) -> CustRes<()> {
        let arg1 = self.args.get(1).ok_or("No args found")?;
        if "--" == arg1 {
            return self.parse_args_as_kv();
        }
        let mut k = String::default();
        let mut v = String::default();
        let mut v_set = false;
        for argstr in &self.args[1..] {
            let (kstr, vstr) = argstr.split_once('=').ok_or("Cannot find = in argument")?;
            match kstr {
                "-k" => {
                    k = vstr.to_owned();
                }
                "-v" => {
                    v = vstr.to_owned();
                    v_set = true;
                }
                _ => {
                    return dummy_err("Failed to recognize argument.");
                }
            }
        }
        if k.is_empty() {
            return dummy_err("Key is needed and it cannot be empty.");
        }
        //if v.is_empty(){
        //	return dummy_err("Value not set.");
        //}
        self.v_provided = v_set;
        self.key = k;
        self.value = v;
        self.prepare_key_hash()
    }
    fn prepare_key_hash(&mut self) -> CustRes<()> {
        let khbytes = calc_hash_of_bytes(&mut self.hasher, self.key.as_bytes())?;
        self.key_hash = bytes2hex(&khbytes)?;
        Ok(())
    }
}

fn dummy_err<T>(msg: &str) -> Result<T, CustomErr> {
    error!("{}", msg);
    Err(CustomErr {})
}

fn calc_hash_of_bytes(hasher: &mut sha2::Sha256, bytes: &[u8]) -> Result<[u8; 32], CustomErr> {
    use sha2::Digest;
    hasher.update(bytes);
    let hash_bytes = hasher.finalize_reset();
    //let hash_bytes = hasher.finalize_boxed_reset();
    //use base64::{engine::general_purpose, Engine as _};
    //return Ok(general_purpose::STANDARD_NO_PAD.encode(hash_bytes));
    let retval: [u8; 32] = hash_bytes.as_slice().try_into()?;
    Ok(retval)
}

fn bytes2hex(bytes: &[u8]) -> Result<String, CustomErr> {
    let mut retval = String::with_capacity(bytes.len() * 2);
    for octet in bytes {
        use std::fmt::Write;
        write!(&mut retval, "{:02x}", octet)?;
    }
    Ok(retval)
}
