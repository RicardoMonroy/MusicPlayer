use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, Paragraph};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = loop {
        if let Err(e) = terminal.draw(|f| {
            let body = Paragraph::new("M0 scaffold. Press q to quit.")
                .block(Block::bordered().title(mp_core::APP_NAME));
            f.render_widget(body, f.area());
        }) {
            break Err(e);
        }
        match event::read() {
            Ok(Event::Key(k)) if k.code == KeyCode::Char('q') => break Ok(()),
            Ok(_) => {}
            Err(e) => break Err(e),
        }
    };
    ratatui::restore();
    result
}
