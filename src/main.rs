mod traefik;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use serde_json::json;
use std::env;
use traefik::check_if_domain_exists;

#[derive(Serialize)]
struct DnsRecord {
    qtype: String,
    qname: String,
    content: String,
    ttl: u32,
}

#[derive(Serialize)]
struct DnsResponse {
    result: Vec<DnsRecord>,
}

#[derive(Clone)]
struct Environment {
    traefik_ip: String,
    traefik_api_port: u16,
    my_zones: Vec<String>,
}

async fn respond_to_a_request(
    environment: &Environment,
    domain: &str,
) -> anyhow::Result<DnsResponse> {
    let mut response = DnsResponse { result: vec![] };

    let exists = check_if_domain_exists(
        &environment.traefik_ip,
        environment.traefik_api_port,
        &domain,
    )
    .await?;

    if exists {
        response.result.push(DnsRecord {
            qtype: "A".into(),
            qname: domain.to_string(),
            content: environment.traefik_ip.clone(),
            ttl: 60,
        });
    }

    Ok(response)
}

fn respond_to_soa_request(environment: &Environment, domain: &str) -> anyhow::Result<DnsResponse> {
    let found = environment.my_zones.iter().any(|zone| zone == domain);

    if found {
        let record = DnsRecord {
            qtype: "SOA".into(),
            qname: domain.to_string(),
            content: format!("remote.local. admin.{}. 1000 900 900 1800 60", domain),
            ttl: 60,
        };

        Ok(DnsResponse {
            result: vec![record],
        })
    } else {
        Ok(DnsResponse { result: vec![] })
    }
}

fn respond_to_ns_request(environment: &Environment, domain: &str) -> anyhow::Result<DnsResponse> {
    let found = environment.my_zones.iter().any(|zone| zone == domain);

    if found {
        let record = DnsRecord {
            qtype: "NS".into(),
            qname: domain.to_string(),
            content: "remote.local.".into(),
            ttl: 60,
        };

        Ok(DnsResponse {
            result: vec![record],
        })
    } else {
        Ok(DnsResponse { result: vec![] })
    }
}

#[get("/dns/lookup/{domain}./{type}")]
async fn lookup(
    environment: web::Data<Environment>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (domain, qtype) = path.into_inner();

    // Only respond to A and ANY requests. We don't support any other.
    if qtype != "A" && qtype != "ANY" {}

    // Ask Traefik.
    let response = match qtype.as_str() {
        "A" | "ANY" => respond_to_a_request(environment.as_ref(), &domain).await,
        "SOA" => respond_to_soa_request(environment.as_ref(), &domain),
        "NS" => respond_to_ns_request(environment.as_ref(), &domain),
        _ => {
            println!("lookup {} {} -> Unhandled request type", qtype, domain);
            Ok(DnsResponse { result: vec![] })
        }
    };

    match response {
        Ok(output) => HttpResponse::Ok().json(output),
        Err(error) => {
            println!("lookup {} {} -> Error {}", qtype, domain, error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/dns/getAllDomainMetadata/{domain}")]
async fn getalldomainmetadata(domain: web::Path<String>) -> impl Responder {
    // No idea what this is for honestly.
    web::Json(
        json!({ "result": { "id": 1, "zone": domain.as_ref(), "kind": "NATIVE", "serial": 11 } }),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let traefik_ip = env::var("TRAEFIK_IP").expect("TRAEFIK_IP environment variable is not set");
    let my_zones = env::var("MY_ZONES")
        .expect("MY_ZONES environment variable is not set")
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let traefik_api_port: u16 = env::var("TRAEFIK_API_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("TRAEFIK_API_PORT must be number between 0 and 65534");
    let listen = env::var("LISTEN").unwrap_or_else(|_| "127.0.0.1:8787".into());

    println!("Hello from traefik to powerdns bridge! https://github.com/Papierkorb/traefik-powerdns-bridge");
    println!("  $TRAEFIK_IP: {}", traefik_ip);
    println!("  $TRAEFIK_API_PORT: {}", traefik_api_port);
    println!("  $LISTEN: {}", listen);
    println!("  $MY_ZONES: {}", my_zones.join(","));

    let environment = Environment {
        traefik_ip,
        traefik_api_port,
        my_zones,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(environment.clone()))
            .service(lookup)
            .service(getalldomainmetadata)
    })
    .bind(&listen)?
    .run()
    .await
}
