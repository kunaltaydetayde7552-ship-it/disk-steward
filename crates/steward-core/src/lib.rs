//! Core of Disk Steward: measures dev-machine disk usage (clone-aware) and drives
//! `auto-compress-apfs` (APFS transparent compression).

pub mod catalog;
pub mod disk;
pub mod docker;
pub mod engine;
pub mod measure;
pub mod model;
pub mod overview;
pub mod risk;
pub mod schedule;
pub mod util;
