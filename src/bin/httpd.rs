use clipshare::data::AppDatabase;
use clipshare::domain::maintenance::Maintenance;
use clipshare::web::renderer::Renderer;
use clipshare::web::views::Views;
use dotenv::dotenv;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "httpd")]
struct Opt {
    #[structopt(
        default_value = "sqlite:data.db",
        help = "connection string to sqlite database"
    )]
    connection_string: String,

    #[structopt(short, long, parse(from_os_str), default_value = "templates/")]
    template_directory: PathBuf,
}

fn main() {
    dotenv().ok();
    let opt = Opt::from_args();

    let rt = tokio::runtime::Runtime::new().expect("failed to spawn tokio runtime");

    let handle = rt.handle().clone();
    let renderer = Renderer::new(opt.template_directory.clone());

    let database = rt.block_on(async move { AppDatabase::new(&opt.connection_string).await });

    let views = Views::new(database.get_pool().clone(), handle.clone());
    let maintenance = Maintenance::spawn(database.get_pool().clone(), handle.clone());

    let config = clipshare::RocketConfig {
        renderer,
        database,
        views,
        maintenance,
    };

    rt.block_on(async move {
        clipshare::rocket(config)
            .launch()
            .await
            .expect("failed to launch rocket server")
    });
}
