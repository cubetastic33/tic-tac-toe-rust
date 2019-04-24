#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use azul::prelude::*;

use rand::Rng;
use std::borrow::BorrowMut;

#[cfg(debug_assertions)]
use std::time::Duration;

const FONT_RALEWAY: &[u8] = include_bytes!("../assets/fonts/Raleway/Raleway-Regular.ttf");

macro_rules! CSS_PATH {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/styles/main.css")
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Cell {
    X,
    O,
    Empty,
}

#[derive(PartialEq)]
enum GameStatus {
    Proceed,
    UserWon,
    IWon,
    Draw,
}

#[derive(Default)]
struct TicTacToe {
    user_is_x: bool,
    cells: Vec<Cell>,
    game_status: GameStatus,
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus::Proceed
    }
}

impl Layout for TicTacToe {
    fn layout(&self, _info: LayoutInfo<Self>) -> Dom<Self> {
        let board = Dom::div()
            .with_id("board")
            .with_child(self.row(1))
            .with_child(self.row(2))
            .with_child(self.row(3));

        let control_panel: Dom<Self> = Dom::div()
            .with_id("control_panel")
            .with_child(
                Dom::label("Start new game as X")
                    .with_class("button")
                    .with_callback(On::MouseUp, Callback(new_game)),
            )
            .with_child(
                Dom::label("Start new game as O")
                    .with_class("button")
                    .with_callback(On::MouseUp, Callback(new_game)),
            )
            .with_child(if self.game_status == GameStatus::Proceed {
                Dom::div()
            } else if self.game_status == GameStatus::UserWon {
                Dom::label("You win!").with_class("banner")
            } else if self.game_status == GameStatus::IWon {
                Dom::label("I win!").with_class("banner")
            } else {
                Dom::label("Draw").with_class("banner")
            });

        let header = Dom::label("Tic-tac-toe").with_id("header");
        let main = Dom::div()
            .with_id("main")
            .with_child(control_panel)
            .with_child(board);
        Dom::div().with_child(header).with_child(main)
    }
}

impl TicTacToe {
    fn row(&self, x: usize) -> Dom<Self> {
        (0..3)
            .map(|y| {
                if self.cells[(x - 1) * 3 + y] == Cell::Empty {
                    NodeData {
                        node_type: NodeType::Div,
                        ids: vec![DomString::Heap(((x - 1) * 3 + y).to_string())],
                        classes: vec![DomString::Static("cell"), DomString::Static("empty")],
                        callbacks: vec![(On::MouseUp.into(), Callback(place_user_counter))],
                        ..Default::default()
                    }
                } else {
                    NodeData {
                        node_type: NodeType::Label(if self.cells[(x - 1) * 3 + y] == Cell::X {
                            DomString::Static("x")
                        } else {
                            DomString::Static("o")
                        }),
                        ids: vec![DomString::Heap(((x - 1) * 3 + y).to_string())],
                        classes: vec![
                            DomString::Static("cell"),
                            if self.cells[(x - 1) * 3 + y] == Cell::X {
                                DomString::Static("x")
                            } else {
                                DomString::Static("o")
                            },
                        ],
                        ..Default::default()
                    }
                }
            })
            .collect::<Dom<Self>>()
            .with_class("row")
    }
}

fn place_user_counter(
    app_state: &mut AppState<TicTacToe>,
    event: &mut CallbackInfo<TicTacToe>,
) -> UpdateScreen {
    // Get position of parent
    let x = event
        .get_index_in_parent(event.target_parent().unwrap())
        .unwrap()
        .0;
    // Get position in parent
    let y = event.target_index_in_parent().unwrap();
    println!("clicked cell = {}", x * 3 + y);
    let mut app = app_state.data.borrow_mut().lock().unwrap();
    // Change the cell to the user's counter
    app.cells[x * 3 + y] = if app.user_is_x { Cell::X } else { Cell::O };
    let cells = app.cells.clone();
    let user_is_x = app.user_is_x.clone();
    app.game_status = check_game_status(&cells, &user_is_x);
    if app.game_status == GameStatus::Proceed {
        // If the game hasn't ended yet, make our move
        let opponents_counter = place_opponent_counter(&cells, &user_is_x);
        println!("opponent's move: {}", opponents_counter);
        app.cells[opponents_counter] =
            if app.user_is_x { Cell::O } else { Cell::X };
        // Check if we won the game
        let cells = app.cells.clone();
        app.game_status = check_game_status(&cells, &user_is_x);
    }
    Redraw
}

