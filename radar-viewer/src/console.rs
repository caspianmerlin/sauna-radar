use macroquad::{texture::Image, color::GREEN, ui::Skin};





#[derive(Debug)]
pub struct Console {
    input_txt: String,
    editbox_skin: Skin,
}
impl Console {
    pub fn new() -> Self {
        Self { input_txt: String::new(), editbox_skin: editbox_skin() }
    }
}




fn editbox_skin() -> Skin {
    let editbox_style = macroquad::ui::root_ui()
    .style_builder()
    .background(
        Image::empty()
    )
    .text_color(GREEN).font_size(20)
    .build();
    
    Skin {
        editbox_style,
        ..macroquad::ui::root_ui().default_skin()
    }
}




