use super::core::XrayFileCore;
use serde_json::{Value, json};

pub trait RoutingOps {
    fn add_balancer_ids<T: ToString>(&self, ids: &[T]) -> Result<(), Box<dyn std::error::Error>>;
    fn remove_balancer_ids<T: ToString>(&self, ids: &[T])
    -> Result<(), Box<dyn std::error::Error>>;
    fn set_balancer_ids<T: ToString>(&self, ids: &[T]) -> Result<(), Box<dyn std::error::Error>>;
}

impl RoutingOps for XrayFileCore {
    fn add_balancer_ids<T: ToString>(&self, ids: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if let Some(balancers) = root["routing"]["balancers"].as_array_mut() {
                for balancer in balancers {
                    if !balancer["selector"].is_array() {
                        balancer["selector"] = json!([]);
                    }
                    if let Some(arr) = balancer["selector"].as_array_mut() {
                        for id in ids {
                            let v = json!(id.to_string());
                            if !arr.contains(&v) {
                                arr.push(v);
                            }
                        }
                    }
                }
            }
        })
    }

    fn remove_balancer_ids<T: ToString>(
        &self,
        ids: &[T],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if let Some(balancers) = root["routing"]["balancers"].as_array_mut() {
                let targets: Vec<String> = ids.iter().map(|x| x.to_string()).collect();
                for balancer in balancers {
                    if let Some(arr) = balancer["selector"].as_array_mut() {
                        arr.retain(|x| !targets.contains(&x.as_str().unwrap_or("").to_string()));
                    }
                }
            }
        })
    }

    fn set_balancer_ids<T: ToString>(&self, ids: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            if let Some(balancers) = root["routing"]["balancers"].as_array_mut() {
                let new_selector: Value = ids.iter().map(|x| json!(x.to_string())).collect();
                for balancer in balancers {
                    balancer["selector"] = new_selector.clone();
                }
            }
        })
    }
}
