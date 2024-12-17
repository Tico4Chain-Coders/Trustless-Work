use soroban_sdk::{Address, Env, String, Vec};
use crate::storage::types::{DataKey, Escrow, Milestone};
use crate::error::ContractError;
use crate::events::escrows_by_engagement_id;
use crate::core::escrow::EscrowManager;

pub struct MilestoneManager;

impl MilestoneManager {
    pub fn change_milestone_status(
        e: Env,
        engagement_id: String,
        milestone_index: i128,
        new_status: String,
        service_provider: Address,
    ) -> Result<(), ContractError> {
        let existing_escrow = EscrowManager::get_escrow_by_id(e.clone(), engagement_id.clone())?;
    
        if existing_escrow.engagement_id != engagement_id {
            return Err(ContractError::EscrowNotInitialized);
        }
    
        if service_provider != existing_escrow.service_provider {
            return Err(ContractError::OnlyServiceProviderChangeMilstoneStatus);
        }
        service_provider.require_auth();
    
        if existing_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }
    
        if milestone_index < 0 || milestone_index >= existing_escrow.milestones.len() as i128 {
            return Err(ContractError::InvalidMileStoneIndex);
        }

        let mut updated_milestones = Vec::<Milestone>::new(&e);
        for (index, milestone) in existing_escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.status = new_status.clone();
            }
            updated_milestones.push_back(new_milestone);
        }
    
        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };
    
        e.storage().instance().set(
            &DataKey::Escrow(engagement_id.clone().into()),
            &updated_escrow,
        );
    
        escrows_by_engagement_id(&e, engagement_id, updated_escrow);
    
        Ok(())
    }
    
    pub fn change_milestone_flag(
        e: Env,
        engagement_id: String,
        milestone_index: i128,
        new_flag: bool,
        client: Address,
    ) -> Result<(), ContractError> {
        let existing_escrow = EscrowManager::get_escrow_by_id(e.clone(), engagement_id.clone())?;
    
        if existing_escrow.engagement_id != engagement_id {
            return Err(ContractError::EscrowNotInitialized);
        }
    
        if client != existing_escrow.client {
            return Err(ContractError::OnlyClientChangeMilstoneFlag);
        }

        client.require_auth();
    
        if existing_escrow.milestones.is_empty() {
            return Err(ContractError::NoMileStoneDefined);
        }

        if milestone_index < 0 || milestone_index >= existing_escrow.milestones.len() as i128 {
            return Err(ContractError::InvalidMileStoneIndex);
        }
    
        let mut updated_milestones = Vec::<Milestone>::new(&e);
        for (index, milestone) in existing_escrow.milestones.iter().enumerate() {
            let mut new_milestone = milestone.clone();
            if index as i128 == milestone_index {
                new_milestone.flag = new_flag;
            }
            updated_milestones.push_back(new_milestone);
        }
    
        let updated_escrow = Escrow {
            milestones: updated_milestones,
            ..existing_escrow
        };
    
        e.storage().instance().set(
            &DataKey::Escrow(engagement_id.clone().into()),
            &updated_escrow,
        );
    
        escrows_by_engagement_id(&e, engagement_id, updated_escrow);
    
        Ok(())
    }

}