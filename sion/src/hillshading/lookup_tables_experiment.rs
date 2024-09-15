use crate::dem_tile::DemTile;
use crate::geo::normalize_angle;
use crate::trig::rad_to_deg;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::io::Write;

pub fn calculate_pq(dem_tile: &DemTile, x: usize, y: usize) -> (i32, i32) {
    let center_index = y * dem_tile.size + x;
    let top_center_index = center_index - dem_tile.size;
    let bottom_center_index = center_index + dem_tile.size;

    let height_tl = dem_tile.height_at_index(top_center_index - 1) as i32;
    let height_bl = dem_tile.height_at_index(bottom_center_index - 1) as i32;
    let height_br = dem_tile.height_at_index(bottom_center_index + 1) as i32;
    let height_tr = dem_tile.height_at_index(top_center_index + 1) as i32;

    let p = (height_br
        + 2 * dem_tile.height_at_index(center_index + 1) as i32
        + height_tr)
        - (height_bl
            + 2 * dem_tile.height_at_index(center_index - 1) as i32
            + height_tl);

    let q = (height_br
        + 2 * dem_tile.height_at_index(bottom_center_index) as i32
        + height_bl)
        - (height_tr
            + 2 * dem_tile.height_at_index(top_center_index) as i32
            + height_tl);
    (p, q)
}

pub fn calculate_slope_and_aspect(p: i32, q: i32) -> (f32, f32) {
    let pf = p as f32;
    let qf = q as f32;

    let max_slope = (pf * pf + qf * qf).sqrt() / 240.;
    let slope = max_slope.atan();
    let aspect = normalize_angle(qf.atan2(pf) - FRAC_PI_2);

    (slope, aspect)
}

pub fn construct_lookup_tables(
    dem: &DemTile,
    aspect_resolution: i32,
) -> (HashMap<i32, i16>, HashMap<i32, i8>) {
    let mut aspect_lookup_table = HashMap::<i32, i8>::new();
    let mut slope_lookup_table = HashMap::<i32, i16>::new();

    for y in 1..dem.size - 1 {
        for x in 1..dem.size - 1 {
            let (p, q) = calculate_pq(dem, x, y);
            let (slope, aspect) = calculate_slope_and_aspect(p, q);

            if q >= 0 && p > 0 {
                let aspect_key = q * aspect_resolution / p;
                let aspect_deg = rad_to_deg(aspect) - 270.;
                aspect_lookup_table.insert(aspect_key, aspect_deg as i8);
            }

            let pi = p / 8;
            let qi = q / 8;
            let slope_key = pi * pi + qi * qi;
            let slope_deg = rad_to_deg(slope);
            slope_lookup_table.insert(slope_key, slope_deg as i16);
        }
    }

    (slope_lookup_table, aspect_lookup_table)
}

fn save_aspect_lookup_table(
    lookup_table: HashMap<i32, i8>,
    resolution: i32,
    max_repeated_values: i32,
) {
    // sort the table by keys
    let mut sorted_keys: Vec<i32> = lookup_table.keys().cloned().collect();
    sorted_keys.sort();

    // make a new dictionary recording the first index of each value
    let mut first_index = HashMap::<i8, i32>::new();
    let mut last_index = HashMap::<i8, i32>::new();
    let mut index = 0;
    let mut last_value = -1;
    for key in &sorted_keys {
        let value = lookup_table.get(key).unwrap();
        if *value != last_value {
            first_index.insert(*value, index);
            last_index.insert(last_value, index - 1);
            last_value = *value;
        }
        index += 1;
    }

    last_index.insert(last_value, index - 1);

    let mut file = std::fs::File::create(format!(
        "target/debug/aspect_lookup_{}.cpp",
        resolution
    ))
    .unwrap();

    writeln!(file, "#include <stdint.h>").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "int16_t aspect_lookup_{}[] = {{", resolution).unwrap();

    let mut comma = "";

    for key in sorted_keys {
        let value = lookup_table.get(&key).unwrap();

        let first_index = first_index.get(value).unwrap();
        let last_index = last_index.get(value).unwrap();
        let occurrences = last_index - first_index + 1;

        if occurrences < max_repeated_values {
            // write hexadecimal byte value
            write!(file, "{}0x{:02x}", comma, value).unwrap();
            comma = ", ";
        }
    }

    writeln!(file, "").unwrap();
    writeln!(file, "}};").unwrap();
    writeln!(file, "").unwrap();

    let mut values_first_index_sorted: Vec<i8> =
        first_index.keys().cloned().collect();
    values_first_index_sorted.sort();
    values_first_index_sorted.reverse();

    for value in values_first_index_sorted {
        let first_index = first_index.get(&value).unwrap();
        let last_index = last_index.get(&value).unwrap();
        let occurrences = last_index - first_index + 1;

        if occurrences >= max_repeated_values {
            // write hexadecimal byte value
            writeln!(
                file,
                "#define ASPECT_{}_{} = {}",
                resolution, value, first_index
            )
            .unwrap();
        }
    }

    file.flush().unwrap();
}

fn save_lookup_table(lookup_table_name: &str, lookup_table: HashMap<i32, i16>) {
    // sort the table by keys
    let mut sorted_keys: Vec<i32> = lookup_table.keys().cloned().collect();
    sorted_keys.sort();

    let mut file = std::fs::File::create(format!(
        "target/debug/{}.txt",
        lookup_table_name
    ))
    .unwrap();

    for key in sorted_keys {
        let value = lookup_table.get(&key).unwrap();
        writeln!(file, "{}: {}", key, value).unwrap();
    }

    file.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dem_tile::DemTile;

    #[test]
    fn calculate_lookup_tables() {
        let dem = DemTile::from_file("tests/data/N46E006.hgt");
        let aspect_resolution = 100;
        let (slope_lookup_table, aspect_lookup_table) =
            construct_lookup_tables(&dem, aspect_resolution);
        save_lookup_table("slope_lookup", slope_lookup_table);

        let max_repeated_values = 25;
        save_aspect_lookup_table(
            aspect_lookup_table,
            aspect_resolution,
            max_repeated_values,
        );
    }
}
