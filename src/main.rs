use scraper::{Html, Selector};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let plaintext = reqwest::get("https://thefluxcollective.bandcamp.com/album/blue-sky-memories")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let html = Html::parse_document(&plaintext);

    let json_selector = Selector::parse("script[type='application/ld+json']").unwrap();

    let json_text = html.select(&json_selector);

    let mut url_list: Vec<(String, String)> = Vec::new();

    for instance in json_text.into_iter() {
        let json: Value = serde_json::from_str(&instance.inner_html()).unwrap();

        let tracks = json["track"]["itemListElement"].as_array().unwrap();

        for track in tracks {
            for property in track["item"]["additionalProperty"].as_array().unwrap() {
                if property["value"].is_string()
                    && property["value"].to_string() != "\"all_rights_reserved\""
                {
                    url_list.push((property["value"].to_string(), track["item"]["name"].to_string()));
                }
            }
        }
    }

    for (url, name) in url_list {
        let url = url.replace("\"", "");
        let name = name.replace("\"", "");

        println!("downloading {} - {}", name, url);
        let data = reqwest::get(url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec();

        fstream::write(format!("{}.mp3", name), data, false).unwrap();
    }
}
