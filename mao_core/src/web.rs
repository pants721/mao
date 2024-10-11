use std::{net::TcpStream, sync::{Arc, Mutex}};

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::{Card, Game};

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lobby {
    #[serde(skip)]
    pub players: Vec<Player>,
    pub id: String,
    #[serde(skip)]
    pub current_game: Option<Game>,
    hand_size: usize,
}

impl Lobby {
    pub fn new(first_player: Player, hand_size: usize, id: String) -> Self {
        Self {
            players: vec![first_player],
            current_game: None,
            id,
            hand_size,
        }
    }

    pub fn start_game(&mut self) -> Result<()> {
        self.current_game = Some(Game::new(
            self.hand_size, 
            self.players.iter().map(|p| p.name.as_str()).collect()
        ));

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub enum ClientRequest {
    // Lobby stuff
    JoinLobby {
        lobby_id: String,
        player_name: String,
    },
    CreateLobby {
        player_name: String,
        hand_size: usize,
        // XXX: password
    },
    StartGame {
        lobby_id: String,
    },


    // Game stuff
    PlayCard {
        player_id: String,
        lobby_id: String,
        card: Card,
    },
    DrawCard {
        player_id: String,
        lobby_id: String, 
    }
    // XXX: Penalty calls
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerResponse {
    pub game_state: Option<Game>,
    pub lobby: Lobby,
}

impl ServerResponse {
    pub fn new(game_state: Option<Game>, lobby: Lobby) -> Self {
        Self { game_state, lobby }
    }
}

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

