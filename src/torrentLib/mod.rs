use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;

use types::BasicAuth;
use types::SessionGet;
use types::TorrentAction;
use types::{Id, Torrent, TorrentGetField, Torrents};
use types::{Nothing, Result, RpcRequest, RpcResponse, RpcResponseArgument};
use types::{TorrentAddArgs, TorrentAdded};

pub mod types;

#[derive(Clone)]
pub struct TransClient {
    url: String,
    auth: Option<BasicAuth>,
}

impl TransClient {
    pub fn with_auth(url: &str, basic_auth: BasicAuth) -> TransClient {
        TransClient {
            url: url.to_string(),
            auth: Some(basic_auth),
        }
    }

    fn rpc_request(&self) -> reqwest::RequestBuilder {
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        if let Some(auth) = &self.auth {
            client
                .post(&self.url)
                .basic_auth(&auth.user, Some(&auth.password))
        } else {
            client.post(&self.url)
        }
        .header(CONTENT_TYPE, "application/json")
    }

    async fn get_session_id(&self) -> String {
        let response = self
            .rpc_request()
            .json(&RpcRequest::session_get())
            .send()
            .await;
        let session_id = match response {
            Ok(ref resp) => match resp.headers().get("x-transmission-session-id") {
                Some(res) => res.to_str().expect("header value should be a string"),
                _ => "",
            },
            _ => "",
        }
        .to_owned();
        session_id
    }

    pub async fn session_get(&self) -> Result<RpcResponse<SessionGet>> {
        self.call(RpcRequest::session_get()).await
    }

    pub async fn torrent_get(
        &self,
        fields: Option<Vec<TorrentGetField>>,
        ids: Option<Vec<Id>>,
    ) -> Result<RpcResponse<Torrents<Torrent>>> {
        self.call(RpcRequest::torrent_get(fields, ids)).await
    }

    pub async fn torrent_action(
        &self,
        action: TorrentAction,
        ids: Vec<Id>,
    ) -> Result<RpcResponse<Nothing>> {
        self.call(RpcRequest::torrent_action(action, ids)).await
    }

    pub async fn torrent_remove(
        &self,
        ids: Vec<Id>,
        delete_local_data: bool,
    ) -> Result<RpcResponse<Nothing>> {
        self.call(RpcRequest::torrent_remove(ids, delete_local_data))
            .await
    }

    pub async fn torrent_add(&self, add: TorrentAddArgs) -> Result<RpcResponse<TorrentAdded>> {
        if add.metainfo == None && add.filename == None {
            panic!("Metainfo or Filename should be provided")
        }
        self.call(RpcRequest::torrent_add(add)).await
    }

    async fn call<RS>(&self, request: RpcRequest) -> Result<RpcResponse<RS>>
    where
        RS: RpcResponseArgument + DeserializeOwned + std::fmt::Debug,
    {
        let rq: reqwest::RequestBuilder = self
            .rpc_request()
            .header("X-Transmission-Session-Id", self.get_session_id().await)
            .json(&request);
        let resp: reqwest::Response = rq.send().await?;
        let rpc_response: RpcResponse<RS> = resp.json().await?;
        Ok(rpc_response)
    }
}
