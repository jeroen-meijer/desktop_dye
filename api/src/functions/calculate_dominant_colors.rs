use lab::Lab;
use prisma::Rgb;

use crate::config::DominantColorAlgorithm;

pub fn calculate_dominant_colors(
    pixels: &Vec<Rgb<u8>>,
    alg: &DominantColorAlgorithm,
    sample_size: &u8,
) -> Vec<Rgb<u8>> {
    match alg {
        DominantColorAlgorithm::Pigmnts => pigmnts_alg(pixels, *sample_size),
        DominantColorAlgorithm::ColorThief => color_thief_alg(pixels, *sample_size),
    }
}

fn pigmnts_alg(pixels: &Vec<Rgb<u8>>, sample_size: u8) -> Vec<Rgb<u8>> {
    let lab_values = pixels
        .iter()
        .map(|rgb| pigmnts::color::LAB::from_rgb(rgb.red(), rgb.green(), rgb.blue()))
        .collect::<Vec<_>>();

    let colors_res = pigmnts::pigments_pixels(
        &lab_values,
        sample_size,
        pigmnts::weights::resolve_mood(&pigmnts::weights::Mood::Dominant),
        None,
    );

    let mut colors = colors_res
        .into_iter()
        .map(|(color, dominance)| {
            let [r, g, b] = Lab::to_rgb(&Lab {
                l: color.l,
                a: color.a,
                b: color.b,
            });

            (Rgb::new(r, g, b), dominance)
        })
        .collect::<Vec<_>>();

    colors.sort_by(|(_, dominance_a), (_, dominance_b)| {
        dominance_a
            .partial_cmp(dominance_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    colors.into_iter().map(|(rgb, _)| rgb).collect()
}

fn color_thief_alg(pixels: &Vec<Rgb<u8>>, sample_size: u8) -> Vec<Rgb<u8>> {
    let colors_res = color_thief::get_palette(
        &pixels
            .iter()
            .flat_map(|rgb| vec![rgb.red(), rgb.green(), rgb.blue()])
            .collect::<Vec<_>>(),
        color_thief::ColorFormat::Rgb,
        1,
        sample_size,
    );

    colors_res
        .unwrap()
        .into_iter()
        .map(|color| Rgb::new(color.r, color.g, color.b))
        .collect()
}
