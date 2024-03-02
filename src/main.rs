use actix_cors::Cors;
use actix_web::{web, App, http, HttpRequest, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;
use regex::Regex;
use url::Url;

#[derive(Deserialize)]
struct FetchQuery {
    url: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct WebRequest {
    surface: String,
    url: String,
}

fn get_base_url_and_path(full_url: &str) -> Option<(String, String)> {
    match Url::parse(full_url) {
        Ok(parsed_url) => {
            let base_url = format!("{}://{}", parsed_url.scheme(), parsed_url.host_str()?);
            let path = parsed_url.path().trim_end_matches('/').to_owned();
            Some((base_url, path))
        },
        Err(_) => None,
    }
}

fn resolve_relative_url(base_url: &str, current_path: &str, relative_url: &str) -> String {
    if relative_url.starts_with("http://") || relative_url.starts_with("https://") {
        return relative_url.to_owned();
    }

    let mut full_url = base_url.to_owned();

    // Check if the relative URL is a file
    if relative_url.contains('.') {
        if !relative_url.starts_with('/') {
            full_url.push('/');
        }
        full_url.push_str(relative_url);
    } else {
        // For directories, append current_path if not already present
        if !relative_url.starts_with(current_path) {
            if !current_path.is_empty() && !current_path.ends_with('/') {
                full_url.push('/');
            }
            full_url.push_str(current_path);
        }

        if !relative_url.starts_with('/') {
            full_url.push('/');
        }

        full_url.push_str(relative_url);

        // Ensure directories end with a slash
        if !full_url.ends_with('/') {
            full_url.push('/');
        }
    }

    // Remove duplicate slashes after scheme
    let parts: Vec<&str> = full_url.splitn(2, "://").collect();
    if parts.len() == 2 {
        let re = Regex::new(r"/{2,}").unwrap();
        full_url = format!("{}://{}", parts[0], re.replace_all(parts[1], "/"));
    }

    full_url
}

fn update_external_links(html_content: &str, base_url: &str, current_path: &str) -> String {
    let href_regex = Regex::new(r#"href="([^"]+)"#).unwrap();
    href_regex.replace_all(html_content, |caps: &regex::Captures| {
        let relative_url = &caps[1];
        let full_url = resolve_relative_url(base_url, current_path, relative_url);
        format!("href=\"{}\"", full_url)
    }).to_string()
}

fn update_image_sources(html_content: &str, base_url: &str, current_path: &str) -> String {
    let img_regex = Regex::new("(<img [^>]*src=\")([^\"]+)").unwrap();
    img_regex.replace_all(html_content, |caps: &regex::Captures| {
        let relative_url = &caps[2];
        let full_url = resolve_relative_url(base_url, current_path, relative_url);
        format!("{}{}\" style=\"max-width:100%\"", &caps[1], full_url)
    }).to_string()
}

fn update_background_urls(html_content: &str, base_url: &str, current_path: &str) -> String {
    let background_regex = Regex::new(r#"background="([^"]+)"#).unwrap();

    background_regex.replace_all(html_content, |caps: &regex::Captures| {
        let relative_url = &caps[1];

        let full_url = resolve_relative_url(base_url, current_path, relative_url);
        format!("background=\"{}\"", full_url)
    }).to_string()
} 


async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Helloo")
}

async fn capture_url(data: web::Json<WebRequest>) -> impl Responder {
    
    //let client = match Client::new();

    if !data.url.ends_with('/') {
        let mut url = data.url.clone();
        url.push('/');
    }
    HttpResponse::Ok().json(data.into_inner())
}

async fn fetch_url(query: web::Query<FetchQuery>, req: HttpRequest) -> impl Responder {
    if req.method() == http::Method::OPTIONS {
        eprintln!("{}", req.method().to_string());
        return HttpResponse::Ok().finish();
    }

    //let client = Client::new();
    let client = match Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build() {
            Ok(client) => client,
            Err(_) => {
                return HttpResponse::InternalServerError().finish();
            }
    };

    match &query.url {
        Some(url) if !url.is_empty() => {
            let mut url = url.clone();
            //if url.ends_with('/') {
                //url.pop();
            //}
            match client.get(&url).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => {
                        eprintln!("{} {}", text, url);
                        let html = Html::parse_document(&text);
                        let (base_url, current_path) = get_base_url_and_path(&url)
                            .unwrap_or_else(|| (String::new(), String::new()));

                        //let body_selector = Selector::parse("body").unwrap();
                        //let body = html.select(&body_selector).next();
                        let body_html = html.select(&Selector::parse("body").unwrap())
                            .next().map_or(String::new(), |b| b.html());
                        //let body_html = body.map_or(String::new(), |b| b.inner_html());
                        //let base_url = get_base_url(&url).unwrap_or_else(|| String::from(""));
                        let modified_html = update_image_sources(&body_html, &base_url, &current_path);
                        let modified_html = update_external_links(&modified_html, &base_url, &current_path);
                        let modified_html = update_background_urls(&modified_html, &base_url, &current_path);
                        //let mut modified_html = update_image_sources(&body_html, &base_url); 
                        //modified_html = update_external_links(&modified_html, &base_url);               
                        
                        HttpResponse::Ok().content_type("text/html").body(modified_html)
                    }
                    Err(_) => HttpResponse::InternalServerError().finish(),
                },
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        _ => {
            let html_string = "<body>Welcome to the Home Page</body>";
            HttpResponse::Ok().content_type("text/html").body(html_string)
        }
    }
}

async fn fetch_quantum_numbers() -> impl Responder {
    dotenv().ok();

    let client = reqwest::Client::new();
    let header_key = env::var("QUANTUM_KEY").unwrap_or_else(|_| "none".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&header_key).unwrap());

    let response = match client.get("https://api.quantumnumbers.anu.edu.au?length=10&type=uint16")
        .headers(headers)
        .send()
        .await {
            Ok(res) => res,
            Err(_) => return HttpResponse::InternalServerError().json("An error processing request"),
        };

    if response.status().is_success() {
        match response.text().await {
            Ok(text) => {
                match serde_json::from_str::<Value>(&text) {
                    Ok(json_data) => HttpResponse::Ok().json(json_data),
                    Err(_) => HttpResponse::BadRequest().json("JSON parsing error"),
                }
            }
            Err(_) => HttpResponse::InternalServerError().json("Error reading response"),
        }
    } else {
        HttpResponse::InternalServerError().json("Server responded with an error")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_method()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("https://a274-184-170-241-39.ngrok-free.app")
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/fetch", web::get().to(fetch_url))
            .route("/quantum", web::get().to(fetch_quantum_numbers))
            .route("/hello", web::get().to(hello_world))
            .route("/capture", web::post().to(capture_url))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
