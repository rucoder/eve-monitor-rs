use ratatui::widgets::Paragraph;

use crossterm::event::KeyEvent;
use log::{info, trace};
use ratatui::{
    layout::{self, Constraint, Flex, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget},
    Frame,
};

use crate::traits::{IEventHandler, IFocusAcceptor, IFocusTracker, IPresenter, IVisible, IWindow};

use super::{
    action::{Action, UiActions},
    focus_tracker::FocusTracker,
    tools::centered_rect_fixed,
    widgets::button::ButtonElement,
    window::{LayoutMap, Window},
};

pub struct Dialog<D> {
    name: String,
    focus: FocusTracker,
    size: (u16, u16),
    buttons: Vec<String>,
    state: D,
    layout: LayoutMap,
}

impl<A: 'static, D: 'static + std::fmt::Debug> Dialog<D> {
    pub fn new(size: (u16, u16), buttons: Vec<String>, focused_button: &str, state: D) -> Self {
        // create buttons and add them to the window builder
        for button_name in buttons.iter() {
            let button = ButtonElement::new(button_name);
            w = w.widget(button_name, Box::new(button));
        }

        Self {
            size,
            buttons,
            state,
            layout: LayoutMap::new(),
        }
    }

    fn on_ok_yes<F>(_f: F) -> Option<UiActions>
    where
        F: Fn(&D) -> Option<UiActions>,
    {
        Some(UiActions::ButtonClicked("Ok".to_string()))
    }

    fn do_layout(&mut self, area: &Rect) {
        let dialog_area = centered_rect_fixed(self.size.0, self.size.1, *area);
        self.layout.insert("frame".to_string(), dialog_area);
        // split the dialog area into two parts: content and buttons
        let max_button_len = self.buttons.iter().map(|b| b.len() + 2).max().unwrap_or(0) as u16;
        let num_buttons = self.buttons.len();

        let layout = layout::Layout::horizontal([
            layout::Constraint::Min(0),
            layout::Constraint::Length(max_button_len),
        ])
        .margin(1)
        .split(dialog_area);

        let content_rect = layout[0];
        let buttons_rect = layout[1];

        // split the buttons area into buttons
        let button_layout = layout::Layout::vertical(vec![Constraint::Length(3); num_buttons])
            .flex(Flex::Start)
            .split(buttons_rect);

        for (i, button) in self.buttons.iter().enumerate() {
            self.layout.insert(button.clone(), button_layout[i]);
        }
        self.layout.insert("content".to_string(), content_rect);
    }

    fn render(&self, area: &Rect, frame: &mut Frame<'_>) {
        info!("Rendering dialog content");
        frame.render_widget(Paragraph::new(format!("{0:?}", self.state)), *area);
    }
}

impl<D: 'static> IWindow for Dialog<D> {}

impl<D> IFocusTracker for Dialog<D> {
    fn focus_next(&mut self) -> Option<String> {
        self.focus.focus_next()
    }

    fn focus_prev(&mut self) -> Option<String> {
        self.focus.focus_prev()
    }

    fn get_focused_view_name(&self) -> Option<String> {
        self.focus.get_focused_view()
    }
}

impl<A: 'static, D: 'static> IPresenter for Dialog<D> {
    // fn do_layout(&mut self, area: &Rect) -> HashMap<String, Rect> {
    //     self.do_layout(area);
    //     // get content area and pass it to window
    //     let content_area = self.layout.get("content").unwrap();

    //     self.w.do_layout(&content_area);
    //     HashMap::new()
    // }

    fn render(&mut self, area: &Rect, frame: &mut Frame<'_>) {
        trace!("Rendering dialog: {}", self.w.name);
        self.do_layout(area);
        // render the dialog
        let frame_rect = self.layout.get("frame").unwrap();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black))
            .title(self.w.name.as_str());

        block.render(*frame_rect, frame.buffer_mut());
        // render the buttons
        for button_name in self.buttons.iter() {
            let button_rect = self.layout.get(button_name).unwrap();
            let button = self.w.widgets.get_mut(button_name).unwrap();
            button.render(button_rect, frame);
        }

        // render the content
        let content_area = self.layout.get("content").unwrap();
        self.w.render(content_area, frame);
    }

    fn is_focus_tracker(&self) -> bool {
        true
    }
}

impl<A, D> IFocusAcceptor for Dialog<D> {
    fn has_focus(&self) -> bool {
        // dialog is always focused
        true
    }

    fn set_focus(&mut self) {
        self.w.set_focus();
    }

    fn clear_focus(&mut self) {
        self.w.clear_focus();
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl<D> IVisible for Dialog<D> {}
impl<A, D> IEventHandler for Dialog<D> {
    type Action = A;
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        trace!("Handling key event for dialog: {}", self.w.name);
        // if Escape is pressed then dismiss the dialog
        if key.code == crossterm::event::KeyCode::Esc {
            trace!("Dismissing dialog: {}", self.w.name);
            return Some(Action::new(self.w.name.clone(), UiActions::DismissDialog));
        }

        let action = self.w.handle_key_event(key);

        // if Cancel is clicked then dismiss the dialog otherwise forward action
        if let Some(action) = action {
            match action.action {
                UiActions::ButtonClicked(name) => match name.as_str() {
                    "Cancel" => {
                        return Some(Action::new(self.w.name.clone(), UiActions::DismissDialog))
                    }
                    _ => {
                        //TODO: call custom button handler to update the state
                        return None;
                    }
                },
                _ => {
                    //TODO: call custom button handler to update the state
                    return Some(action);
                }
            }
        } else {
            None
        }
    }
}
