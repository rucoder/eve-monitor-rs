use crate::{
    model::device::network::NetworkInterfaceStatus,
    traits::{IPresenter, IWindow},
    ui::ipdialog::create_ip_dialog,
};
use core::fmt::Debug;
use crossterm::event::{KeyCode, KeyModifiers};
use log::{debug, info, warn};
use ratatui::{
    layout::{
        Constraint::{self, Fill, Length},
        Layout, Rect,
    },
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, Clear, Tabs, Widget},
};
use std::rc::Rc;
use strum::{Display, EnumCount, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    events::Event,
    model::{device::dmesg::DmesgViewer, model::Model},
    terminal::TerminalWrapper,
    traits::IEventHandler,
    ui::action::UiActions,
};

use super::{
    action::Action,
    app_page::ApplicationsPage,
    homepage::HomePage,
    layer_stack::LayerStack,
    networkpage::create_network_page,
    statusbar::{create_status_bar, StatusBarState},
    summary_page::SummaryPage,
    window::Window,
};

use std::result::Result::Ok;

use anyhow::Result;

pub struct Ui {
    pub terminal: TerminalWrapper,
    pub action_tx: UnboundedSender<Action>,
    pub views: Vec<LayerStack>,
    pub selected_tab: UiTabs,
    pub status_bar: Window<StatusBarState>,
    first_frame: bool,
}

#[derive(Default, Copy, Clone, Display, EnumIter, Debug, FromRepr, EnumCount)]
pub enum UiTabs {
    #[default]
    //Debug,
    Summary,
    Home,
    Network,
    Applications,
    Dmesg,
}

impl Debug for Ui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ui :)")
    }
}

impl Ui {
    pub fn new(action_tx: UnboundedSender<Action>, terminal: TerminalWrapper) -> Result<Self> {
        Ok(Self {
            terminal,
            action_tx,
            views: vec![LayerStack::new(); UiTabs::COUNT],
            selected_tab: UiTabs::default(),
            status_bar: create_status_bar(),
            first_frame: true,
        })
    }

