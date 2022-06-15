use crate::{gen, style, UserSettings};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Rain {
    pub charaters: Vec<Vec<char>>,
    pub locations: Vec<usize>,
    pub length: Vec<usize>,
    pub colors: Vec<Vec<style::Color>>,
    pub time: Vec<(Instant, Duration)>,
    pub width: u16,
    pub height: u16,
}

impl Rain {
    pub fn new<F>(create_color: F, width: u16, height: u16, us: &UserSettings) -> Self
    where
        F: Fn(style::Color, style::Color, u8) -> Vec<style::Color>,
    {
        let w = (width / us.group.width()) as usize;
        let h = height as usize;
        let charaters = vec![vec![' '; h]; w];
        let locations = vec![0; w];
        let length = gen::lengths(w, h);
        let colors = gen::colors(
            create_color,
            us.head_color,
            w,
            &length,
            us.rain_color.into(),
        );
        let time = gen::times(w, us.speed);
        Self {
            charaters,
            locations,
            length,
            colors,
            time,
            width,
            height,
        }
    }
}
