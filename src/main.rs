#![cfg_attr(all(feature = "cargo-metadata", not(feature = "toml-config")), feature(file_set_times))]
/// 【`Strategy`设计模式】的【依赖注入】项
#[cfg(all(feature = "cargo-metadata", not(feature = "toml-config")))]
mod cargo_metadata {
    use crate::TAction;
    use ::cargo_toml::Manifest;
    use ::derive_builder::Builder;
    use ::std::{collections::HashMap, error::Error, io::Write, fs::File, io::Read, path::Path, time::{Duration, SystemTime}};
    use ::toml::Value;
    #[derive(Builder)]
    pub struct Action<'a> {
        manifest_path: &'a Path,
        #[builder(setter(skip))]
        manifest: Option<Manifest>
    }
    impl<'a> TAction<'a> for Action<'a> {
        fn get_manifest_path(&self) -> &'a Path {
            self.manifest_path
        }
        fn get_cached_last_modified_time(&mut self) -> Result<Option<u64>, Box<dyn Error>> {
            let mut manifest_str = String::new();
            let mut manifest_file = File::open(self.manifest_path)?;
            manifest_file.read_to_string(&mut manifest_str)?;
            self.manifest = Some(Manifest::<Value>::from_slice(manifest_str.as_bytes())?);
            Ok(self.manifest.as_mut().and_then(|manifest| {
                manifest.package.as_mut()
            }).and_then(|package| {
                package.metadata.as_mut()
            }).and_then(|metadata| {
                metadata.as_table_mut()
            }).and_then(|key_values| {
                key_values.get(&Action::KEY.to_string())
            }).and_then(|old_time| {
                old_time.as_integer()
            }).map(|old_time| {
                old_time as u64
            }))
        }
        fn put_last_modified_time(&mut self, last_modified_time: u64) -> Result<(), Box<dyn Error>> {
            let manifest = self.manifest.as_mut().map(|manifest| {
                manifest.package.as_mut().map(|package| {
                    let md = package.metadata.as_mut().map_or_else(|| {
                        let hm: HashMap<String, Value> = [(Action::KEY.to_string(), Value::Integer(last_modified_time as i64))].into();
                        Some(hm.into())
                    }, |metadata| {
                        metadata.as_table_mut().map_or_else(|| {
                            let hm: HashMap<String, Value> = [(Action::KEY.to_string(), Value::Integer(last_modified_time as i64))].into();
                            Some(hm.into())
                        }, |key_values| {
                            key_values.insert(Action::KEY.to_string(), Value::Integer(last_modified_time as i64));
                            None
                        })
                    });
                    if md.is_some() {
                        package.metadata = md;
                    }
                });
                manifest
            });
            if let Some(manifest) = manifest {
                let serialized = toml::to_string_pretty(manifest)?;
                let mut manifest_file = File::options().write(true).open(self.manifest_path)?;
                manifest_file.write_all(serialized.as_bytes())?;
                manifest_file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(last_modified_time))?;
            }
            Ok(())
        }
    }
}
#[cfg(all(feature = "toml-config", not(feature = "cargo-metadata")))]
mod toml_file {
    use crate::TAction;
    use ::derive_builder::Builder;
    use ::std::{collections::HashMap, error::Error, io::Write, fs::File, io::Read, path::{Path, PathBuf}};
    use ::toml::{map::Map, Value};
    #[derive(Builder)]
    pub struct Action<'a> {
        manifest_path: &'a Path,
        #[builder(setter(skip))]
        config: Option<Value>
    }
    impl<'a> Action<'a> {
        fn get_cache_file_path(&self) -> PathBuf {
            let mut cache_file_path = PathBuf::from(self.manifest_path.parent().unwrap());
            cache_file_path.push("cargo-offline-config.toml");
            cache_file_path
        }
    }
    impl<'a> TAction<'a> for Action<'a> {
        fn get_manifest_path(&self) -> &'a Path {
            self.manifest_path
        }
        fn get_cached_last_modified_time(&mut self) -> Result<Option<u64>, Box<dyn Error>> {
            let cache_file_path = self.get_cache_file_path();
            if cache_file_path.is_file() {
                let mut cache_file_str = String::new();
                let mut cache_file = File::open(&cache_file_path)?;
                cache_file.read_to_string(&mut cache_file_str)?;
                self.config = Some(toml::from_slice(cache_file_str.as_bytes())?);
                return Ok(self.config.as_mut().and_then(|config| {
                    config.as_table()
                }).and_then(|key_values| {
                    key_values.get(&Action::KEY.to_string())
                }).and_then(|last_modified_time| {
                    last_modified_time.as_integer()
                }).map(|last_modified_time| {
                    last_modified_time as u64
                }));
            }
            Ok(None)
        }
        fn put_last_modified_time(&mut self, last_modified_time: u64) -> Result<(), Box<dyn Error>> {
            let cache_file_path = self.get_cache_file_path();
            if self.config.is_none() {
                self.config.replace(Value::Table(Map::new()));
            }
            let config = self.config.as_mut().map(|config| {
                let md = config.as_table_mut().map_or_else(|| {
                    let hm: HashMap<String, Value> = [(Action::KEY.to_string(), Value::Integer(last_modified_time as i64))].into();
                    Some(hm.into())
                }, |key_values| {
                    key_values.insert(Action::KEY.to_string(), Value::Integer(last_modified_time as i64));
                    None
                });
                if let Some(md) = md {
                    *config = md;
                }
                config
            });
            if let Some(config) = config {
                let serialized = toml::to_string_pretty(config)?;
                let mut cache_file = File::options().write(true).create(true).open(&cache_file_path)?;
                cache_file.write_all(serialized.as_bytes())?;
            }
            Ok(())
        }
    }
}
/// 程序内启用了【`Builder`设计模式】与【`Strategy`设计模式】
use ::std::{error::Error, iter::Iterator, env::{VarError, self}, fs::File, path::Path, process::Command, time::SystemTime};
/// 【`Strategy`设计模式】的【依赖注入】规格定义
trait TAction<'a> {
    const KEY: &'a str = "last-modified-system-time";
    fn get_manifest_path(&self) -> &'a Path;
    fn get_cached_last_modified_time(&mut self) -> Result<Option<u64>, Box<dyn Error>>;
    fn put_last_modified_time(&mut self, last_modified_time: u64) -> Result<(), Box<dyn Error>>;
}
/// 【`Strategy`设计模式】的`IoC`容器
fn ioc_container<'a, T>(action: Option<T>) -> Result<(), Box<dyn Error>> where T: TAction<'a> {
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
        let is_write = if let Some(cached_last_modified_time) = cached_last_modified_time {
            cached_last_modified_time < last_modified_time
        } else {
            true
        };
        if is_write {
            action.put_last_modified_time(last_modified_time)?;
        } else if !args.contains(&"--offline".to_string()) {
            args.push("--offline".to_string());
        }
    }
    #[cfg(debug_assertions)]
    dbg!(&cargo_bin, &args);
    Command::new(cargo_bin).args(args).spawn()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
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
