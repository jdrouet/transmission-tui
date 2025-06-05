use color_eyre::Result;

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    set_panic_hook();

    let terminal = ratatui::init();
    let client = transmission_rpc::TransClient::new(
        std::env::var("TRANSMISSION_URL").unwrap().parse().unwrap(),
    );

    let res = transmission_tui::Application::new(client)
        .run(terminal)
        .await;
    ratatui::restore();

    res
}
