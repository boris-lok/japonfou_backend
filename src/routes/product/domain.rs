use crate::routes::product_id_generator;

#[derive(serde::Deserialize, Debug)]
pub struct CreateProductRequest {
    pub name: String,
    pub currency: i16,
    pub price: f64,
}

pub struct NewProduct {
    pub id: i64,
    pub name: String,
    pub currency: i16,
    pub price: f64,
}

impl NewProduct {
    pub async fn parse(req: CreateProductRequest) -> Result<Self, String> {
        let id = async {
            let generator = product_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        Ok(Self {
            id,
            name: req.name,
            currency: req.currency,
            price: req.price,
        })
    }
}
