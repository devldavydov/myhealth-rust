use anyhow::Result;

pub trait Service {
    fn run(&mut self) -> Result<()>;
}
