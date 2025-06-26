mod font;

fn main() {
    let font = font::Font::new(font::DEFAULT_8X16);

    println!("Hello, world!");

    let width = 100;
    let mut arr = [false; 3000];
    let mut x_offset: usize = 0;
    for ch in "hello".chars() {
        font.render_char(ch, |x, y, v| {
            arr[y * width + x + x_offset] = v;
        });
        x_offset += font.width;
    }

    for y in 0..font.height {
        for x in 0..width {
            let ch = if arr[y * width + x] { "*" } else { " " };
            print!("{}", ch);
        }
        println!("");
    }
}
