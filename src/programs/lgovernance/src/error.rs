//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the Governance program
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum GovernanceError {
    /// Invalid instruction passed to program
    #[error("Invalid instruction passed to program")]
    InvalidInstruction = 600, // Start Governance custom errors from 500 to avoid conflicts with programs invoked via CPI

    /// Realm with the given name and governing mints already exists
    #[error("Realm with the given name and governing mints already exists")]
    RealmAlreadyExists,

    /// Invalid Realm
    #[error("Invalid realm")]
    InvalidRealm,

    /// Invalid Governing Token Mint
    #[error("Invalid Governing Token Mint")]
    InvalidGoverningTokenMint,

    /// Governing Token Owner must sign transaction
    #[error("Governing Token Owner must sign transaction")]
    GoverningTokenOwnerMustSign,

    /// Governing Token Owner or Delegate  must sign transaction
    #[error("Governing Token Owner or Delegate  must sign transaction")]
    GoverningTokenOwnerOrDelegateMustSign, // 505

    /// All votes must be relinquished to withdraw governing tokens
    #[error("All votes must be relinquished to withdraw governing tokens")]
    AllVotesMustBeRelinquishedToWithdrawGoverningTokens,

    /// Invalid Token Owner Record account address
    #[error("Invalid Token Owner Record account address")]
    InvalidTokenOwnerRecordAccountAddress,

    /// Invalid GoverningMint for TokenOwnerRecord
    #[error("Invalid GoverningMint for TokenOwnerRecord")]
    InvalidGoverningMintForTokenOwnerRecord,

    /// Invalid Realm for TokenOwnerRecord
    #[error("Invalid Realm for TokenOwnerRecord")]
    InvalidRealmForTokenOwnerRecord,

    /// Invalid Proposal for ProposalTransaction,
    #[error("Invalid Proposal for ProposalTransaction,")]
    InvalidProposalForProposalTransaction, // 510

    /// Invalid Signatory account address
    #[error("Invalid Signatory account address")]
    InvalidSignatoryAddress, // 511

    /// Signatory already signed off
    #[error("Signatory already signed off")]
    SignatoryAlreadySignedOff, // 512

    /// Signatory must sign
    #[error("Signatory must sign")]
    SignatoryMustSign, // 513

    /// Invalid Proposal Owner
    #[error("Invalid Proposal Owner")]
    InvalidProposalOwnerAccount, // 514

    /// Invalid Proposal for VoterRecord
    #[error("Invalid Proposal for VoterRecord")]
    InvalidProposalForVoterRecord, // 515

    /// Invalid Proposal for vote weight
    #[error("Invalid Proposal for vote weight")]
    InvalidProposalForVoteWeight, // 516

    /// Invalid Proposal for option
    #[error("Invalid Proposal for option")]
    InvalidProposalForOption, // 517

    /// Invalid GoverningTokenOwner  for VoteRecord
    #[error("Invalid GoverningTokenOwner for VoteRecord")]
    InvalidGoverningTokenOwnerForVoteRecord,

    /// Invalid Scope for VoteRecord
    #[error("Invalid Scope for VoteRecord")]
    InvalidScopeVoteRecord,

    /// Invalid Governance config: Vote threshold percentage out of range"
    #[error("Invalid Governance config: Vote threshold percentage out of range")]
    InvalidVoteThresholdPercentage,

    /// Proposal for the given Governance, Governing Token Mint and index already exists
    #[error("Proposal for the given Governance, Governing Token Mint and index already exists")]
    ProposalAlreadyExists,

    /// Token Owner already voted on the Proposal
    #[error("Token Owner already voted on the Proposal")]
    VoteAlreadyExists,

    /// Vote is missing
    #[error("Vote is missing")]
    VoteMissing,

    /// Votes already counted for option
    #[error("Votes already counted for option")]
    VotesAlreadyCounted,

    /// Owner doesn't have enough governing tokens to create Proposal
    #[error("Owner doesn't have enough governing tokens to create Proposal")]
    NotEnoughTokensToCreateProposal,

    /// Invalid State: Can't edit Signatories
    #[error("Invalid State: Can't edit Signatories")]
    InvalidStateCannotEditSignatories,

    /// Invalid state cannot edit scopes
    #[error("Invalid state cannot edit scopes")]
    InvalidStateCannotEditscopes,

    /// Invalid state cannot edit options
    #[error("Invalid state cannot edit option")]
    InvalidStateCannotEditOptions,

    /// Invalid state cannot finalize draft
    #[error("Invalid state cannot finalize draft")]
    InvalidStateCannotFinalizeDraft,

    /// Invalid Proposal state
    #[error("Invalid Proposal state")]
    InvalidProposalState,
    /// Invalid State: Can't edit transactions
    #[error("Invalid State: Can't edit transactions")]
    InvalidStateCannotEditTransactions,

    /// Invalid State: Can't execute transaction
    #[error("Invalid State: Can't execute transaction")]
    InvalidStateCannotExecuteTransaction,

    /// Can't execute transaction within its hold up time
    #[error("Can't execute transaction within its hold up time")]
    CannotExecuteTransactionWithinHoldUpTime,

    /// Transaction already executed
    #[error("Transaction already executed")]
    TransactionAlreadyExecuted,

    /// Invalid Transaction index
    #[error("Invalid Transaction index")]
    InvalidTransactionIndex,

    /// Transaction hold up time is below the min specified by Governance
    #[error("Transaction hold up time is below the min specified by Governance")]
    TransactionHoldUpTimeBelowRequiredMin,

    /// Transaction at the given index for the Proposal already exists
    #[error("Transaction at the given index for the Proposal already exists")]
    TransactionAlreadyExists,

    /// Invalid State: Can't sign off
    #[error("Invalid State: Can't sign off")]
    InvalidStateCannotSignOff, // 530

    /// Invalid State: Can't vote
    #[error("Invalid State: Can't vote")]
    InvalidStateCannotVote,

    /// Invalid State: Can't finalize vote
    #[error("Invalid State: Can't finalize vote")]
    InvalidStateCannotFinalize,

    /// Invalid State: Can't cancel Proposal
    #[error("Invalid State: Can't cancel Proposal")]
    InvalidStateCannotCancelProposal,

    /// Vote already relinquished
    #[error("Vote already relinquished")]
    VoteAlreadyRelinquished,

    /// Can't finalize vote. Voting still in progress
    #[error("Can't finalize vote. Voting still in progress")]
    CannotFinalizeVotingInProgress,

    /// Proposal voting time expired
    #[error("Proposal voting time expired")]
    ProposalVotingTimeExpired,

    /// Invalid Signatory Mint
    #[error("Invalid Signatory Mint")]
    InvalidSignatoryMint,

    /// Proposal does not belong to the given Governance
    #[error("Proposal does not belong to the given Governance")]
    InvalidGovernanceForProposal,

    /// Proposal does not belong to the given Scope
    #[error("Proposal does not belong to the given Scope")]
    InvalidGovernanceForscope,

    /// Proposal does not belong to given Governing Mint"
    #[error("Proposal does not belong to given Governing Mint")]
    InvalidGoverningMintForProposal,

    /// Current mint authority must sign transaction
    #[error("Current mint authority must sign transaction")]
    MintAuthorityMustSign,

    /// Invalid mint authority
    #[error("Invalid mint authority")]
    InvalidMintAuthority,

    /// Mint has no authority
    #[error("Mint has no authority")]
    MintHasNoAuthority,

    /// ---- SPL Token Tools Errors ----

    /// Invalid Token account owner
    #[error("Invalid Token account owner")]
    SplTokenAccountWithInvalidOwner,

    /// Invalid Mint account owner
    #[error("Invalid Mint account owner")]
    SplTokenMintWithInvalidOwner,

    /// Token Account is not initialized
    #[error("Token Account is not initialized")]
    SplTokenAccountNotInitialized,

    /// Token Account doesn't exist
    #[error("Token Account doesn't exist")]
    SplTokenAccountDoesNotExist,

    /// Token account data is invalid
    #[error("Token account data is invalid")]
    SplTokenInvalidTokenAccountData,

    /// Token mint account data is invalid
    #[error("Token mint account data is invalid")]
    SplTokenInvalidMintAccountData,

    /// Token Mint is not initialized
    #[error("Token Mint account is not initialized")]
    SplTokenMintNotInitialized,

    /// Token Mint account doesn't exist
    #[error("Token Mint account doesn't exist")]
    SplTokenMintDoesNotExist,

    /// ---- Bpf Upgradable Loader Tools Errors ----

    /// Invalid ProgramData account Address
    #[error("Invalid ProgramData account address")]
    InvalidProgramDataAccountAddress,

    /// Invalid ProgramData account data
    #[error("Invalid ProgramData account Data")]
    InvalidProgramDataAccountData,

    /// Provided upgrade authority doesn't match current program upgrade authority
    #[error("Provided upgrade authority doesn't match current program upgrade authority")]
    InvalidUpgradeAuthority,

    /// Current program upgrade authority must sign transaction
    #[error("Current program upgrade authority must sign transaction")]
    UpgradeAuthorityMustSign,

    /// Given program is not upgradable
    #[error("Given program is not upgradable")]
    ProgramNotUpgradable,

    /// Invalid token owner
    #[error("Invalid token owner")]
    InvalidTokenOwner,

    /// Current token owner must sign transaction
    #[error("Current token owner must sign transaction")]
    TokenOwnerMustSign,

    /// Given VoteThresholdPercentageType is not supported
    #[error("Given VoteThresholdPercentageType is not supported")]
    VoteThresholdPercentageTypeNotSupported,

    /// Given VoteWeightSource is not supported
    #[error("Given VoteWeightSource is not supported")]
    VoteWeightSourceNotSupported,

    /// Proposal cool off time is not supported
    #[error("Proposal cool off time is not supported")]
    ProposalCoolOffTimeNotSupported,

    /// Governance PDA must sign
    #[error("Governance PDA must sign")]
    GovernancePdaMustSign,

    /// Transaction already flagged with error
    #[error("Transaction already flagged with error")]
    TransactionAlreadyFlaggedWithError,

    /// Invalid Realm for Governance
    #[error("Invalid Realm for Governance")]
    InvalidRealmForGovernance,

    /// Invalid Authority for Realm
    #[error("Invalid Authority for Realm")]
    InvalidAuthorityForRealm,

    /// Realm has no authority
    #[error("Realm has no authority")]
    RealmHasNoAuthority,

    /// Realm authority must sign
    #[error("Realm authority must sign")]
    RealmAuthorityMustSign, // 566

    /// Invalid governing token holding account
    #[error("Invalid governing token holding account")]
    InvalidGoverningTokenHoldingAccount,

    /// Realm council mint change is not supported
    #[error("Realm council mint change is not supported")]
    RealmCouncilMintChangeIsNotSupported,

    /// Not supported mint max vote weight sourcef
    #[error("Not supported mint max vote weight source")]
    MintMaxVoteWeightSourceNotSupported,

    /// Invalid max vote weight supply fraction
    #[error("Invalid max vote weight supply fraction")]
    InvalidMaxVoteWeightSupplyFraction,

    /// Owner doesn't have enough governing tokens to create Governance
    #[error("Owner doesn't have enough governing tokens to create Governance")]
    NotEnoughTokensToCreateGovernance,

    /// Too many outstanding proposals
    #[error("Too many outstanding proposals")]
    TooManyOutstandingProposals,

    /// All proposals must be finalized to withdraw governing tokens
    #[error("All proposals must be finalized to withdraw governing tokens")]
    AllProposalsMustBeFinalisedToWithdrawGoverningTokens,

    /// Invalid VoterWeightRecord for Realm
    #[error("Invalid VoterWeightRecord for Realm")]
    InvalidVoterWeightRecordForRealm,

    /// Invalid VoterWeightRecord for GoverningTokenMint
    #[error("Invalid VoterWeightRecord for GoverningTokenMint")]
    InvalidVoterWeightRecordForGoverningTokenMint,

    /// Invalid VoterWeightRecord for TokenOwner
    #[error("Invalid VoterWeightRecord for TokenOwner")]
    InvalidVoterWeightRecordForTokenOwner,

    /// VoterWeightRecord expired
    #[error("VoterWeightRecord expired")]
    VoterWeightRecordExpired,

    /// Invalid RealmConfig for Realm
    #[error("Invalid RealmConfig for Realm")]
    InvalidRealmConfigForRealm,

    /// TokenOwnerRecord already exists
    #[error("TokenOwnerRecord already exists")]
    TokenOwnerRecordAlreadyExists,

    /// TokenOwnerRecord missing
    #[error("TokenOwnerRecord missing")]
    TokenOwnerRecordMissing,

    /// TokenOwnerBudgetRecord missing
    #[error("TokenOwnerBudgetRecord missing")]
    TokenOwnerBudgetRecordMissing,

    /// TokenOnwerBudgetRecord already exist
    #[error("TokenOnwerBudgetRecord already exist")]
    TokenOwnerBudgetRecordAlreadyExist,

    /// Governing token deposits not allowed
    #[error("Governing token deposits not allowed")]
    GoverningTokenDepositsNotAllowed,

    /// Invalid vote choice weight percentage
    #[error("Invalid vote choice weight percentage")]
    InvalidVoteChoiceWeightPercentage,

    /// Vote type not supported
    #[error("Vote type not supported")]
    VoteTypeNotSupported,

    /// InvalidProposalOptions
    #[error("Invalid proposal options")]
    InvalidProposalOptions,

    /// Proposal is not not executable
    #[error("Proposal is not not executable")]
    ProposalIsNotExecutable,

    /// Invalid vote
    #[error("Invalid vote")]
    InvalidVote,

    /// Cannot execute defeated option
    #[error("Cannot execute defeated option")]
    CannotExecuteDefeatedOption,

    /// VoterWeightRecord invalid action
    #[error("VoterWeightRecord invalid action")]
    VoterWeightRecordInvalidAction,

    /// VoterWeightRecord invalid action target
    #[error("VoterWeightRecord invalid action target")]
    VoterWeightRecordInvalidActionTarget,

    /// Invalid MaxVoterWeightRecord for Realm
    #[error("Invalid MaxVoterWeightRecord for Realm")]
    InvalidMaxVoterWeightRecordForRealm,

    /// Invalid MaxVoterWeightRecord for GoverningTokenMint
    #[error("Invalid MaxVoterWeightRecord for GoverningTokenMint")]
    InvalidMaxVoterWeightRecordForGoverningTokenMint,

    /// MaxVoterWeightRecord expired
    #[error("MaxVoterWeightRecord expired")]
    MaxVoterWeightRecordExpired,

    /// Not supported VoteType
    #[error("Not supported VoteType")]
    NotSupportedVoteType,

    /// RealmConfig change not allowed
    #[error("RealmConfig change not allowed")]
    RealmConfigChangeNotAllowed,

    /// GovernanceConfig change not allowed
    #[error("GovernanceConfig change not allowed")]
    GovernanceConfigChangeNotAllowed,

    /// Invalid vote mint
    #[error("Invalid vote mint")]
    InvalidVoteMint,

    /// Invalid vote scope
    #[error("Invalid vote scope")]
    InvalidVotescope,

    /// Scope not applicable
    #[error("Scope not applicable")]
    ScopeNotApplicableForInstruction,

    /// Invalid channel
    #[error("Invalid channel")]
    InvalidChannel,

    /// Invalid option for vote
    #[error("Invalid option for vote")]
    InvalidOptionForVote,

    /// Option already exist
    #[error("Option already exist")]
    OptionAlreadyExist,

    /// Invalid option for instructions
    #[error("Invalid option for instructions")]
    InvalidOptionForInstructions,

    /// Max vote weights has not been calculated
    #[error("Max vote weights has not been calculated")]
    MaxWeightsNotCalculated,

    /// Invalid authority for channel
    #[error("Invalid authority for channel")]
    InvalidAuthorityForChannel,

    /// Invalid authority for governance
    #[error("Invalid authority for governance")]
    InvalidAuthorityForGovernance,

    /// Invalid creator for proposal
    #[error("Invalid creator for proposal")]
    InvalidCreatorForProposal,

    /// Missing scopes for proposal
    #[error("Missing scopes for propasal")]
    MissingscopesForProposal,

    /// Invalid tag record
    #[error("Invalid tag record")]
    InvalidTagRecord,

    /// Invalid token balance
    #[error("Invalid token balance")]
    InvalidTokenBalance,

    /// Token holder account already exist
    #[error("Token holder account already exist")]
    TokenHolderAccountAlreadyExist,

    /// Invalid deny option for proposal
    #[error("Invalid deny option for proposal")]
    InvalidDenyOptionForProposal,

    /// Option equal to deny option
    #[error("Option equal to deny option")]
    OptionEqualToDenyOption,

    /// Invalid delegation state for updates
    #[error("Invalid delegation state for updates")]
    InvalidDelegationStateForUpdates,

    /// Invalid vote record
    #[error("Invalid vote record")]
    InvalidVoteRecord,

    /// Invalid owner for delegation record
    #[error("Invalid owner for delegation record")]
    InvalidOwnerForDelegationRecord,

    /// Invalid previous vote for vote record
    #[error("Invalid previous vote for vote record")]
    InvalidPreviousVoteForVoteRecord,

    /// Invalid delegation state for undelegation
    #[error("Invalid delegation state for undelegation")]
    InvalidDelegatioStateForUndelegation,

    /// DelegationRecord missing
    #[error("DelegationRecord missing")]
    DelegationRecordMissing,

    /// DelegatingDelegateNotAllowed not allowed
    #[error("DelegatingDelegateNotAllowed not allowed")]
    DelegatingDelegateNotAllowed,

    /// InvalidSyncDirection
    #[error("InvalidSyncDirection")]
    InvalidSyncDirection,

    // InvalidTagRecordFactory
    #[error("InvalidTagRecordFactory")]
    InvalidTagRecordFactory,

    // InvalidVotePowerSource
    #[error("InvalidVotePowerSource")]
    InvalidVotePowerSource,

    // VotePowerOriginRecordAlreadyExist
    #[error("VotePowerOriginRecordAlreadyExist")]
    VotePowerOriginRecordAlreadyExist,
}
impl PrintProgramError for GovernanceError {
    fn print<E>(&self) {
        msg!("GOVERNANCE-ERROR: {}", &self.to_string());
    }
}

impl From<GovernanceError> for ProgramError {
    fn from(e: GovernanceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for GovernanceError {
    fn type_of() -> &'static str {
        "Governance Error"
    }
}
