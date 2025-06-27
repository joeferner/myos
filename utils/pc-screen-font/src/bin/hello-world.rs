use pc_screen_font::{Font, FontData, include_font_data};

include_font_data!(DEFAULT_8X16, "Tamsyn8x16b.psf");

fn main() {
    let font = Font::new(DEFAULT_8X16);

    let message = "Hello World!";

    let ch_width = font.width + 1;
    let width = ch_width * message.len();
    let mut arr = vec![];
    for _i in 0..width * font.height {
        arr.push(false);
    }

    let mut x_offset: usize = 0;
    for ch in message.chars() {
        font.render_char(ch, |x, y, v| {
            arr[y * width + x + x_offset] = v;
        });
        x_offset += ch_width;
    }

    for y in 0..font.height {
        for x in 0..width {
            let ch = if arr[y * width + x] { "*" } else { " " };
            print!("{}", ch);
        }
        println!("");
    }
}
