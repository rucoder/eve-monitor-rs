use std::{cell::RefCell, collections::HashMap};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::ipc::eve_types::{
    AppInstanceStatus, AppInstanceSummary, AppsList, DataSecAtRestStatus, DeviceNetworkStatus,
    DownloaderStatus, ErrorAndTime, EveNodeStatus, EveOnboardingStatus, EveVaultStatus, PCRStatus,
    SwState,
};

use super::device::network::NetworkInterfaceStatus;

#[derive(Debug, Clone, Default)]
pub enum OnboardingStatus {
    #[default]
    Unknown,
    Onboarding,
    Onboarded(Uuid),
    Error(String),
}

#[derive(Debug, Default)]
pub struct NodeStatus {
    pub server: Option<String>,
    pub app_summary: AppInstanceSummary,
    pub onboarding_status: OnboardingStatus,
}

#[derive(Debug)]
pub enum AppInstanceState {
    Normal(SwState),
    Error(SwState, String),
}

#[derive(Debug)]
pub struct AppInstance {
    pub name: String,
    pub uuid: Uuid,
    pub version: String,
    pub state: AppInstanceState,
}

#[derive(Debug)]
pub struct EveError {
    pub error: String,
    pub time: DateTime<Utc>,
}

impl From<ErrorAndTime> for EveError {
    fn from(error_and_time: ErrorAndTime) -> Self {
        Self {
            error: error_and_time.error_description.error,
            time: error_and_time.error_description.error_time,
        }
    }
}

#[derive(Debug)]
pub enum VaultStatus {
    Unknown,
    EncriptionDisabled(EveError, bool),
    Unlocked(bool),
    Locked(EveError, Option<Vec<i32>>),
}

pub type Model = RefCell<MonitorModel>;
#[derive(Debug)]
pub struct MonitorModel {
    pub dmesg: Vec<rmesg::entry::Entry>,
    pub network: Vec<NetworkInterfaceStatus>,
    pub downloader: Option<DownloaderStatus>,
    pub node_status: NodeStatus,
    pub apps: HashMap<Uuid, AppInstance>,
    pub vault_status: VaultStatus,
}

impl From<EveVaultStatus> for VaultStatus {
    fn from(vault_status: EveVaultStatus) -> Self {
        let tpm_used = vault_status.pcr_status == PCRStatus::PcrEnabled;
        match vault_status.status {
            DataSecAtRestStatus::DataSecAtRestUnknown => Self::Unknown,
            DataSecAtRestStatus::DataSecAtRestDisabled => {
                let reason = EveError::from(vault_status.error_and_time);
                Self::EncriptionDisabled(reason, tpm_used)
            }
            DataSecAtRestStatus::DataSecAtRestEnabled => Self::Unlocked(tpm_used),
            DataSecAtRestStatus::DataSecAtRestError => {
                let err = EveError::from(vault_status.error_and_time);

                let pcrs = if err.error.contains("Vault key unavailable") {
                    vault_status.missmatching_pcrs
                } else {
                    None
                };
                Self::Locked(err, pcrs)
            }
        }
    }
}

impl From<AppInstanceStatus> for AppInstance {
    fn from(app: AppInstanceStatus) -> Self {
        let state = if !app
            .error_and_time_with_source
            .error_description
            .error
            .is_empty()
        {
            AppInstanceState::Error(
                app.state,
                app.error_and_time_with_source.error_description.error,
            )
        } else {
            AppInstanceState::Normal(app.state)
        };

        AppInstance {
            name: app.display_name,
            uuid: app.uuid_and_version.uuid,
            version: app.uuid_and_version.version,
            state,
        }
    }
}

impl From<AppsList> for HashMap<Uuid, AppInstance> {
    fn from(apps_list: AppsList) -> Self {
        apps_list
            .apps
            .into_iter()
            .map(|app| (app.uuid_and_version.uuid.clone(), AppInstance::from(app)))
            .collect()
    }
}

impl From<EveNodeStatus> for NodeStatus {
    fn from(node_status: EveNodeStatus) -> Self {
        let onboarding_status = match (node_status.onboarded, node_status.node_uuid) {
            (true, Some(uuid)) => OnboardingStatus::Onboarded(uuid),
            (true, None) => OnboardingStatus::Error("Node UUID is missing".to_string()),
            (false, _) => OnboardingStatus::Onboarding,
        };
        NodeStatus {
            server: node_status.server.clone(),
            app_summary: node_status.app_instance_summary.unwrap_or_default(),
            onboarding_status,
        }
    }
}

impl MonitorModel {
    fn get_network_settings(
        &self,
        network_status: DeviceNetworkStatus,
    ) -> Option<Vec<NetworkInterfaceStatus>> {
        let ports = network_status.ports.as_ref()?;
        Some(ports.iter().map(|p| p.into()).collect())
    }
    pub fn update_app_status(&mut self, state: AppInstanceStatus) {
        let app_guid = &state.uuid_and_version.uuid;
        self.apps
            .entry(*app_guid)
            .and_modify(|e| *e = AppInstance::from(state.clone()))
            .or_insert(AppInstance::from(state));
    }

    pub fn update_app_list(&mut self, apps_list: AppsList) {
        self.apps = HashMap::from(apps_list);
    }

    pub fn update_downloader_status(&mut self, status: DownloaderStatus) {
        self.downloader = Some(status);
    }

    pub fn update_node_status(&mut self, status: EveNodeStatus) {
        self.node_status = NodeStatus::from(status);
    }

    pub fn update_app_summary(&mut self, app_summary: AppInstanceSummary) {
        self.node_status.app_summary = app_summary;
    }

    pub fn update_network_status(&mut self, net_status: DeviceNetworkStatus) {
        self.network = self.get_network_settings(net_status).unwrap_or_default();
    }

    pub fn update_vault_status(&mut self, vault_status: EveVaultStatus) {
        self.vault_status = VaultStatus::from(vault_status);
    }

    pub fn update_onboarding_status(&mut self, status: EveOnboardingStatus) {
        self.node_status.onboarding_status = OnboardingStatus::Onboarded(status.device_uuid);
    }
}

impl Default for MonitorModel {
    fn default() -> Self {
        MonitorModel {
            dmesg: Vec::with_capacity(1000),
            network: Vec::new(),
            downloader: None,
            node_status: NodeStatus::default(),
            apps: HashMap::new(),
            vault_status: VaultStatus::Unknown,
        }
    }
}
