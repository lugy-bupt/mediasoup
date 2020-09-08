#[macro_export]
macro_rules! uuid_based_wrapper_type {
    ($struct_name: ident) => {
        #[derive(
            Debug,
            Copy,
            Clone,
            serde::Deserialize,
            serde::Serialize,
            Hash,
            Ord,
            PartialOrd,
            Eq,
            PartialEq,
        )]
        pub struct $struct_name(uuid::Uuid);

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                uuid::Uuid::fmt(&self.0, f)
            }
        }

        impl From<$struct_name> for uuid::Uuid {
            fn from(id: $struct_name) -> Self {
                id.0
            }
        }

        impl $struct_name {
            pub(super) fn new() -> Self {
                $struct_name(uuid::Uuid::new_v4())
            }
        }
    };
}