use crate::Video;

pub const DEFAULT_URL: &str = "https://www.googleapis.com/youtube/v3/videos";

pub fn extract_id(url: &str) -> String {
    let split = url.split("=").collect::<Vec<&str>>();
    return split[1].to_string();
}
pub fn get_video_title(v: crate::Video) -> Video {
    let mut v = v;
    let url = DEFAULT_URL.to_string()
        + "?"
        + "&type=video&part=snippet&id="
        + v.id.as_str()
        + "&key=AIzaSyC9keOcdbVqDuQCtZYHzivGbX8l5jwVPaw";

    let json: serde_json::Value = ureq::get(url.as_str()).call().unwrap().into_json().unwrap();

    let title = json["items"][0]["snippet"]["title"].as_str().unwrap();
    v.title = title.to_string();
    v
}

pub fn correct_title(v: crate::Video) -> Video {
    let mut v = v;
    let mut chars: Vec<char> = v.title.chars().collect();

    if chars[0].to_string() == r#"'"# {
        chars.remove(0);
    }

    if chars[chars.len() - 1].to_string() == r#"'"# {
        chars.remove(chars.len() - 1);
    }

    let mut new_name = chars.iter().collect::<String>();

    let mut tmp: Vec<String>;
    if new_name.contains('[') {
        tmp = new_name.split('[').map(|s| s.to_string()).collect();

        let mut s: String = String::new();
        if v.title.contains("m4a") {
            s = "m4a".to_string();
        } else {
            s = tmp[1].split('.').collect::<Vec<&str>>()[1].to_string();
        }
        new_name = tmp[0].clone() + "." + &s;
    }
    if new_name.contains(' ') {
        new_name = new_name.replace(' ', "_");
    }
    let mut chars: Vec<char> = new_name.chars().collect();

    if chars[chars.len() - 5] == '_' {
        chars.remove(chars.len() - 5);
    }

    if chars[chars.len() - 6] == '_' {
        chars.remove(chars.len() - 6);
    }

    let mut new_name = chars.iter().collect::<String>();

    new_name = new_name.trim().to_string();

    if new_name.contains('/') {
        new_name = new_name.replace('/', "");
    }

    if new_name.contains("__") {
        new_name = new_name.replace("__", "_");
    }
    if new_name.contains("___") {
        new_name = new_name.replace("___", "_");
    }
    if new_name.contains("_(Official_Music_Video)") {
        new_name = new_name.replace("_(Official_Music_Video)", "");
    }
    if new_name.contains("BAND-MAID") {
        new_name = new_name.replace("BAND-MAID_", "");
    }
    if new_name.contains("_(Music_Video)") {
        new_name = new_name.replace("_(Music_Video)", "");
    }
    if new_name.contains("_(Music)") {
        new_name = new_name.replace("_(Music)", "");
    }
    if new_name.contains("_(Official_Live_Video)") {
        new_name = new_name.replace("_(Official_Live_Video)", "");
    }
    if new_name.contains("Official_Music_Video") {
        new_name = new_name.replace("Official_Music_Video", "");
    }

    new_name = new_name.replace(" ", "");

    v.title = new_name;
    return v;
}
