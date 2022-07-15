#[derive(clap::Parser, Debug)]
#[clap(name = "lens-creator", about = "Lens Creator Service")]
pub struct LensCreatorConfig {
    #[clap(env="LENS_MANAGER_CLIENT_URL")]
    pub lens_manager_client_url: String,
}
