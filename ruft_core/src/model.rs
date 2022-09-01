use std::collections::HashMap;

pub type TermIndex = u64;
pub type ServerId = u16;
pub type LogIndex = usize;

pub trait Command = Clone;

pub type Log<C> = Vec<LogEntry<C>>;

#[derive(Clone)]
pub struct LogEntry<C: Command> {
    pub index: LogIndex,

    /// command for state machine.
    pub command: C,

    /// term when entry was received by leader.
    pub term: TermIndex,
}

// pub struct Log<C> {
//     /// log entries
//     entries: Vec<LogEntry<C>>,
// }
//
// impl <C> Log<C> {
//     pub fn len(&self) -> usize {
//         self.entries.len()
//     }
//
//     pub fn insert(&mut self, pos: usize, entry: LogEntry<C>) {
//         self.entries.insert(pos, entry)
//     }
//
//     pub fn remove(&mut self, pos: usize) -> Option<LogEntry<C>> {
//         if pos < self.entries.len() {
//             Some(self.entries.remove(pos))
//         } else {
//             None
//         }
//     }
// }

/// Persistent state on all servers.
/// Updated on stable storage before responding to RPCs.
pub struct PersistentServerState<C: Command> {
    /// latest term server has seen.
    /// initialized to 0 on first boot, increases monotonically.
    pub current_term: TermIndex,

    /// candidate_id that received vote in current term (or null if none, i.e 0).
    pub voted_for: ServerId,

    /// log entries.
    pub log: Log<C>,
}

/// Volatile state on all servers.
pub struct VolatileServerState {
    /// index of highest log entry known to be committed.
    /// initialized to 0, increases monotonically.
    pub commit_index: LogIndex,

    /// index of highest log entry applied to state machine.
    /// initialized to 0, increases monotonically.
    pub last_applied: LogIndex,
}

/// Volatile state on leaders.
/// Reinitialized after election.
pub struct VolatileLeaderState {
    /// for each server, index of the next log entry to send to that server.
    /// initialized to leader last log index + 1.
    pub next_index: HashMap<ServerId, LogIndex>,

    /// for each server, index of highest log entry known to be replicated on server.
    /// initialized to 0, increases monotonically.
    pub match_index: HashMap<ServerId, LogIndex>,
}

/// Invoked by leader to replicate log entries.
/// Also used as heartbeat.
pub struct AppendEntriesRpcArguments<C: Command> {
    /// leader's term.
    pub term: TermIndex,

    /// so follower can redirect clients.
    pub leader_id: ServerId,

    /// index of log entry immediately preceeding new ones.
    pub prev_log_index: LogIndex,

    /// term of prev_log_index entry.
    pub prev_log_term: TermIndex,

    /// log entries to store.
    /// empty for heartbeat.
    /// may send more than one for efficiency.
    pub entries: Vec<LogEntry<C>>,

    /// leader's commit index.
    pub leader_commit: LogIndex,
}

pub struct AppendEntriesRpcResult {
    /// current term, for leader to update itself.
    pub term: TermIndex,

    /// true if follower contained entry matching prev_log_index and prev_log_term.
    pub success: bool,
}

/// Invoked by candidates to gather votes
pub struct RequestVoteRpcArguments {
    /// candidtae's term
    pub term: TermIndex,

    /// candidate requesting vote
    pub candidate_id: ServerId,

    /// index of candidate's last log entry
    pub last_log_index: LogIndex,

    /// term of candidate's last log entry
    pub last_log_term: TermIndex,
}

pub struct RequestVoteRpcResult {
    /// current term, for candidate to update itself
    pub term: TermIndex,

    /// true means candidate received vote
    pub vote_granted: bool,
}