    fn tabs() -> Tabs<'static> {
        let tab_titles = UiTabs::iter().map(UiTabs::to_tab_title);
        let block = Block::new().title(" Use ctrl + ◄ ► to change tab");
        Tabs::new(tab_titles)
            .block(block)
            .highlight_style(Modifier::REVERSED)
            .divider(" ")
            .padding("", "")
    }

    pub fn init(&mut self) {
        // let w = self.create_main_wnd();

        //self.views[UiTabs::Debug as usize].push(Box::new(w));

        // let s = IpDialogState {
        //     ip: "10.208.13.10".to_string(),
        //     mode: "DHCP".to_string(),
        //     gw: "1.1.1.1".to_string(),
        // };

        // let d: Dialog<MonActions> = Dialog::new(
        //     (50, 20),
        //     "confirm",
        //     vec!["Ok", "Cancel"],
        //     "Cancel",
        //     MonActions::NetworkInterfaceUpdated(s),
        // );

        self.views[UiTabs::Summary as usize].push(Box::new(SummaryPage::new()));
        self.views[UiTabs::Home as usize].push(Box::new(HomePage::new()));

        // self.views[UiTabs::Home as usize].push(Box::new(d));

        self.views[UiTabs::Network as usize].push(Box::new(create_network_page()));

        self.views[UiTabs::Applications as usize].push(Box::new(ApplicationsPage::new()));
        self.views[UiTabs::Dmesg as usize].push(Box::new(DmesgViewer::new()));
    }

    pub fn draw(&mut self, model: Rc<Model>) {
        let screen_layout = Layout::vertical([Length(3), Fill(0), Length(3)]);
        let tabs_widget = Ui::tabs();

        //TODO: handle terminal event
        let _ = self.terminal.draw(|frame| {
            let area = frame.size();
            let [tabs, body, statusbar_rect] = screen_layout.areas(area);

            if self.first_frame {
                self.first_frame = false;
                frame.render_widget(Clear, area);
            }
            tabs_widget
                .select(self.selected_tab as usize)
                .render(tabs, frame.buffer_mut());

            // redraw from the bottom up
            let stack = &mut self.views[self.selected_tab as usize];
            let last_index = stack.len().saturating_sub(1);
            for (index, layer) in stack.iter_mut().enumerate() {
                layer.render(&body, frame, &model, index == last_index);
            }
            // render status bar
            self.status_bar
                .render(&statusbar_rect, frame, &model, false);
        });
    }

    fn invalidate(&mut self) {
        self.action_tx
            .send(Action::new("app", UiActions::Redraw))
            .unwrap();
    }

    pub fn handle_event(&mut self, event: Event) -> Option<Action> {
        if event != Event::Tick {
            debug!("Ui handle_event {:?}", event);
        }

        match event {
            // only for debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('e')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+q: application Quit requested");
                self.action_tx
                    .send(Action::new("user", UiActions::Quit))
                    .unwrap();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('r')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+r: manual Redraw requested");
                self.invalidate();
            }
            // For debugging purposes
            Event::Key(key)
                if (key.code == KeyCode::Char('p')) && (key.modifiers == KeyModifiers::CONTROL) =>
            {
                debug!("CTRL+p: manual layer.pop() requested");
                self.pop_layer();
            }

            Event::Key(key)
                if (key.code == KeyCode::Char('a'))
                    && (key.modifiers == KeyModifiers::CONTROL)
                    && cfg!(debug_assertions) =>
            {
                debug!("CTRL+a: manual panic requested");
                panic!("Manual panic requested");
            }
            // show dialog on ctrl+d
            // Event::Key(key)
            //     if (key.code == KeyCode::Char('d')) && (key.modifiers == KeyModifiers::CONTROL) =>
            // {
            //     debug!("CTRL+d: show dialog");

            //     // let s = IpDialogState {
            //     //     ip: "10.208.13.10".to_string(),
            //     //     mode: "DHCP".to_string(),
            //     //     gw: "1.1.1.1".to_string(),
            //     // };

            //     // let d: Dialog<MonActions> = Dialog::new(
            //     //     (50, 30),
            //     //     "confirm".to_string(),
            //     //     vec!["Ok".to_string(), "Cancel".to_string()],
            //     //     "Cancel",
            //     //     MonActions::NetworkInterfaceUpdated(s),
            //     // );

            //     let d = create_ip_dialog();
            //     self.push_layer(d);
            // }

            // show network edit dialog on ctrl+e
            // Event::Key(key)
            //     if (key.code == KeyCode::Char('e')) && (key.modifiers == KeyModifiers::CONTROL) =>
            // {
            //     debug!("CTRL+e: show dialog");

            //     // let s = IpDialogState {
            //     //     ip: "10.208.13.10".to_string(),
            //     //     mode: "DHCP".to_string(),
            //     //     gw: "1.1.1.1".to_string(),
            //     // };

            //     // let d: NetworkDialog = NetworkDialog::new();
            //     // self.views[self.selected_tab as usize].push(Box::new(d));
            // }

            // handle Tab switching
            // Event::Key(key)
            //     if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left) =>
            // {
            //     debug!("CTRL+Left: switching tab view");
            //     self.selected_tab = self.selected_tab.previous();
            // }
            // Event::Key(key)
            //     if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right) =>
            // {
            //     debug!("CTRL+Right: switching tab view");
            //     self.selected_tab = self.selected_tab.next();
            // }

            // forward all other key events to the top layer
            Event::Key(key) => {
                if let Some(action) = self.views[self.selected_tab as usize]
                    .last_mut()?
                    .handle_event(Event::Key(key))
                {
                    match action.action {
                        UiActions::DismissDialog => {
                            self.pop_layer();
                        }

                        UiActions::ButtonClicked(name) => match name.as_str() {
                            "Ok" => {
                                self.pop_layer();
                            }
                            "Cancel" => {
                                self.pop_layer();
                            }
                            _ => {}
                        },

                        _ => {
                            return Some(action);
                        }
                    }
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Left {
                    debug!("CTRL+Left: switching tab view");
                    self.selected_tab = self.selected_tab.previous();
                }

                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Right {
                    debug!("CTRL+Right: switching tab view");
                    self.selected_tab = self.selected_tab.next();
                }
            }
            Event::Tick => {
                // forward tick event to all layers. Collect actions
                for layer in self.views[self.selected_tab as usize].iter_mut() {
                    if let Some(action) = layer.handle_event(Event::Tick) {
                        self.action_tx.send(action).unwrap();
                    }
                }
                // and to the status bar
                self.status_bar.handle_event(Event::Tick);
            }
            _ => {
                debug!("Unhandled event: {:?}", event);
            }
        }

        None
    }

    fn push_layer(&mut self, d: impl IWindow + 'static) {
        self.views[self.selected_tab as usize].push(Box::new(d))
    }

    fn pop_layer(&mut self) -> Option<Box<dyn IWindow>> {
        self.views[self.selected_tab as usize].pop()
    }

    pub fn show_ip_dialog(&mut self, iface: NetworkInterfaceStatus) {
        let d = create_ip_dialog(&iface);
        self.push_layer(d);
    }
}

impl UiTabs {
    fn to_tab_title(self) -> Line<'static> {
        let text = self.to_string();
        format!(" {text} ").bg(Color::Black).into()
    }

    /// Get the previous tab, if there is no previous tab return the current tab.
    fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}
