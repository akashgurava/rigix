mod definitions {
    use rigix_macros::{Model, Request};
    use uuid::Uuid;

    #[derive(Request)]
    pub struct CreateAccount {
        id: i64,
        uuid: Uuid,
        name: String,
        active: bool,
        balance: f64,
        tags: Vec<String>,
    }

    impl CreateAccount {
        pub fn new(
            id: i64,
            uuid: Uuid,
            name: String,
            active: bool,
            balance: f64,
            tags: Vec<String>,
        ) -> Self {
            Self {
                id,
                uuid,
                name,
                active,
                balance,
                tags,
            }
        }
    }

    #[derive(Model, Clone, PartialEq, Debug)]
    pub struct Account {
        id: i64,
        uuid: Uuid,
        name: String,
        balance: f64,
    }

    impl Account {
        pub fn new(id: i64, uuid: Uuid, name: String, balance: f64) -> Self {
            Self {
                id,
                uuid,
                name,
                balance,
            }
        }
    }

    #[derive(Model)]
    pub struct FamilyDetails {
        id: i64,
        family_name: String,
        accounts: Vec<Account>,
    }

    impl FamilyDetails {
        pub fn new(id: i64, family_name: String, accounts: Vec<Account>) -> Self {
            Self {
                id,
                family_name,
                accounts,
            }
        }
    }
}

use definitions::{Account, CreateAccount, FamilyDetails};
use uuid::Uuid;

fn main() {
    let dummy_uuid = Uuid::new_v4();

    // 1. Verify Request getters and into_parts
    let req = CreateAccount::new(
        101,
        dummy_uuid,
        "Savings".to_string(),
        true,
        500.25,
        vec!["emergency".to_string(), "personal".to_string()],
    );

    let id: i64 = req.id();
    assert_eq!(id, 101);

    let uuid_val: Uuid = req.uuid();
    assert_eq!(uuid_val, dummy_uuid);

    let name: &str = req.name();
    assert_eq!(name, "Savings");

    let active: bool = req.active();
    assert!(active);

    let balance: f64 = req.balance();
    assert!((balance - 500.25).abs() < f64::EPSILON);

    let tags: &Vec<String> = req.tags();
    assert_eq!(tags.len(), 2);
    assert_eq!(&tags[0], "emergency");

    let (req_id, req_uuid, req_name, req_active, req_bal, req_tags) = req.into_parts();
    assert_eq!(req_id, 101);
    assert_eq!(req_uuid, dummy_uuid);
    assert_eq!(req_name, "Savings");
    assert!(req_active);
    assert!((req_bal - 500.25).abs() < f64::EPSILON);
    assert_eq!(req_tags, vec!["emergency".to_string(), "personal".to_string()]);

    // 2. Verify Model getters and into_parts (including nested models)
    let acc1 = Account::new(1, dummy_uuid, "Checking".to_string(), 1200.0);
    let acc2 = Account::new(2, dummy_uuid, "Vault".to_string(), 8500.0);

    let acc_id: i64 = acc1.id();
    assert_eq!(acc_id, 1);
    let acc_name: &str = acc1.name();
    assert_eq!(acc_name, "Checking");
    let acc_bal: f64 = acc1.balance();
    assert!((acc_bal - 1200.0).abs() < f64::EPSILON);

    let family = FamilyDetails::new(50, "Miller Family".to_string(), vec![acc1.clone(), acc2.clone()]);
    assert_eq!(family.id(), 50);
    assert_eq!(family.family_name(), "Miller Family");
    assert_eq!(family.accounts().len(), 2);
    assert_eq!(family.accounts()[0], acc1);
    assert_eq!(family.accounts()[1], acc2);

    let (fam_id, fam_name, fam_accounts) = family.into_parts();
    assert_eq!(fam_id, 50);
    assert_eq!(fam_name, "Miller Family");
    assert_eq!(fam_accounts.len(), 2);
    assert_eq!(fam_accounts[0], acc1);
}
