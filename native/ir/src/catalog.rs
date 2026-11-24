//! BEN catalog 读取：给 BEM→IR 提供骨骼/材质/对象路径的宿主侧身份。
//!
//! catalog 是 `tools/CustomModel/catalog/chr_*.json`（`kind = bem-character-catalog`），
//! 包含每个部件（component）的 mesh 名、骨骼表、材质与贴图槽属性，以及
//! Unity 对象路径（`evidence.observations.*.path`）。这些信息让 IR 能生成
//! `[[skeleton.map]]` / `[[material.override]]`，而不是只写进 `[ext.ben]`。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::Error;

#[derive(Debug, Clone)]
pub struct TextureProperty {
    pub property: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct CatalogComponent {
    pub mesh_name: String,
    pub bones: Vec<String>,
    pub materials: Vec<String>,
    /// 每个材质槽的贴图属性（`_BaseMap` → 贴图对象名）。
    pub texture_properties: Vec<Vec<TextureProperty>>,
    /// Unity 对象路径（优先 UI 模型观察记录）。
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub character_id: String,
    pub components: BTreeMap<u32, CatalogComponent>,
}

impl Catalog {
    pub fn load(dir: &Path, character_id: &str) -> Result<Self, Error> {
        // 文件名不一定等于 character_id（例如 chr_0003_endminf 的 catalog 是 endminf-pc.json），
        // 所以先按文件名试，再扫描目录按 `character_id` 字段匹配。
        let mut candidates = Vec::new();
        let direct = dir.join(format!("{character_id}.json"));
        if direct.is_file() {
            candidates.push(direct.clone());
        }
        if let Ok(entries) = fs::read_dir(dir) {
            let mut others: Vec<_> = entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.extension().map(|ext| ext == "json").unwrap_or(false))
                .collect();
            others.sort();
            for path in others {
                if !candidates.contains(&path) {
                    candidates.push(path);
                }
            }
        }

        let mut root: Option<Value> = None;
        let mut used_path = std::path::PathBuf::new();
        for path in candidates {
            let Ok(bytes) = fs::read(&path) else { continue };
            let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            let kind = value.get("kind").and_then(Value::as_str).unwrap_or("");
            if kind != "bem-character-catalog" {
                continue;
            }
            let id = value.get("character_id").and_then(Value::as_str).unwrap_or("");
            if direct == path || id == character_id {
                root = Some(value);
                used_path = path;
                break;
            }
        }
        let root = root.ok_or_else(|| {
            Error::Ir(format!(
                "catalog 目录里找不到 character_id={character_id}：{}",
                dir.display()
            ))
        })?;
        let _ = used_path;
        let mut components = BTreeMap::new();
        if let Some(map) = root.get("components").and_then(Value::as_object) {
            for (key, value) in map {
                let Ok(id) = key.parse::<u32>() else { continue };
                components.insert(id, parse_component(value));
            }
        }
        Ok(Self {
            character_id: character_id.to_string(),
            components,
        })
    }

    pub fn component(&self, id: u32) -> Option<&CatalogComponent> {
        self.components.get(&id)
    }
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_component(value: &Value) -> CatalogComponent {
    let texture_properties = value
        .get("material_texture_properties")
        .and_then(Value::as_array)
        .map(|materials| {
            materials
                .iter()
                .map(|material| {
                    material
                        .as_array()
                        .map(|properties| {
                            properties
                                .iter()
                                .filter_map(|property| {
                                    Some(TextureProperty {
                                        property: property.get("property")?.as_str()?.to_string(),
                                        name: property.get("name")?.as_str()?.to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();

    // 对象路径：优先包含 uimodel 的观察，其次第一个带 path 的观察。
    let path = value
        .get("evidence")
        .and_then(|evidence| evidence.get("observations"))
        .and_then(Value::as_object)
        .and_then(|observations| {
            let mut fallback = None;
            for (key, observation) in observations {
                let Some(path) = observation.get("path").and_then(Value::as_str) else {
                    continue;
                };
                if key.contains("uimodel") {
                    return Some(path.to_string());
                }
                fallback.get_or_insert_with(|| path.to_string());
            }
            fallback
        });

    CatalogComponent {
        mesh_name: value
            .get("mesh_name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        bones: string_list(value.get("bone_names")),
        materials: string_list(value.get("materials")),
        texture_properties,
        path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_components_and_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("chr_test.json");
        let mut file = fs::File::create(&path).expect("catalog file");
        file.write_all(
            br#"{"kind":"bem-character-catalog","components":{"0":{"mesh_name":"m","bone_names":["b0"],"materials":["M"],"material_texture_properties":[[{"property":"_BaseMap","name":"T"}]],"evidence":{"observations":{"chr_test_uimodel":{"path":"chr_test_uimodel/Mesh/m"}}}}}}"#,
        )
        .expect("write");
        let catalog = Catalog::load(dir.path(), "chr_test").expect("load");
        let component = catalog.component(0).expect("component");
        assert_eq!(component.materials, vec!["M".to_string()]);
        assert_eq!(component.bones, vec!["b0".to_string()]);
        assert_eq!(
            component.path.as_deref(),
            Some("chr_test_uimodel/Mesh/m")
        );
        assert_eq!(component.texture_properties[0][0].property, "_BaseMap");
    }
}
