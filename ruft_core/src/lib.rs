use std::collections::HashMap;
use std::cmp;

type TermIndex = u64;
type ServerId = u16;

type LogEntryCommand = String;
type LogIndex = usize;
type Log = Vec<LogEntry>;

#[derive(Clone)]
struct LogEntry {
    index: LogIndex,

    /// command for state machine.
    command: LogEntryCommand,

    /// term when entry was received by leader.
    term: TermIndex,
}

/// Persistent state on all servers.
/// Updated on stable storage before responding to RPCs.
struct PersistentServerState {
    /// latest term server has seen.
    /// initialized to 0 on first boot, increases monotonically.
    current_term: TermIndex,

    /// candidate_id that received vote in current term (or null if none, i.e 0).
    voted_for: ServerId,

    /// log entries.
    log: Log,
}

/// Volatile state on all servers.
struct VolatileServerState {
    /// index of highest log entry known to be committed.
    /// initialized to 0, increases monotonically.
    commit_index: LogIndex, 

    /// index of highest log entry applied to state machine.
    /// initialized to 0, increases monotonically.
    last_applied: LogIndex,
}

/// Volatile state on leaders.
/// Reinitialized after election.
struct VolatileLeaderState {
    /// for each server, index of the next log entry to send to that server.
    /// initialized to leader last log index + 1.
    next_index: HashMap<ServerId, LogIndex>,

    /// for each server, index of highest log entry known to be replicated on server.
    /// initialized to 0, increases monotonically.
    match_index: HashMap<ServerId, LogIndex>,
}

/// Invoked by leader to replicate log entries.
/// Also used as heartbeat.
struct AppendEntriesRpcArguments {
    /// leader's term.
    term: TermIndex,

    /// so follower can redirect clients.
    leader_id: ServerId,

    /// index of log entry immediately preceeding new ones.
    prev_log_index: LogIndex,

    /// term of prev_log_index entry.
    prev_log_term: TermIndex,

    /// log entries to store.
    /// empty for heartbeat.
    /// may send more than one for efficiency.
    entries: Vec<LogEntry>,

    /// leader's commit index.
    leader_commit: LogIndex,
}

trait HasTerm {
    fn get_term(&self) -> TermIndex;
}

macro_rules! implement_has_term_trait {
    ($id:ident) => {
        impl HasTerm for $id {
            fn get_term(&self) -> TermIndex {
                self.term
            }
        }
    }
}

implement_has_term_trait!(AppendEntriesRpcArguments);
implement_has_term_trait!(AppendEntriesRpcResult);
implement_has_term_trait!(RequestVoteRpcArguments);
implement_has_term_trait!(RequestVoteRpcResult);

struct AppendEntriesRpcResult {
    /// current term, for leader to update itself.
    term: TermIndex,

    /// true if follower contained entry matching prev_log_index and prev_log_term.
    success: bool,
}

impl AppendEntriesRpcResult {
    fn success(term: TermIndex) -> Self {
        Self  {
            success: true,
            term,
        }
    }

    fn fail() -> Self {
        Self {
            success: false,
            term: 0,
        }
    }
}

struct AppendEntriesRpcLogic;

impl AppendEntriesRpcLogic {
    fn receive_append_entries_rpc(server: &mut Server, arguments: AppendEntriesRpcArguments) -> AppendEntriesRpcResult {
        // Induction validating the log matching property
        let is_invalid = [
            arguments.term < server.persistent_server_state.current_term,
            server.persistent_server_state.log.len() < arguments.prev_log_index,
            server.persistent_server_state.log[arguments.prev_log_index].term != arguments.prev_log_term,
        ].into_iter().any(|x| x);

        if is_invalid {
            return AppendEntriesRpcResult::fail()
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
            server.persistent_server_state.log.insert(entry.index, entry.clone());
        }

        // update volatile state
        if arguments.leader_commit > server.volatile_server_state.commit_index {
            server.volatile_server_state.commit_index = cmp::min(
                arguments.leader_commit,
                entries.last().map_or(arguments.leader_commit, |entry| entry.index),
            );
        }

        return AppendEntriesRpcResult::success(0);
    }
}

/// Invoked by candidates to gather votes
struct RequestVoteRpcArguments {
    /// candidtae's term
    term: TermIndex,

    /// candidate requesting vote
    candidate_id: ServerId,

    /// index of candidate's last log entry
    last_log_index: LogIndex,

    /// term of candidate's last log entry
    last_log_term: TermIndex
}

struct RequestVoteRpcResult {
    /// current term, for candidate to update itself
    term: TermIndex,

    /// true means candidate received vote
    vote_granted: bool,
}

struct RequestVoteRpcLogic;

impl RequestVoteRpcLogic {
    fn receive_request_vote_rpc(server: &mut Server, arguments: RequestVoteRpcArguments) -> RequestVoteRpcResult {
        RequestVoteRpcResult {
            term: server.persistent_server_state.current_term,
            vote_granted: arguments.term < server.persistent_server_state.current_term,
        }
    }
}

enum ServerState {
    Leader,
    Follower,
    Candidate,
}

struct Server {
    state: ServerState,
    persistent_server_state: PersistentServerState,
    volatile_server_state: VolatileServerState,
    volatile_leader_state: Option<VolatileLeaderState>,

    /// TODO: this is temporary. the server should receive an external state machine
    state_machine: String,
}

impl Server {
    fn initialize(persistent_server_state: PersistentServerState) -> Self {
        let volatile_server_state = VolatileServerState {
            commit_index: 0,
            last_applied: 0,
        };

        Server {
            state: ServerState::Follower,
            state_machine: "".to_string(),
            persistent_server_state,
            volatile_server_state,
            volatile_leader_state: None,
        }
    }

    fn convert_to(&mut self, state: ServerState) {
        // TODO
    }

    fn apply_to_state_machine(&mut self, command: LogEntryCommand) {
        self.state_machine += &command
    }
}

struct AllServersRules;

impl AllServersRules {
    fn apply_next_entry_if_needed(server: &mut Server) {
        if server.volatile_server_state.commit_index > server.volatile_server_state.last_applied {
            if server.volatile_server_state.commit_index > server.volatile_server_state.last_applied {
                server.volatile_server_state.last_applied += 1;
                let command = server.persistent_server_state.log[server.volatile_server_state.last_applied].command.clone();
                server.apply_to_state_machine(command);
            }
        }
    }

    fn convert_to_follower_if_needed<H: HasTerm>(server: &mut Server, has_term: &H) {
        if server.persistent_server_state.current_term < has_term.get_term() {
            server.persistent_server_state.current_term = has_term.get_term();
            server.convert_to(ServerState::Follower);
        }
    }
}

struct FollowerRules;

impl FollowerRules {
    /// TODO: if election timeout elapses without receiving AppendEntries RPC from current leader
    /// or granting vote to candidate: convert to candidate
    fn todo() {
    }

}

