pub mod proposal_option;
pub mod proposal_transaction;

use std::slice::Iter;

use borsh::maybestd::io::Write;
use shared::account::{get_account_data, MaxSize};
use shared::content::ContentSource;
use solana_program::account_info::next_account_info;
use solana_program::clock::{Slot, UnixTimestamp};

use solana_program::msg;
use solana_program::program_pack::IsInitialized;
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::accounts::AccountType;
use crate::error::GovernanceError;
use crate::{state::enums::TransactionExecutionStatus, PROGRAM_AUTHORITY_SEED};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use self::proposal_option::{get_proposal_option_data, ProposalOption};

use super::enums::{InstructionExecutionFlags, ProposalState, VoteTipping};

use super::rules::rule::{AcceptenceCriteria, Rule, RuleTimeConfig};
use super::rules::rule_weight::RuleWeight;
use super::vote_record::Vote;
use proposal_transaction::ProposalTransactionV2;

/// Proposal option vote result
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum OptionVoteResult {
    /// Vote on the option is not resolved yet
    None,

    /// Vote on the option is completed and the option passed
    Succeeded,

    /// Vote on the option is completed and the option was defeated
    Defeated,
}

/// Proposal vote type
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum VoteType {
    /// Single choice vote with mutually exclusive choices
    /// In the SingeChoice mode there can ever be a single winner
    /// If multiple options score the same highest vote then the Proposal is not resolved and considered as Failed
    /// Note: Yes/No vote is a single choice (Yes) vote with the deny option (No)
    SingleChoice,

    /// Multiple options can be selected with up to max_voter_options per voter
    /// and with up to max_winning_options of successful options
    /// Ex. voters are given 5 options, can choose up to 3 (max_voter_options)
    /// and only 1 (max_winning_options) option can win and be executed
    MultiChoice {
        /// The max number of options a voter can choose
        /// By default it equals to the number of available options
        #[allow(dead_code)]
        max_voter_options: Option<u8>,

        /// The max number of wining options
        /// For executable proposals it limits how many options can be executed for a Proposal
        /// By default it equals to the number of available options
        #[allow(dead_code)]
        max_winning_options: Option<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct CommonRuleConfig {
    pub vote_tipping: VoteTipping,
    pub time_config: RuleTimeConfig,
}

impl CommonRuleConfig {
    pub fn set_strictest(&mut self, compare: &Rule) {
        self.vote_tipping = self
            .vote_tipping
            .get_strictest(&compare.config.vote_config.vote_tipping);
        self.time_config = self.time_config.get_strictest(&compare.config.time_config);
    }
}

/// Governance Proposal
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct ProposalV2 {
    /// AccountType = Proposal
    pub account_type: AccountType,

    /// Governance account the Proposal belongs to
    pub governance: Pubkey,

    /// Current proposal state
    pub state: ProposalState,

    // TODO: add state_at timestamp to have single field to filter recent proposals in the UI
    /// The creator of the proposal
    pub creator: Pubkey,

    /// The number of signatories assigned to the Proposal
    pub signatories_count: u8,

    /// The number of signatories who already signed
    pub signatories_signed_off_count: u8,

    /// Vote type
    pub vote_type: VoteType,

    /// Common/strictest config for rules
    /*    pub common_rule_config: CommonRuleConfig, */

    /// Number of available options
    pub options_count: u16,

    /// Number of available options that has been vote counted for
    pub options_counted_count: u16,

    /// Number of available options that has been vote counted for
    pub options_executed_count: u16,

    /// Amount of rules expected
    pub rules_count: u8,

    /// All the rules used for the proposal
    pub rules_max_vote_weight: Vec<RuleWeight>,

    /// Does deny option exist if this has some value
    pub deny_option: Option<Pubkey>,

    /// Winning options
    pub winning_options: Vec<u16>,

    pub defeated_options: Vec<u16>,

    /*
    /// Strictest acceptence_criteria
    pub strictest_criteria: AcceptenceCriteria,

    /// Strictest time config found
    pub strictest_time_config: RuleTimeConfig, */
    /// The total weight of the Proposal rejection votes
    /// If the proposal has no deny option then the weight is None
    /// Only proposals with the deny option can have executable instructions attached to them
    /// Without the deny option a proposal is only non executable survey
    /*   pub deny_vote_weight: Option<u64>,

    /// The total weight of Veto votes
    /// Note: Veto is not supported in the current version
    pub veto_vote_weight: Option<u64>,

    /// The total weight of  votes
    /// Note: Abstain is not supported in the current version
    pub abstain_vote_weight: Option<u64>, */

    /// Optional start time if the Proposal should not enter voting state immediately after being signed off
    /// Note: start_at is not supported in the current version
    pub start_voting_at: Option<UnixTimestamp>,

    /// When the Proposal was created and entered Draft state
    pub draft_at: UnixTimestamp,

    /// When Signatories started signing off the Proposal
    pub signing_off_at: Option<UnixTimestamp>,

    /// When the Proposal began voting as UnixTimestamp
    pub voting_at: Option<UnixTimestamp>,

    /// When the Proposal began voting as UnixTimestamp
    pub max_vote_weights_calculated_at: Option<UnixTimestamp>,

    /// When the Proposal began voting as Slot
    /// Note: The slot is not currently used but the exact slot is going to be required to support snapshot based vote weights
    pub voting_at_slot: Option<Slot>,

    /// When the Proposal ended voting and entered either Succeeded or Defeated
    pub voting_completed_at: Option<UnixTimestamp>,

    /// When the Proposal entered Executing state
    pub executing_at: Option<UnixTimestamp>,

    /// When the Proposal entered final state Completed or Cancelled and was closed
    pub closed_at: Option<UnixTimestamp>,

    /// Instruction execution flag for ordered and transactional instructions
    /// Note: This field is not used in the current version
    pub execution_flags: InstructionExecutionFlags,

    /// The max vote weight for the Governing Token mint at the time Proposal was decided
    /// It's used to show correct vote results for historical proposals in cases when the mint supply or max weight source changed
    /// after vote was completed.
    /*   pub max_vote_weight: Option<u64>, */

    /// The max vote weight for the Governing Token mint at the time Proposal was decided
    /// It's used to show correct vote results for historical proposals in cases when the mint supply or max weight source changed
    /// after vote was completed.
    /*  pub max_vote_weights: Option<Vec<VoteWeight>>, */

    /// Max voting time for the proposal if different from parent Governance  (only higher value possible)
    /// Note: This field is not used in the current version
    pub max_voting_time: Option<u32>,

    /// The vote threshold percentage at the time Proposal was decided
    /// It's used to show correct vote results for historical proposals in cases when the threshold
    /// was changed for governance config after vote was completed.
    /// TODO: Use this field to override for the threshold from parent Governance (only higher value possible)
    pub vote_threshold_percentage: Option<AcceptenceCriteria>,

    /// Info
    pub source: ContentSource,
}

impl IsInitialized for ProposalV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Proposal
    }
}
impl MaxSize for ProposalV2 {
    fn get_max_size(&self) -> Option<usize> {
        Some(1000) //TODO FIX
    }
}

