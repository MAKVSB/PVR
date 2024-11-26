use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, };
use ratatui::{
    layout::{Alignment, Rect}, style::{Color, Style, Stylize}, text::Text, widgets::{Block, BorderType, List, ListItem, ListState, Paragraph}, Frame
};

#[derive(Debug)]
pub enum ListSelectorKeyResponse {
    Selected,
    CursorMoved,
    None,
    Pass,
}

#[derive(Debug)]
pub struct ListSelectorLabels {
    pub empty: String,
    pub title: String,
}

#[derive(Debug)]
pub struct ListSelector<T>
where
    T: Into<Text<'static>>,
{
    items: Option<Vec<T>>,
    state: ListState,
    selected: Vec<usize>,
    allow_multiple: bool,
    labels: ListSelectorLabels,
    loading: bool,
}
impl<T> ListSelector<T>
where
    T: Into<Text<'static>> + Clone + std::fmt::Debug,
{
    pub fn new(items: Option<Vec<T>>, labels: ListSelectorLabels, allow_multiple: bool) -> Self {
        Self {
            items,
            state: ListState::default(),
            selected: Vec::new(),
            allow_multiple,
            labels,
            loading: false,
        }
    }
    
    pub fn get_selected_items(&mut self) -> Vec<&T>{
        match &self.items {
            Some(items) => {
                items.iter().enumerate()
                    .filter(|(i, _item)| self.selected.contains(i))
                    .map(|(_i, item)| item)
                    .collect::<Vec<&T>>()
                    .clone()
            },
            None => Vec::new(),
        }
    }

    pub fn get_cursor_item(&mut self) -> Option<&T>{
        match self.state.selected() {
            Some(i) => {
                match &self.items {
                    Some(data) => Some(&data[i]),
                    None => None,
                }
            },
            None => {None},
        }
    }

    pub fn clear_selected(&mut self) {
        self.selected.clear();
    }

    pub fn append_items(&mut self, items: Vec<T>) {
        self.loading = false;
        if self.items.is_none() {
            self.items = Some(items);
            return
        }
        self.items.as_mut().unwrap().append(&mut items.clone());
    }

    pub fn set_items(&mut self, items: Option<Vec<T>>) {
        self.loading = false;
        self.items = items;
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, active: bool) {
        let border_style = if active {
            Style::new().fg(Color::White)
        } else {
            Style::new().fg(Color::DarkGray)
        };

        let block = Block::bordered()
            .title(self.labels.title.as_str())
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(border_style);

        if self.loading {
            let par = Paragraph::new("Loading...")
            .block(
                Block::bordered()
                    .title(self.labels.title.as_ref())
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .centered();
            frame.render_widget(par, area);
            return;
        }
        
        match self.items.as_ref() {
            Some(playlist_data) => {
                let items = playlist_data.iter()
                .enumerate()
                .map(|(i, playlist)| {
                    if self.selected.contains(&i) {
                        ListItem::from(playlist.clone()).bg(Color::Cyan)
                    } else {
                        ListItem::from(playlist.clone()).bg(Color::Black)
                    }
                })
                .collect::<Vec<ListItem>>();
            let list = List::new(items)
                .block(block)
                .style(Style::default().fg(Color::Cyan).bg(Color::Black))
                .highlight_style(Style::new().italic())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true);
            frame.render_stateful_widget(list, area, &mut self.state);
            },
            None => {
                let par = Paragraph::new(self.labels.empty.clone())
                .block(
                    Block::bordered()
                        .title(self.labels.title.as_ref())
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(Style::default().fg(Color::Cyan).bg(Color::Black))
                .centered();
                frame.render_widget(par, area);
            },
        };
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> ListSelectorKeyResponse {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('w') => {
                self.state.select_previous();
                ListSelectorKeyResponse::CursorMoved
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.state.select_next();
                ListSelectorKeyResponse::CursorMoved
            }
            KeyCode::Char('a') => {
                if self.allow_multiple && key_event.modifiers == KeyModifiers::CONTROL {
                    self.selected = (0..self.items.as_ref().unwrap().len()).collect();
                    ListSelectorKeyResponse::Selected
                } else {
                    ListSelectorKeyResponse::Pass
                }
            }
            KeyCode::Enter => {
                if let Some(val) = self.state.selected() {
                    match self.allow_multiple {
                        true => {
                            let pos = self.selected.iter().position(|v| *v == val);
                            match pos {
                                Some(pos) => {
                                    self.selected.swap_remove(pos);
                                },
                                None => {
                                    self.selected.push(val);
                                },
                            }
                        },
                        false => {
                            self.selected.clear();
                            self.selected.push(val);
                        },
                    }
                    ListSelectorKeyResponse::Selected
                } else {
                    ListSelectorKeyResponse::None
                }
            }
            _ => ListSelectorKeyResponse::Pass
        }
    }
}