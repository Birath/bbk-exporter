use std::os::unix::fs::PermissionsExt;

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[tokio::test]
async fn test_metric_endpoint() {
    let dummy_bbk_program_path = std::env::temp_dir().join("test_bbk.sh");
    {
        let mut dummy_bbk = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&dummy_bbk_program_path)
            .await
            .unwrap();
        dummy_bbk.write_all(b"#!/bin/sh\necho '250.445,254.074,4.47409,anycast-global-ipv4.bredbandskollen.se,ISP AB,support_id,11111111'").await.unwrap();
        let mut perms = dummy_bbk.metadata().await.unwrap().permissions();
        perms.set_mode(0o755);
        dummy_bbk.set_permissions(perms).await.unwrap();
        dummy_bbk.flush().await.unwrap();
    }
    let handle = tokio::spawn(async move {
        bbk_exporter::run_exporter(10032, dummy_bbk_program_path, Vec::new()).await
    });
    let expected_output = "# HELP bbk_download_speed_mbps Download speed in Mbit/s
# TYPE bbk_download_speed_mbps gauge
bbk_download_speed_mbps{network_operator=\"ISP AB\",server=\"anycast-global-ipv4.bredbandskollen.se\"} 250.445
# HELP bbk_latency_ms Latency to test server in ms
# TYPE bbk_latency_ms gauge
bbk_latency_ms{network_operator=\"ISP AB\",server=\"anycast-global-ipv4.bredbandskollen.se\"} 4.47409
# HELP bbk_upload_speed_mbps Upload speed in Mbit/s
# TYPE bbk_upload_speed_mbps gauge
bbk_upload_speed_mbps{network_operator=\"ISP AB\",server=\"anycast-global-ipv4.bredbandskollen.se\"} 254.074
";
    let resp = reqwest::get("http://localhost:10032/metrics")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(resp, expected_output);
    // Until the server can be closed gracefully this will have to do
    handle.abort();
}
