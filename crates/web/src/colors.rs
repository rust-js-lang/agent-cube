pub fn color(value: char) -> (u32, &'static str) {
    match value {
        'U' => (0xf1efdf, "white"),
        'R' => (0xe45e40, "red"),
        'F' => (0x95c975, "green"),
        'D' => (0xe9d35f, "yellow"),
        'L' => (0xf1a84b, "orange"),
        'B' => (0x649cbf, "blue"),
        _ => panic!("invalid sticker color"),
    }
}
