use reqwest::Error;
use serde::Deserialize;
use std::io::{self, Write};

#[derive(Deserialize, Debug)]
struct Weather {
    temperature_2m: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    hourly: Weather,
}

const CITIES: [(&str, f64, f64); 4] = [
    ("Adelaide, Australia", -34.9285, 138.6007),
    ("Melbourne, Australia", -37.8136, 144.9631),
    ("London, UK", 51.5072, -0.1276),
    ("Beijing, China", 39.9042, 116.4074),
];

fn get_city_choice(input: &str) -> Result<(f64, f64), &'static str> {
    let choice: usize = input.trim().parse().map_err(|_| "Invalid choice")?;
    if choice < 1 || choice > CITIES.len() {
        return Err("Choice out of range");
    }
	// returns lat/long tuple
    Ok((CITIES[choice - 1].1, CITIES[choice - 1].2))
}

fn construct_url(latitude: f64, longitude: f64) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        latitude, longitude
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Select a city:");
    // Print each city from CITIES constant
	for (i, city) in CITIES.iter().enumerate() {
        println!("{}. {}", i + 1, city.0);
    }

    println!("Enter the number of your choicre: ");
    
	// Instantiate input param
	let mut choice = String::new();
	// Get input and set to input param
    io::stdin().read_line(&mut choice).unwrap();

	// Get chosen city with Ok and Err response from get_city_choice function
    let (latitude, longitude) = match get_city_choice(&choice) {
        // Setting Ok return param to new coords param
		Ok(coords) => coords,
        Err(e) => {
            eprintln!("{}", e);
			// Main is returned on error and thus program is exited.
            return Ok(());
        }
    };

	// sends the two params in the coords tuple to the url construct function for api call
    let url = construct_url(latitude, longitude);

	// Using reqwest for async wait. Setting json from the response from the genetated url (with the lat/long params)
    let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;

	// Check if the temperature_2m param in the response has values, and if so, prints the first.
    if let Some(temp) = response.hourly.temperature_2m.first() {
        println!("Current temperature: {}°C", temp);
    } else {
        println!("No temperature data available.");
    }

    Ok(())
}
