use serde_json::{Value, json};

pub trait JsonExt {
    fn ensure_array_path(&mut self, path: &[&str]) -> Option<&mut Vec<Value>>;

    fn add_unique_str_to_array(&mut self, val: &str);

    fn remove_str_from_array(&mut self, val: &str);
}

impl JsonExt for Value {
    fn ensure_array_path(&mut self, path: &[&str]) -> Option<&mut Vec<Value>> {
        let mut current = self;
        for key in path {
            if !current.is_object() {
                *current = json!({});
            }

            current = current.as_object_mut()?.entry(*key).or_insert(json!({}));
        }
        if !current.is_array() {
            *current = json!([]);
        }
        current.as_array_mut()
    }

    fn add_unique_str_to_array(&mut self, val: &str) {
        if let Some(arr) = self.as_array_mut() {
            let v = json!(val);
            if !arr.contains(&v) {
                arr.push(v);
            }
        }
    }

    fn remove_str_from_array(&mut self, val: &str) {
        if let Some(arr) = self.as_array_mut() {
            arr.retain(|x| x.as_str() != Some(val));
        }
    }
}

pub trait IdList {
    fn to_ids(&self) -> Vec<String>;
}

impl IdList for &str {
    fn to_ids(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IdList for &[&str] {
    fn to_ids(&self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl IdList for i32 {
    fn to_ids(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl IdList for &[i32] {
    fn to_ids(&self) -> Vec<String> {
        self.iter().map(|n| n.to_string()).collect()
    }
}

impl IdList for Vec<i32> {
    fn to_ids(&self) -> Vec<String> {
        self.iter().map(|n| n.to_string()).collect()
    }
}

impl IdList for &Vec<i32> {
    fn to_ids(&self) -> Vec<String> {
        self.iter().map(|n| n.to_string()).collect()
    }
}

pub fn json_add_ids(arr: &mut Vec<Value>, ids: Vec<String>) {
    for id in ids {
        let v = json!(id);
        if !arr.contains(&v) {
            arr.push(v);
        }
    }
}

pub fn json_remove_ids(arr: &mut Vec<Value>, ids: Vec<String>) {
    arr.retain(|x| !ids.contains(&x.as_str().unwrap_or("").to_string()));
}