fn place_opponent_counter(cells: &Vec<Cell>, user_is_x: &bool) -> usize {
    let mut x_counters = vec![];
    let mut o_counters = vec![];
    /*let neighbours: [Vec<usize>; 9] = [
        vec![1, 3, 4],
        vec![0, 2, 3, 4, 5],
        vec![1, 4, 5],
        vec![0, 1, 4, 6, 7],
        vec![0, 1, 2, 3, 5, 6, 7, 8],
        vec![1, 2, 4, 7, 8],
        vec![3, 4, 7],
        vec![3, 4, 5, 6, 8],
        vec![4, 5, 7],
    ];*/
    let possible_win_positions: [Vec<usize>; 9] = [
        vec![1, 3, 4],
        vec![4],
        vec![1, 4, 5],
        vec![4],
        vec![],
        vec![4],
        vec![3, 4, 7],
        vec![4],
        vec![4, 5, 7],
    ];
    for (i, cell) in cells.iter().enumerate() {
        if *cell == Cell::X {
            x_counters.push(i);
        } else if *cell == Cell::O {
            o_counters.push(i);
        }
    }
    let (opponents_counters, users_counters) = if *user_is_x {
        (o_counters, x_counters)
    } else {
        (x_counters, o_counters)
    };
    println!("our counters = {:?}", opponents_counters);
    println!("user's counters = {:?}", users_counters);
    if opponents_counters.len() >= 2 {
        // If we've placed at least 2 counters so there's a chance to win
        for cell_index in opponents_counters.iter() {
            // Iterate over our placed counters
            println!(
                "neighbours of {} to check: {:?}",
                cell_index, possible_win_positions[*cell_index]
            );
            for neighbour in possible_win_positions[*cell_index].iter() {
                if opponents_counters.contains(neighbour)
                    && !users_counters.contains(&(neighbour * 2 - cell_index))
                    && neighbour * 2 - cell_index < 9 {
                    // The next cell in the line isn't occupied by the user and isn't outside the board
                    // Return the index of the next cell to complete the line and win
                    return neighbour * 2 - cell_index;
                }
            }
        }
    }
    println!("our counters = {:?}", opponents_counters);
    println!("user's counters = {:?}", users_counters);
    if users_counters.len() >= 2 {
        // If the user has placed at least 2 counters so there's a chance they could win
        for cell_index in users_counters.iter() {
            // Iterate over the counters they've placed
            println!(
                "neighbours of {} to check: {:?}",
                cell_index, possible_win_positions[*cell_index]
            );
            for neighbour in possible_win_positions[*cell_index].iter() {
                println!("Does the user have {}? {}", neighbour, users_counters.contains(neighbour));
                println!("Do we have {}? {}", neighbour * 2 - cell_index, opponents_counters.contains(&(neighbour * 2 - cell_index)));
                if users_counters.contains(neighbour)
                    && !opponents_counters.contains(&(neighbour * 2 - cell_index))
                    && neighbour * 2 - cell_index < 9 {
                    // The next cell in the line isn't occupied by us and isn't outside the grid
                    // Return the index of the next cell to stop the user from winning this way
                    return neighbour * 2 - cell_index;
                }
            }
        }
    }
    // Select a random empty cell to place the opponent's counter
    let mut rng = rand::thread_rng();
    let cell = rng.gen_range(0, 9);
    if cells[cell] == Cell::Empty {
        // We have found an empty cell, return its index
        return cell;
    }
    // The cell wasn't empty, try again
    return place_opponent_counter(cells, user_is_x);
}

