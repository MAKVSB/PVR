use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Clear},
    Frame,
};

#[derive(Debug)]
pub struct LoadingPopup {
    pub active: bool,
}
impl LoadingPopup {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        /// helper function to create a centered rect using up certain percentage of the available rect `r`
        fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }

        if self.active {
            let block = Block::bordered().title("Loading");
            let area = popup_area(area, 60, 20);
            frame.render_widget(Clear, area); //this clears out the background
            frame.render_widget(block, area);
        }
    }
}
