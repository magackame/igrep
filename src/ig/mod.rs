mod entries;
mod result_list;
mod search_config;
mod searcher;
mod sink;

pub use entries::EntryType;
pub use search_config::SearchConfig;

use std::{process::Command, sync::mpsc};

use result_list::ResultList;
use searcher::{Event, Searcher};

#[derive(PartialEq)]
pub enum State {
    Idle,
    Searching,
    OpenFile(bool),
    Exit,
}

pub struct Ig {
    rx: mpsc::Receiver<Event>,
    state: State,
    searcher: Searcher,
    pub result_list: ResultList,
}

impl Ig {
    pub fn new(config: SearchConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            rx,
            state: State::Idle,
            searcher: Searcher::new(config, tx),
            result_list: ResultList::default(),
        }
    }

    pub fn open_file_if_requested(&mut self) {
        if let State::OpenFile(idle) = self.state {
            if let Some((file_name, line_number)) = self.result_list.get_selected_entry() {
                let mut child_process = Command::new("nvim")
                    .arg(file_name)
                    .arg(format!("+{}", line_number))
                    .spawn()
                    .expect("Error: Failed to run editor.");
                child_process.wait().expect("Error: Editor failed to exit.");
            }

            self.state = if idle { State::Idle } else { State::Searching };
        }
    }

    pub fn handle_searcher_event(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::NewEntry(e) => self.result_list.add_entry(e),
                Event::SearchingFinished => self.state = State::Idle,
                Event::Error => self.state = State::Exit,
            }
        }
    }

    pub fn search(&mut self) {
        if self.state == State::Idle {
            self.result_list.clear();
            self.state = State::Searching;
            self.searcher.search();
        }
    }

    pub fn open_file(&mut self) {
        self.state = State::OpenFile(self.state == State::Idle);
    }

    pub fn exit(&mut self) {
        self.state = State::Exit;
    }

    pub fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    pub fn is_searching(&self) -> bool {
        self.state == State::Searching
    }

    pub fn exit_requested(&self) -> bool {
        self.state == State::Exit
    }
}
