use crate::{gen, style, thread_rng, Characters, Rain, Rng};
use itertools::izip;
use std::time::Instant;

pub fn update(rain: &mut Rain, group: &Characters) {
    let g = group.as_vec_u32();
    let now = Instant::now();
    for ((time, delay), location, len, ch) in izip!(
        &mut rain.time,
        &mut rain.locations,
        &rain.length,
        &mut rain.charaters
    ) {
        if *time <= now {
            if *location < *len {
                let new = char::from_u32(g[thread_rng().gen_range(0..g.len())]).unwrap_or('#');
                let _ = ch.pop().unwrap_or('%');
                ch.insert(0, new);
            } else {
                let last = ch.pop().unwrap_or('%');
                ch.insert(0, last);
            }
            *time += *delay;
            *location += 1;
        }
    }
}

// pub fn reset<F>(create_color: F, rain: &mut Rain, us: &UserSettings)
// where
//     F: Fn(style::Color, style::Color, u8) -> Vec<style::Color>,
// {
//     let mut rng = thread_rng();
//     let h16 = rain.height;
//     let hsize = rain.height as usize;
//     let now = Instant::now();
//     for i in rain.queue.iter() {
//         if rain.locations[*i] > hsize + rain.length[*i] {
//             rain.charaters[*i] = gen::create_drop_chars(h16, &us.group);
//             rain.locations[*i] = 0;
//             rain.length[*i] = rng.gen_range(4..hsize - 10);
//             rain.colors[*i] = create_color(
//                 us.rain_color.into(),
//                 us.head_color.into(),
//                 rain.length[*i] as u8,
//             );
//             rain.time[*i] = (
//                 now,
//                 Duration::from_millis(rng.gen_range(us.speed.0..us.speed.1)),
//             );
//         }
//     }
// }