impl ProposalV2 {
    /// Checks if Signatories can be edited (added or removed) for the Proposal in the given state
    pub fn assert_can_edit_signatories(&self) -> Result<(), ProgramError> {
        self.assert_is_draft_state()
            .map_err(|_| GovernanceError::InvalidStateCannotEditSignatories.into())
    }

    /// Checks if Proposal can be singed off
    /*   pub fn assert_can_sign_off(&self) -> Result<(), ProgramError> {
        match self.state {
            ProposalState::Draft | ProposalState::SigningOff => Ok(()),
            ProposalState::Executing
            | ProposalState::ExecutingWithErrors
            | ProposalState::Completed
            | ProposalState::Cancelled
            | ProposalState::Voting
            | ProposalState::Succeeded
            | ProposalState::Defeated => Err(GovernanceError::InvalidStateCannotSignOff.into()),
        }
    } */

    pub fn assert_can_finalize_draft(&self, creator: &AccountInfo) -> Result<(), ProgramError> {
        self.assert_edit_authority(creator)?;
        if self.state != ProposalState::Draft {
            return Err(GovernanceError::InvalidStateCannotFinalizeDraft.into());
        }
        Ok(())
    }

    /// Checks the Proposal is in Voting state
    fn assert_is_voting_state(&self) -> Result<(), ProgramError> {
        if self.state != ProposalState::Voting {
            return Err(GovernanceError::InvalidProposalState.into());
        }
        Ok(())
    }

    // Checks edit authority
    fn assert_edit_authority(&self, creator: &AccountInfo) -> Result<(), ProgramError> {
        if !creator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if &self.creator != creator.key {
            return Err(GovernanceError::InvalidCreatorForProposal.into());
        }
        return Ok(());
    }

    /// Checks the Proposal is in Draft state
    fn assert_is_draft_state(&self) -> Result<(), ProgramError> {
        if self.state != ProposalState::Draft {
            return Err(GovernanceError::InvalidProposalState.into());
        }

        Ok(())
    }

    /// Checks if Proposal can be voted on
    /*   pub fn assert_can_cast_vote(
           &self,
           current_unix_timestamp: UnixTimestamp,
       ) -> Result<(), ProgramError> {
           self.assert_is_voting_state()
               .map_err(|_| PostError::InvalidStateCannotVote)?;

           // Check if we are still within the configured max_voting_time period
           if self.has_vote_time_ended(&self.strictest_time_config, current_unix_timestamp) {
               return Err(PostError::ProposalVotingTimeExpired.into());
           }

           Ok(())
       }
    */

    pub fn set_completed_voting_state(&mut self, state: ProposalState, unix_timestamp: i64) {
        self.state = state;
        self.voting_completed_at = Some(unix_timestamp);
    }
    /// Checks whether the voting time has ended for the proposal
    pub fn has_vote_time_ended(
        &self,
        config: &RuleTimeConfig,
        current_unix_timestamp: UnixTimestamp,
    ) -> bool {
        // Check if we passed vote_end_time determined by the configured max_voting_time period
        self.voting_at
            .unwrap()
            .checked_add(config.max_voting_time as i64)
            .unwrap()
            < current_unix_timestamp
    }

    /// Checks if Proposal can be finalized
    pub fn assert_can_finalize_vote(
        &self,
        config: &RuleTimeConfig,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<(), ProgramError> {
        self.assert_is_voting_state()
            .map_err(|_| GovernanceError::InvalidStateCannotFinalize)?;

        // We can only finalize the vote after the configured max_voting_time has expired and vote time ended
        if !self.has_vote_time_ended(config, current_unix_timestamp) {
            return Err(GovernanceError::CannotFinalizeVotingInProgress.into());
        }

        Ok(())
    }

    /// Finalizes vote by moving it to final state Succeeded or Defeated if max_voting_time has passed
    /// If Proposal is still within max_voting_time period then error is returned
    pub fn finalize_vote(
        &mut self,
        rule_time_config: &RuleTimeConfig,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<(), ProgramError> {
        self.assert_can_finalize_vote(rule_time_config, current_unix_timestamp)?;
        // TODO: set voting_completed_at based on the time when the voting ended and not when we finalized the proposal
        self.voting_completed_at = Some(current_unix_timestamp);

        // Capture vote params to correctly display historical results
        /*  self.max_vote_weight = Some(max_voter_weight);
        self.vote_threshold_percentage = Some(config.criteria); */

        Ok(())
    }

    /// Resolves final proposal state after vote ends
    /// It inspects all proposals options and resolves their final vote results
    /* fn resolve_final_vote_state(
        &mut self,
        criteria: &AcceptenceCriteria,
        max_vote_weight: u64,
    ) -> Result<ProposalState, ProgramError> {
        let mut final_state = match self.vote_type {
            VoteType::SingleChoice {winning_option} => {
                match winning_option {
                    Some(option) => {
                        ProposalState::Succeeded
                    },
                    None => ProposalState::Defeated
                }

            }
            VoteType::MultiChoice => {}
        }
    } */
    /// Resolves final proposal state after vote ends
    /// It inspects all proposals options and resolves their final vote results
    /* fn resolve_final_vote_state(
        &mut self,
        criteria: &AcceptenceCriteria,
        max_vote_weight: u64,
    ) -> Result<ProposalState, ProgramError> {
        // Get the min vote weight required for options to pass
        let min_vote_threshold_weight =
            get_min_vote_threshold_weight(criteria, max_vote_weight).unwrap();

        // If the proposal has a reject option then any other option must beat it regardless of the configured min_vote_threshold_weight
        let deny_vote_weight = self
            .options
            .iter()
            .find(|option| option.option_type == ProposalOptionType::Deny)
            .map(|option| option.vote_weight)
            .unwrap_or(0);

        let mut best_succeeded_option_weight = 0;
        let mut best_succeeded_option_count = 0u16;

        for option in self.options.iter_mut() {
            // Any positive vote (Yes) must be equal or above the required min_vote_threshold_weight and higher than the reject option vote (No)
            // The same number of positive (Yes) and rejecting (No) votes is a tie and resolved as Defeated
            // In other words  +1 vote as a tie breaker is required to succeed for the positive option vote
            if option.vote_weight >= min_vote_threshold_weight
                && option.vote_weight > deny_vote_weight
            {
                option.vote_result = OptionVoteResult::Succeeded;

                match option.vote_weight.cmp(&best_succeeded_option_weight) {
                    Ordering::Greater => {
                        best_succeeded_option_weight = option.vote_weight;
                        best_succeeded_option_count = 1;
                    }
                    Ordering::Equal => {
                        best_succeeded_option_count =
                            best_succeeded_option_count.checked_add(1).unwrap()
                    }
                    Ordering::Less => {}
                }
            } else {
                option.vote_result = OptionVoteResult::Defeated;
            }
        }

        let mut final_state = if best_succeeded_option_count == 0 {
            // If none of the individual options succeeded then the proposal as a whole is defeated
            ProposalState::Defeated
        } else {
            match self.vote_type {
                VoteType::SingleChoice => {
                    let proposal_state = if best_succeeded_option_count > 1 {
                        // If there is more than one winning option then the single choice proposal is considered as defeated
                        best_succeeded_option_weight = u64::MAX; // no winning option
                        ProposalState::Defeated
                    } else {
                        ProposalState::Succeeded
                    };

                    // Coerce options vote results based on the winning score (best_succeeded_vote_weight)
                    for option in self.options.iter_mut() {
                        option.vote_result = if option.vote_weight == best_succeeded_option_weight {
                            OptionVoteResult::Succeeded
                        } else {
                            OptionVoteResult::Defeated
                        };
                    }

                    proposal_state
                }
                VoteType::MultiChoice {
                    max_voter_options: _n,
                    max_winning_options: _m,
                } => {
                    // If any option succeeded for multi choice then the proposal as a whole succeeded as well
                    ProposalState::Succeeded
                }
            }
        };

        // None executable proposal is just a survey and is considered Completed once the vote ends and no more actions are available
        // There is no overall Success or Failure status for the Proposal however individual options still have their own status
        if self.deny_vote_weight.is_none() {
            final_state = ProposalState::Completed;
        }

        Ok(final_state)
    } */

    /// Adjusts max voter weight to ensure it's not lower than total cast votes
    /*   fn coerce_max_voter_weight(&self, max_voter_weight: u64) -> u64 {
           let deny_vote_weight = self.deny_vote_weight.unwrap_or(0);

           let max_option_vote_weight = self.options.iter().map(|o| o.vote_weight).max().unwrap();

           let total_vote_weight = max_option_vote_weight
               .checked_add(deny_vote_weight)
               .unwrap();

           max_voter_weight.max(total_vote_weight)
       }
    */
    /// Checks if vote can be tipped and automatically transitioned to Succeeded or Defeated state
    /// If the conditions are met the state is updated accordingly
    /* pub fn try_tip_vote(
        &mut self,
        max_voter_weight: u64,
        config: &RuleVoteConfig,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<bool, ProgramError> {
        if self.vote_type != VoteType::SingleChoice
        // Tipping should not be allowed for opinion only proposals (surveys without rejection) to allow everybody's voice to be heard
        || self.deny_option.is_none()
        || self.options_count != 2
        {
            return Ok(false);
        };

        if let Some(tipped_state) = self.try_get_tipped_vote_state(max_voter_weight, config) {
            self.state = tipped_state;
            self.voting_completed_at = Some(current_unix_timestamp);

            // Capture vote params to correctly display historical results
            /*   self.max_vote_weight = Some(max_voter_weight); */
            self.vote_threshold_percentage = Some(config.criteria.clone());

            Ok(true)
        } else {
            Ok(false)
        }
    } */

