use std::sync::Arc;

use color_eyre::{eyre::eyre, Result};
use giphy_api::Client;

/// Giphy API wrapper
#[derive(Clone)]
pub struct GiphyApi {
    api: Arc<Client>,
}

impl GiphyApi {
    pub fn new(api_key: &str) -> Result<Self> {
        let client = Client::new(api_key);

        let api = Arc::new(client);

        Ok(Self { api })
    }

    #[tracing::instrument(skip(self), ret, err)]
    pub async fn get_random_cat_gif(&self) -> Result<url::Url> {
        const CAT_QUERY: &str = "cute cat";

        Ok(self
            .api
            .gifs()
            .random(CAT_QUERY, "PG")
            .await
            .map_err(|e| eyre!("failed to get GIF: {e:?}"))?
            .data
            .and_then(|gif| {
                gif.images
                    .and_then(|images| images.looping.map(|looping| looping.image.mp_4))
            })
            .ok_or_else(|| eyre!("no looping image in response"))?
            .parse()?)
    }
}

#[cfg(test)]
mod tests {}
