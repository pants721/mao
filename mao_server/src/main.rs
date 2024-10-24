use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc, thread::spawn};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard, OwnedMutexGuard};

use tokio_tungstenite::accept_async;
use tungstenite::{
    accept,
    Message,
};
use anyhow::{anyhow, Result};

use mao_core::web::{self, random_string, ClientRequest, Lobby, ServerResponse};

#[derive(Debug)]
pub struct Server {
    lobbies: HashMap<String, Arc<Mutex<Lobby>>>,
}

async fn handle_client(server: Arc<Mutex<Server>>, raw_stream: TcpStream) -> Result<()> {
    let mut ws_stream = accept_async(raw_stream).await?;
    let (mut outgoing, mut incoming) = ws_stream.split();

    while let Some(msg) = incoming.next().await {
        let mut server = server.lock().await;
        let msg = msg?;

        let mut response = Message::Text("None".to_string());
        match msg {
            Message::Text(s) => {
                match serde_json::from_str::<ClientRequest>(&s)? {
                    ClientRequest::JoinLobby { lobby_id, player_name } => {
                        let player = web::Player::new(player_name);
                        if let Some(og_lobby) = server.lobbies.get_mut(&lobby_id) {
                            let mut locked_lobby = og_lobby.lock().await;
                            locked_lobby.players.push(player);
                            response = Message::Text(serde_json::to_string(
                                &ServerResponse::new(
                                    None,
                                    locked_lobby.clone(),
                                )
                            )?);
                        }
                    },
                    ClientRequest::CreateLobby { player_name, hand_size } => {
                        let player = web::Player::new(player_name);
                        let lobby_id = random_string(5);
                        let lobby = Lobby::new(player, hand_size, lobby_id);

                        response = Message::Text(serde_json::to_string(
                            &ServerResponse::new(
                                None,
                                lobby.clone(),
                            )
                        )?);

                        server.lobbies.insert(lobby.id.clone(), Arc::new(Mutex::new(lobby)));
                    },
                    ClientRequest::StartGame { lobby_id } => {
                        if let Some(og_lobby) = server.lobbies.get(&lobby_id) {
                            let mut locked_lobby = og_lobby.lock().await;
                            locked_lobby.start_game()?;
                            if let Some(game) = &mut locked_lobby.current_game {
                                response = Message::Text(serde_json::to_string(
                                    &ServerResponse::new(
                                        Some(game.clone()), 
                                        locked_lobby.clone()
                                    )
                                )?);
                            }
                        }
                    },
                    ClientRequest::PlayCard { player_id, lobby_id, card } => {
                        if let Some(og_lobby) = server.lobbies.get(&lobby_id) {
                            let mut locked_lobby = og_lobby.lock().await;
                            if let Some(game) = &mut locked_lobby.current_game {
                                game.play_card(card, &player_id)?;
                                response = Message::Text(serde_json::to_string(
                                    &ServerResponse::new(
                                        Some(game.clone()), 
                                        locked_lobby.clone()
                                    )
                                )?);
                            }
                        }
                    },
                    ClientRequest::DrawCard { player_id, lobby_id } => {
                        if let Some(og_lobby) = server.lobbies.get(&lobby_id) {
                            let mut locked_lobby = og_lobby.lock().await;
                            if let Some(game) = &mut locked_lobby.current_game {
                                game.draw_card(&player_id).unwrap();
                                response = Message::Text(serde_json::to_string(
                                    &ServerResponse::new(
                                        Some(game.clone()), 
                                        locked_lobby.clone()
                                    )
                                )?);
                            }
                        }
                    }
                }
            },
            _ => return Err(anyhow!("Non-string messages not supported"))
        }
        outgoing.send(response.clone()).await?;
        std::mem::drop(server);
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<()>{
    let addr = "127.0.0.1:3012".to_string();
    env_logger::init();

    let listener = TcpListener::bind(&addr).await?;
    let og_server = Arc::new(Mutex::new(Server {
        lobbies: HashMap::new(),
    }));

    while let Ok((stream, socket_addr)) = listener.accept().await {
        println!("New WebSocket connection: {}", socket_addr);
        let server = og_server.clone();
        tokio::spawn(async move {
            handle_client(server, stream).await.unwrap();
        });
    }

    Ok(())

    // for stream in tcp.incoming() {
    //     let server = og_server.clone();
    //     spawn(move || match stream {
    //         Ok(stream) => {
    //             let mut locked_server = server.lock().unwrap();
    //             if let Err(e) = locked_server.handle_client(stream) {
    //                 eprintln!("{}", e);
    //             }
    //         },
    //         Err(e) => {}
    //     });
    // }

    // Ok(())
}
