# Train delay logger

> A simple Rust command-line app for logging delayed trains (> 15 minutes) between two stations

## Usage

```bash
$ SERVICE_USERNAME= SERVICE_PASSWORD= train-delay-logger DEP_CODE ARR_CODE START_HOUR END_HOUR YYYY-MM-DD

# Example
# Trains between London Euston and Liverpool Lime Street, from 1pm to 10pm on 1st July 2018
$ SERVICE_USERNAME= SERVICE_PASSWORD= train-delay-logger EUS LIV 13 22 2018-07-01
```

_See the API Authentication section on how to get values for the `SERVICE_USERNAME` and `SERVICE_PASSWORD` environment variables._

## API Authentication

You must provide the `SERVICE_USERNAME` and `SERVICE_PASSWORD` environment variables in order to gain access to the [Open Rail Data Historical Service Performance API](https://wiki.openraildata.com/index.php/HSP).

You can register for an account to get these details via this URL: https://datafeeds.nationalrail.co.uk/#/
