pub mod contracts;
pub mod handlers;

pub use contracts::{
    CityResponse, CityWithStateResponse, CountryResponse, StateResponse, StateWithCountryResponse,
};

use crate::infra::state::AppState;
use axum::{routing::get, Router};

/// Creates the geo_regions router with all Country, State, and City routes
pub fn router() -> Router<AppState> {
    let countries = Router::new()
        .route(
            "/",
            get(handlers::list_countries).post(handlers::create_country),
        )
        .route(
            "/{id}",
            get(handlers::get_country)
                .put(handlers::update_country)
                .delete(handlers::delete_country),
        );

    let states = Router::new()
        .route("/", get(handlers::list_states).post(handlers::create_state))
        .route(
            "/{id}",
            get(handlers::get_state)
                .put(handlers::update_state)
                .delete(handlers::delete_state),
        );

    let cities = Router::new()
        .route("/", get(handlers::list_cities).post(handlers::create_city))
        .route(
            "/{id}",
            get(handlers::get_city)
                .put(handlers::update_city)
                .delete(handlers::delete_city),
        );

    Router::new()
        .nest("/countries", countries)
        .nest("/states", states)
        .nest("/cities", cities)
}
