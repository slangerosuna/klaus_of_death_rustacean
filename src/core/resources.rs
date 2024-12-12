use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref RESOURCES_DIR: String = {
        let bin_dir = std::env::current_exe().expect("Can't find path to executable");
        let bin_dir = bin_dir.parent().unwrap();

        let resources_dir = format!("{}/kod_resources", bin_dir.display());

        resources_dir
    };
}

pub fn get_resource_string(resource: &str) -> String {
    let path = format!("{}/{}", *RESOURCES_DIR, resource);
    std::fs::read_to_string(path).unwrap()
}

pub fn get_resource_bin(resource: &str) -> Vec<u8> {
    let path = format!("{}/{}", *RESOURCES_DIR, resource);
    std::fs::read(path).unwrap()
}

pub fn get_resource_ron<T>(resource: &str) -> T
where
    for<'de> T: Deserialize<'de>,
{
    let data = get_resource_string(resource);
    ron::from_str(&data).unwrap()
}

pub fn get_resource_toml<T>(resource: &str) -> T
where
    for<'de> T: Deserialize<'de>,
{
    let data = get_resource_string(resource);
    toml::from_str(&data).unwrap()
}

pub fn get_resource_bincode<T>(resource: &str) -> T
where
    for<'de> T: Deserialize<'de>,
{
    let data = get_resource_bin(resource);
    bincode::deserialize(&data).unwrap()
}
