pub mod test_package {
    tonic::include_proto!("test_package");
}

use test_package::add_service_client::AddServiceClient;
use test_package::{AddReply, AddRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = AddServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(AddRequest { lhs: 20, rhs: 20 });

    let response: tonic::Response<AddReply> = client.add(request).await?;

    println!("RESPONSE={:?}", response.into_inner());

    Ok(())
}