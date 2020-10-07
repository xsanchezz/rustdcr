/// ProposalRecord is an entire proposal and it's content.
///
/// Signature is a signature of the proposal merkle root where the merkle root
/// contains all Files and Metadata of the proposal.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct ProposalRecord {
    /// Short proposal name.
    pub name: String,
    /// Current state of proposal
    #[serde(rename = "state")]
    pub proposal_state: i64,
    /// Current status of proposal
    #[serde(rename = "status")]
    pub proposal_status: i64,
    /// Last update of proposal
    pub timestamp: i64,
    /// ID of user who submitted proposal
    #[serde(rename = "userid")]
    pub user_id: String,
    /// Username of user who submitted proposal
    pub username: String,
    /// Key used for signature.
    #[serde(rename = "publickey")]
    pub public_key: String,
    /// Signature of merkle root
    pub signature: String,
    /// Number of comments on the proposal
    #[serde(rename = "numcomments")]
    pub number_of_comments: usize,
    /// Record version
    pub version: String,
    /// Message associated to the status change
    #[serde(rename = "statuschangemessage")]
    pub status_change_message: String,
    /// UNIX timestamp of when proposal was published
    #[serde(rename = "publishedat")]
    pub pubished_at: i64,
    /// UNIX timestamp of when proposal was censored
    #[serde(rename = "censoredat")]
    pub censored_at: i64,
    /// UNIX timestamp of when proposal was abandoned
    #[serde(rename = "abandonedat")]
    pub abandoned_at: i64,
    /// Token of linked parent proposal
    #[serde(rename = "linkto")]
    pub link_to: String,
    /// UNIX timestamp of RFP deadline
    #[serde(rename = "linkby")]
    pub link_by: i64,
    /// Tokens of public props that have linked to this this prop
    #[serde(rename = "linkfrom")]
    pub link_from: Vec<String>,
}

/// VoteOption describes a single vote option.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct VoteOption {
    /// Single unique word identifying vote (e.g. yes)
    pub id: String,
    /// Longer description of the vote.
    pub description: String,
    /// Bits used for this option
    pub bits: u64,
}

/// Vote represents the vote options for vote that is identified by its token.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct Vote {
    /// Token that identifies vote.
    pub token: String,
    /// Valid votebits.
    pub mask: u64,
    /// Duration in blocks
    pub duration: u32,
    /// Percent of eligible votes required for quorum
    #[serde(rename = "quorumpercentage")]
    pub quorom_percentage: u32,
    /// Percent of total votes required to pass
    #[serde(rename = "passpercentage")]
    pub pass_percentage: u32,
    /// Vote options
    pub options: Vec<VoteOption>,
}

/// StartVote starts the voting process for a proposal.
///
/// THIS ROUTE HAS BEEN DEPRECATED
/// A proposal vote must be initiated using the v2 StartVote route.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct StartVote {
    /// Key used for signature.
    #[serde(rename = "publickey")]
    pub public_key: String,
    /// Vote
    pub vote: Vote,
    /// Signature of Votehash.
    pub signature: String,
}

/// StartVoteReply returns the eligible ticket pool.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct StartVoteReply {
    /// Block height.
    #[serde(rename = "startblockheight")]
    pub start_block_height: String,
    /// Block hash.
    #[serde(rename = "startblockhash")]
    pub start_block_hash: String,
    /// Height of vote end.
    #[serde(rename = "endheight")]
    pub end_height: String,
    /// Valid voting tickets.
    #[serde(rename = "eligibletickets")]
    pub eligible_tickets: Vec<String>,
}

/// ProposalVoteTuple is the proposal, vote and vote details.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct ProposalVote {
    /// Proposal.
    pub propsal: ProposalRecord,
    /// Vote bits and mask
    #[serde(rename = "startvote")]
    pub start_vote: StartVote,
    #[serde(rename = "startvotereply")]
    /// Eligible tickets and other details
    pub start_vote_reply: StartVoteReply,
}

/// Retrieves all active votes.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct ActiveVoteResult {
    pub votes: Vec<ProposalVote>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct VoteStatus {
    #[serde(rename = "votesstatus")]
    pub vote_status: Vec<Vote>,
}