fn check_game_status(cells: &Vec<Cell>, user_is_x: &bool) -> GameStatus {
    let mut x_counters = vec![];
    let mut o_counters = vec![];
    let mut empty_cells = vec![];
    let neighbours: [Vec<usize>; 9] = [
        vec![1, 3, 4],
        vec![0, 2, 3, 4, 5],
        vec![1, 4, 5],
        vec![0, 1, 4, 6, 7],
        vec![0, 1, 2, 3, 5, 6, 7, 8],
        vec![1, 2, 4, 7, 8],
        vec![3, 4, 7],
        vec![3, 4, 5, 6, 8],
        vec![4, 5, 7],
    ];
    let forward_neighbours: [Vec<usize>; 8] = [
        vec![1, 3, 4],
        vec![2, 3, 4, 5],
        vec![4, 5],
        vec![4, 6, 7],
        vec![5, 6, 7, 8],
        vec![7, 8],
        vec![7],
        vec![8],
    ];
    for (i, cell) in cells.iter().enumerate() {
        if *cell == Cell::X {
            x_counters.push(i);
        } else if *cell == Cell::O {
            o_counters.push(i);
        } else {
            empty_cells.push(i);
        }
    }

    for counter in [x_counters.clone(), o_counters].iter() {
        // Iterate over both the players
        if counter.len() >= 3 {
            // If this player has placed at least 3 counters, so there's a possibility they won
            for cell_index in counter[0..counter.len() - 2].iter() {
                // Skip checking for the last two indices because a winning pattern needs to have 2 counters after it
                for neighbour in forward_neighbours[*cell_index].iter() {
                    if counter.contains(neighbour) {
                        if !neighbours[*cell_index].contains(&(neighbour * 2 - cell_index))
                            && counter.contains(&(neighbour * 2 - cell_index))
                        {
                            if counter[0] == x_counters[0] {
                                return if *user_is_x {
                                    GameStatus::UserWon
                                } else {
                                    GameStatus::IWon
                                };
                            } else {
                                return if *user_is_x {
                                    GameStatus::IWon
                                } else {
                                    GameStatus::UserWon
                                };
                            }
                        }
                    }
                }
            }
        }
    }
    if empty_cells.len() == 0 {
        // All the cells have been filled, and neither players have won, which means it's a draw
        return GameStatus::Draw;
    }
    GameStatus::Proceed
}

fn new_game(
    app_state: &mut AppState<TicTacToe>,
    event: &mut CallbackInfo<TicTacToe>,
) -> UpdateScreen {
    let mut app = app_state.data.borrow_mut().lock().unwrap();
    // Set user's counter
    app.user_is_x = [true, false][event.target_index_in_parent().unwrap()];
    let cells = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
    ];
    app.cells = cells.clone();
    app.game_status = GameStatus::Proceed;
    if !app.user_is_x {
        app.cells[place_opponent_counter(&cells, &false)] = Cell::X;
    }
    Redraw
}

fn main() {
    let mut app = App::new(
        TicTacToe {
            user_is_x: true,
            cells: vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            game_status: GameStatus::Proceed,
        },
        AppConfig::default(),
    )
    .unwrap();

    let mut window_create_options = WindowCreateOptions::default();
    window_create_options.state.title = String::from("Tic-tac-toe");
    window_create_options.state.is_maximized = true;
    #[cfg(debug_assertions)]
    let window = {
        let hot_reloader = css::hot_reload(CSS_PATH!(), Duration::from_millis(500));
        app.create_hot_reload_window(window_create_options, hot_reloader).unwrap()
    };

    #[cfg(not(debug_assertions))]
    let window = {
        let css = css::from_str(include_str!(CSS_PATH!())).unwrap();
        app.create_window(window_create_options, css).unwrap()
    };

    let font_id = app.add_css_font_id("Raleway-Regular");
    app.add_font(font_id, FontSource::Embedded(FONT_RALEWAY));

    app.run(window).unwrap();
}
