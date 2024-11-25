pub mod app;
pub mod args;
mod cmd;
mod config;
mod messages;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