    /// Checks if vote can be tipped and automatically transitioned to Succeeded or Defeated state
    /// If yes then Some(ProposalState) is returned and None otherwise
    /* #[allow(clippy::float_cmp)]
    pub fn try_get_tipped_vote_state(
        &mut self,
        option: &ProposalOption,
        deny_option: Option<&ProposalOption>,
    ) -> Option<ProposalState> {
        // Vote tipping is currently supported for SingleChoice votes with single Yes and No (rejection) options only
        // Note: Tipping for multiple options (single choice and multiple choices) should be possible but it requires a great deal of considerations
        //       and I decided to fight it another day
        if self.vote_type != VoteType::SingleChoice
        // Tipping should not be allowed for opinion only proposals (surveys without rejection) to allow everybody's voice to be heard
        || deny_option.is_none()
        || self.options_count != 2
        {
            return None;
        };

        if option.vote_result == OptionVoteResult::Succeeded {
            return Some(ProposalState::Succeeded);
        }

        if let Some(deny_option) = deny_option {
            if deny_option.vote_result == OptionVoteResult::Succeeded {
                option.vote_result = OptionVoteResult::Defeated;
                return Some(ProposalState::Defeated);
            }
        }
        match config.vote_tipping {
            VoteTipping::Disabled => {}
            VoteTipping::Strict => {
                if yes_vote_weight >= min_vote_threshold_weight
                    && yes_vote_weight > (max_vote_weight.saturating_sub(yes_vote_weight))
                {
                    yes_option.vote_result = OptionVoteResult::Succeeded;
                    return Some(ProposalState::Succeeded);
                }
            }
            VoteTipping::Early => {
                if yes_vote_weight >= min_vote_threshold_weight
                    && yes_vote_weight > deny_vote_weight
                {
                    yes_option.vote_result = OptionVoteResult::Succeeded;
                    return Some(ProposalState::Succeeded);
                }
            }
        }

        // If vote tipping isn't disabled entirely, allow a vote to complete as
        // "defeated" if there is no possible way of reaching majority or the
        // min_vote_threshold_weight for another option. This tipping is always
        // strict, there's no equivalent to "early" tipping for deny votes.
        if config.vote_tipping != VoteTipping::Disabled
            && (deny_vote_weight > (max_vote_weight.saturating_sub(min_vote_threshold_weight))
                || deny_vote_weight >= (max_vote_weight.saturating_sub(deny_vote_weight)))
        {
            yes_option.vote_result = OptionVoteResult::Defeated;
            return Some(ProposalState::Defeated);
        }

        None
    } */

    /// Checks if Proposal can be canceled in the given state
    /* pub fn assert_can_cancel(
        &self,
        config: &RuleTimeConfig,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<(), ProgramError> {
        match self.state {
            ProposalState::Draft | ProposalState::SigningOff => Ok(()),
            ProposalState::Voting => {
                // Note: If there is no tipping point the proposal can be still in Voting state but already past the configured max_voting_time
                // In that case we treat the proposal as finalized and it's no longer allowed to be canceled
                if self.has_vote_time_ended(config, current_unix_timestamp) {
                    return Err(GovernanceError::ProposalVotingTimeExpired.into());
                }
                Ok(())
            }
            ProposalState::Executing
            | ProposalState::ExecutingWithErrors
            | ProposalState::Completed
            | ProposalState::Cancelled
            | ProposalState::Succeeded
            | ProposalState::Defeated => {
                Err(GovernanceError::InvalidStateCannotCancelProposal.into())
            }
        }
    } */

    /// Checks if Instructions can be edited (inserted or removed) for the Proposal in the given state
    /// It also asserts whether the Proposal is executable (has the reject option)
    pub fn assert_can_edit_instructions(
        &self,
        creator_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        self.assert_edit_authority(creator_info)?;

        if self.assert_is_draft_state().is_err() {
            return Err(GovernanceError::InvalidStateCannotEditTransactions.into());
        }

        // For security purposes only proposals with the reject option can have executable instructions
        if self.deny_option.is_none() {
            return Err(GovernanceError::ProposalIsNotExecutable.into());
        }

        Ok(())
    }

