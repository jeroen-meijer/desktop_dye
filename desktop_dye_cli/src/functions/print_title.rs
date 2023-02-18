use colored::Colorize;
use prisma::{Lerp, Rgb};
use rand::Rng;

pub fn print_title() {
    const PACKAGE_NAME: &str = "DesktopDye";
    let mut possible_colors = vec![
        Rgb::new(152, 31, 172),
        Rgb::new(255, 0, 106),
        Rgb::new(0, 140, 255),
        Rgb::new(255, 140, 0),
    ];
    let mut rng = rand::thread_rng();
    let start_color = possible_colors.remove(rng.gen_range(0..possible_colors.len()));
    let end_color = possible_colors.remove(rng.gen_range(0..possible_colors.len()));

    let chars = PACKAGE_NAME.chars().collect::<Vec<_>>();
    let char_count = chars.len();
    let colors = (0..char_count)
        .map(|i| start_color.lerp(&end_color, i as f64 / char_count as f64))
        .collect::<Vec<_>>();

    let colored_package_name = chars
        .into_iter()
        .zip(colors.into_iter())
        .map(|(c, color)| {
            c.to_string()
                .on_truecolor(color.red(), color.green(), color.blue())
                .to_string()
        })
        .collect::<String>()
        .bold()
        .truecolor(255, 255, 255);

    println!(
        "\n{} v{}\n",
        colored_package_name,
        env!("CARGO_PKG_VERSION")
    );
}
