use crate::common::{
    fetchers,
    parsers::{self, proxy_config::ProxyConfig},
};

pub async fn get_configs(url: &str) -> Result<Vec<ProxyConfig>, Box<dyn std::error::Error>> {
    let body = match fetchers::config::fetch(url).await {
        Ok(body) => body,
        Err(e) => {
            eprintln!("Error fetching config from {}: {}", url, e);
            return Err(e);
        }
    };

    let raw_subs = match parsers::proxy_config::decode_config_from_base64(body.as_str()) {
        Ok(subs) => subs,
        Err(e) => {
            eprintln!("Error decoding config: {}", e);
            return Err(e);
        }
    };

    let subs = match parsers::proxy_config::work(raw_subs.as_str()) {
        Ok(subs) => subs,
        Err(e) => {
            eprintln!("Error processing config: {:?}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error processing config",
            )));
        }
    };

    Ok(subs)
}
