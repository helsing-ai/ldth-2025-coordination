mod generated;

use generated::simulation_client::SimulationClient;
use std::str::FromStr;
use tonic::{Request, Status, metadata::AsciiMetadataValue, transport::Endpoint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = "changeme";
    let header_value = format!("bearer {token}");
    let url = "http://172.237.124.96:21234";

    let authenticate = |mut req: Request<()>| -> Result<Request<()>, Status> {
        req.metadata_mut().insert(
            "authorization",
            AsciiMetadataValue::from_str(&header_value).unwrap(),
        );
        Ok(req)
    };

    let channel = Endpoint::from_str(url)?.connect().await?;
    let mut client = SimulationClient::with_interceptor(channel, authenticate);

    let sim = client.start(()).await?.into_inner();
    println!("start parameters: {sim:?}");

    Ok(())
}
