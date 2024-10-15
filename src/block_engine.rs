use std::str::FromStr;

use tonic::codegen::tokio_stream::Stream;
use tonic::transport::Channel;

use crate::proto;

use crate::proto::auth::Token;
use crate::proto::block_engine::block_engine_validator_client::BlockEngineValidatorClient;
use crate::proto::block_engine::SubscribeBundlesRequest;
use crate::proto::dto::MempoolPacket;

pub struct MevtonBlockEngine {
    block_engine_client: BlockEngineValidatorClient<Channel>,
    access_token: Option<Token>,
}

impl MevtonBlockEngine {
    pub async fn new(block_engine_url: &str, access_token: Token) -> Result<Self, Box<dyn std::error::Error>> {
        let block_engine_client = BlockEngineValidatorClient::connect(block_engine_url.to_string()).await?;

        Ok(Self {
            block_engine_client,
            access_token: Some(access_token),
        })
    }

    pub async fn stream_mempool(
        &mut self,
        stream: impl Stream<Item = MempoolPacket> + Send + 'static
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut request = tonic::Request::new(stream);

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(
                    &format!("Bearer {}", access_token.value)
                )?,
            );
        }

        self.block_engine_client.stream_mempool(request).await?;

        Ok(())
    }

    pub async fn subscribe_bundles<F>(
        &mut self,
        on_data: F,
    ) -> Result<(), Box<dyn std::error::Error>>
        where
            F: Fn(proto::dto::Bundle) + Send + 'static,
    {
        let mut request = tonic::Request::new(SubscribeBundlesRequest {});

        if let Some(access_token) = &self.access_token {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::from_str(
                    &format!("Bearer {}", access_token.value)
                )?,
            );
        }

        let mut stream = self.block_engine_client.subscribe_bundles(request).await?.into_inner();

        tokio::spawn(async move {
            while let Some(response) = stream.message().await.unwrap_or(None) {
                on_data(response);
            }
        });

        Ok(())
    }
}
