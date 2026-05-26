use anyhow::Result;

use aisess::app;

fn main() -> Result<()> {
    let app = app::App::new();
    app.run()
}
