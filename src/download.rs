use anyhow::{Result, Error, anyhow};
use futures_util::StreamExt;
use linya::{Bar, Progress};
use bytes::{BufMut, BytesMut};
use colored::Colorize;
use id3::{Tag, Timestamp, Version, frame::{Comment, Picture, PictureType}};
use chrono::prelude::*;

use crate::Song;
use std::fs;
use std::path::PathBuf;

pub async fn download_songs(song_list: Vec<Song>, is_debug: bool) -> Result<u64, Error> {
    let mut num_total_bytes: u64 = 0;

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

        let request = reqwest::get(audio_url).await?;

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
        let mut num_song_bytes: usize = 0;
        let mut buf = BytesMut::with_capacity(content_length);

        while let Some(item) = stream.next().await {
            let item = item?;
            buf.put(item.clone());

            num_total_bytes += item.len() as u64;
            num_song_bytes += item.len();

            progress.set_and_draw(&bar, num_song_bytes);
        }

        if !is_debug {
            let mut music_path = std::env::current_dir()?;
            music_path.push(song.clone().artist);
            music_path.push(song.clone().album);
    
            if !music_path.exists() {
                fs::create_dir_all(&music_path)?;
            }
    
            music_path.push(format!("{}.mp3", song.name));
    
            match fstream::write(&music_path, buf, false) {
                Some(_) => (),
                None => return Err(anyhow!("Couldn't write file to disk: {}", song.name)),
            }
    
            write_music_tags(music_path, song).await?;
        }
    }

    Ok(num_total_bytes)
}

async fn write_music_tags(music_path: PathBuf, song: Song) -> Result<(), Error> {
    let mut tag = Tag::new();
    tag.set_album(song.album.to_string());
    tag.set_artist(song.artist.to_string());
    tag.set_title(song.name.to_string());
    tag.set_track(song.track_num as u32);
    
    let date = DateTime::parse_from_rfc2822(&song.release_date)?;
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
        .await?
        .bytes()
        .await?
        .to_vec();
    
    tag.add_picture(Picture {
        mime_type: "image/jpeg".to_string(),
        picture_type: PictureType::Other,
        description: "album art".to_string(),
        data: picture_data,
    });

    tag.write_to_path(music_path, Version::Id3v24)?;

    Ok(())
}