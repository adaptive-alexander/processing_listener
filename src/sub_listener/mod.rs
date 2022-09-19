use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::Status;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::Client;
use google_cloud_pubsub::subscription::SubscriptionConfig;

/// # sub_listener
/// Listens to a topic subscription and runs callback
pub async fn sub_listener(
    call_back: fn(&PubsubMessage), // Pass back data from pub/sub
    topic: &str,
    sub: &str,
) -> Result<(), Status> {
    // Create pubsub client.
    // - If the server is running on GCP the project_id is from metadata server
    let client = Client::default().await.unwrap();

    // Get the topic to subscribe to.
    let topic = client.topic(topic);

    // Configure subscription.
    let config = SubscriptionConfig::default();

    // Create subscription
    let subscription = client.subscription(sub);
    if !subscription.exists(None, None).await? {
        subscription
            .create(topic.fully_qualified_name(), config, None, None)
            .await?;
    }
    // Token for cancel.
    let cancel = CancellationToken::new();

    // Receive blocks until the ctx is cancelled or an error occurs.
    subscription
        .receive(
            move |message, cancel| async move {
                // Handle data.
                call_back(&message.message);

                // Ack or Nack message.
                message.ack().await.expect("Failed acknowledging message");
            },
            cancel.clone(),
            None,
        )
        .await
        .expect("Receiving message encountered an error");

    Ok(())
}
