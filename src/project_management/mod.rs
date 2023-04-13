mod requirements;
pub use requirements::Requirements;

macro_rules! enum_without_associated_data {
    ($name:ident, $($variant:ident),+ $(,)?) => {
        #[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
        pub enum $name {
            $($variant),+
        }
    };
}

macro_rules! enum_with_associated_data {
    ($name:ident($associated_data:ident), $($variant:ident),+ $(,)?) => {
        #[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
        pub enum $name {
            $($variant($associated_data)),+
        }
    };
}

macro_rules! struct_with_fields {
    ($name:ident, $($field_name:ident $type:ty),+ $(,)?) => {
        #[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
        pub struct $name {
            $(pub $field_name: $type),+
        }
    };
}

struct_with_fields!(Details, name String, details String, lower_bound Option<f64>, upper_bound Option<f64>);
struct_with_fields!(FunctionalRequirementDetails, name String, description String, priority Priority, acceptance_criteria Vec<String>, dependencies Vec<String>);
struct_with_fields!(RiskDetails, description String, impact String, mitigation_strategy String);

enum_with_associated_data!(Constraint(Details), Accessibility, Budget, Collaboration, Compatibility, Communication, Customizability, Documentation, Efficiency, Flexibility, Interoperability, Legal, Localization, Maintainability, Modularity, Performance, Portability, Privacy, Regulatory, Reliability, Resilience, Resources, Scalability, Security, Technology, Time, Usability);

enum_with_associated_data!(FunctionalRequirement(FunctionalRequirementDetails), Auditing, Authentication, Authorization, Collaboration, Configuration, ContentManagement, DataImportExport, DataManagement, DataValidation, ErrorHandling, Filtering, Integration, Localization, Logging, Navigation, Notification, Personalization, Reporting, Search, UserManagement);

enum_with_associated_data!(NonFunctionalRequirement(Details), Adaptability, Analytics, ErrorHandling, FaultTolerance, Flexibility, Interoperability, Maintainability, Monitoring, Performance, Recoverability, Reliability, ResourceEfficiency, Robustness, Scalability, Security, Testability, Usability, VersionControl);

enum_without_associated_data!(Priority, Critical, High, Low, Medium, NiceToHave);
enum_with_associated_data!(Risk(RiskDetails), Budget, Capacity, ChangeAdoption, ChangeManagement, Compliance, Competitive, CustomerRetention, Cybersecurity, Data, DisasterRecovery, Environmental, Ethical, Financial, Geopolitical, HumanResource, Infrastructure, Integration, IntellectualProperty, Knowledge, LegalAndRegulatory, Market, Operational, OrganizationalCulture, Procurement, ProjectManagement, Quality, Reputational, Requirements, Resource, Risk, Schedule, Security, Stakeholder, Technical, TechnologicalObsolescence, Vendor);

enum_with_associated_data!(SuccessCriteria(Details), Adaptability, BusinessAlignment, Compliance, CostEffectiveness, CustomerSatisfaction, Innovation, LearningAndGrowth, MarketShareGrowth, Performance, ProjectDelivery, QualityAssurance, ResourceUtilization, ReturnOnInvestment, RiskMitigation, Scalability, Security, StakeholderEngagement, Timeliness, Usability, UserAcceptance);