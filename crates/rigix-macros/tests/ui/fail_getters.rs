mod definitions {
    use rigix_macros::{Model, Request};
    use uuid::Uuid;

    #[derive(Request)]
    pub struct UserRequest {
        id: i64,
        uuid: Uuid,
        name: String,
    }

    impl UserRequest {
        pub fn new(id: i64, uuid: Uuid, name: String) -> Self {
            Self { id, uuid, name }
        }
    }

    #[derive(Model)]
    pub struct UserModel {
        id: i64,
        uuid: Uuid,
        name: String,
    }

    impl UserModel {
        pub fn new(id: i64, uuid: Uuid, name: String) -> Self {
            Self { id, uuid, name }
        }
    }
}

use definitions::{UserModel, UserRequest};
use uuid::Uuid;

fn main() {
    let dummy_uuid = Uuid::new_v4();
    let mut req = UserRequest::new(1, dummy_uuid, "Alice".to_string());
    let mut model = UserModel::new(2, dummy_uuid, "Bob".to_string());

    // 1. Direct private field access must fail
    let _ = req.id;
    let _ = model.id;

    // 2. Direct field mutation must fail
    req.name = "Charlie".to_string();
    model.name = "Dave".to_string();

    // 3. Calling non-existent getters must fail
    req.non_existent();
    model.non_existent();

    // 4. Use of struct after into_parts consumption must fail (ownership consumed)
    let _ = req.into_parts();
    let _ = req.name();

    let _ = model.into_parts();
    let _ = model.name();
}
