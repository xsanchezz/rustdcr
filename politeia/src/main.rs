mod api;

#[tokio::main]
async fn main() {
    fetch_data().await;
}

async fn fetch_data() {
    let get_votes_link = format!(
        "{}{}",
        api::constants::POLITEIA_HOST_V1,
        api::constants::REQUEST_GET_VOTE_STATUS
    );

    let client = reqwest::ClientBuilder::default()
        .cookie_store(true)
        .connection_verbose(true)
        .build()
        .expect("Could not build client");

    let response = client.get(&get_votes_link).send().await.unwrap();

    println!("{:?}", response);

    let response = response
        .bytes()
        .await
        .expect("Unable to convert response to a ");

    let votes: api::types::VoteStatus =
        serde_json::from_slice(&response).expect("Unable to unmarshall result");

    println!("Total Voted: {}", votes.vote_status.len())
}
