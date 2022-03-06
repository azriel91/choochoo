use std::{
    convert::Infallible,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// Identifies a resource logically. `String` newtype.
///
/// A logical resource ID is defined by code, and does not change.
///
/// A physical resource ID is one generated during execution, which generally is
/// random or computed.
///
/// Examples of logical IDs and corresponding physical IDs:
///
/// | Logical ID               | Physical ID                            |
/// | ------------------------ | -------------------------------------- |
/// | `app_server_instance_id` | `ef34a9a4-0c02-45a6-96ec-a4db06d4980c` |
/// | `app_server.address`     | `10.0.0.1`                             |
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ResIdLogical(pub String);

impl Deref for ResIdLogical {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResIdLogical {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ResIdLogical {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ResIdLogical {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<ResIdLogical, Infallible> {
        Ok(ResIdLogical(s.to_string()))
    }
}
