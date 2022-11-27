use std::error::Error;

use mmorpg::{handler::Context, constants::Constants, map::Map};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = reqwest::get("https://raw.githubusercontent.com/oh-jinsu/mmorpg-data/main/map/AA-01.json").await?;

    let map: Map = serde_json::from_slice(&res.bytes().await?).unwrap();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let constants = Constants::init()?;

    let app = Context::new(constants, map, listener);

    app.run().await
}
