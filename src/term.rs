use crate::{cursor, queue, style, terminal, Direction, Rain, Result, Stdout};

pub fn clear(w: &mut Stdout) -> Result<()> {
    queue!(w, terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

fn rotate_screen(rain: &Rain) -> Vec<Vec<char>> {
    (0..rain.charaters[0].len())
        .map(|x| {
            (0..rain.charaters.len())
                .map(|y| rain.charaters[y][x])
                .collect::<Vec<char>>()
        })
        .collect::<Vec<Vec<char>>>()
}

fn make_printable(rain: &Vec<Vec<char>>) -> String {
    rain.iter()
        .enumerate()
        .map(|(_y, line)| {
            format!(
                "{}\r\n",
                line.iter()
                    .enumerate()
                    .map(|(_x, ch)| format!("{}", ch))
                    .collect::<String>()
            )
        })
        .collect::<String>()
}

pub fn draw(w: &mut Stdout, rain: &Rain, _spacing: u16, _direction: &Direction) -> Result<()> {
    let rotated_screen = rotate_screen(&rain);
    let printable_screen = make_printable(&rotated_screen);
    queue!(w, cursor::MoveTo(0, 0), style::Print(printable_screen),)?;
    Ok(())
}

// // TODO: Clean this crap up
// // Draw takes rain data and places it on screen.
// pub fn draw(w: &mut Stdout, rain: &Rain, spacing: u16, direction: &Direction) -> Result<()> {
//     // NOTE: Maybe move this into its own functions to be generated at startup
//     // to lessen the amount of branching done.
//     // Further investigation into the assembly code to see if this is worth it.
//     use Direction::*;
//     // Since we do not keep track of the x and y value of the rain we need to swap
//     // values depending on desired direction.
//     let move_to = match direction {
//         Down => |x: u16, y: u16, _: u16| cursor::MoveTo(x, y),
//         Up => |x: u16, y: u16, offest: u16| cursor::MoveTo(x, offest - y),
//         Right => |x: u16, y: u16, _: u16| cursor::MoveTo(y, x),
//         Left => |x: u16, y: u16, offest: u16| cursor::MoveTo(offest - y, x),
//     };
//     // By subtracting height - location you get opposite location on screen.
//     let offset = match direction {
//         Down | Right => 0,
//         Up | Left => rain.height,
//     };
//
//     // -------------------------------------
//
//     let (mut chr, mut col, mut len, mut clr);
//     let height = rain.height as usize;
//     for row in rain.queue.iter() {
//         // character
//         chr = &rain.charaters[*row];
//         // location
//         col = &rain.locations[*row];
//         // length
//         len = &rain.length[*row];
//         // color
//         clr = &rain.colors[*row];
//
//         let start = col.saturating_sub(*len).clamp(0, chr.len());
//         let end = (col + 1).clamp(1, chr.len());
//         let slice = chr[start..end].iter();
//
//         let cstart = if col > len {
//             clr.len().saturating_sub(slice.len())
//         } else {
//             0
//         };
//
//         let color = &clr[cstart..];
//
//         for (y, (ch, _c)) in slice.rev().zip(color.iter().copied()).enumerate() {
//             queue!(
//                 w,
//                 move_to(
//                     *row as u16 * spacing,
//                     (*col.min(&height) - y) as u16,
//                     offset
//                 ),
//                 style::SetForegroundColor(_c),
//                 style::Print(ch),
//             )?;
//         }
//         // This Deletes old tail character of rain.
//         if col >= len {
//             queue!(
//                 w,
//                 move_to(
//                     *row as u16 * spacing,
//                     col.saturating_sub(*len) as u16,
//                     offset
//                 ),
//                 style::Print(" ".repeat(spacing as usize)),
//             )?;
//         }
//     }
//     Ok(())
// }
