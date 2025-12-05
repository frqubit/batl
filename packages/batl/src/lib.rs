//! # Battalion
//! 
//! Battalion is a CLI tool for managing codebase relationships. It uses a simple heirarchy of **repositories** and **workspaces** to link codebases together when needed, and keep them separate when not.
//! 
//! ## Installation
//! 
//! ```bash
//! cargo install batl
//! batl setup
//! 
//! # (optional) Install batlas
//! batl repository fetch battalion/batlas
//! batl repository exec -n battalion/batlas build
//! batl repository exec -n battalion/batlas install
//! ```
//! 
//! ## Usage
//! 
//! ```bash
//! # Create a new repository
//! batl repository init prototypes/awesome-project
//! 
//! # Create a new workspace
//! batl workspace init --ref prototypes/awesome-project
//! 
//! # Create a library
//! batl repository init prototypes/awesome-library
//! 
//! # cd into the workspace
//! cd $(batl workspace which prototypes/awesome-project)
//! 
//! # ...or if you use batlas with VSCode...
//! batlas prototypes/awesome-project code %!
//! 
//! # create a link while in directory of workspace
//! batl link init -n library prototypes/awesome-library
//! 
//! # Start building!
//! ```


#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

// Current requirement, might fix later idk
#![allow(clippy::multiple_crate_versions, reason = "required, may fix")]
#![allow(clippy::min_ident_chars, reason = "rule is overly noisy, may reenable in future")]

// Remove clippy contradictions here
// #![allow(clippy::blanket_clippy_restriction_lints)]
#![allow(clippy::implicit_return, reason = "implicit return is repo convention")]
#![allow(clippy::self_named_module_files, reason = "self named module is repo convention")]
#![allow(clippy::unseparated_literal_suffix, reason = "no underscore suffix is repo convention")]
// #![allow(clippy::pub_with_shorthand)]
#![allow(clippy::question_mark_used, reason = "err? is repo convention")]
#![allow(clippy::absolute_paths, reason = "abspaths not against repo convention")]
#![allow(clippy::pub_use, reason = "required for library")]


pub mod error;
pub mod system;
pub mod resource;
pub mod version;
