//! MOTORI AUTO-EDIT: kombinon beat map-in e muzikës me highlight-et e videove
//! dhe gjeneron një timeline të plotë ku çdo prerje bie saktësisht mbi beat.

pub mod engine;
pub mod styles;

pub use engine::{run, TimelineClip};
pub use styles::{EditStyle, all_styles, style_by_id};
