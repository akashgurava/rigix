use rigix_macros::{Model, Request};
use uuid::Uuid;

#[derive(Request)]
struct ValidRequest {
    id: i64,
    uuid: Uuid,
    name: String,
    active: bool,
}

#[derive(Model)]
struct ValidModel {
    id: i64,
    uuid: Uuid,
    name: String,
    active: bool,
}

fn main() {
    let dummy_uuid = Uuid::new_v4();

    let _ = ValidRequest {
        id: 1,
        uuid: dummy_uuid,
        name: "CreateUser".to_string(),
        active: true,
    };

    let _ = ValidModel {
        id: 1,
        uuid: dummy_uuid,
        name: "User".to_string(),
        active: true,
    };
}
