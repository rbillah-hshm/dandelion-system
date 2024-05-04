#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
const SCALE_FACTOR: f32 = 50.0;

const SITUPS_BIAS: f32 = 1.0;
const PUSHUPS_BIAS: f32 = 10.0;
const DISTANCE_BIAS: f32 = 50.0;
fn round_order(number: f32, order: i32) -> f32 {
    (number * f32::powi(10.0, order)).floor() / f32::powi(10.0, order)
}
fn get_first_significant_figure(number: f32) -> f32 {
    number / f32::powi(10.0, number.log10().floor() as i32)
}
fn determine_raw_exp(data: WorkoutData) -> f32 {
    let raw_situps = (data.situps * SCALE_FACTOR as i32) as f32 * f32::powf(1.1, SITUPS_BIAS);
    let raw_pushups = data.pushups * SCALE_FACTOR * f32::powf(1.1, PUSHUPS_BIAS);
    let raw_distance = data.run_distance * SCALE_FACTOR * f32::powf(1.1, DISTANCE_BIAS);
    round_order(raw_situps + raw_pushups + raw_distance, 2)
}
fn determine_level(exp: f32) -> (i32, f32) {
    let level = f32::log((exp / 1000.0) + 1.0, 1.75);
    (
        level.floor() as i32,
        round_order(
            (f32::powf((level + 1.0).floor(), 1.75) - 1.0) * 1000.0
                - ((f32::powf(level, 1.75) - 1.0) * 1000.0),
            2,
        ),
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkoutData {
    situps: i32,
    pushups: f32,
    run_distance: f32, // Miles
}
fn window_conf() -> Conf {
    Conf {
        window_title: "DANDELION_SYSTEM".to_string(),
        window_resizable: false,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() -> io::Result<()> {
    request_new_screen_size(0.5 * 1920.0, 0.5 * 1080.0);
    let mut rank_map = HashMap::new();
    rank_map.insert(0, "E");
    rank_map.insert(10, "D");
    rank_map.insert(20, "C");
    rank_map.insert(30, "B");
    rank_map.insert(40, "A");
    rank_map.insert(50, "S");
    rank_map.insert(100, "N");
    loop {
        clear_background(YELLOW);
        if ((screen_width() != 0.5 * 1920.0) || screen_height() != 0.5 * 1080.0) {
            next_frame().await;
            continue;
        }
        let data_base_path = Path::new("data_base");
        let written_file = File::open(&data_base_path.join("written.json"))?;
        let written_file_reader = BufReader::new(&written_file);
        let mut saved_data = String::new();
        for line in written_file_reader.lines() {
            saved_data.push_str(line.unwrap().trim());
        }
        let deserialized_json = Deserializer::from_str(&saved_data).into_iter::<Value>();
        let mut routine_struct = None;
        for value in deserialized_json {
            let unwrapped_value = value.unwrap();
            let situps = match (unwrapped_value.get("situps")) {
                Some(value) => value.as_i64().unwrap(),
                None => 0,
            } as i32;
            let pushups = match (unwrapped_value.get("pushups")) {
                Some(value) => value.as_f64().unwrap(),
                None => 0.0,
            } as f32;
            let run_distance = match (unwrapped_value.get("run_distance")) {
                Some(value) => value.as_f64().unwrap(),
                None => 0.0,
            } as f32;
            routine_struct = Some(WorkoutData {
                situps,
                pushups,
                run_distance,
            })
        }
        let unwrapped_routine = routine_struct.unwrap();
        let (level, remaining_exp) = determine_level(determine_raw_exp(unwrapped_routine));
        println!("{}, {}", level, remaining_exp);
        let mut rank_association = String::new();
        let ones_level = get_first_significant_figure(level as f32);
        if (level < 10) {
            rank_association.push_str(rank_map.get(&0).unwrap());
        } else if (level < 50) {
            rank_association.push_str(
                rank_map
                    .get(
                        &((ones_level.floor() * f32::powf(10.0, (level as f32).log10().floor()))
                            as i32),
                    )
                    .unwrap(),
            )
        } else if (level < 100) {
            rank_association.push_str(rank_map.get(&50).unwrap());
        } else {
            rank_association.push_str(rank_map.get(&100).unwrap());
        }
        draw_text(
            format!("Level: {}", level).as_str(),
            screen_width() / 2.0,
            screen_height() / 2.0,
            50.0,
            BLACK,
        );
        draw_text(
            format!("{}exp left!", remaining_exp).as_str(),
            screen_width() / 2.0,
            (screen_height() / 2.0) - 50.0,
            50.0,
            BLACK,
        );
        draw_text(
            format!("Rank: {}", rank_association).as_str(),
            screen_width() / 2.0,
            (screen_height() / 2.0) + 50.0,
            50.0,
            BLACK,
        );
        next_frame().await;
    }
}
