use anyhow::{Result, Error};
use anyhow::anyhow;
use serde_json::Value;

use crate::Song;

pub fn parse_track(json: Value) -> Result<Vec<Song>, Error> {
    let mut song_list: Vec<Song> = Vec::new();

    let album = remove_first_and_last(json["inAlbum"]["name"].to_string());
    let image_url = remove_first_and_last(json["image"].to_string());
    let site_url = remove_first_and_last(json["@id"].to_string());
    let artist = remove_first_and_last(json["byArtist"]["name"].to_string());
    let release_date = remove_first_and_last(json["datePublished"].to_string());
    let description = remove_first_and_last("".to_string());
    let name = remove_first_and_last(json["name"].to_string()).replace(":", ".");

    let mut audio_url: Option<String> = None;
    let mut track_num: usize = 0;

    for property in json["additionalProperty"].as_array().unwrap() {
        if property["name"].to_string().replace("\"", "") == "tracknum" {
            track_num = match property["value"].as_u64() {
                Some(data) => data,
                None => return Err(anyhow!("tracknum is not f64")),
            } as usize;
        }

        if property["name"].to_string().replace("\"", "") == "file_mp3-128" {
            audio_url = Some(property["value"].to_string().replace("\"", ""));
        }
    }

    song_list.push(Song {
        album,
        artist,
        track_num,
        name,
        audio_url,
        image_url,
        site_url,
        release_date,
        description,
    });

    return Ok(song_list);
}

pub fn parse_album(json: Value) -> Result<Vec<Song>, Error> {
    let mut song_list: Vec<Song> = Vec::new();

    let album = remove_first_and_last(json["name"].to_string());
    let image_url = remove_first_and_last(json["image"].to_string());
    let site_url = remove_first_and_last(json["@id"].to_string());
    let artist = remove_first_and_last(json["byArtist"]["name"].to_string());
    let tracks = json["track"]["itemListElement"].as_array().unwrap();
    let release_date = remove_first_and_last(json["datePublished"].to_string());
    let description = remove_first_and_last(json["description"].to_string());

    'track_iter: for track in tracks {
        let name = remove_first_and_last(track["item"]["name"].to_string()).replace(":", ".");
        let track_num = track["position"].as_u64().unwrap() as usize;

        let mut has_audio_preview = false;

        for property in track["item"]["additionalProperty"].as_array().unwrap() {
            if property["name"].to_string() == "\"file_mp3-128\"" {
                let audio_url = remove_first_and_last(property["value"].to_string());
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

        // Not the cleanest way to do this but cmon I had to.
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

    return Ok(song_list);
}

fn remove_first_and_last(value: String) -> String {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    return chars.as_str().to_string();
}