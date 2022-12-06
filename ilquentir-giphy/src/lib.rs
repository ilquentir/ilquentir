use std::sync::Arc;

use color_eyre::Result;
use giphy::v1::{r#async::{AsyncApi, RunnableAsyncRequest}, gifs::RandomRequest};
use reqwest::Client;

/// Giphy API wrapper
#[derive(Clone)]
pub struct GiphyApi {
    api: Arc<AsyncApi>,
}

impl GiphyApi {
    pub fn new(api_key: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .build()?;

        let api = Arc::new(AsyncApi::new(api_key.to_owned(), client));

        Ok(Self { api })
    }

    #[tracing::instrument(skip(self), ret, err)]
    pub async fn get_random_cat_gif(&self) -> Result<url::Url> {
        const CAT_QUERY: &str = "cute cat";

        Ok(RandomRequest::new().with_tag(CAT_QUERY)
            .send_to(&self.api)
            .await?
            .data
            .images
            .looping
            .mp4
            .parse()?)
    }
}

#[cfg(test)]
mod tests {}
