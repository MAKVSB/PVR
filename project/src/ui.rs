use ratatui::{
    layout::{Constraint, Layout}, widgets::Paragraph, Frame
};

use crate::app::App;
pub use crate::widgets;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let l = Layout::vertical([
        Constraint::Percentage(100),
        Constraint::Length(1),
    ]);
    let [main_area, help_area] = l.areas(frame.area());

    let columns = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]);
    let [spotify_column, youtube_column] = columns.areas(main_area);

    app.spotify_column.render(frame, spotify_column);
    app.youtube_column.render(frame, youtube_column);

    let help_message = Paragraph::new("Use ↓↑ to move, [enter] to select, ←→ to transfer, [a] to add, [r] to refresh, [del] to delete, [o] to open in browser.").centered();
    frame.render_widget(help_message, help_area);

    app.popup.render(frame, frame.area());
}
