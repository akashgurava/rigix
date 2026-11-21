mod definitions {
    use rigix_macros::{Request, Response};
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

    #[derive(Response)]
    pub struct UserResponse {
        id: i64,
        uuid: Uuid,
        name: String,
    }

    impl UserResponse {
        pub fn new(id: i64, uuid: Uuid, name: String) -> Self {
            Self { id, uuid, name }
        }
    }
}

use definitions::{UserRequest, UserResponse};
use uuid::Uuid;

fn main() {
    let dummy_uuid = Uuid::new_v4();
    let mut req = UserRequest::new(1, dummy_uuid, "Alice".to_string());
    let mut res = UserResponse::new(2, dummy_uuid, "Bob".to_string());

    // 1. Direct private field access must fail
    let _ = req.id;
    let _ = res.id;

    // 2. Direct field mutation must fail
    req.name = "Charlie".to_string();
    res.name = "Dave".to_string();

    // 3. Calling non-existent getters must fail
    req.non_existent();
    res.non_existent();

    // 4. Use of struct after into_parts consumption must fail (ownership consumed)
    let _ = req.into_parts();
    let _ = req.name();

    let _ = res.into_parts();
    let _ = res.name();
}
