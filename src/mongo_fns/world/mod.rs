pub mod person;
pub mod village;

#[macro_export]
macro_rules! model {
    ($sv:vis struct $name:ident { $($fv:vis $fname:ident : $ftype:ty),* } ) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        $sv struct $name {
            #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
            id: Option<mongodb::bson::oid::ObjectId>,
            $($fv $fname: $ftype,)*
        }

        impl $name {
            #[allow(dead_code)]
            $sv fn new($($fname: $ftype,)*) -> Self {
                $name {
                    id: None,
                    $($fname,)*
                }
            }

            #[allow(dead_code)]
            pub fn get_id(&self) -> &Option<mongodb::bson::oid::ObjectId> {
                &self.id
            }
        }
    }
}

pub use model;
