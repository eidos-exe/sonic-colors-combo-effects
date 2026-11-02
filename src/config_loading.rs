use crate::data::{CharacterData,parse_effect_config};
use std::{collections::HashMap,path::Path,fs};
pub fn load(root:&Path,filename:&str,defaults:&str)->(HashMap<String,CharacterData>,String) {
    let mut config=parse_effect_config(defaults).expect("Invalid bundled combo defaults");
    let mut selected=String::from("bundled defaults");
    if let Ok(entries)=fs::read_dir(root) {
        let mut paths:Vec<_>=entries.filter_map(Result::ok).map(|e|e.path().join(filename)).collect();
        paths.sort();
        for path in paths {
            if let Ok(text)=fs::read_to_string(&path) {
                if let Ok(loaded)=parse_effect_config(&text) {config=loaded;selected=path.to_string_lossy().into_owned();}
            }
        }
    }
    (config,selected)
}
