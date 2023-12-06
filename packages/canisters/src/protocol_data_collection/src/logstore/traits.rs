pub trait PayloadContent: serde::Serialize {
    fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
