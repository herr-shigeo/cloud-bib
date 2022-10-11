use crate::{
    error::BibErrorResponse,
    item::{search_items, RentalSetting},
};
use mongodb::Database;

pub async fn get_rental_setting(db: &Database) -> Result<RentalSetting, BibErrorResponse> {
    let mut setting = RentalSetting::default();
    setting.id = 1;
    let mut setting = match search_items(&db, &setting).await {
        Ok(setting) => setting,
        Err(e) => {
            return Err(BibErrorResponse::DataNotFound(e.to_string()));
        }
    };

    if setting.len() != 1 {
        return Err(BibErrorResponse::DataDuplicated);
    }
    let setting = setting.pop().unwrap();
    Ok(setting)
}
