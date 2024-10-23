use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tungstenite::{connect, Message};
use web::{ClientRequest, ServerResponse};
use anyhow::Result;
use wasm_bindgen::prelude::*;
use leptos::*;

use mao_core::*;


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (mut ws_stream, response) = connect_async("ws://localhost:3012/socket").await?;

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    ws_stream.send(Message::Text(
        serde_json::to_string(&ClientRequest::CreateLobby { hand_size: 2, player_name: "lucas".to_string() }).unwrap()
    )).await?;

    if let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        println!("Received: {}", msg);
        let res = serde_json::from_str::<ServerResponse>(&msg.to_string()).unwrap();
        ws_stream.send(Message::Text(serde_json::to_string(&ClientRequest::StartGame { lobby_id: res.lobby.id.clone() })?)).await?;

    }

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        println!("Received: {}", msg);
        let res = serde_json::from_str::<ServerResponse>(&msg.to_string()).unwrap();
        ws_stream.send(Message::Text(serde_json::to_string(&ClientRequest::DrawCard {player_id: "lucas".to_string(), lobby_id: res.lobby.id.clone() })?)).await?;
    }

    Ok(())
    // socket.close(None);
}
