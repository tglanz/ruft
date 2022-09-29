use std::cmp;
use std::marker::PhantomData;
use thiserror::Error;

use crate::model::*;

#[derive(Error, Debug)]
pub enum GenericError {
    #[error("{0}")]
    Generic(String),
}

pub trait BlobStorage {
    fn save(&mut self, id: String, blob: Vec<u8>) -> Result<(), GenericError>;
    fn load(&mut self, id: String) -> Result<Vec<u8>, GenericError>;
}

pub trait Rpc {
    fn send_recieve<M, R>(&mut self, message: M) -> Result<R, GenericError>;
    fn broadcast<M>(&mut self, message: M) -> Result<(), GenericError>;
}

pub enum ServerState {
    Leader,
    Follower,
    Candidate,
}

pub trait StateMachine<C: Command> {
    fn apply(&mut self, command: C) -> Result<(), GenericError>;
}


pub struct Server<B: BlobStorage, C: Command, S: StateMachine<C>> {
    state: ServerState,

    persistent_server_state: PersistentServerState<C>,
    volatile_server_state: VolatileServerState,
    volatile_leader_state: Option<VolatileLeaderState>,

    blob_storage: B, 
    state_machine: S,
}

impl<B: BlobStorage, C: Command, S: StateMachine<C>> Server<B, C, S> {
    pub fn create(blob_storage: B, state_machine: S) -> Result<Self, GenericError> {
        let volatile_server_state = VolatileServerState {
            commit_index: 0,
            last_applied: 0,
        };

        // TODO: load from blob storage
        let persistent_server_state = PersistentServerState {
            current_term: 0,
            log: vec![],
            voted_for: 0,
        };

        Ok(Server {
            state: ServerState::Follower,
            persistent_server_state,
            volatile_server_state,
            volatile_leader_state: None,
            blob_storage,
            state_machine,
        })
    }

    fn convert_to(&mut self, state: ServerState) {
        // TODO
    }

    fn apply_to_state_machine(&mut self, command: C) {
        self.state_machine.apply(command).unwrap();
    }

    fn receive_command(&mut self, command: C) {

    }
}

pub trait HasTerm {
    fn get_term(&self) -> TermIndex;
}

macro_rules! implement_has_term_trait {
    ($id:ty) => {
        impl HasTerm for $id {
            fn get_term(&self) -> TermIndex {
                self.term
            }
        }
    };
}

// implement_has_term_trait!(AppendEntriesRpcArguments<C>);
impl<C: Command> HasTerm for AppendEntriesRpcArguments<C> {
    fn get_term(&self) -> TermIndex {
        self.term
    }
}

implement_has_term_trait!(AppendEntriesRpcResult);
implement_has_term_trait!(RequestVoteRpcArguments);
implement_has_term_trait!(RequestVoteRpcResult);

pub struct AllServersRules;

impl AllServersRules {
    fn apply_next_entry_if_needed<B, C, S>(server: &mut Server<B, C, S>)
    where
        B: BlobStorage,
        C: Command,
        S: StateMachine<C>,
    {
        if server.volatile_server_state.commit_index > server.volatile_server_state.last_applied {
            if server.volatile_server_state.commit_index > server.volatile_server_state.last_applied
            {
                server.volatile_server_state.last_applied += 1;
                let command = server.persistent_server_state.log
                    [server.volatile_server_state.last_applied]
                    .command
                    .clone();
                server.apply_to_state_machine(command);
            }
        }
    }

    fn convert_to_follower_if_needed<H, B, C, S>(server: &mut Server<B, C, S>, has_term: H)
    where
        H: HasTerm,
        B: BlobStorage,
        C: Command,
        S: StateMachine<C>,
    {
        if server.persistent_server_state.current_term < has_term.get_term() {
            server.persistent_server_state.current_term = has_term.get_term();
            server.convert_to(ServerState::Follower);
        }
    }
}

pub struct FollowerRules;

impl FollowerRules {
    /// TODO: if election timeout elapses without receiving AppendEntries RPC from current leader
    /// or granting vote to candidate: convert to candidate
    fn todo() {}
}

struct AppendEntriesRpcLogic;

impl AppendEntriesRpcLogic {
    fn receive_append_entries_rpc<B, C, S>(
        server: &mut Server<B, C, S>,
        arguments: AppendEntriesRpcArguments<C>,
    ) -> AppendEntriesRpcResult
    where
        B: BlobStorage,
        C: Command,
        S: StateMachine<C>,
    {
        // Induction validating the log matching property
        let is_invalid = [
            arguments.term < server.persistent_server_state.current_term,
            server.persistent_server_state.log.len() < arguments.prev_log_index,
            server.persistent_server_state.log[arguments.prev_log_index].term
                != arguments.prev_log_term,
        ]
        .into_iter()
        .any(|x| x);

        if is_invalid {
            return AppendEntriesRpcResult {
                term: 0,
                success: false,
            };
        }

        // sort entries for somewhat efficient checks
        let mut entries = arguments.entries.clone();
        entries.sort_by_key(|entry| entry.term);

        // trim all local entries that the leader isn't aware of
        for entry in entries.iter() {
            if server.persistent_server_state.log.len() < entry.index {
                break;
            }

            if server.persistent_server_state.log[entry.index].term != entry.term {
                while server.persistent_server_state.log.len() >= entry.index {
                    server.persistent_server_state.log.pop();
                }
                break;
            }
        }

        // replicate new leader's entries
        for entry in entries.iter() {
            server
                .persistent_server_state
                .log
                .insert(entry.index, entry.clone());
        }

        // update volatile state
        if arguments.leader_commit > server.volatile_server_state.commit_index {
            server.volatile_server_state.commit_index = cmp::min(
                arguments.leader_commit,
                entries
                    .last()
                    .map_or(arguments.leader_commit, |entry| entry.index),
            );
        }

        return AppendEntriesRpcResult {
            term: 0,
            success: true,
        };
    }
}

struct RequestVoteRpcLogic;

impl RequestVoteRpcLogic {
    fn receive_request_vote_rpc<B, C, S>(
        server: &mut Server<B, C, S>,
        arguments: RequestVoteRpcArguments,
    ) -> RequestVoteRpcResult
    where
        B: BlobStorage,
        C: Command,
        S: StateMachine<C>,
    {
        RequestVoteRpcResult {
            term: server.persistent_server_state.current_term,
            vote_granted: arguments.term < server.persistent_server_state.current_term,
        }
    }
}
