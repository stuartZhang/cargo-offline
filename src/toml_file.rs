use crate::{MxResult, TAction};
use ::derive_builder::Builder;
use ::std::{collections::HashMap, io::Write, fs::File, io::Read, path::{Path, PathBuf}};
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
    fn get_cached_last_modified_time(&mut self) -> MxResult<Option<u64>> {
        let cache_file_path = self.get_cache_file_path();
        if cache_file_path.is_file() {
            let cache_file_str = {
                let mut cache_file_str = String::new();
                let mut cache_file = File::open(&cache_file_path)?;
                cache_file.read_to_string(&mut cache_file_str)?;
                cache_file_str
            };
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
    fn put_last_modified_time(&mut self, last_modified_time: u64) -> MxResult<()> {
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