#[macro_use]
extern crate clap;

use clap::Arg;
use colored::Colorize;
use serde_json::Value;
use chrono::prelude::*;
use anyhow::{Result, Error};
use scraper::{Html, Selector};
use anyhow::anyhow;
use size_display::Size;

mod parse;
mod download;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(error) => {
            nice_error(error.to_string().as_str());
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<(), Error> {
    let app = app_from_crate!()
    .arg(
        Arg::with_name("url")
            .short("u")
            .long("url")
            .takes_value(true)
            .required(true)
            .help("Download from bandcamp URL"),
    )
    .arg(
        Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Don't actually save the songs"),
    )
    .get_matches();

    let url = app.value_of("url").unwrap();
    let debug = app.is_present("debug");

    let download_type = match url.split("/").nth(3) {
        Some(data) => data,
        None => return Err(anyhow!("Can't find download type from url")),
    };

    println!("{} {} {}", "Downloading".bold(), download_type.bold(), "page...".bold());
    let plaintext = reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let html = Html::parse_document(&plaintext);
    let json_selector = match Selector::parse("script[type='application/ld+json']") {
        Ok(data) => data,
        Err(_) => return Err(anyhow!("Couldn't find script in webpage source.")),
    };

    let json_text = html.select(&json_selector);
    let json = match json_text.into_iter().nth(0) {
        Some(data) => data,
        None => return Err(anyhow!("Couldn't get first json text.")),
    };
    let json: Value = serde_json::from_str(&json.inner_html())?;

    let song_list = match download_type {
        "album" => match parse::parse_album(json) {
            Ok(data) => data,
            Err(error) => return Err(error),
        },
        "track" => match parse::parse_track(json) {
            Ok(data) => data,
            Err(error) => return Err(error),
        },
        &_ => return Err(anyhow!("Unsupported URL type. Not album or track.")),
    };

    let before_downloaded = Utc::now();
    let download_size = download::download_songs(song_list.clone(), debug).await?;
    let time_difference = Utc::now().signed_duration_since(before_downloaded).num_seconds();

    let num_tracks = match song_list.len() {
        1 => format!("1 Track"),
        _ => format!("{} Tracks", song_list.len()),
    };

    println!("{}", format!("Downloaded {} and {:.2} in {} Seconds.", num_tracks, Size(download_size), time_difference).yellow());

    Ok(())
}

fn nice_error(message: &str) {
    eprintln!("{} {}", "error:".red().bold(), message);
    eprintln!("\nUSAGE:\n    bcdl --url <url>\n");
    eprintln!("For more information try {}", "--help".green());
}

#[derive(Debug, Clone)]
pub struct Song {
    pub album: String,
    pub artist: String,
    pub track_num: usize,
    pub name: String,
    pub audio_url: Option<String>,
    pub image_url: String,
    pub site_url: String,
    pub release_date: String,
    pub description: String,
}