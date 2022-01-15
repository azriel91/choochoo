use std::{
    convert::Infallible,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// Identifiers a physical resource. `String` newtype.
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ResourceIdPhysical(pub String);

impl Deref for ResourceIdPhysical {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResourceIdPhysical {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ResourceIdPhysical {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ResourceIdPhysical {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<ResourceIdPhysical, Infallible> {
        Ok(ResourceIdPhysical(s.to_string()))
    }
}
