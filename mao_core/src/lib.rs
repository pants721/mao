use anyhow::{anyhow, Result};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

pub mod web;

#[derive(Deserialize, Serialize, Clone, Debug, strum::Display, PartialEq, PartialOrd)]
pub enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

#[derive(Deserialize, Serialize, Clone, Debug, strum::Display, PartialEq, PartialOrd)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    pub fn name(&self) -> String {
        format!("{} of {}", self.rank, self.suit).to_string()
    }

    pub fn stackable(&self, b: &Card) -> bool {
        self.suit == b.suit || self.rank == b.rank
    }
}

pub fn std_deck() -> Vec<Card> {
    vec![
        Card::new(Suit::Hearts, Rank::Ace),
        Card::new(Suit::Hearts, Rank::Two),
        Card::new(Suit::Hearts, Rank::Three),
        Card::new(Suit::Hearts, Rank::Four),
        Card::new(Suit::Hearts, Rank::Five),
        Card::new(Suit::Hearts, Rank::Six),
        Card::new(Suit::Hearts, Rank::Seven),
        Card::new(Suit::Hearts, Rank::Eight),
        Card::new(Suit::Hearts, Rank::Nine),
        Card::new(Suit::Hearts, Rank::Ten),
        Card::new(Suit::Hearts, Rank::Jack),
        Card::new(Suit::Hearts, Rank::Queen),
        Card::new(Suit::Hearts, Rank::King),
        Card::new(Suit::Spades, Rank::Ace),
        Card::new(Suit::Spades, Rank::Two),
        Card::new(Suit::Spades, Rank::Three),
        Card::new(Suit::Spades, Rank::Four),
        Card::new(Suit::Spades, Rank::Five),
        Card::new(Suit::Spades, Rank::Six),
        Card::new(Suit::Spades, Rank::Seven),
        Card::new(Suit::Spades, Rank::Eight),
        Card::new(Suit::Spades, Rank::Nine),
        Card::new(Suit::Spades, Rank::Ten),
        Card::new(Suit::Spades, Rank::Jack),
        Card::new(Suit::Spades, Rank::Queen),
        Card::new(Suit::Spades, Rank::King),
        Card::new(Suit::Clubs, Rank::Ace),
        Card::new(Suit::Clubs, Rank::Two),
        Card::new(Suit::Clubs, Rank::Three),
        Card::new(Suit::Clubs, Rank::Four),
        Card::new(Suit::Clubs, Rank::Five),
        Card::new(Suit::Clubs, Rank::Six),
        Card::new(Suit::Clubs, Rank::Seven),
        Card::new(Suit::Clubs, Rank::Eight),
        Card::new(Suit::Clubs, Rank::Nine),
        Card::new(Suit::Clubs, Rank::Ten),
        Card::new(Suit::Clubs, Rank::Jack),
        Card::new(Suit::Clubs, Rank::Queen),
        Card::new(Suit::Clubs, Rank::King),
        Card::new(Suit::Diamonds, Rank::Ace),
        Card::new(Suit::Diamonds, Rank::Two),
        Card::new(Suit::Diamonds, Rank::Three),
        Card::new(Suit::Diamonds, Rank::Four),
        Card::new(Suit::Diamonds, Rank::Five),
        Card::new(Suit::Diamonds, Rank::Six),
        Card::new(Suit::Diamonds, Rank::Seven),
        Card::new(Suit::Diamonds, Rank::Eight),
        Card::new(Suit::Diamonds, Rank::Nine),
        Card::new(Suit::Diamonds, Rank::Ten),
        Card::new(Suit::Diamonds, Rank::Jack),
        Card::new(Suit::Diamonds, Rank::Queen),
        Card::new(Suit::Diamonds, Rank::King),
    ]
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Player {
    name: String,
    hand: Vec<Card>
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), hand: vec![] }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Game {
    deck: Vec<Card>,
    play_stack: Vec<Card>,
    pub players: Vec<Player>,
    // XXX: who is the king mao
}

impl Game {
    pub fn new(hand_size: usize, player_names: Vec<&str> ) -> Self {
        let mut rng = rand::thread_rng();
        let mut deck = std_deck();
        deck.shuffle(&mut rng);

        let mut players = vec![];

        for name in player_names {
            let mut player = Player::new(name);
            for _ in 0..hand_size {
                let card = deck.pop().expect("Handsize x number of players exceeds deck size");
                player.hand.push(card);
            }
            players.push(player);
        }

        let first_card = deck.pop().expect("Handsize x number of players exceeds deck size");

        Self {
            deck,
            players,
            play_stack: vec![first_card],
        }
    }

    pub fn top_card(&self) -> &Card {
        self.play_stack.last().expect("Play stack is empty")
    }

    pub fn valid_play(&self, card: &Card) -> bool {
        card.stackable(self.top_card())
    }

    fn get_player(&mut self, name: &str) -> Option<&mut Player> {
        if let Some(idx) = self.players.iter().position(|p| p.name == name) {
            return self.players.get_mut(idx);
        }

        None
    }

    pub fn play_card(&mut self, card: Card, player_name: &str) -> Result<()> {
        if !self.valid_play(&card) {
            return Err(anyhow!("Invalid card played"));
        }

        let player = self.get_player(player_name).expect("Could not find player by given name");

        if let Some(idx) = player.hand.iter().position(|c| *c == card) {
            let c = player.hand.remove(idx);
            self.play_stack.push(c);

            // we will never need to see further back than 4 in the play stack
            if self.play_stack.len() > 4 {
                let bot = self.play_stack.remove(0);
                self.deck.push(bot);
                self.deck.shuffle(&mut thread_rng());
            }
        }

        Ok(())
    }

    pub fn draw_card(&mut self, player_name: &str) -> Result<Option<Card>> {
        if self.deck.is_empty() && self.play_stack.len() > 2 {
            while self.play_stack.len() > 1 {
                let c = self.play_stack.remove(0);
                self.deck.push(c);
                self.deck.shuffle(&mut thread_rng());
            }
        }

        if self.deck.is_empty() && self.play_stack.len() == 1 {
            return Ok(None);
        }

        if let Some(idx) = self.players.iter().position(|p| p.name == player_name) {
            let player = self.players.get_mut(idx).unwrap();
            let top_card = self.deck.pop().expect("Deck is somehow empty");
            player.hand.push(top_card.clone());
            return Ok(Some(top_card));
        }

        Err(anyhow!("Could not find player by given name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_name() {
        let card = Card::new(Suit::Hearts, Rank::Ace);
        assert_eq!(card.name(), "Ace of Hearts".to_string());
    }

    #[test]
    fn new_game() {
        let game = Game::new(8, vec!["lucas", "jesus", "messi", "hitler"]);
        assert!(game.players.len() == 4);
        for player in game.players {
            assert!(player.hand.len() == 8);
        }
    }

    #[test]
    fn draw_card() {
        let mut game = Game::new(2, vec!["lucas", "lucas2"]);
        game.draw_card("lucas").unwrap();
        assert!(game.get_player("lucas").unwrap().hand.len() == 3);
    }

    #[test]
    fn play_card() {
        let mut game = Game {
            players: vec![Player {
                name: "lucas".to_string(),
                hand: vec![Card::new(Suit::Diamonds, Rank::Ace)],
            }],
            deck: vec![],
            play_stack: vec![Card::new(Suit::Diamonds, Rank::King)],
        };

        assert!(game.play_card(Card::new(Suit::Diamonds, Rank::Ace), "lucas").is_ok())
    }
}