    /// Checks if Rules can be edited (inserted or removed) for the Proposal
    pub fn assert_can_edit_rules(&self, creator_info: &AccountInfo) -> Result<(), ProgramError> {
        self.assert_edit_authority(creator_info)?;

        if self.assert_is_draft_state().is_err() {
            return Err(GovernanceError::InvalidStateCannotEditRules.into());
        }

        Ok(())
    }

    pub fn assert_can_edit_options(&self, creator_info: &AccountInfo) -> Result<(), ProgramError> {
        self.assert_edit_authority(creator_info)?;

        if self.assert_is_draft_state().is_err() {
            return Err(GovernanceError::InvalidStateCannotEditOptions.into());
        }
        Ok(())
    }

    /// Checks if Instructions can be executed for the Proposal in the given state
    pub fn assert_can_execute_transaction(
        &self,
        proposal_transaction_data: &ProposalTransactionV2,
        proposal_option_data: &ProposalOption,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<(), ProgramError> {
        match self.state {
            ProposalState::Succeeded
            | ProposalState::Executing
            | ProposalState::ExecutingWithErrors => {}
            ProposalState::Draft
            | ProposalState::Completed
            | ProposalState::Voting
            | ProposalState::Cancelled
            | ProposalState::Defeated => {
                return Err(GovernanceError::InvalidStateCannotExecuteTransaction.into())
            }
        }

        if proposal_option_data.index != proposal_transaction_data.option_index {
            return Err(GovernanceError::InvalidOptionForInstructions.into());
        }

        if proposal_option_data.vote_result != OptionVoteResult::Succeeded {
            return Err(GovernanceError::CannotExecuteDefeatedOption.into());
        }

        if self
            .voting_completed_at
            .unwrap()
            .checked_add(proposal_transaction_data.hold_up_time as i64)
            .unwrap()
            >= current_unix_timestamp
        {
            return Err(GovernanceError::CannotExecuteTransactionWithinHoldUpTime.into());
        }

        if proposal_transaction_data.executed_at.is_some() {
            return Err(GovernanceError::TransactionAlreadyExecuted.into());
        }

        Ok(())
    }

    /// Checks if the instruction can be flagged with error for the Proposal in the given state
    pub fn assert_can_flag_transaction_error(
        &self,
        proposal_transaction_data: &ProposalTransactionV2,
        proposal_option_data: &ProposalOption,
        current_unix_timestamp: UnixTimestamp,
    ) -> Result<(), ProgramError> {
        // Instruction can be flagged for error only when it's eligible for execution
        self.assert_can_execute_transaction(
            proposal_transaction_data,
            proposal_option_data,
            current_unix_timestamp,
        )?;

        if proposal_transaction_data.execution_status == TransactionExecutionStatus::Error {
            return Err(GovernanceError::TransactionAlreadyFlaggedWithError.into());
        }

        Ok(())
    }

    /// Asserts the given vote is valid for the proposal
    pub fn assert_valid_vote(&self, vote: &Vote) -> Result<(), ProgramError> {
        /*
        Allow multichoice with deny or not?
        if let Some(key) = self.deny_option {


            if vote.len() != 1 || vote.get(0).unwrap() == &key {
                return Err(GovernanceError::InvalidVote.into());
            }
        } */
        msg!(" ?? {} ", vote.len());
        if vote.len() == 0 {
            return Err(GovernanceError::InvalidVote.into());
        }

        match self.vote_type {
            VoteType::SingleChoice { .. } => {
                if vote.len() != 1 {
                    return Err(GovernanceError::InvalidVote.into());
                }
            }
            VoteType::MultiChoice {
                max_voter_options, ..
            } => {
                if let Some(max_options) = max_voter_options {
                    if vote.len() > max_options as usize {
                        return Err(GovernanceError::InvalidVote.into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: &mut W) -> Result<(), ProgramError> {
        BorshSerialize::serialize(&self, writer)?;
        Ok(())
    }

    /// Assert options to create proposal are valid for the Proposal vote_type
    pub fn assert_valid_proposal_options(&self, vote_type: &VoteType) -> Result<(), ProgramError> {
        if self.options_count <= 1 {
            return Err(GovernanceError::InvalidProposalOptions.into());
        }

        if let VoteType::MultiChoice {
            max_voter_options,
            max_winning_options,
            ..
        } = *vote_type
        {
            if let Some(max_voter_options) = max_voter_options {
                if self.options_count < max_voter_options as u16 {
                    return Err(GovernanceError::InvalidProposalOptions.into());
                }
            }
            if let Some(max_winning_options) = max_winning_options {
                if self.options_count < max_winning_options as u16 {
                    return Err(GovernanceError::InvalidProposalOptions.into());
                }
            }
        }

        Ok(())
    }

    /// If add is true, then vote weight will be added, else vote weiight will be removed
    pub fn perform_voting(
        &self,
        program_id: &Pubkey,
        amount: u64,
        add: bool,
        governing_token_mint: &Pubkey,
        rule: &Pubkey,
        rule_data: &Rule,
        proposal: &Pubkey,
        accounts_iter: &mut Iter<AccountInfo>,
    ) -> Result<Vec<u16>, ProgramError> {
        let mut vote = Vec::new();
        msg!("B");
        let mut option_info_next = next_account_info(accounts_iter);
        msg!("BB");

        while let Ok(option_info) = option_info_next {
            // Vote with rule weight
            // Check create vote recor
            msg!("::: {} {}", option_info.key, option_info.data_is_empty());
            let mut option_data = get_proposal_option_data(program_id, option_info, proposal)?;
            msg!("BBB");

            option_data.update_weight(amount, add, governing_token_mint, rule, rule_data)?;
            msg!("BBBB");

            option_data.serialize(&mut *option_info.data.borrow_mut())?;
            option_info_next = next_account_info(accounts_iter);
            vote.push(option_data.index);
        }
        msg!("---B");
        self.assert_valid_vote(&vote)?;
        msg!("---C");
        Ok(vote)
    }
}

/// Converts threshold in percentages to actual vote weight
/// and returns the min weight required for a proposal option to pass
/* fn get_min_vote_threshold_weight(
    vote_threshold_percentage: &AcceptenceCriteria,
    max_vote_weight: u64,
) -> Result<u64, ProgramError> {
    let yes_vote_threshold_percentage = match vote_threshold_percentage {
        AcceptenceCriteria::Threshold(percentage) => *percentage,
        _ => {
            return Err(PostError::VoteThresholdPercentageTypeNotSupported.into());
        }
    };

    let numerator = (yes_vote_threshold_percentage as u128)
        .checked_mul(max_vote_weight as u128)
        .unwrap();

    let mut yes_vote_threshold = numerator.checked_div(100).unwrap();

    if yes_vote_threshold.checked_mul(100).unwrap() < numerator {
        yes_vote_threshold = yes_vote_threshold.checked_add(1).unwrap();
    }

    Ok(yes_vote_threshold as u64)
}
 */
/// Deserializes Proposal account and checks owner program
pub fn get_proposal_data(
    program_id: &Pubkey,
    proposal_info: &AccountInfo,
) -> Result<ProposalV2, ProgramError> {
    let proposal = get_account_data::<ProposalV2>(program_id, proposal_info)?;
    Ok(proposal)
}
/// Deserializes Proposal and validates it belongs to the given Governance
pub fn get_proposal_data_for_governance(
    program_id: &Pubkey,
    proposal_info: &AccountInfo,
    governance: &Pubkey,
) -> Result<ProposalV2, ProgramError> {
    let proposal_data = get_proposal_data(program_id, proposal_info)?;

    if proposal_data.governance != *governance {
        return Err(GovernanceError::InvalidGovernanceForProposal.into());
    }

    Ok(proposal_data)
}

/// Deserializes Proposal and validates it belongs to the given Creator
pub fn get_proposal_data_for_creator(
    program_id: &Pubkey,
    proposal_info: &AccountInfo,
    creator_info: &AccountInfo,
) -> Result<ProposalV2, ProgramError> {
    if !creator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let proposal_data = get_proposal_data(program_id, proposal_info)?;

    if &proposal_data.creator != creator_info.key {
        return Err(GovernanceError::InvalidProposalOwnerAccount.into());
    }

    Ok(proposal_data)
}

/// Returns Proposal PDA seeds
pub fn get_proposal_address_seeds<'a>(
    governance: &'a Pubkey,
    proposal_index_le_bytes: &'a [u8],
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        PROGRAM_AUTHORITY_SEED,
        governance.as_ref(),
        proposal_index_le_bytes,
        bump_seed,
    ]
}

/// Returns Proposal PDA address
pub fn get_proposal_address<'a>(
    program_id: &Pubkey,
    governance: &'a Pubkey,
    proposal_index_le_bytes: &'a [u8],
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PROGRAM_AUTHORITY_SEED,
            governance.as_ref(),
            proposal_index_le_bytes,
        ],
        program_id,
    )
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    use crate::state::{
        vote_record::{Vote, VoteChoice},
    };

    use proptest::prelude::*;

    fn create_test_proposal() -> ProposalV2 {
        ProposalV2 {
            account_type: AccountType::Post,
            governance: Pubkey::new_unique(),
            max_vote_weight: Some(10),
            state: ProposalState::Draft,
            token_owner_record: Pubkey::new_unique(),
            signatories_count: 10,
            signatories_signed_off_count: 5,
            description_link: "This is my description".to_string(),
            name: "This is my name".to_string(),

            strictest_criteria: AcceptenceCriteria::Threshold(50),
            strictest_time_config: RuleTimeConfig {
                max_voting_time: 0,
                min_transaction_hold_up_time: 0,
                proposal_cool_off_time: 0,
            },
            max_vote_weights: None,

            start_voting_at: Some(0),
            draft_at: 10,
            signing_off_at: Some(10),

            voting_at: Some(10),
            voting_at_slot: Some(500),

            voting_completed_at: Some(10),
            executing_at: Some(10),
            closed_at: Some(10),

            vote_type: VoteType::SingleChoice,
            options: vec![ProposalOption {
                label: "yes".to_string(),
                vote_weight: 0,
                vote_result: OptionVoteResult::None,
                transactions_executed_count: 10,
                transactions_count: 10,
                transactions_next_index: 10,
            }],
            deny_vote_weight: Some(0),
            abstain_vote_weight: Some(0),
            veto_vote_weight: Some(0),

            execution_flags: InstructionExecutionFlags::Ordered,

            max_voting_time: Some(0),
            vote_threshold_percentage: Some(AcceptenceCriteria::Threshold(100)),

            reserved: [0; 64],
        }
    }

    fn create_test_multi_option_proposal() -> ProposalV2 {
        let mut proposal = create_test_proposal();
        proposal.options = vec![
            ProposalOption {
                label: "option 1".to_string(),
                vote_weight: 0,
                vote_result: OptionVoteResult::None,
                transactions_executed_count: 10,
                transactions_count: 10,
                transactions_next_index: 10,
            },
            ProposalOption {
                label: "option 2".to_string(),
                vote_weight: 0,
                vote_result: OptionVoteResult::None,
                transactions_executed_count: 10,
                transactions_count: 10,
                transactions_next_index: 10,
            },
            ProposalOption {
                label: "option 3".to_string(),
                vote_weight: 0,
                vote_result: OptionVoteResult::None,
                transactions_executed_count: 10,
                transactions_count: 10,
                transactions_next_index: 10,
            },
        ];

        proposal
    }

    fn create_test_rule_config() -> RuleConfig {
        RuleConfig {
            criteria: AcceptenceCriteria::default(),
            vote_tipping: VoteTipping::Disabled,
            info: None,
            name: None,
            rule_condition: None,
            vote_weights: vec![],
        }
    }

    #[test]
    fn test_max_size() {
        let mut proposal = create_test_proposal();
        proposal.vote_type = VoteType::MultiChoice {
            max_voter_options: 1,
            max_winning_options: 1,
        };

        let size = proposal.try_to_vec().unwrap().len();

        assert_eq!(proposal.get_max_size(), Some(size));
    }

    #[test]
    fn test_multi_option_proposal_max_size() {
        let mut proposal = create_test_multi_option_proposal();
        proposal.vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let size = proposal.try_to_vec().unwrap().len();

        assert_eq!(proposal.get_max_size(), Some(size));
    }

    prop_compose! {
        fn vote_results()(governing_token_supply in 1..=u64::MAX)(
            governing_token_supply in Just(governing_token_supply),
            vote_count in 0..=governing_token_supply,
        ) -> (u64, u64) {
            (vote_count as u64, governing_token_supply as u64)
        }
    }

    fn editable_signatory_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![Just(ProposalState::Draft)]
    }

