use std::io;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rattunnel::{FilePicker, FilePickerState, PickerResult};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FilePickerState::builder()
        .start_dir(".")
        .build();

    loop {
        terminal.draw(|f| {
            f.render_stateful_widget(FilePicker::default(), f.area(), &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            state.handle_event(Event::Key(key));
        }

        match state.result() {
            PickerResult::Selected(paths) => {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;
                println!("Selected:");
                for path in paths {
                    println!("  {}", path.display());
                }
                return Ok(());
            }
            PickerResult::Cancelled => {
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;
                println!("Cancelled");
                return Ok(());
            }
            PickerResult::Pending => {}
        }
    }
}
