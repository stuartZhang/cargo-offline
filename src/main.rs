#![cfg_attr(all(feature = "cargo-metadata", not(feature = "toml-config")), feature(file_set_times))]
/// 【`Strategy`设计模式】的【依赖注入】项
#[cfg(all(feature = "cargo-metadata", not(feature = "toml-config")))]
mod cargo_metadata;
#[cfg(all(feature = "toml-config", not(feature = "cargo-metadata")))]
mod toml_file;
/// 程序内启用了【`Builder`设计模式】与【`Strategy`设计模式】
use ::proc_lock::proc_lock;
use ::std::{error::Error, iter::Iterator, env::{VarError, self}, fs::File, path::Path, process::Command, time::SystemTime};
type MxResult<T> = Result<T, Box<dyn Error>>;
/// 【`Strategy`设计模式】的【依赖注入】规格定义
trait TAction<'a> {
    const KEY: &'a str = "last-modified-system-time";
    fn get_manifest_path(&self) -> &'a Path;
    fn get_cached_last_modified_time(&mut self) -> MxResult<Option<u64>>;
    fn put_last_modified_time(&mut self, last_modified_time: u64) -> MxResult<()>;
}
/// 【`Strategy`设计模式】的`IoC`容器
#[cfg_attr(debug_assertions, proc_lock(name = "cargo-offline.debug.lock"))]
#[cfg_attr(not(debug_assertions), proc_lock(name = "cargo-offline.lock"))]
fn ioc_container<'a, T>(action: Option<T>) -> MxResult<()> where T: TAction<'a> {
    let mut args: Vec<String> = match env::args().nth(1) {
        Some(arg1st) if arg1st == "offline" => env::args().skip(2).collect(),
        _ => env::args().skip(1).collect()
    };
    let cargo_bin = env::var("CARGO").or_else(|_| -> Result<String, VarError> {
        Ok("cargo".to_string())
    })?;
    if let Some(mut action) = action {
        let last_modified_time = {
            let manifest_file = File::open(action.get_manifest_path())?;
            manifest_file.metadata()?.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
        };
        let cached_last_modified_time = action.get_cached_last_modified_time()?;
        let is_write = cached_last_modified_time.map_or(true, |cached_last_modified_time| {
            cached_last_modified_time < last_modified_time
        });
        if is_write {
            action.put_last_modified_time(last_modified_time)?;
        } else {
            let offline_arg = "--offline".to_string();
            if !args.contains(&offline_arg) {
                args.push(offline_arg);
            }
        }
    }
    #[cfg(debug_assertions)]
    dbg!(&cargo_bin, &args);
    let mut child = Command::new(cargo_bin).args(args).spawn()?;
    let exit_code = child.wait()?;
    #[cfg(debug_assertions)]
    dbg!(exit_code);
    Ok(())
}
fn main() -> MxResult<()> {
    let manifest_path = locate_cargo_manifest::locate_manifest();
    ioc_container(if let Ok(manifest_path) = manifest_path.as_ref() {
        #[cfg(all(feature = "cargo-metadata", not(feature = "toml-config")))]
        let action = cargo_metadata::ActionBuilder::default().manifest_path(manifest_path).build()?;
        #[cfg(all(feature = "toml-config", not(feature = "cargo-metadata")))]
        let action = toml_file::ActionBuilder::default().manifest_path(manifest_path).build()?;
        Some(action)
    } else {
        None
    })?;
    Ok(())
}
