pub mod components;
pub mod ui;
pub mod app;
pub mod event_handler;
#[derive(PartialEq, Clone, Copy, Debug)] // Added Clone, Copy, Debug for convenience
pub enum ActiveTarget {
    Editor,
    Term,
    FileView,
    PrimarySideBar,
    SecondarySideBar,
    Panel,
}

#[derive(Debug)] // Added Debug
pub struct Tab<T> {
    pub content: T,
    pub title: String,
}