use crate::{MxResult, TAction};
use ::cargo_toml::Manifest;
use ::derive_builder::Builder;
use ::std::{collections::HashMap, io::Write, fs::File, io::Read, path::Path, time::{Duration, SystemTime}};
use ::toml::{map::Map, Value};
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
    fn get_cached_last_modified_time(&mut self) -> MxResult<Option<u64>> {
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
    fn put_last_modified_time(&mut self, last_modified_time: u64) -> MxResult<()> {
        let manifest = self.manifest.as_mut().map(|manifest| {
            macro_rules! modify_metadata {
                ($origin: expr) => {
                    $origin.as_mut().map(|root| {
                        let md = root.metadata.as_mut().map_or_else(|| -> Option<Value> {
                            let hm: HashMap<String, Value> = [(Action::KEY.to_string(), Value::Integer(last_modified_time as i64))].into();
                            Some(hm.into())
                        }, |metadata| {
                            metadata.as_table_mut().map_or_else(|| -> Option<Value> {
                                let hm: HashMap<String, Value> = [(Action::KEY.to_string(), Value::Integer(last_modified_time as i64))].into();
                                Some(hm.into())
                            }, |key_values: &mut Map<String, Value>| -> Option<Value> {
                                key_values.insert(Action::KEY.to_string(), Value::Integer(last_modified_time as i64));
                                None
                            })
                        });
                        if md.is_some() {
                            root.metadata = md;
                        }
                    });
                };
            }
            if manifest.workspace.is_some() {
                modify_metadata!(manifest.workspace);
            } else if manifest.package.is_some() {
                modify_metadata!(manifest.package);
            }
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