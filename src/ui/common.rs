use super::*;

pub fn clip_copy(clip: &mut EguiClipboard, data: &[String]) {
    clip.set_text(&data.join(","));
}

pub fn clip_paste(clip: &mut EguiClipboard, data: &mut [String]) {
    clip.get_text()
        .unwrap_or_default()
        .split(",")
        .chain(std::iter::repeat("0"))
        .map(str::trim)
        .map(String::from)
        .zip(data)
        .for_each(|(value, data)| *data = value);
}
