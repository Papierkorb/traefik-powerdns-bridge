type ArrayOfObjects = Vec<serde_json::Map<String, serde_json::Value>>;

fn has_domain(routers: &ArrayOfObjects, domain: &str) -> bool {
    let rule = format!("Host(`{}`)", domain);

    routers.iter().any(|route| {
        route
            .get("rule")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .contains(&rule)
    })
}

pub async fn check_if_domain_exists(
    traefik_ip: &str,
    api_port: u16,
    domain: &str,
) -> anyhow::Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/api/http/routers", traefik_ip, api_port);
    let routers: ArrayOfObjects = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(has_domain(&routers, domain))
}
