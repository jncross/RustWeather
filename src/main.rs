use reqwest::Error;
use serde::Deserialize;
use std::io::{self};

// Weather object with parameters matching individual response parameters
#[derive(Deserialize, Debug)]
struct Weather {
    temperature_2m: Option<Vec<f64>>,       // Option type to handle missing data
    temperature_2m_min: Option<Vec<f64>>,   
    temperature_2m_max: Option<Vec<f64>>,   
    time: Option<Vec<String>>,    
    precipitation_probability: Option<Vec<f64>>,
}

// Parameters for the full response depending on call made
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    hourly: Option<Weather>,
    daily: Option<Weather>,
}

// List of cities with their names and coordinates (latitude, longitude)
const CITIES: [(&str, f64, f64, &str); 4] = [
    ("Adelaide, Australia", -34.9285, 138.6007, "Australia/Adelaide"),
    ("Melbourne, Australia", -37.8136, 144.9631, "Australia/Melbourne"),
    ("London, UK", 51.5072, -0.1276, "Europe/London"),
    ("Beijing, China", 39.9042, 116.4074, "Asia/Shanghai"),
];

// Handles user input from the top level menu
fn get_city_choice(input: &str) -> Result<(f64, f64, &str), &'static str> {
    let choice: usize = input.trim().parse().map_err(|_| "Invalid choice")?;
    if choice < 1 || choice > CITIES.len() {
        return Err("Choice out of range");
    }
    Ok((CITIES[choice - 1].1, CITIES[choice - 1].2, CITIES[choice - 1].3))  // Return coordinates and timezone of chosen city
}


// Function to construct the API URL based on user choice (current weather or min/max temps)
fn construct_url(latitude: f64, longitude: f64, timezone: &str, option: &str) -> String {
    match option {
		//Current weather
        "1" => format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,precipitation_probability&timezone={}",
            latitude, longitude, timezone
        ),
		// Weekly weather
        "2" => format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_min,temperature_2m_max&timezone={}",
            latitude, longitude, timezone
        ),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_city_choice_valid() {
        let result = get_city_choice("1").unwrap();
        assert_eq!(result, (-34.9285, 138.6007, "Australia/Adelaide"));
    }

    #[test]
    fn test_get_city_choice_invalid_choice() {
        let result = get_city_choice("5");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_city_choice_invalid_input() {
        let result = get_city_choice("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_construct_url_current_weather() {
        let url = construct_url(-34.9285, 138.6007, "Australia/Adelaide", "1");
        assert_eq!(url, "https://api.open-meteo.com/v1/forecast?latitude=-34.9285&longitude=138.6007&hourly=temperature_2m,precipitation_probability&timezone=Australia/Adelaide");
    }

    #[test]
    fn test_construct_url_invalid_option() {
        let url = construct_url(-34.9285, 138.6007, "Australia/Adelaide", "3");
        assert_eq!(url, "");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Print each city from CITIES constant
    for (i, city) in CITIES.iter().enumerate() {
        println!("{}. {}", i + 1, city.0);
    }
    // Option for custom latitude and longitude input
    println!("{}. Enter custom latitude and longitude", CITIES.len() + 1);

    println!("Enter the number of your choice: ");
    
    // Instantiate input param
    let mut choice = String::new();
    // Get input and set to input param
    io::stdin().read_line(&mut choice).unwrap();

    // Determine if user wants to enter custom latitude and longitude
    let (latitude, longitude, timezone) = if choice.trim() == (CITIES.len() + 1).to_string() {
        println!("Enter latitude: ");
        let mut lat_input = String::new();
        io::stdin().read_line(&mut lat_input).unwrap();
		// validate input
        let latitude: f64 = lat_input.trim().parse().map_err(|_| {
            eprintln!("Invalid latitude");
            std::process::exit(1);
        })?;

        println!("Enter longitude: ");
        let mut long_input = String::new();
        io::stdin().read_line(&mut long_input).unwrap();
        let longitude: f64 = long_input.trim().parse().map_err(|_| {
            eprintln!("Invalid longitude");
            std::process::exit(1);
        })?;

        println!("Enter timezone: ");
        let mut tz_input = String::new();
        io::stdin().read_line(&mut tz_input).unwrap();
		//Timezone in the same IANA format as the const variables e.g. Australia/Melbourne, America/New_York
		//This sanitizes the input to string
        let timezone: &str = Box::leak(tz_input.trim().to_string().into_boxed_str());

        (latitude, longitude, timezone)
    } else {
        // Get chosen city with Ok and Err response from get_city_choice function
        match get_city_choice(&choice) {
            // Setting Ok return param to new coords param
            Ok(coords) => coords,
            Err(e) => {
                eprintln!("{}", e);
                // Main is returned on error and thus program is exited.
                return Ok(());
            }
        }
    };

    println!("Choose an option:");
    println!("1. Current weather");
    println!("2. Min/Max temperature for the next seven days");
    let mut option = String::new();
    io::stdin().read_line(&mut option).unwrap();

    // Send the two params in the coords tuple to the URL construct function for API call
    let url = construct_url(latitude, longitude, timezone, &option.trim());
    if url.is_empty() {
        eprintln!("Invalid option");
        return Ok(());
    }

    // Using reqwest for async wait. Setting json from the response from the generated url (with the lat/long params)
    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;

    match option.trim() {
        "1" => {
            if let Some(hourly) = response.hourly {
                // Check if the temperature_2m param in the response has values, and if so, prints the first.
                if let Some(temp) = hourly.temperature_2m.and_then(|v| v.first().cloned()) {
                    println!("Current temperature: {}°C", temp);
                } else {
                    println!("No temperature data available.");
                }
                // Check if the precipitation_probability param in the response has values, and if so, prints the first.
                if let Some(probability) = hourly.precipitation_probability.and_then(|v| v.first().cloned()) {
                    println!("Chance of rain: {}%", probability);
                } else {
                    println!("No precipitation data available.");
                }
            } else {
                println!("No hourly data available.");
            }
        }
        "2" => {
            if let Some(daily) = response.daily {
                if let (Some(min_temps), Some(max_temps)) = (daily.temperature_2m_min, daily.temperature_2m_max) {
                    if let Some(dates) = daily.time {
                        println!("Minimum/Maximum temperatures for the next seven days:");
						// Prints each date with min/max from the daily array that was received in the call. Open-meteo returns 7 days by default.
                        for ((min, max), date) in min_temps.iter().zip(&max_temps).zip(dates.iter()) {
                            println!("Date: {}, Min: {}°C, Max: {}°C", date, min, max);
                        }
                    } else {
                        println!("No date data available.");
                    }
                } else {
                    println!("No min/max temperature data available.");
                }
            } else {
                println!("No daily data available.");
            }
        }
        _ => eprintln!("Invalid option"),
    }

	println!("Press enter to exit:");
    let mut option = String::new();
    io::stdin().read_line(&mut option).unwrap();
    Ok(())
}
