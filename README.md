# Bredbandskollen Prometheus exporter

A Prometheus exporter for exporting internet speed test results from [Bredbandskollen's CLI](https://github.com/dotse/bbk/tree/master/). Made mostly for fun since Aolde already made an exporter a while back, [bredbandskollen-prometheus-exporter](https://github.com/aolde/bredbandskollen-prometheus-exporter).

## Metrics

| Metric                  | Description                  |
| ----------------------- | ---------------------------- |
| bbk_download_speed_mbps | Download speed in Mbit/s     |
| bbk_upload_speed_mbps   | Upload speed in Mbit/s       |
| bbk_latency_ms          | Latency to test server in ms |

## Labels

All metrics are labeled with the following labels

| Name             | Description                                                        |
| ---------------- | ------------------------------------------------------------------ |
| server           | The test server host name                                          |
| network_operator | The network operator that BBK reported was used for the speed test |
