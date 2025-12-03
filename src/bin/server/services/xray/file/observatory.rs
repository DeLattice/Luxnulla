use super::core::XrayFileCore;
use serde_json::{Value, json};

pub trait ObservatoryOps {
    fn add_observatory_ids<T: ToString>(&self, ids: &[T])
    -> Result<(), Box<dyn std::error::Error>>;
    fn remove_observatory_ids<T: ToString>(
        &self,
        ids: &[T],
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn set_observatory_ids<T: ToString>(&self, ids: &[T])
    -> Result<(), Box<dyn std::error::Error>>;
}

impl ObservatoryOps for XrayFileCore {
    fn add_observatory_ids<T: ToString>(
        &self,
        ids: &[T],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if !root["observatory"].is_object() {
                root["observatory"] = json!({});
            }
            if !root["observatory"]["subjectSelector"].is_array() {
                root["observatory"]["subjectSelector"] = json!([]);
            }

            if let Some(arr) = root["observatory"]["subjectSelector"].as_array_mut() {
                for id in ids {
                    let v = json!(id.to_string());
                    if !arr.contains(&v) {
                        arr.push(v);
                    }
                }
            }
        })
    }

    fn remove_observatory_ids<T: ToString>(
        &self,
        ids: &[T],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if let Some(arr) = root["observatory"]["subjectSelector"].as_array_mut() {
                let targets: Vec<String> = ids.iter().map(|x| x.to_string()).collect();
                arr.retain(|x| !targets.contains(&x.as_str().unwrap_or("").to_string()));
            }
        })
    }

    fn set_observatory_ids<T: ToString>(
        &self,
        ids: &[T],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if !root["observatory"].is_object() {
                root["observatory"] = json!({});
            }
            root["observatory"]["subjectSelector"] =
                ids.iter().map(|x| json!(x.to_string())).collect();
        })
    }
}
