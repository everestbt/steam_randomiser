use bytes::Bytes;

pub fn get_game_cover_blocking(app_id: &i32) -> Option<Bytes> {
    let url = "https://shared.steamstatic.com/store_item_assets/steam/apps/".to_owned() + &app_id.to_string() + "/library_600x900_2x.jpg";
    reqwest::blocking::get(url).expect("Failed to load url").bytes().ok()
}
