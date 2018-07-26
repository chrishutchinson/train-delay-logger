extern crate chrono;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use self::chrono::{Duration, NaiveTime};
use self::reqwest::header::{Authorization, Basic, ContentType};
use self::reqwest::Result;
use std::collections::HashMap;
use std::env;
use std::io::Write;

#[derive(Deserialize, Debug)]
pub struct TrainLocation {
  pub location: String,
  pub gbtt_ptd: String,
  pub gbtt_pta: String,
  pub actual_td: String,
  pub actual_ta: String,
  pub late_canc_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct TrainDetails {
  pub date_of_service: String,
  pub toc_code: String,
  pub rid: String,
  pub locations: Vec<TrainLocation>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Train {
  service_attributes_details: TrainDetails,
}

impl Train {
  pub fn new(rid: &str) -> Result<Train> {
    let username = match env::var("SERVICE_USERNAME") {
      Ok(u) => u,
      Err(_) => {
        writeln!(
          ::std::io::stderr(),
          "Please provide the SERVICE_USERNAME environment variable"
        ).unwrap();
        ::std::process::exit(1);
      }
    };

    let password = match env::var("SERVICE_PASSWORD") {
      Ok(u) => u,
      Err(_) => {
        writeln!(
          ::std::io::stderr(),
          "Please provide the SERVICE_PASSWORD environment variable"
        ).unwrap();
        ::std::process::exit(1);
      }
    };

    let mut map = HashMap::new();
    map.insert("rid", &rid);

    let client = reqwest::Client::new();
    let mut res = client
      .post("https://hsp-prod.rockshore.net/api/v1/serviceDetails")
      .header(ContentType::json())
      .header(Authorization(Basic {
        username,
        password: Some(password),
      }))
      .json(&map)
      .send()?;

    res.json()
  }

  pub fn get_departure_details(&self) -> Option<&TrainLocation> {
    self.service_attributes_details.locations.iter().nth(0)
  }

  pub fn get_destination_details(&self, destination: &String) -> Option<&TrainLocation> {
    self
      .service_attributes_details
      .locations
      .iter()
      .find(|location| location.location == destination.to_string())
  }

  pub fn get_total_delay(&self, destination: &String) -> Option<Duration> {
    let destination_details = match self.get_destination_details(&destination) {
      Some(d) => d,
      _ => return None,
    };

    let actual_arrival_time = NaiveTime::parse_from_str(&destination_details.actual_ta, "%H%M");

    let scheduled_arrival_time = NaiveTime::parse_from_str(&destination_details.gbtt_pta, "%H%M");

    match (actual_arrival_time, scheduled_arrival_time) {
      (Ok(a), Ok(b)) => Some(NaiveTime::signed_duration_since(a, b)),
      _ => None,
    }
  }

  pub fn was_delayed_on_arrival(&self, minutes: i64, destination: &String) -> bool {
    match self.get_total_delay(&destination) {
      Some(delay) => delay >= Duration::minutes(minutes),
      None => false,
    }
  }
}
