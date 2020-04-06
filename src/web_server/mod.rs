use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
pub struct TableInfo {
  pub target_height: Option<i32>,
  pub current_height: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct ChangeHeightRequest {
  pub target_height: i32,
}

#[tokio::main]
pub async fn start_server(
  tx_set_target_height: Sender<i32>,
  rx_table_info_request: Sender<()>,
  rx_table_info_response: Receiver<TableInfo>,
) {
  let with_set_target_height_filter = warp::any().map(move || tx_set_target_height.clone());
  let with_table_info_request_filter = warp::any().map(move || rx_table_info_request.clone());
  let with_table_info_response_filter = warp::any().map(move || rx_table_info_response.clone());
  let table_info = warp::path!("table")
    .and(warp::get())
    .and(with_table_info_request_filter.clone())
    .and(with_table_info_response_filter.clone())
    .map(
      |table_info_request: Sender<()>, table_info_response: Receiver<TableInfo>| {
        table_info_request.send(()).unwrap();
        let response = table_info_response.recv().unwrap();
        warp::reply::json(&response)
      },
    );

  let set_table_info = warp::path!("table")
    .and(warp::patch())
    .and(warp::body::content_length_limit(1024 * 16))
    .and(with_set_target_height_filter.clone())
    .and(with_table_info_request_filter.clone())
    .and(with_table_info_response_filter.clone())
    .and(warp::body::json())
    .map(
      |set_target_height: Sender<i32>,
       table_info_request: Sender<()>,
       table_info_response: Receiver<TableInfo>,
       body: ChangeHeightRequest| {
        set_target_height.send(body.target_height).unwrap();
        table_info_request.send(()).unwrap();
        let response = table_info_response.recv().unwrap();
        warp::reply::json(&response)
      },
    );

  let routes = table_info.or(set_table_info);

  let addr = std::env::var("ADDRESS").unwrap_or("localhost:7777".to_string()).parse<IpAddr>();
  println!("{:?}", addr); // TODO pass IP and PORT as env var
  warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
