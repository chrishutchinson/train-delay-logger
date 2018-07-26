extern crate chrono;
extern crate reqwest;

use self::reqwest::header::{Authorization, Basic, ContentType};
use self::reqwest::Result;
use chrono::{DateTime, FixedOffset};
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
struct ServiceMetricsHeader {
  from_location: String,
  to_location: String,
}

#[derive(Deserialize, Debug)]
pub struct ServiceAttributes {
  pub origin_location: String,
  pub destination_location: String,
  pub gbtt_ptd: String,
  pub gbtt_pta: String,
  pub toc_code: String,
  pub matched_services: String,
  pub rids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ServiceMetric {
  pub tolerance_value: String,
  pub num_not_tolerance: String,
  pub num_tolerance: String,
  pub percent_tolerance: String,
  pub global_tolerance: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Service {
  pub service_attributes_metrics: ServiceAttributes,
  #[serde(rename = "Metrics")]
  pub metrics: Vec<ServiceMetric>,
}

#[derive(Deserialize, Debug)]
pub struct ServiceMetrics {
  header: ServiceMetricsHeader,
  #[serde(rename = "Services")]
  pub services: Vec<Service>,
}

pub struct ServiceQuery {
  departure_station: String,
  destination_station: String,
  departure_hour: i8,
  arrival_hour: i8,
  date: DateTime<FixedOffset>,
}

impl ServiceQuery {
  pub fn new(
    departure_station: String,
    destination_station: String,
    departure_hour: i8,
    arrival_hour: i8,
    date: DateTime<FixedOffset>,
  ) -> ServiceQuery {
    ServiceQuery {
      departure_station,
      destination_station,
      departure_hour,
      arrival_hour,
      date,
    }
  }

  pub fn query_for_services(self) -> Result<ServiceMetrics> {
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

    println!(
      "Querying for trains from {} to {} between the hours of {}:00 and {}:00",
      self.departure_station, self.destination_station, self.departure_hour, self.arrival_hour
    );

    let from_time = format!("{}00", self.departure_hour);
    let to_time = format!("{}00", self.arrival_hour);
    let formatted_date = self.date.format("%Y-%m-%d").to_string();
    let day_type = match process_day_type(&self.date) {
      DayType::Weekday => "WEEKDAY",
      DayType::Saturday => "SATURDAY",
      DayType::Sunday => "SUNDAY",
    };

    let mut map = HashMap::new();
    map.insert("days", day_type);
    map.insert("from_date", &formatted_date);
    map.insert("to_date", &formatted_date);
    map.insert("from_loc", &self.departure_station);
    map.insert("to_loc", &self.destination_station);
    map.insert("from_time", &from_time);
    map.insert("to_time", &to_time);

    let client = reqwest::Client::new();
    let mut res = client
      .post("https://hsp-prod.rockshore.net/api/v1/serviceMetrics")
      .header(ContentType::json())
      .header(Authorization(Basic {
        username,
        password: Some(password),
      }))
      .json(&map)
      .send()?;

    res.json()
  }
}

enum DayType {
  Weekday,
  Saturday,
  Sunday,
}

fn process_day_type(date: &DateTime<FixedOffset>) -> DayType {
  let day_number = isize::from_str(&format!("{}", &date.format("%u"))).unwrap();

  match day_number {
    6 => DayType::Saturday,
    7 => DayType::Sunday,
    _ => DayType::Weekday,
  }
}
