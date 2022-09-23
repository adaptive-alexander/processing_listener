use google_cloud_gax::cancel::CancellationToken;
use google_cloud_pubsub::client::Client;
use google_cloud_pubsub::subscription::SubscriptionConfig;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use options::{
    options_struct::Options,
    utilities,
};
use options::pricing_models::black_scholes;
use rayon::prelude::*;
use tokio::fs;

/// # sub_listener
/// Listens to a topic subscription and runs callback
pub async fn sub_listener(
    topic: &str,
    sub: &str,
) {
    // Create pubsub client.
    // - If the server is running on GCP the project_id is from metadata server
    let client = Client::default().await.unwrap();

    // Get the topic to subscribe to.
    let topic = client.topic(topic);

    // Configure subscription.
    let config = SubscriptionConfig::default();

    // Create subscription
    let subscription = client.subscription(sub);
    if !subscription.exists(None, None).await.unwrap() {
        subscription
            .create(topic.fully_qualified_name(), config, None, None)
            .await.unwrap();
    }
    // Token for cancel.
    let cancel = CancellationToken::new();

    // Receive blocks until the ctx is cancelled or an error occurs.
    subscription
        .receive(
            move |message, _cancel| async move {
                File::open(r"./input/inp_file").unwrap().write_all(&*message.message.data).expect("Failed writing file from bytes");

                // Timing initialization
                let mut opts = Options::from_file(&PathBuf::from(Path::new(r"./input/inp_file")), Box::new(black_scholes::BlackScholesModel::new()));

                // Chunk options
                let mut chunked_opts = utilities::chunk_opt(opts, 1000);

                // Parallel computation of options_old
                chunked_opts.par_iter_mut().for_each(|x| x.get_prices());
                chunked_opts.par_iter_mut().for_each(|x| x.get_greeks());

                // Collect Options
                opts = utilities::collect_chunks(chunked_opts);

                // Write and time output
                opts.write_csv(PathBuf::from(Path::new(r"./output/out_file")))
                    .expect("Failed writing output to csv.");

                // Return file


                // Remove processed files
                fs::remove_file(Path::new(r"./input/inp_file"));
                fs::remove_file(Path::new(r"./output/out_file"));

                // Ack or Nack message.
                message.ack().await.expect("Failed acknowledging message");
            },
            cancel.clone(),
            None,
        )
        .await
        .expect("Receiving message encountered an error");
}
