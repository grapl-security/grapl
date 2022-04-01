mod grapl_logo;
pub mod index;
mod login;
mod r#static;
pub mod plugin_registry;

pub(super) fn config(cfg: &mut actix_web::web::ServiceConfig) {
    grapl_logo::config(cfg);
    index::config(cfg);
    login::config(cfg);
    r#static::config(cfg);
}
