use super::{Constraint, Risk, SuccessCriteria, NonFunctionalRequirement, FunctionalRequirement};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Requirements {
    pub stakeholder_needs: Vec<String>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub constraints: Vec<Constraint>,
    pub risks: Vec<Risk>,
    pub success_criteria: Vec<SuccessCriteria>,
}