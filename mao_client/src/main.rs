use tungstenite::{connect, Message};

use mao_core::*;
use web::{ClientRequest, ServerResponse};
use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, response) = connect("ws://localhost:3012/socket").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (header, _value) in response.headers() {
        println!("* {header}");
    }

    socket.send(Message::Text(
        serde_json::to_string(&ClientRequest::CreateLobby { hand_size: 2, player_name: "lucas".to_string() }).unwrap()
    )).unwrap();
    let msg = socket.read().expect("Error reading message");
    println!("Received: {msg}");
    let res = serde_json::from_str::<ServerResponse>(&msg.to_string()).unwrap();
    socket.send(Message::Text(serde_json::to_string(&ClientRequest::StartGame { lobby_id: res.lobby.id.clone() })?))?;

    loop {
        socket.send(Message::Text(serde_json::to_string(&ClientRequest::DrawCard {player_id: "lucas".to_string(), lobby_id: res.lobby.id.clone() })?))?;
        let msg = socket.read().expect("Error reading message");
        println!("Received: {msg}");
        let res = serde_json::from_str::<ServerResponse>(&msg.to_string()).unwrap();
    }
    // socket.close(None);
}
