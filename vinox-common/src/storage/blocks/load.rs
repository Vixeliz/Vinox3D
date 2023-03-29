use directories::ProjectDirs;
use std::fs;

use walkdir::WalkDir;

use super::descriptor::BlockDescriptor;

pub fn load_all_blocks() -> Vec<BlockDescriptor> {
    let mut result = Vec::new();
    if let Some(proj_dirs) = ProjectDirs::from("com", "vinox", "vinox") {
        for entry in WalkDir::new(proj_dirs.data_dir().join("assets/blocks"))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().unwrap_or_default() == "ron" {
                if let Ok(ron_string) = fs::read_to_string(entry.path()) {
                    let ron_result = ron::from_str::<BlockDescriptor>(ron_string.as_str());
                    if let Ok(block) = ron_result {
                        if let Some(auto_geo) = block.clone().auto_geo {
                            for geo in auto_geo.iter() {
                                let mut new_block = block.clone();
                                new_block.auto_geo = None;
                                new_block.geometry = Some(geo.clone());
                                new_block.has_item = Some(false);
                                new_block.name = block.name.clone() + "." + &geo.get_geo_name();
                                result.push(new_block);
                            }
                        }

                        result.push(block);
                    } else {
                        println!("{ron_result:?}, {:?}", entry.path());
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::storage::{blocks::descriptor::BlockDescriptor, items::descriptor::ToolType};

    #[test]
    fn ron_loads() {
        let ron_type = "
            BlockDescriptor(
                namespace: \"vinox\",
                name: \"grass\",
                tool_type: Some(Shovel)
            )
        ";
        if let Ok(ron_string) = ron::from_str::<BlockDescriptor>(ron_type) {
            assert_eq!(
                ron_string,
                BlockDescriptor {
                    namespace: "vinox".to_string(),
                    name: "grass".to_string(),
                    tool_type: Some(ToolType::Shovel),
                    ..Default::default()
                }
            )
        }
    }
}
