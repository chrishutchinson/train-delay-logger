#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate serde;
extern crate serde_json;

mod service_query;
mod train;

use chrono::DateTime;
use service_query::ServiceQuery;
use std::io::Write;
use std::str::FromStr;
use train::Train;

fn parse_args_into_service_query(args: &Vec<String>) -> ServiceQuery {
    ServiceQuery::new(
        args[0].to_string(),
        args[1].to_string(),
        i8::from_str(&args[2]).expect("error parsing start hour"),
        i8::from_str(&args[3]).expect("error parsing end hour"),
        DateTime::parse_from_str(&format!("{} 00:00:00 +0000", &args[4]), "%Y-%m-%d %T %z")
            .expect("error parsing date"),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 5 {
        writeln!(
            std::io::stderr(),
            "Usage: train-delay DEPARTURE_STATION DESTINATION_STATION DEPARTURE_HOUR ARRIVAL_HOUR"
        ).unwrap();
        std::process::exit(1);
    }

    let service_query = parse_args_into_service_query(&args);

    let service_response = match service_query.query_for_services() {
        Err(_) => {
            writeln!(
                std::io::stderr(),
                "Unable to load data for the given arguments"
            ).unwrap();
            std::process::exit(1);
        }
        Ok(services) => services,
    };

    let trains: Vec<Train> = service_response
        .services
        .into_iter()
        .map(|service| {
            match Train::new(&service
                .service_attributes_metrics
                .rids
                .into_iter()
                .nth(0)
                .expect("No valid RID"))
            {
                Err(_) => {
                    writeln!(std::io::stderr(), "Unable to load data for train service").unwrap();
                    std::process::exit(1);
                }
                Ok(train) => train,
            }
        })
        .collect();

    let delayed_trains = trains
        .into_iter()
        .filter(|train| train.was_delayed_on_arrival(15, &args[1]))
        .collect::<Vec<_>>();

    println!(
        "Found {} delayed trains between {} and {} from {}:00 to {}:00",
        delayed_trains.len(),
        &args[0],
        &args[1],
        &args[2],
        &args[3]
    );

    delayed_trains.into_iter().for_each(|train| {
        let departure_details = train
            .get_departure_details()
            .expect("Unable to get departure details");

        let destination_details = train
            .get_destination_details(&args[1])
            .expect("Unable to get destination details");

        let total_delay = train
            .get_total_delay(&args[1])
            .expect("Unable to get total delay");

        println!(
            "{} from {} to {} was delayed by {} minutes",
            departure_details.gbtt_ptd,
            departure_details.location,
            destination_details.location,
            total_delay.num_minutes()
        );
    });
}
