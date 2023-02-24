use anyhow::Result;

use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use crate::ruft_proto::{
    node_server::{
        Node, NodeServer,
    },
    AppendEntriesRequest,
    AppendEntriesResponse,
    RequestVoteRequest,
    RequestVoteResponse, self,
};

#[derive(Debug, Default)]
pub struct NodeService { }

impl NodeService {
    pub async fn serve(address: impl AsRef<str>) -> Result<()> {
        println!("serving: {}", address.as_ref());
        Server::builder()
            .add_service(NodeServer::new(NodeService::default()))
            .serve(address.as_ref().parse()?)
            .await?;
        println!("closing: {}", address.as_ref());
        Ok(())
    }
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        Ok(Response::new(AppendEntriesResponse { 
            // TODO: logic
            message: request.get_ref().message.clone()
        }))
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        Ok(Response::new(RequestVoteResponse {
            // TODO: logic
            message: request.get_ref().message.clone()
        }))
    }
}