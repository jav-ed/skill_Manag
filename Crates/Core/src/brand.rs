//! Every product-specific on-disk or environment name lives here, so a rename touches one file.

/// Binary and crate name.
pub const NAME: &str = "skillmirror";

/// Directory below the user's config home that holds the vault pointer.
pub const CONFIG_DIR: &str = "skillmirror";

/// Environment variable that overrides the vault path.
pub const ENV_VAULT: &str = "SKILLMIRROR_VAULT";

/// Environment variable that overrides the scan root.
pub const ENV_ROOT: &str = "SKILLMIRROR_ROOT";
