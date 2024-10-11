use std::{collections::HashMap, net::{TcpListener, TcpStream}, ops::{Deref, DerefMut}, sync::{Arc, Mutex}, thread::{self, spawn}};

use log::{info, warn};
use rand::{distributions::Alphanumeric, Rng};
use tungstenite::{
    accept_hdr,
    accept,
    Message,
};
use anyhow::Result;

use mao_core::web::{self, random_string, ClientRequest, Lobby, ServerResponse};
use mao_core::Game;


#[derive(Debug)]
pub struct Server {
    lobbies: HashMap<String, Arc<Mutex<Lobby>>>,
}

impl Server {
    fn handle_client(&mut self, stream: TcpStream) -> Result<()> {
        let mut socket = accept(stream)?;
        loop {
            match socket.read()? {
                Message::Text(msg) => {
                    match serde_json::from_str::<ClientRequest>(&msg)? {
                        ClientRequest::JoinLobby { lobby_id, player_name } => {
                            let player = web::Player::new(player_name);
                            if let Some(og_lobby) = self.lobbies.get_mut(&lobby_id) {
                                let mut locked_lobby = og_lobby.lock().unwrap();
                                locked_lobby.players.push(player);
                                socket.send(Message::Text(serde_json::to_string(
                                    &ServerResponse::new(
                                        None,
                                        locked_lobby.clone(),
                                    )
                                )?))?;
                            }
                        },
                        ClientRequest::CreateLobby { player_name, hand_size } => {
                            let player = web::Player::new(player_name);
                            let lobby_id = random_string(5);
                            let lobby = Lobby::new(player, hand_size, lobby_id);

                            socket.send(Message::Text(serde_json::to_string(
                                &ServerResponse::new(
                                    None,
                                    lobby.clone(),
                                )
                            )?))?;

                            self.lobbies.insert(lobby.id.clone(), Arc::new(Mutex::new(lobby)));
                            dbg!(&self);
                        },
                        ClientRequest::StartGame { lobby_id } => {
                            if let Some(og_lobby) = self.lobbies.get(&lobby_id) {
                                let mut locked_lobby = og_lobby.lock().unwrap();
                                locked_lobby.start_game()?;
                                if let Some(game) = &mut locked_lobby.current_game {
                                    socket.send(Message::Text(serde_json::to_string(
                                        &ServerResponse::new(
                                            Some(game.clone()), 
                                            locked_lobby.clone()
                                        )
                                    )?))?;
                                }
                            }
                        },
                        ClientRequest::PlayCard { player_id, lobby_id, card } => {
                            if let Some(og_lobby) = self.lobbies.get(&lobby_id) {
                                let mut locked_lobby = og_lobby.lock().unwrap();
                                if let Some(game) = &mut locked_lobby.current_game {
                                    game.play_card(card, &player_id)?;
                                    socket.send(Message::Text(serde_json::to_string(
                                        &ServerResponse::new(
                                            Some(game.clone()), 
                                            locked_lobby.clone()
                                        )
                                    )?))?;
                                }
                            }
                        },
                        ClientRequest::DrawCard { player_id, lobby_id } => {
                            if let Some(og_lobby) = self.lobbies.get(&lobby_id) {
                                let mut locked_lobby = og_lobby.lock().unwrap();
                                if let Some(game) = &mut locked_lobby.current_game {
                                    game.draw_card(&player_id).unwrap();
                                    socket.send(Message::Text(serde_json::to_string(
                                        &ServerResponse::new(
                                            Some(game.clone()), 
                                            locked_lobby.clone()
                                        )
                                    ).unwrap())).unwrap();
                                }
                            }
                        }
                    }
                }
                Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {}
            }
        }
    }
}


fn main() -> Result<()>{

    env_logger::init();

    let game_og = Arc::new(Mutex::new(Game::new(1, vec!["lucas", "mia"])));
    let tcp = TcpListener::bind("127.0.0.1:3012").unwrap();
    let og_server = Arc::new(Mutex::new(Server {
        lobbies: HashMap::new(),
    }));
    for stream in tcp.incoming() {
        let server = og_server.clone();
        spawn(move || match stream {
            Ok(stream) => {
                let mut locked_server = server.lock().unwrap();
                if let Err(e) = locked_server.handle_client(stream) {
                    eprintln!("{}", e);
                }
            },
            Err(e) => {}
        });
    }

    Ok(())
}
