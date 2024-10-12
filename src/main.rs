use std::net::SocketAddr;

use std::fs;
use std::path::Path;
use axum::Router;
use clap::Parser;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    action: String,
}   

fn main() {
    println!("Hello, world!");

    // parse the command line arguments
    let args = Args::parse();

    if args.action == "generate" {
        gen_site();
    } else if args.action == "serve" {
        serve_site();
    }
}

fn gen_site() {
    let content_dir = Path::new("content");
    let markdown_files = read_markdown_files(content_dir);

    println!("Found {} markdown files", markdown_files.len());

    let public_dir = Path::new("public");
    
    fs::create_dir_all(public_dir).expect("Failed to create public directory");

    for markdown_file in markdown_files.iter() {
        let html = markdown::to_html(markdown_file);

        let html_filename = format!("{}.html", markdown_file);
        let html_path = public_dir.join(html_filename);

        fs::write(html_path, html).expect("Failed to write HTML file");
    }

    println!("Generated {} HTML files in the public directory", markdown_files.len());

}

fn read_markdown_files(dir: &Path) -> Vec<String> {
    let mut markdown_files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    markdown_files.extend(read_markdown_files(&path));
                } else if let Some(extension) = path.extension() {
                    if extension == "md" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            markdown_files.push(content);
                        }
                    }
                }
            }
        }
    }
    markdown_files
}

#[tokio::main]
async fn serve_site() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

        serve(using_serve_dir(), 3001).await;
}

fn using_serve_dir() -> Router {
    // serve the file in the "public" directory under `/assets`
    Router::new().nest_service("/", ServeDir::new("public"))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}