    proptest! {
        #[test]
        fn test_assert_can_edit_signatories(state in editable_signatory_states()) {

            let mut proposal = create_test_proposal();
            proposal.state = state;
            proposal.assert_can_edit_signatories().unwrap();

        }

    }

    fn none_editable_signatory_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![
            Just(ProposalState::Voting),
            Just(ProposalState::Succeeded),
            Just(ProposalState::Executing),
            Just(ProposalState::ExecutingWithErrors),
            Just(ProposalState::Completed),
            Just(ProposalState::Cancelled),
            Just(ProposalState::Defeated),
            Just(ProposalState::SigningOff),
        ]
    }

    proptest! {
        #[test]
            fn test_assert_can_edit_signatories_with_invalid_state_error(state in none_editable_signatory_states()) {
                // Arrange
                let mut proposal = create_test_proposal();
                proposal.state = state;

                // Act
                let err = proposal.assert_can_edit_signatories().err().unwrap();

                // Assert
                assert_eq!(err, PostError::InvalidStateCannotEditSignatories.into());
        }

    }

    fn sign_off_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![Just(ProposalState::SigningOff), Just(ProposalState::Draft),]
    }
    proptest! {
        #[test]
        fn test_assert_can_sign_off(state in sign_off_states()) {
            let mut proposal = create_test_proposal();
            proposal.state = state;
            proposal.assert_can_sign_off().unwrap();
        }
    }

    fn none_sign_off_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![
            Just(ProposalState::Voting),
            Just(ProposalState::Succeeded),
            Just(ProposalState::Executing),
            Just(ProposalState::ExecutingWithErrors),
            Just(ProposalState::Completed),
            Just(ProposalState::Cancelled),
            Just(ProposalState::Defeated),
        ]
    }

    proptest! {
        #[test]
        fn test_assert_can_sign_off_with_state_error(state in none_sign_off_states()) {
                // Arrange
                let mut proposal = create_test_proposal();
                proposal.state = state;

                // Act
                let err = proposal.assert_can_sign_off().err().unwrap();

                // Assert
                assert_eq!(err, PostError::InvalidStateCannotSignOff.into());
        }
    }

    fn cancellable_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![
            Just(ProposalState::Draft),
            Just(ProposalState::SigningOff),
            Just(ProposalState::Voting),
        ]
    }

    proptest! {
        #[test]
        fn test_assert_can_cancel(state in cancellable_states()) {

            // Arrange
            let mut proposal = create_test_proposal();
            let rule_config = create_test_rule_config();

            // Act
            proposal.state = state;

            // Assert
            proposal.assert_can_cancel(&proposal.strictest_time_config,1).unwrap();

        }

    }

    fn none_cancellable_states() -> impl Strategy<Value = ProposalState> {
        prop_oneof![
            Just(ProposalState::Succeeded),
            Just(ProposalState::Executing),
            Just(ProposalState::ExecutingWithErrors),
            Just(ProposalState::Completed),
            Just(ProposalState::Cancelled),
            Just(ProposalState::Defeated),
        ]
    }

    proptest! {
        #[test]
            fn test_assert_can_cancel_with_invalid_state_error(state in none_cancellable_states()) {
                // Arrange
                let mut proposal = create_test_proposal();
                proposal.state = state;

                let rule_config = create_test_rule_config();

                // Act
                let err = proposal.assert_can_cancel(&proposal.strictest_time_config,1).err().unwrap();

                // Assert
                assert_eq!(err, PostError::InvalidStateCannotCancelProposal.into());
        }

    }

    #[derive(Clone, Debug)]
    pub struct VoteCastTestCase {
        #[allow(dead_code)]
        name: &'static str,
        governing_token_supply: u64,
        vote_threshold_percentage: u8,
        yes_votes_count: u64,
        no_votes_count: u64,
        expected_tipped_state: ProposalState,
        expected_finalized_state: ProposalState,
    }

    fn vote_casting_test_cases() -> impl Strategy<Value = VoteCastTestCase> {
        prop_oneof![
            //  threshold < 50%
            Just(VoteCastTestCase {
                name: "45:10 @40 -- Nays can still outvote Yeahs",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 45,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "49:50 @40 -- In best case scenario it can be 50:50 tie and hence Defeated",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 49,
                no_votes_count: 50,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "40:40 @40 -- Still can go either way",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 40,
                no_votes_count: 40,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "45:45 @40 -- Still can go either way",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 45,
                no_votes_count: 45,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "50:10 @40 -- Nay sayers can still tie up",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 50,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "50:50 @40 -- It's a tie and hence Defeated",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 50,
                no_votes_count: 50,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "45:51 @ 40 -- Nays won",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 45,
                no_votes_count: 51,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "40:55 @ 40 -- Nays won",
                governing_token_supply: 100,
                vote_threshold_percentage: 40,
                yes_votes_count: 40,
                no_votes_count: 55,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            // threshold == 50%
            Just(VoteCastTestCase {
                name: "50:10 @50 -- +1 tie breaker required to tip",
                governing_token_supply: 100,
                vote_threshold_percentage: 50,
                yes_votes_count: 50,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "10:50 @50 -- +1 tie breaker vote not possible any longer",
                governing_token_supply: 100,
                vote_threshold_percentage: 50,
                yes_votes_count: 10,
                no_votes_count: 50,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "50:50 @50 -- +1 tie breaker vote not possible any longer",
                governing_token_supply: 100,
                vote_threshold_percentage: 50,
                yes_votes_count: 50,
                no_votes_count: 50,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "51:10 @ 50 -- Nay sayers can't outvote any longer",
                governing_token_supply: 100,
                vote_threshold_percentage: 50,
                yes_votes_count: 51,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Succeeded,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "10:51 @ 50 -- Nays won",
                governing_token_supply: 100,
                vote_threshold_percentage: 50,
                yes_votes_count: 10,
                no_votes_count: 51,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            // threshold > 50%
            Just(VoteCastTestCase {
                name: "10:10 @ 60 -- Can still go either way",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 10,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "55:10 @ 60 -- Can still go either way",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 55,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "60:10 @ 60 -- Yeah reached the required threshold",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 60,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Succeeded,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "61:10 @ 60 -- Yeah won",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 61,
                no_votes_count: 10,
                expected_tipped_state: ProposalState::Succeeded,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "10:40 @ 60 -- Yeah can still outvote Nay",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 10,
                no_votes_count: 40,
                expected_tipped_state: ProposalState::Voting,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "60:40 @ 60 -- Yeah won",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 60,
                no_votes_count: 40,
                expected_tipped_state: ProposalState::Succeeded,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "10:41 @ 60 -- Aye can't outvote Nay any longer",
                governing_token_supply: 100,
                vote_threshold_percentage: 60,
                yes_votes_count: 10,
                no_votes_count: 41,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
            Just(VoteCastTestCase {
                name: "100:0",
                governing_token_supply: 100,
                vote_threshold_percentage: 100,
                yes_votes_count: 100,
                no_votes_count: 0,
                expected_tipped_state: ProposalState::Succeeded,
                expected_finalized_state: ProposalState::Succeeded,
            }),
            Just(VoteCastTestCase {
                name: "0:100",
                governing_token_supply: 100,
                vote_threshold_percentage: 100,
                yes_votes_count: 0,
                no_votes_count: 100,
                expected_tipped_state: ProposalState::Defeated,
                expected_finalized_state: ProposalState::Defeated,
            }),
        ]
    }

    proptest! {
        #[test]
        fn test_try_tip_vote(test_case in vote_casting_test_cases()) {
            // Arrange
            let mut proposal = create_test_proposal();

            proposal.options[0].vote_weight = test_case.yes_votes_count;
            proposal.deny_vote_weight = Some(test_case.no_votes_count);

            proposal.state = ProposalState::Voting;

            let mut rule_config = create_test_rule_config();
            rule_config.criteria =  AcceptenceCriteria::Threshold(test_case.vote_threshold_percentage);

            let current_timestamp = 15_i64;



            let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(test_case.governing_token_supply).unwrap();

            // Act
            proposal.try_tip_vote(max_voter_weight, &rule_config,current_timestamp).unwrap();

            // Assert
            assert_eq!(proposal.state,test_case.expected_tipped_state,"CASE: {:?}",test_case);

            if test_case.expected_tipped_state != ProposalState::Voting {
                assert_eq!(Some(current_timestamp),proposal.voting_completed_at);

            }

            match proposal.options[0].vote_result {
                OptionVoteResult::Succeeded => {
                    assert_eq!(ProposalState::Succeeded,test_case.expected_tipped_state)
                },
                OptionVoteResult::Defeated => {
                    assert_eq!(ProposalState::Defeated,test_case.expected_tipped_state)
                },
                OptionVoteResult::None =>  {
                    assert_eq!(ProposalState::Voting,test_case.expected_tipped_state)
                },
            };

        }

        #[test]
        fn test_finalize_vote(test_case in vote_casting_test_cases()) {
            // Arrange
            let mut proposal = create_test_proposal();

            proposal.options[0].vote_weight = test_case.yes_votes_count;
            proposal.deny_vote_weight = Some(test_case.no_votes_count);

            proposal.state = ProposalState::Voting;

            let mut rule_config = create_test_rule_config();
            rule_config.criteria = AcceptenceCriteria::Threshold(test_case.vote_threshold_percentage);

            let current_timestamp = 16_i64;


            let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(test_case.governing_token_supply).unwrap();

            // Act
            proposal.finalize_vote(max_voter_weight, &rule_config,current_timestamp).unwrap();

            // Assert
            assert_eq!(proposal.state,test_case.expected_finalized_state,"CASE: {:?}",test_case);
            assert_eq!(Some(current_timestamp),proposal.voting_completed_at);

            match proposal.options[0].vote_result {
                OptionVoteResult::Succeeded => {
                    assert_eq!(ProposalState::Succeeded,test_case.expected_finalized_state)
                },
                OptionVoteResult::Defeated => {
                    assert_eq!(ProposalState::Defeated,test_case.expected_finalized_state)
                },
                OptionVoteResult::None =>  {
                    panic!("Option result must be resolved for finalized vote")
                },
            };

        }
    }

    prop_compose! {
        fn full_vote_results()(governing_token_supply in 1..=u64::MAX, yes_vote_threshold in 1..100)(
            governing_token_supply in Just(governing_token_supply),
            yes_vote_threshold in Just(yes_vote_threshold),

            yes_votes_count in 0..=governing_token_supply,
            no_votes_count in 0..=governing_token_supply,

        ) -> (u64, u64, u64, u8) {
            (yes_votes_count as u64, no_votes_count as u64, governing_token_supply as u64,yes_vote_threshold as u8)
        }
    }

    proptest! {
        #[test]
        fn test_try_tip_vote_with_full_vote_results(
            (yes_votes_count, no_votes_count, governing_token_supply, yes_vote_threshold_percentage) in full_vote_results(),

        ) {
            // Arrange

            let mut proposal = create_test_proposal();

            proposal.options[0].vote_weight = yes_votes_count;
            proposal.deny_vote_weight = Some(no_votes_count.min(governing_token_supply-yes_votes_count));


            proposal.state = ProposalState::Voting;


            let mut rule_config = create_test_rule_config();
            let  yes_vote_threshold_percentage = AcceptenceCriteria::Threshold(yes_vote_threshold_percentage);
            rule_config.criteria = yes_vote_threshold_percentage.clone();

            let current_timestamp = 15_i64;


            let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(governing_token_supply).unwrap();

            // Act
            proposal.try_tip_vote(max_voter_weight, &rule_config, current_timestamp).unwrap();

            // Assert
            let yes_vote_threshold_count = get_min_vote_threshold_weight(&yes_vote_threshold_percentage,governing_token_supply).unwrap();

            let no_vote_weight = proposal.deny_vote_weight.unwrap();

            if yes_votes_count >= yes_vote_threshold_count && yes_votes_count > (governing_token_supply - yes_votes_count)
            {
                assert_eq!(proposal.state,ProposalState::Succeeded);
            } else if no_vote_weight > (governing_token_supply - yes_vote_threshold_count)
                || no_vote_weight >= (governing_token_supply - no_vote_weight ) {
                assert_eq!(proposal.state,ProposalState::Defeated);
            } else {
                assert_eq!(proposal.state,ProposalState::Voting);
            }
        }
    }

    proptest! {
        #[test]
        fn test_finalize_vote_with_full_vote_results(
            (yes_votes_count, no_votes_count, governing_token_supply, yes_vote_threshold_percentage) in full_vote_results(),

        ) {
            // Arrange
            let mut proposal = create_test_proposal();

            proposal.options[0].vote_weight = yes_votes_count;
            proposal.deny_vote_weight = Some(no_votes_count.min(governing_token_supply-yes_votes_count));

            proposal.state = ProposalState::Voting;


            let mut rule_config = create_test_rule_config();
            let  yes_vote_threshold_percentage = AcceptenceCriteria::Threshold(yes_vote_threshold_percentage);

            rule_config.criteria = yes_vote_threshold_percentage.clone();

            let current_timestamp = 16_i64;


            let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(governing_token_supply).unwrap();

            // Act
            proposal.finalize_vote(max_voter_weight, &rule_config,current_timestamp).unwrap();

            // Assert
            let no_vote_weight = proposal.deny_vote_weight.unwrap();

            let yes_vote_threshold_count = get_min_vote_threshold_weight(&yes_vote_threshold_percentage,governing_token_supply).unwrap();

            if yes_votes_count >= yes_vote_threshold_count &&  yes_votes_count > no_vote_weight
            {
                assert_eq!(proposal.state,ProposalState::Succeeded);
            } else {
                assert_eq!(proposal.state,ProposalState::Defeated);
            }
        }
    }
    /*
    #[test]
    fn test_try_tip_vote_with_reduced_community_mint_max_vote_weight() {
        // Arrange
        let mut proposal = create_test_proposal();

        proposal.options[0].vote_weight = 60;
        proposal.deny_vote_weight = Some(10);

        proposal.state = ProposalState::Voting;

        let mut rule_config = create_test_rule_config();
        rule_config.criteria = AcceptenceCriteria::Threshold(60);

        let current_timestamp = 15_i64;

        let community_token_supply = 200;

        // reduce max vote weight to 100
        realm.config.community_mint_max_vote_weight_source =
            MintMaxVoteWeightSource::SupplyFraction(
                MintMaxVoteWeightSource::SUPPLY_FRACTION_BASE / 2,
            );

        let max_voter_weight = proposal
            .get_max_voter_weight_from_mint_supply(community_token_supply)
            .unwrap();

        // Act
        proposal
            .try_tip_vote(max_voter_weight, &rule_config, current_timestamp)
            .unwrap();

        // Assert
        assert_eq!(proposal.state, ProposalState::Succeeded);
        assert_eq!(proposal.max_vote_weight, Some(100));
    } */

    #[test]
    pub fn test_finalize_vote_with_expired_voting_time_error() {
        // Arrange
        let mut proposal = create_test_proposal();
        proposal.state = ProposalState::Voting;
        let rule_config = create_test_rule_config();

        let current_timestamp = proposal.voting_at.unwrap() + rule_config.max_voting_time as i64;

        let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(100).unwrap();

        // Act
        let err = proposal
            .finalize_vote(max_voter_weight, &rule_config, current_timestamp)
            .err()
            .unwrap();

        // Assert
        assert_eq!(err, PostError::CannotFinalizeVotingInProgress.into());
    }

    #[test]
    pub fn test_finalize_vote_after_voting_time() {
        // Arrange
        let mut proposal = create_test_proposal();
        proposal.state = ProposalState::Voting;
        let rule_config = create_test_rule_config();

        let current_timestamp =
            proposal.voting_at.unwrap() + rule_config.max_voting_time as i64 + 1;

        let max_voter_weight = proposal.get_max_voter_weight_from_mint_supply(100).unwrap();

        // Act
        let result = proposal.finalize_vote(max_voter_weight, &rule_config, current_timestamp);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    pub fn test_assert_can_vote_with_expired_voting_time_error() {
        // Arrange
        let mut proposal = create_test_proposal();
        proposal.state = ProposalState::Voting;
        let rule_config = create_test_rule_config();

        let current_timestamp =
            proposal.voting_at.unwrap() + rule_config.max_voting_time as i64 + 1;

        // Act
        let err = proposal
            .assert_can_cast_vote(&rule_config, current_timestamp)
            .err()
            .unwrap();

        // Assert
        assert_eq!(err, PostError::ProposalVotingTimeExpired.into());
    }

    #[test]
    pub fn test_assert_can_vote_within_voting_time() {
        // Arrange
        let mut proposal = create_test_proposal();
        proposal.state = ProposalState::Voting;
        let rule_config = create_test_rule_config();

        let current_timestamp = proposal.voting_at.unwrap() + rule_config.max_voting_time as i64;

        // Act
        let result = proposal.assert_can_cast_vote(&rule_config, current_timestamp);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    pub fn test_assert_valid_vote_with_deny_vote_for_survey_only_proposal_error() {
        // Arrange
        let mut proposal = create_test_proposal();
        proposal.deny_vote_weight = None;

        // Survey only proposal can't be denied
        let vote = Vote::Deny;

        // Act
        let result = proposal.assert_valid_vote(&vote);

        // Assert
        assert_eq!(result, Err(PostError::InvalidVote.into()));
    }

    #[test]
    pub fn test_assert_valid_vote_with_too_many_options_error() {
        // Arrange
        let proposal = create_test_proposal();

        let choices = vec![
            VoteChoice {
                rank: 0,
                weight_percentage: 100,
            },
            VoteChoice {
                rank: 0,
                weight_percentage: 100,
            },
        ];

        let vote = Vote::Approve(choices.clone());

        // Ensure
        assert!(proposal.options.len() != choices.len());

        // Act
        let result = proposal.assert_valid_vote(&vote);

        // Assert
        assert_eq!(result, Err(PostError::InvalidVote.into()));
    }

    #[test]
    pub fn test_assert_valid_vote_with_no_choice_for_single_choice_error() {
        // Arrange
        let proposal = create_test_proposal();

        let choices = vec![VoteChoice {
            rank: 0,
            weight_percentage: 0,
        }];

        let vote = Vote::Approve(choices.clone());

        // Ensure
        assert_eq!(proposal.options.len(), choices.len());

        // Act
        let result = proposal.assert_valid_vote(&vote);

        // Assert
        assert_eq!(result, Err(PostError::InvalidVote.into()));
    }

    #[test]
    pub fn test_assert_valid_vote_with_to_many_choices_for_single_choice_error() {
        // Arrange
        let proposal = create_test_multi_option_proposal();
        let choices = vec![
            VoteChoice {
                rank: 0,
                weight_percentage: 100,
            },
            VoteChoice {
                rank: 0,
                weight_percentage: 100,
            },
            VoteChoice {
                rank: 0,
                weight_percentage: 0,
            },
        ];

        let vote = Vote::Approve(choices.clone());

        // Ensure
        assert_eq!(proposal.options.len(), choices.len());

        // Act
        let result = proposal.assert_valid_vote(&vote);

        // Assert
        assert_eq!(result, Err(PostError::InvalidVote.into()));
    }

    #[test]
    pub fn test_assert_valid_vote_with_no_choices_for_multi_choice_error() {
        // Arrange
        let mut proposal = create_test_multi_option_proposal();
        proposal.vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let choices = vec![
            VoteChoice {
                rank: 0,
                weight_percentage: 0,
            },
            VoteChoice {
                rank: 0,
                weight_percentage: 0,
            },
            VoteChoice {
                rank: 0,
                weight_percentage: 0,
            },
        ];

        let vote = Vote::Approve(choices.clone());

        // Ensure
        assert_eq!(proposal.options.len(), choices.len());

        // Act
        let result = proposal.assert_valid_vote(&vote);

        // Assert
        assert_eq!(result, Err(PostError::InvalidVote.into()));
    }

    #[test]
    pub fn test_assert_valid_proposal_options_with_invalid_choice_number_for_multi_choice_vote_error(
    ) {
        // Arrange
        let vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let options = vec!["option 1".to_string(), "option 2".to_string()];

        // Act
        let result = assert_valid_proposal_options(&options, &vote_type);

        // Assert
        assert_eq!(result, Err(PostError::InvalidProposalOptions.into()));
    }

    #[test]
    pub fn test_assert_valid_proposal_options_with_no_options_for_multi_choice_vote_error() {
        // Arrange
        let vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let options = vec![];

        // Act
        let result = assert_valid_proposal_options(&options, &vote_type);

        // Assert
        assert_eq!(result, Err(PostError::InvalidProposalOptions.into()));
    }

    #[test]
    pub fn test_assert_valid_proposal_options_with_no_options_for_single_choice_vote_error() {
        // Arrange
        let vote_type = VoteType::SingleChoice;

        let options = vec![];

        // Act
        let result = assert_valid_proposal_options(&options, &vote_type);

        // Assert
        assert_eq!(result, Err(PostError::InvalidProposalOptions.into()));
    }

    #[test]
    pub fn test_assert_valid_proposal_options_for_multi_choice_vote() {
        // Arrange
        let vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let options = vec![
            "option 1".to_string(),
            "option 2".to_string(),
            "option 3".to_string(),
        ];

        // Act
        let result = assert_valid_proposal_options(&options, &vote_type);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    pub fn test_assert_valid_proposal_options_for_multi_choice_vote_with_empty_option_error() {
        // Arrange
        let vote_type = VoteType::MultiChoice {
            max_voter_options: 3,
            max_winning_options: 3,
        };

        let options = vec![
            "".to_string(),
            "option 2".to_string(),
            "option 3".to_string(),
        ];

        // Act
        let result = assert_valid_proposal_options(&options, &vote_type);

        // Assert
        assert_eq!(result, Err(PostError::InvalidProposalOptions.into()));
    }
}
 */
