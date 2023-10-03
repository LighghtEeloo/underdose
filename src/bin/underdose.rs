// 0. Configuration (done)
// 1. Perfect State Executor (Executor) (done)
// 2. Perfect State Detective
// 3. Garbage Collector (Dreamer) (done)

fn main() -> anyhow::Result<()> {
    env_logger::init();
    underdose::Cli::new().main()?;
    Ok(())
}
