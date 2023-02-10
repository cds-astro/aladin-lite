use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CmapLabel(String);

impl AsRef<str> for CmapLabel {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
