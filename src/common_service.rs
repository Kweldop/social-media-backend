use rocket::http::ContentType;

pub fn validate_image(ct: &ContentType) -> bool {
    if ct.top().as_str() != "image" {
        return false;
    }

    matches!(ct.sub().as_str(), "png" | "jpeg" | "jpg" | "webp")
}
