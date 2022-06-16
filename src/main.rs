use serde_derive::Serialize;
use std::{env, fs};
use std::collections::HashMap;
use time::{Date, Time, format_description};

const VERSION: &str = "0.03";

#[derive(Debug, Serialize)]
struct Fixture {
    time: String,
    competition: String,
    opponent: String,
    sort_key: Time,
    class: String,
}

#[derive(Debug, Serialize)]
struct Day {
    date: String,
    fixtures: Vec<Fixture>,
    sort_key: Date,
}

#[derive(Debug, Serialize)]
struct Location {
    location: String,
    days: Vec<Day>,
}

fn main() {
    println!("TGFC Fixtures v{}", VERSION);

    let args: Vec<String> = env::args().collect();

    if args.len() == 4 {
        let csv_path = &args[1];
        let template_path = &args[2];
        let output_path = &args[3];

        println!("Loading template '{}'", template_path);
        let template_data = fs::read_to_string(template_path)
            .expect(&format!("Unable to load template {}", template_path));

        println!("Loading csv input '{}'", csv_path);
        let csv_data = fs::read_to_string(csv_path)
            .expect(&format!("Unable to load file {}", csv_path));

        let mut csv = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(csv_data.as_bytes());

        let mut fixture_map: HashMap<String, HashMap<Date, Vec<Fixture>>> = HashMap::new();

        for result in csv.records() {
            let record = result.expect("unexpected record found in csv file");

            if record.len() >= 5 {
                let location = record.get(0).expect("unable to get location");
                let competition = record.get(1).expect("unable to get competition");
                let date = record.get(2).expect("unable to get date");
                let time = record.get(3).expect("unable to get location");
                let opponent = record.get(4).expect("unable to get location");

                if location.is_empty() ||
                   competition.is_empty() ||
                   date.is_empty() ||
                   time.is_empty() ||
                   opponent.is_empty() {
                    continue;
                }

                println!("Found fixture:");
                println!("\tField: {}", location);
                println!("\tComp: {}", competition);
                println!("\tOpponent: {}", opponent);
                println!("\tTime: {} {}", date, time);

                let expected_date_format = format_description::parse("[day]/[month]/[year]").unwrap();
                let date = Date::parse(date, &expected_date_format).expect("unable to parse date");

                let expected_time_format = format_description::parse("[hour repr:12]:[minute]:[second] [period]").unwrap();
                let time = Time::parse(time, &expected_time_format).expect("unable to parse time");
                let wanted_time_format = format_description::parse("[hour repr:12]:[minute] [period]").unwrap();
                let time_string = time.format(&wanted_time_format).expect("unable to format time");

                let opponent_words: Vec<String> = opponent
                    .to_lowercase()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect();

                let opponent_word_0 = opponent_words.get(0).map(|s| s.as_str());
                let opponent_word_1 = opponent_words.get(1).map(|s| s.as_str());

                let prefix_count = match (opponent_word_0, opponent_word_1) {
                    (Some("the"), _) |
                    (Some("mt"), _) |
                    (Some("st"), _) |
                    (Some("north"), _) |
                    (Some("south"), _) |
                    (Some("brisbane"), _) |
                    (Some("western"), _) |
                    (Some("ipswich"), _) => 1,

                    (Some("gold"), Some("coast")) => 2,
                    (Some("sunshine"), Some("coast")) => 2,

                    (Some(_), Some(_)) | (Some(_), None) => 0,

                    (None, _) => {
                        unreachable!("unhandled opponent name");
                    }
                };

                let mut class = opponent_words[0].clone();
                for i in 0 .. prefix_count {
                    class.push_str("_");
                    class.push_str(&opponent_words[i+1]);
                }

                let fixture = Fixture {
                    competition: competition.into(),
                    opponent: opponent.into(),
                    time: time_string.into(),
                    sort_key: time,
                    class,
                };

                let location_slot = fixture_map
                    .entry(location.into())
                    .or_insert(HashMap::new());

                let day_slot = location_slot
                    .entry(date)
                    .or_insert(Vec::new());

                day_slot.push(fixture);
            } else {
                println!("WARN: Unexpected record {:?}", record);
            }
        }

        let mut locations = Vec::new();

        for (location_key, days) in fixture_map {
            let mut location = Location {
                days: Vec::new(),
                location: location_key,
            };

            for (date, mut fixtures) in days {
                fixtures.sort_by_key(|fixture| fixture.sort_key);

                let wanted_date_format = format_description::parse("[weekday] [day] [month]").unwrap();
                let date_string = date.format(&wanted_date_format).expect("unable to format date");

                let day = Day {
                    date: date_string.into(),
                    fixtures,
                    sort_key: date,
                };

                location.days.push(day);
            }

            location.days.sort_by_key(|day| day.sort_key);

            locations.push(location);
        }

        locations.sort_by_key(|location| location.location.clone());

        let mut context = tera::Context::new();
        context.insert("locations", &locations);

        let output = match tera::Tera::one_off(
            &template_data,
            &context,
            true,
        ) {
            Ok(output) => output,
            Err(err) => {
                panic!("{:?}", err);
            }
        };

        fs::write(output_path, output)
            .expect(&format!("Unable to write file {}", output_path));

        println!("Wrote {}", output_path);
    } else {
        println!("USAGE: tgfcfixtures [csv file] [template file] [output file]");
    }
}
