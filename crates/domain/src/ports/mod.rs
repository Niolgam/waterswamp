pub mod auth;
pub mod departments; // New module - not yet exported to avoid conflicts
pub mod email;
pub mod facilities; // New module - not yet exported to avoid conflicts
pub mod geo_regions; // New module - not yet exported to avoid conflicts
pub mod location; // Legacy - will be removed after full migration
pub mod mfa;
pub mod user;

pub use auth::*;
// pub use departments::*; // Commented out until migration is complete
pub use email::*;
// pub use facilities::*; // Commented out until migration is complete
// pub use geo_regions::*; // Commented out until migration is complete
pub use location::*; // Legacy - will be removed after full migration
pub use mfa::*;
pub use user::*;
