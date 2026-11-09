use rigix_macros::{Model, Request};

mod test_module {
    use super::*;

    #[derive(Request)]
    struct SinglePubRequest {
        pub id: i64,
        name: String,
    }

    #[derive(Request)]
    struct MultiPubRequest {
        pub standard_pub: i64,
        pub(crate) crate_pub: String,
        pub(super) super_pub: bool,
        pub(in crate::test_module) path_pub: u32,
        private_field: String,
    }

    #[derive(Model)]
    struct SinglePubModel {
        pub id: i64,
        name: String,
    }

    #[derive(Model)]
    struct MultiPubModel {
        pub standard_pub: i64,
        pub(crate) crate_pub: String,
        pub(super) super_pub: bool,
        pub(in crate::test_module) path_pub: u32,
        private_field: String,
    }
}

fn main() {}
