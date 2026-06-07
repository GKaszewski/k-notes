mod auth;
mod data;
mod notes;
mod tags;

use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

fn build() -> utoipa::openapi::OpenApi {
    let mut api = auth::AuthDoc::openapi();
    api.info = utoipa::openapi::InfoBuilder::new()
        .title("k-notes API")
        .version("1.0.0")
        .description(Some(
            "Self-hosted note-taking API. \
             Authenticate with `POST /api/v1/auth/login` to receive a Bearer token.",
        ))
        .build();

    api.merge(notes::NotesDoc::openapi());
    api.merge(tags::TagsDoc::openapi());
    api.merge(data::DataDoc::openapi());
    SecurityAddon.modify(&mut api);
    api
}

pub fn serve(router: Router) -> Router {
    tracing::info!("API docs available at /docs (Swagger) and /scalar");
    let spec = build();
    router
        .merge(SwaggerUi::new("/docs").url("/openapi.json", spec.clone()))
        .merge(Scalar::with_url("/scalar", spec))
}
