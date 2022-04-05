use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

/// The status of instruction execution
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum TransactionExecutionStatus {
    /// Transaction was not executed yet
    None,

    /// Transaction was executed successfully
    Success,

    /// Transaction execution failed
    Error,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Asset {
    NonAsset, // Not for sale, just a regular "post" (no one would want to buy this)
              // Add more markets here, like auction, then this would describe the owner token etc
}

// What state a Proposal is in
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum ProposalState {
    /// Draft - Proposal enters Draft state when it's created
    Draft,
    /*
    /// SigningOff - The Proposal is being signed off by Signatories
    /// Proposal enters the state when first Signatory Sings and leaves it when last Signatory signs
    SigningOff, */
    /// Taking votes
    Voting,

    /// Voting ended with success
    Succeeded,

    /// Voting on Proposal succeeded and now instructions are being executed
    /// Proposal enter this state when first instruction is executed and leaves when the last instruction is executed
    Executing,

    /// Completed
    Completed,

    /// Cancelled
    Cancelled,

    /// Defeated
    Defeated,

    /// Same as Executing but indicates some instructions failed to execute
    /// Proposal can't be transitioned from ExecutingWithErrors to Completed state
    ExecutingWithErrors,
}

impl Default for ProposalState {
    fn default() -> Self {
        ProposalState::Draft
    }
}

/// The type of vote tipping to use on a Proposal.
///
/// Vote tipping means that under some conditions voting will complete early.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum VoteTipping {
    /// Tip when there is no way for another option to win and the vote threshold
    /// has been reached. This ignores voters withdrawing their votes.
    ///
    /// Currently only supported for the "yes" option in single choice votes.
    Strict,

    /// Tip when an option reaches the vote threshold and has more vote weight
    /// than any other options.
    ///
    /// Currently only supported for the "yes" option in single choice votes.
    Early,

    /// Never tip the vote early.
    Disabled,
}
/*
impl VoteTipping {
    pub fn get_strictest(&self, other: &Self) -> Self {
        match &self {
            Self::Strict => {
                if other == &VoteTipping::Disabled {
                    return other.clone();
                }
                self.clone()
            }
            Self::Early => {
                if other != &VoteTipping::Early {
                    return other.clone();
                }
                self.clone()
            }

            Self::Disabled => {
                self.clone()
            }
        }
    }
} */

/// Transaction execution flags defining how instructions are executed for a Proposal
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum InstructionExecutionFlags {
    /// No execution flags are specified
    /// Instructions can be executed individually, in any order, as soon as they hold_up time expires
    None,

    /// Instructions are executed in a specific order
    /// Note: Ordered execution is not supported in the current version
    /// The implementation requires another account type to track deleted instructions
    Ordered,

    /// Multiple instructions can be executed as a single transaction
    /// Note: Transactions are not supported in the current version
    /// The implementation requires another account type to group instructions within a transaction
    UseTransaction,
}

/// The source of max vote weight used for voting
/// Values below 100% mint supply can be used when the governing token is fully minted but not distributed yet
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum MintMaxVoteWeightSource {
    /// Fraction (10^10 precision) of the governing mint supply is used as max vote weight
    /// The default is 100% (10^10) to use all available mint supply for voting
    SupplyFraction(u64),

    /// Absolute value, irrelevant of the actual mint supply, is used as max vote weight
    /// Note: this option is not implemented in the current version
    Absolute(u64),
}

impl MintMaxVoteWeightSource {
    /// Base for mint supply fraction calculation
    pub const SUPPLY_FRACTION_BASE: u64 = 10_000_000_000;

    /// 100% of mint supply
    pub const FULL_SUPPLY_FRACTION: MintMaxVoteWeightSource =
        MintMaxVoteWeightSource::SupplyFraction(MintMaxVoteWeightSource::SUPPLY_FRACTION_BASE);
}
