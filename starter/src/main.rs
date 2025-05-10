mod generated;

use generated::{UnitCommand, Vector2, simulation_client::SimulationClient};
use std::str::FromStr;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Status, metadata::AsciiMetadataValue, transport::Endpoint};

const SIMULATION_ID_HEADER: &str = "x-simulation-id";
const UNIT_ID_HEADER: &str = "x-unit-id";

#[tokio::main]
async fn main() -> eyre::Result<()> {
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

    for (unit_id, initial_pos) in sim.sensor_units {
        manage_unit(&mut client, &sim.id, &unit_id, initial_pos).await?;
    }

    Ok(())
}

async fn manage_unit<T>(
    client: &mut SimulationClient<T>,
    sim_id: &str,
    unit_id: &str,
    initial_pos: Vector2,
) -> eyre::Result<()>
where
    T: tonic::client::GrpcService<tonic::body::Body>,
    T::Error: Into<tonic::codegen::StdError>,
    T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + Send + 'static,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let cmd_stream = ReceiverStream::new(rx);

    let mut request = Request::new(cmd_stream);
    request.metadata_mut().insert(
        SIMULATION_ID_HEADER,
        AsciiMetadataValue::from_str(sim_id).unwrap(),
    );
    request.metadata_mut().insert(
        UNIT_ID_HEADER,
        AsciiMetadataValue::from_str(unit_id).unwrap(),
    );
    let mut status_stream = client.unit_control(request).await?.into_inner();

    tokio::spawn(async move {
        let _ = initial_pos;
        while let Some(_unit_status) = status_stream.message().await? {
            // TODO: some logic here - note you can send commands
            // anytime, you don't need to wait for status updates
            tx.send(UnitCommand { command: None }).await?;
        }

        Ok::<_, eyre::Report>(())
    });

    Ok(())
}
