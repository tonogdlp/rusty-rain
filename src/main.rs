use clap::{crate_description, crate_name, crate_version, Arg, Command};
use crossterm::{
    cursor, event, execute, queue,
    style::{self, Color, ContentStyle, StyledContent},
    terminal, Result,
};
use ezemoji::*;
use rand::{thread_rng, Rng};
// use unicode_width::UnicodeWidthChar;

// use std::fmt;
use std::io::{stdout, Stdout, Write};
use std::time::{Duration, Instant};

const MAXSPEED: u64 = 40;
const MINSPEED: u64 = 200;
const AUTHOR: &str = "
▞▀▖       ▌        ▞▀▖▞▀▖▞▀▖▛▀▘
▌  ▞▀▖▌  ▌▛▀▖▞▀▖▌ ▌▚▄▘▙▄  ▗▘▙▄
▌ ▖▌ ▌▐▐▐ ▌ ▌▌ ▌▚▄▌▌ ▌▌ ▌▗▘ ▖ ▌
▝▀ ▝▀  ▘▘ ▀▀ ▝▀ ▗▄▘▝▀ ▝▀ ▀▀▘▝▀
Email: cowboy8625@protonmail.com
";

fn main() -> Result<()> {
    let mut stdout = stdout();
    let user_settings = cargs();
    let (width, height) = match user_settings.direction {
        Direction::Left | Direction::Right => {
            let (w, h) = terminal::size()?;
            (h, w)
        }
        Direction::Up | Direction::Down => terminal::size()?,
    };
    let (width, height) = (width, height - 1);

    let create_color = color_function(user_settings.shading);

    let mut rain = Rain::new(create_color, width, height, &user_settings);
    let mut is_running = true;

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    while is_running {
        user_input(
            &mut stdout,
            &mut rain,
            &user_settings,
            create_color,
            &mut is_running,
        )?;
        draw(
            &mut stdout,
            &rain,
            user_settings.group.width(),
            &user_settings.direction,
        )?;
        stdout.flush()?;
        update(&mut rain, &user_settings);
        // reset(create_color, &mut rain, &user_settings);
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
// User Input

fn user_input(
    stdout: &mut Stdout,
    rain: &mut Rain,
    user_settings: &UserSettings,
    create_color: fn(Color, Color, u8) -> Vec<Color>,
    mode: &mut bool,
) -> Result<()> {
    use event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
    if poll(Duration::from_millis(50))? {
        match read()? {
            Event::Key(keyevent) => {
                if keyevent == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
                    || keyevent == KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
                {
                    *mode = false;
                }
            }
            Event::Resize(w, h) => {
                clear(stdout)?;
                *rain = Rain::new(create_color, w, h, user_settings);
            }
            _ => {}
        }
    }
    Ok(())
}

// Update
fn update(rain: &mut Rain, us: &UserSettings) {
    let mut rng = thread_rng();
    let g = us.group.as_vec_u32();
    let rng_char = || char::from_u32(g[thread_rng().gen_range(0..g.len())]).unwrap_or('#');
    let now = Instant::now();
    for ((((time, delay), location), len), ch) in rain
        .time
        .iter_mut()
        .zip(&mut rain.locations)
        .zip(&mut rain.length)
        .zip(&mut rain.charaters)
    {
        if *time <= now {
            if *location < rain.height as usize {
                // Remove tail of rain
                let _ = ch.pop().unwrap_or('&');
                ch.insert(*location, rng_char());
            }
            if is_tail_in_screen(location, len, rain.height as usize) {
                // Remove tail of rain
                let loc = location.saturating_sub(*len + 1);
                ch.remove(loc);
                ch.insert(loc, ' ');
            }
            if is_reset(location, len, rain.height as usize) {
                // Reset Line and give a new length
                let now = Instant::now();
                let (slowest, fastest) = us.speed;
                *time = now;
                *delay = Duration::from_millis(rng.gen_range(slowest..fastest));
                *len = rng.gen_range(4..(rain.height as usize) - 10);
                *location = 0;
            }
            *time += *delay;
            *location += 1;
        }
    }
}
fn is_tail_in_screen(location: &usize, length: &usize, height: usize) -> bool {
    let tail_loc = location.saturating_sub(*length);
    tail_loc < height && location > length
}

fn is_reset(location: &usize, length: &usize, height: usize) -> bool {
    location.saturating_sub(*length) > height
}

// Rain Struct

#[derive(Debug)]
struct Rain {
    charaters: Vec<Vec<char>>,
    locations: Vec<usize>,
    length: Vec<usize>,
    colors: Vec<Vec<Color>>,
    time: Vec<(Instant, Duration)>,
    width: u16,
    height: u16,
    base_color: Color,
}
impl Rain {
    fn new<F>(create_color: F, width: u16, height: u16, us: &UserSettings) -> Self
    where
        F: Fn(Color, Color, u8) -> Vec<Color>,
    {
        let w = (width / us.group.width()) as usize;
        let h = height as usize;
        // TODO: Maybe we need a enum instead of a char to handle width
        let charaters = vec![vec![' '; h]; w];
        let locations = vec![0; w];
        let length = lengths(w, h);
        let colors = colors(
            create_color,
            us.head_color,
            w,
            &length,
            us.rain_color.into(),
        );
        let time = times(w, us.speed);
        Self {
            charaters,
            locations,
            length,
            colors,
            time,
            width,
            height,
            base_color: Color::Rgb { r: 0, g: 255, b: 0 },
        }
    }
}

/// Generates the color function on startup to remove branching if statements from code.
fn color_function(shading: bool) -> fn(Color, Color, u8) -> Vec<Color> {
    // This Creates a closure off of the args
    // given to the program at start that will crates the colors for the rain
    match shading {
        // Creates shading colors
        true => |bc: Color, head: Color, length: u8| {
            let mut c: Vec<Color> = Vec::with_capacity(length as usize);
            let (mut nr, mut ng, mut nb);
            if let Color::Rgb { r, g, b } = bc {
                for i in 0..length {
                    nr = r / length;
                    ng = g / length;
                    nb = b / length;
                    c.push((nr * i, ng * i, nb * i).into());
                }
                c.push(head);
                c.reverse();
            }
            c
        },
        // creates with out color
        _ => |bc: Color, head: Color, length: u8| {
            let mut c: Vec<Color> = Vec::with_capacity(length as usize);
            c.push(head);
            if let Color::Rgb { r, g, b } = bc {
                for _ in 0..length {
                    c.push((r, g, b).into());
                }
            }
            c
        },
    }
}

// TODO: I feel like slowest and fastest are labeled wrong.........
/// Generates Timing for rain to fall. AKA the speed of the rain fall.
fn times(width: usize, (slowest, fastest): (u64, u64)) -> Vec<(Instant, Duration)> {
    let now = Instant::now();
    let mut rng = thread_rng();
    (0..width)
        .map(|_| (now, Duration::from_millis(rng.gen_range(slowest..fastest))))
        .collect()
}

/// Generates the visable length of each column.
fn lengths(width: usize, height: usize) -> Vec<usize> {
    let mut rng = thread_rng();
    (0..width).map(|_| rng.gen_range(4..height - 10)).collect()
}

/// Uses Generates function to create all the color of the Rain/CharacterSheet.
fn colors<F: Fn(Color, Color, u8) -> Vec<Color>>(
    create_color: F,
    head: (u8, u8, u8),
    width: usize,
    length: &[usize],
    bc: Color,
) -> Vec<Vec<Color>> {
    let mut colors = Vec::with_capacity(width);
    for l in length.iter() {
        colors.push(create_color(bc, head.into(), *l as u8));
    }
    colors
}

// Direction

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum CharWidth {
    Single,
    Double,
}

impl CharWidth {
    fn width(self) -> u16 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CharacterSheet {
    All(AllEmojis),
    Alphalow(LowerAlpha),
    Alphaup(UpperAlpha),
    Arrow(Arrow),
    Bin(Bin),
    Cards(Cards),
    Clock(Clock),
    Crab(Crab),
    Dominosh(HorizontalDominos),
    Dominosv(VerticalDominos),
    Earth(Earth),
    Emojis(Emojis),
    Jap(Japanese),
    LargeLetters(LargeLetter),
    Moon(Moon),
    Num(Numbers),
    NumberedBalls(NumberedBalls),
    NumberedCubes(NumberedCubes),
    Plants(Plant),
    Smile(Smile),
    Shapes(Shape),
}

impl CharacterSheet {
    fn width(&self) -> u16 {
        match self {
            Self::All(_) => CharWidth::Double.width(),
            Self::Alphalow(_) => CharWidth::Single.width(),
            Self::Alphaup(_) => CharWidth::Single.width(),
            Self::Arrow(_) => CharWidth::Double.width(),
            Self::Bin(_) => CharWidth::Single.width(),
            Self::Cards(_) => CharWidth::Double.width(),
            Self::Clock(_) => CharWidth::Double.width(),
            Self::Crab(_) => CharWidth::Double.width(),
            Self::Dominosh(_) => CharWidth::Double.width(),
            Self::Dominosv(_) => CharWidth::Single.width(),
            Self::Earth(_) => CharWidth::Double.width(),
            Self::Emojis(_) => CharWidth::Double.width(),
            Self::Jap(_) => CharWidth::Single.width(),
            Self::LargeLetters(_) => CharWidth::Double.width(),
            Self::Moon(_) => CharWidth::Double.width(),
            Self::Num(_) => CharWidth::Single.width(),
            Self::NumberedBalls(_) => CharWidth::Double.width(),
            Self::NumberedCubes(_) => CharWidth::Double.width(),
            Self::Plants(_) => CharWidth::Double.width(),
            Self::Smile(_) => CharWidth::Double.width(),
            Self::Shapes(_) => CharWidth::Double.width(),
        }
    }

    fn as_vec_u32(&self) -> Vec<u32> {
        match self {
            Self::All(c) => c.as_vec_u32(),
            Self::Alphalow(c) => c.as_vec_u32(),
            Self::Alphaup(c) => c.as_vec_u32(),
            Self::Arrow(c) => c.as_vec_u32(),
            Self::Bin(c) => c.as_vec_u32(),
            Self::Cards(c) => c.as_vec_u32(),
            Self::Clock(c) => c.as_vec_u32(),
            Self::Crab(c) => c.as_vec_u32(),
            Self::Dominosh(c) => c.as_vec_u32(),
            Self::Dominosv(c) => c.as_vec_u32(),
            Self::Earth(c) => c.as_vec_u32(),
            Self::Emojis(c) => c.as_vec_u32(),
            Self::Jap(c) => c.as_vec_u32(),
            Self::LargeLetters(c) => c.as_vec_u32(),
            Self::Moon(c) => c.as_vec_u32(),
            Self::Num(c) => c.as_vec_u32(),
            Self::NumberedBalls(c) => c.as_vec_u32(),
            Self::NumberedCubes(c) => c.as_vec_u32(),
            Self::Plants(c) => c.as_vec_u32(),
            Self::Smile(c) => c.as_vec_u32(),
            Self::Shapes(c) => c.as_vec_u32(),
        }
    }
}

impl From<ezemoji::AllEmojis> for CharacterSheet {
    fn from(e: ezemoji::AllEmojis) -> Self {
        Self::All(e)
    }
}

impl From<ezemoji::LowerAlpha> for CharacterSheet {
    fn from(e: ezemoji::LowerAlpha) -> Self {
        Self::Alphalow(e)
    }
}

impl From<ezemoji::UpperAlpha> for CharacterSheet {
    fn from(e: ezemoji::UpperAlpha) -> Self {
        Self::Alphaup(e)
    }
}

impl From<ezemoji::Arrow> for CharacterSheet {
    fn from(e: ezemoji::Arrow) -> Self {
        Self::Arrow(e)
    }
}

impl From<ezemoji::Bin> for CharacterSheet {
    fn from(e: ezemoji::Bin) -> Self {
        Self::Bin(e)
    }
}

impl From<ezemoji::Cards> for CharacterSheet {
    fn from(e: ezemoji::Cards) -> Self {
        Self::Cards(e)
    }
}

impl From<ezemoji::Clock> for CharacterSheet {
    fn from(e: ezemoji::Clock) -> Self {
        Self::Clock(e)
    }
}

impl From<ezemoji::Crab> for CharacterSheet {
    fn from(e: ezemoji::Crab) -> Self {
        Self::Crab(e)
    }
}

impl From<ezemoji::HorizontalDominos> for CharacterSheet {
    fn from(e: ezemoji::HorizontalDominos) -> Self {
        Self::Dominosh(e)
    }
}

impl From<ezemoji::VerticalDominos> for CharacterSheet {
    fn from(e: ezemoji::VerticalDominos) -> Self {
        Self::Dominosv(e)
    }
}

impl From<ezemoji::Earth> for CharacterSheet {
    fn from(e: ezemoji::Earth) -> Self {
        Self::Earth(e)
    }
}

impl From<ezemoji::Emojis> for CharacterSheet {
    fn from(e: ezemoji::Emojis) -> Self {
        Self::Emojis(e)
    }
}

impl From<ezemoji::Japanese> for CharacterSheet {
    fn from(e: ezemoji::Japanese) -> Self {
        Self::Jap(e)
    }
}

impl From<ezemoji::LargeLetter> for CharacterSheet {
    fn from(e: ezemoji::LargeLetter) -> Self {
        Self::LargeLetters(e)
    }
}

impl From<ezemoji::Moon> for CharacterSheet {
    fn from(e: ezemoji::Moon) -> Self {
        Self::Moon(e)
    }
}

impl From<ezemoji::Numbers> for CharacterSheet {
    fn from(e: ezemoji::Numbers) -> Self {
        Self::Num(e)
    }
}

impl From<ezemoji::NumberedBalls> for CharacterSheet {
    fn from(e: ezemoji::NumberedBalls) -> Self {
        Self::NumberedBalls(e)
    }
}

impl From<ezemoji::NumberedCubes> for CharacterSheet {
    fn from(e: ezemoji::NumberedCubes) -> Self {
        Self::NumberedCubes(e)
    }
}

impl From<ezemoji::Plant> for CharacterSheet {
    fn from(e: ezemoji::Plant) -> Self {
        Self::Plants(e)
    }
}

impl From<ezemoji::Smile> for CharacterSheet {
    fn from(e: ezemoji::Smile) -> Self {
        Self::Smile(e)
    }
}

impl From<ezemoji::Shape> for CharacterSheet {
    fn from(e: ezemoji::Shape) -> Self {
        Self::Shapes(e)
    }
}

// Terminal IO

fn clear(w: &mut Stdout) -> Result<()> {
    queue!(w, terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

fn add_color(rain: &Rain) -> Vec<Vec<StyledContent<char>>> {
    assert_eq!(rain.charaters.len(), rain.length.len());
    rain.charaters
        .iter()
        .zip(&rain.colors)
        .zip(&rain.length)
        .map(|((line, colors), _len)| {
            let mut l = 0;
            // assert_eq!(colors.len(), l);
            line.iter()
                .map(|c| {
                    StyledContent::new(
                        {
                            let mut cs = ContentStyle::new();
                            cs.foreground_color = if c == &' ' {
                                None
                            } else {
                                let color = colors.get(l).map(Clone::clone);
                                l = l.saturating_add(1);
                                color
                            };
                            cs
                        },
                        *c,
                    )
                })
                .collect::<Vec<StyledContent<char>>>()
        })
        .collect::<Vec<Vec<StyledContent<char>>>>()
}

fn rotate_screen(screen: &Vec<Vec<StyledContent<char>>>) -> Vec<Vec<StyledContent<char>>> {
    let w = screen.len();
    let h = screen[0].len();
    (0..h)
        .map(|i| {
            (0..w)
                // .map(|j| screen[j][h - i - 1])
                .map(|j| screen[w - j - 1][i])
                .collect::<Vec<StyledContent<char>>>()
        })
        .collect::<Vec<Vec<StyledContent<char>>>>()
}

fn make_printable(rain: &Vec<Vec<StyledContent<char>>>) -> String {
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

fn draw(w: &mut Stdout, rain: &Rain, _spacing: u16, _direction: &Direction) -> Result<()> {
    let colored_screen = add_color(&rain);
    let rotated_screen = rotate_screen(&colored_screen);
    let printable_screen = make_printable(&rotated_screen);
    queue!(w, cursor::MoveTo(0, 0), style::Print(printable_screen))?;
    Ok(())
}

// User Settings
#[derive(Debug, Clone)]
struct UserSettings {
    rain_color: (u8, u8, u8),
    head_color: (u8, u8, u8),
    group: CharacterSheet,
    shading: bool,
    speed: (u64, u64),
    direction: Direction,
}

impl UserSettings {
    fn new(
        rain_color: (u8, u8, u8),
        head_color: (u8, u8, u8),
        group: CharacterSheet,
        shading: bool,
        speed: (u64, u64),
        direction: Direction,
    ) -> Self {
        Self {
            rain_color,
            head_color,
            group,
            shading,
            speed,
            direction,
        }
    }
}

// Command Line Arguments
fn cargs() -> UserSettings {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(AUTHOR)
        .about(crate_description!())
        .arg(
            Arg::new("color")
                .short('C')
                .long("color")
                .help(
                    "Set color of Rain with color string name or tuple
OPTIONS:
    white,
    red,
    blue,
    green,
    r,g,b
    ",
                )
                .takes_value(true),
        )
        .arg(
            Arg::new("direction")
                .short('d')
                .long("direction")
                .help(
                    "Set the direction of the Rain.
Default is set to down/south
OPTIONS:
    up, north,
    down, south,
    left, west,
    right, east
    ",
                )
                .takes_value(true),
        )
        .arg(
            Arg::new("head")
                .short('H')
                .long("head")
                .help(
                    "Set the color of the first char in Rain.
OPTIONS:
    white,
    red,
    blue,
    green,
    r,g,b
    ",
                )
                .takes_value(true),
        )
        .arg(
            Arg::new("characters")
                .short('c')
                .long("chars")
                .help(
                    "Set what kind of characters are printed as rain.
OPTIONS:
    all            - This shows most of the Character Groups all at once.
    alphalow       - Lower Case Alphabet Characters
    alphaup        - Upper Case Alphabet Characters
    arrow          - Arrow Emojis or Fancy Characters
    bin            - All Ones and Zeros
    cards          - Playing Cards
    clock          - 🕑
    crab           - 🦀
    dominosh       - 🀽
    dominosv       - 🁫
    earth          - 🌎
    emojis         - This is just a bunch of random Emojis
    jap            - Japanese Characters
    large-letters  - Cool Looking Large Letters
    moon           - 🌕
    num            - Good ol fashion Numbers
    numbered-balls - These are like pool balls
    numbered-cubes - These are like the pool balls but just cubes
    plants         - Plants of sorts
    smile          - 😃
    shapes         - Squares and Circles of a few colors
    ",
                )
                .takes_value(true),
        )
        .arg(
            Arg::new("speed")
                .short('S')
                .long("speed")
                .help("Set speed of rain MAX,MIN -S 200,400")
                .takes_value(true),
        )
        .arg(
            Arg::new("shade")
                .short('s')
                .long("shade")
                .help("Set Rain shading to fade or stay constant")
                .takes_value(false),
        )
        .get_matches();

    let color = match matches
        .get_one::<String>("color")
        .unwrap_or(&"green".into())
        .as_str()
    {
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 255, 0),
        "cyan" => (0, 139, 139),
        "blue" => (0, 0, 255),
        a => a.to_string().into_tuple(),
    };

    let head = match matches
        .get_one::<String>("head")
        .unwrap_or(&"white".into())
        .as_str()
    {
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 255, 0),
        "blue" => (0, 0, 255),
        "cyan" => (0, 139, 139),
        a => a.to_string().into_tuple(),
    };

    let group = match matches
        .get_one::<String>("characters")
        .unwrap_or(&"bin".into())
        .as_str()
    {
        "all" => AllEmojis.into(),
        "alphalow" => LowerAlpha.into(),
        "alphaup" => UpperAlpha.into(),
        "arrow" => Arrow.into(),
        "bin" => Bin.into(),
        "cards" => Cards.into(),
        "clock" => Clock.into(),
        "crab" => Crab.into(),
        "dominosh" => HorizontalDominos.into(),
        "dominosv" => VerticalDominos.into(),
        "earth" => Earth.into(),
        "emojis" => Emojis.into(),
        "jap" => Japanese.into(),
        "large-letters" => LargeLetter.into(),
        "moon" => Moon.into(),
        "num" => Numbers.into(),
        "numbered-balls" => NumberedBalls.into(),
        "numbered-cubes" => NumberedCubes.into(),
        "plants" => Plant.into(),
        "smile" => Smile.into(),
        "shapes" => Shape.into(),
        _ => Bin.into(),
    };

    let speed = match matches.get_one::<String>("speed") {
        Some(value) => value.to_string().into_tuple(),
        None => (MAXSPEED, MINSPEED),
    };

    let direction = match matches
        .get_one::<String>("direction")
        .unwrap_or(&"down".into())
        .as_str()
    {
        "up" | "north" => Direction::Up,
        "down" | "south" => Direction::Down,
        "left" | "west" => Direction::Left,
        "right" | "east" => Direction::Right,
        e => {
            eprintln!("'{}' is not reconized direction.", e);
            std::process::exit(1);
        }
    };

    let shading = matches.get_one::<bool>("shade").copied().unwrap_or(false);

    UserSettings::new(color, head, group, shading, speed, direction)
}

impl StrTuple<(u64, u64)> for String {
    fn into_tuple(self) -> (u64, u64) {
        let mut nums = Vec::new();
        for num in self.split(',') {
            nums.push(
                num.parse::<u64>()
                    .expect("This is not the correct format, expecting 0,0,0 or name like white"),
            );
        }
        let a = nums[0];
        let b = nums[1];
        (a, b)
    }
}

impl StrTuple<(u8, u8, u8)> for String {
    fn into_tuple(self) -> (u8, u8, u8) {
        let mut nums = Vec::new();
        for num in self.split(',') {
            nums.push(
                num.parse::<u8>()
                    .expect("This is not the correct format, expecting 0,0,0 or name like white"),
            );
        }
        let a = nums[0];
        let b = nums[1];
        let c = nums[2];
        (a, b, c)
    }
}

trait StrTuple<T> {
    fn into_tuple(self) -> T;
}
