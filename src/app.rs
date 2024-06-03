use std::collections::HashMap;
use std::io::{Stdout, stdout};
use std::sync::{Arc, Mutex};
use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, EventStream, KeyCode};
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::FutureExt;
use ratatui::prelude::{CrosstermBackend};
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use tracing::{debug, info};
use crate::component::Component;

pub struct App {
    pub event_receiver: UnboundedReceiver<Event>,
    pub event_sender: UnboundedSender<Event>,
    pub action_receiver: UnboundedReceiver<Action>,
    pub action_sender: UnboundedSender<Action>,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub components: HashMap<String, Box<dyn Component>>,
    pub cur_component: Arc<Mutex<String>>,
    pub fps: f64,
}

impl App {
    pub fn new() -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (action_sender, action_receiver) = mpsc::unbounded_channel();
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let app = Self {
            event_sender,
            event_receiver,
            action_sender,
            action_receiver,
            terminal,
            fps: 60.0,
            components: HashMap::new(),
            cur_component: Arc::new(Mutex::new(String::from("HomePage"))),
        };
        return Ok(app);
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting the application......");
        crossterm::execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        // UI thread to listen to key events
        let _event_sender = self.event_sender.clone();
        let _fps = self.fps;
        let _cur_component = self.cur_component.clone();

        tokio::spawn(async move {
            let render_delay = std::time::Duration::from_secs_f64(1.0 / _fps);
            let mut render_interval = tokio::time::interval(render_delay);
            let mut reader = EventStream::new();
            loop {
                let next_event = reader.next().fuse();
                let next_render = render_interval.tick();
                tokio::select! {
                    _ = next_render => {
                        let val =  _cur_component.lock().unwrap();
                        let event = Event::Render(val.clone());
                        debug!("UI thread sending event {:?}", &event);
                        _event_sender.send(event).unwrap();

                    }
                    maybe_event = next_event => {
                        match maybe_event {
                            Some(Ok(event)) => {
                                if let crossterm::event::Event::Key(key) = event {
                                    let event = Event::Key(key);
                                    debug!("UI thread sending event {:?}", &event);
                                    _event_sender.send(event).unwrap()
                                }
                            }
                            Some(Err(e)) => eprintln!("Error: {:?}\r", e),
                            None => break,
                        }
                    }
                }
            }
        });

        // Data thread to perform actions based on events
        loop {
            if let Some(e) = self.next_event().await {
                match e {
                    Event::Key(key) => {
                        // Special handle to exit the program
                        if key.code == KeyCode::Char('q') {
                            self.action_sender.send(Action::Quit).unwrap()
                        }
                        let component_name = self.cur_component.lock().unwrap();
                        if let Some(component) = self.components.get_mut(component_name.as_str()) {
                            debug!("Data thread received event {:?}", &key);
                            component.handle_key_events(key)?;
                        } else {
                            eprintln!("Component {} doesn't exist!", component_name);
                            std::process::exit(1);
                        }
                    }
                    Event::Render(ref component_name) => {
                        debug!("Data thread received event {:?}", &e);
                        self.action_sender.send(Action::Render(component_name.clone())).unwrap()
                    }
                }
            }

            while let Ok(action) = self.action_receiver.try_recv() {
                match action {
                    Action::Render(ref component_name) => {
                        debug!("Data thread executing action {:?}", &action);
                        let mut cur_component_name = self.cur_component.lock().unwrap();
                        if *component_name != *cur_component_name {
                            debug!("Component changed from {} to {}", &*cur_component_name, component_name);
                            *cur_component_name = component_name.clone();
                            // The action here is to change the page to render. Need to clear the event queue that has the old page event.
                            // The example race condition:
                            // 2024-06-03T03:28:59.108900Z DEBUG src/app.rs:66: UI thread sending event Render("SelectBossPage")
                            // 2024-06-03T03:28:59.112023Z DEBUG src/app.rs:114: Data thread executing action Render("GamePage")
                            // 2024-06-03T03:28:59.112030Z DEBUG src/app.rs:117: Component changed from SelectBossPage to GamePage
                            // 2024-06-03T03:28:59.123166Z DEBUG src/app.rs:106: Data thread received event Render("SelectBossPage")
                            // 2024-06-03T03:28:59.123180Z DEBUG src/app.rs:114: Data thread executing action Render("SelectBossPage")
                            // 2024-06-03T03:28:59.123185Z DEBUG src/app.rs:117: Component changed from GamePage to SelectBossPage
                            while let Ok(_) = self.event_receiver.try_recv() {}
                        }
                        if let Some(component) = self.components.get_mut(&*cur_component_name) {
                            self.terminal.draw(|frame| component.draw(frame, frame.size()).unwrap())?;
                        } else {
                            eprintln!("Component {} doesn't exist!", &*cur_component_name);
                            std::process::exit(1);
                        }
                    }
                    Action::Update(ref component_name, ref message) => {
                        debug!("Data thread executing action {:?}", &action);
                        if let Some(component) = self.components.get_mut(component_name) {
                            component.update(Action::Update(component_name.clone(), message.clone())).unwrap()
                        }
                    }
                    Action::Quit => {
                        self.exit()?;
                    }
                }
            }
        }
    }

    pub fn register_component(&mut self, name: String, mut component: Box<dyn Component>) -> Result<()> {
        if let Some(_) = self.components.get(&name) {
            eprintln!("Duplicate component name {}", name);
            std::process::exit(1);
        } else {
            info!("Registering component: {}", &name);
            component.register_action_handler(self.action_sender.clone())?;
            self.components.insert(name, component);
            Ok(())
        }
    }

    pub fn exit(&self) -> Result<()> {
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::execute!(stdout(), LeaveAlternateScreen)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        std::process::exit(0);
    }

    async fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.recv().await
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Render(String),
    Key(KeyEvent),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Action {
    Render(String),          // component name
    Quit,
    Update(String, String),  // component name -> message
}