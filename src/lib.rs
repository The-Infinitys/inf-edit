pub mod app;
pub mod components;
pub mod event_handler;
pub mod ui;
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveTarget {
    Editor,
    PrimarySideBar,
    SecondarySideBar,
    Panel,
}

#[derive(Debug)] // Added Debug
pub struct Tab<T> {
    pub content: T,
    pub title: String,
}
