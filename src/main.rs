#[macro_use]
extern crate clap;

use std::{fs, path::PathBuf};

use id3::{Tag, Timestamp, Version, frame::{Comment, Picture, PictureType}};
use bytes::{BufMut, BytesMut};
use clap::Arg;
use colored::Colorize;
use futures_util::StreamExt;
use linya::{Bar, Progress};
use scraper::{html::Select, Html, Selector};
use serde_json::Value;
use chrono::prelude::*;

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

#[tokio::main]
async fn main() {
    let app = app_from_crate!()
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .takes_value(true)
                .required(true)
                .help("Download from bandcamp URL"),
        )
        .get_matches();

    let url = match app.value_of("url") {
        Some(data) => data,
        None => panic!(""),
    };

    let plaintext = reqwest::get(url).await.unwrap().text().await.unwrap();

    let html = Html::parse_document(&plaintext);
    let json_selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let json_text = html.select(&json_selector);
    let song_list = parse_album(json_text);

    download_songs(song_list).await;
}

fn parse_album(json_text: Select) -> Vec<Song> {
    let mut song_list: Vec<Song> = Vec::new();

    let json = json_text.into_iter().nth(0).unwrap();
    let json: Value = serde_json::from_str(&json.inner_html()).unwrap();

    let album = json["name"].to_string().replace("\"", "");
    let image_url = json["image"].to_string().replace("\"", "");
    let site_url = json["@id"].to_string().replace("\"", "");
    let artist = json["publisher"]["name"].to_string().replace("\"", "");
    let tracks = json["track"]["itemListElement"].as_array().unwrap();
    let release_date = json["datePublished"].to_string().replace("\"", "");
    let description = remove_first_and_last(json["description"].to_string());

    'track_iter: for track in tracks {
        let name = track["item"]["name"].to_string().replace("\"", "");
        let track_num = track["position"].as_u64().unwrap() as usize;

        let mut has_audio_preview = false;

        for property in track["item"]["additionalProperty"].as_array().unwrap() {
            if property["name"].to_string() == "\"file_mp3-128\"" {
                let audio_url = property["value"].to_string().replace("\"", "");
                has_audio_preview = true;

                for song in song_list.clone() {
                    if song.track_num == track_num {
                        continue 'track_iter;
                    }
                }

                let song = Song {
                    album: album.clone(),
                    artist: artist.clone(),
                    track_num,
                    name: name.clone(),
                    audio_url: Some(audio_url),
                    image_url: image_url.clone(),
                    site_url: site_url.clone(),
                    release_date: release_date.clone(),
                    description: description.clone(),
                };

                song_list.push(song);
            }
        }

        match has_audio_preview {
            true => (),
            false => {
                let song = Song {
                    album: album.clone(),
                    artist: artist.clone(),
                    track_num,
                    name: name.clone(),
                    audio_url: None,
                    image_url: image_url.clone(),
                    site_url: site_url.clone(),
                    release_date: release_date.clone(),
                    description: description.clone(),
                };

                song_list.push(song);
            }
        }
    }

    return song_list;
}

fn remove_first_and_last(value: String) -> String {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    return chars.as_str().to_string();
}

fn nice_error(message: &str) {
    eprintln!("{} {}", "error:".red().bold(), message);
    eprintln!("\nUSAGE:\n    bcdl --url <url>\n");
    eprintln!("For more information try {}", "--help".green());
}

async fn download_songs(song_list: Vec<Song>) {
    for song in song_list {
        let audio_url = match song.clone().audio_url {
            Some(data) => data,
            None => {
                eprintln!(
                    "{} Audio URL not found for \"{}\"",
                    "Error:".red().bold(),
                    song.clone().name
                );
                continue;
            }
        };

        let request = reqwest::get(audio_url).await.unwrap();

        let content_length = request.content_length().unwrap() as usize;

        let mut progress = Progress::new();
        let bar: Bar = progress.bar(
            content_length,
            &format!(
                "{} Track {} - {}",
                "Downloading".green().bold(),
                song.track_num,
                song.name
            ),
        );

        let mut stream = request.bytes_stream();

        let mut num_bytes: usize = 0;
        let mut buf = BytesMut::with_capacity(content_length);

        while let Some(item) = stream.next().await {
            let item = item.unwrap();
            buf.put(item.clone());

            let amt = item.len();
            num_bytes += amt;

            progress.set_and_draw(&bar, num_bytes);
        }

        let mut music_path = std::env::current_dir().unwrap();
        music_path.push(song.clone().artist);
        music_path.push(song.clone().album);

        if !music_path.exists() {
            fs::create_dir_all(&music_path).unwrap();
        }

        music_path.push(format!("{}.mp3", song.name));

        fstream::write(&music_path, buf, false).unwrap();

        write_music_tags(music_path, song).await;
    }
}

async fn write_music_tags(music_path: PathBuf, song: Song) {
    let mut tag = Tag::new();
    tag.set_album(song.album.to_string());
    tag.set_artist(song.artist.to_string());
    tag.set_title(song.name.to_string());
    tag.set_track(song.track_num as u32);
    
    let date = DateTime::parse_from_rfc2822(&song.release_date).unwrap();
    let timestamp = Timestamp {
        year: date.year(),
        day: Some(date.day() as u8),
        month: Some(date.month() as u8),
        hour: Some(date.hour() as u8),
        minute: Some(date.minute() as u8),
        second: Some(date.second() as u8),
    };

    tag.set_date_released(timestamp);
    tag.set_date_recorded(timestamp);

    let year = date.year();
    tag.set_year(year);

    tag.add_comment(Comment { lang: "US".to_string(), description: "Site".to_string(), text: song.site_url.to_string() });
    tag.add_comment(Comment { lang: "US".to_string(), description: "Description".to_string(), text: song.description.to_string() });


    let picture_data = reqwest::get(song.image_url.to_string())
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();

    tag.add_picture(Picture {
        mime_type: "image/jpeg".to_string(),
        picture_type: PictureType::Other,
        description: "album art".to_string(),
        data: picture_data,
    });

    tag.write_to_path(music_path, Version::Id3v24).unwrap();    
}