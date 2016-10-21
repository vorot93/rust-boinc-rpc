extern crate std;

#[derive(Clone, Copy, Debug)]
pub enum Component {
    CPU,
    GPU,
    Network,
}

#[derive(Clone, Copy, Debug)]
pub enum RunMode {
    Always,
    Auto,
    Never,
    Restore,
}

#[derive(Clone, Copy, Debug)]
pub enum CpuSched {
    Uninitialized,
    Preempted,
    Scheduled,
}

#[derive(Clone, Copy, Debug)]
pub enum ResultState {
    New,
    FilesDownloading,
    FilesDownloaded,
    ComputeError,
    FilesUploading,
    FilesUploaded,
    Aborted,
    UploadFailed,
}

#[derive(Clone, Copy, Debug)]
pub enum Process {
    Uninitialized = 0,
    Executing = 1,
    Suspended = 9,
    AbortPending = 5,
    QuitPending = 8,
    CopyPending = 10,
}

#[derive(Clone, Debug, Default)]
pub struct VersionInfo {
    pub major: Option<i64>,
    pub minor: Option<i64>,
    pub release: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct HostInfo {
    pub tz_shift: Option<i64>,
    pub domain_name: Option<String>,
    pub serialnum: Option<String>,
    pub ip_addr: Option<String>,
    pub host_cpid: Option<String>,

    pub p_ncpus: Option<i64>,
    pub p_vendor: Option<String>,
    pub p_model: Option<String>,
    pub p_features: Option<String>,
    pub p_fpops: Option<f64>,
    pub p_iops: Option<f64>,
    pub p_membw: Option<f64>,
    pub p_calculated: Option<f64>,
    pub p_vm_extensions_disabled: Option<bool>,

    pub m_nbytes: Option<f64>,
    pub m_cache: Option<f64>,
    pub m_swap: Option<f64>,

    pub d_total: Option<f64>,
    pub d_free: Option<f64>,

    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub product_name: Option<String>,

    pub mac_address: Option<String>,

    pub virtualbox_version: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ProjectInfo {
    pub name: Option<String>,
    pub summary: Option<String>,
    pub url: Option<String>,
    pub general_area: Option<String>,
    pub specific_area: Option<String>,
    pub description: Option<String>,
    pub home: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub image: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AccountManagerInfo {
    pub url: Option<String>,
    pub name: Option<String>,
    pub have_credentials: Option<bool>,
    pub cookie_required: Option<bool>,
    pub cookie_failure_url: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Message {
    pub project_name: Option<String>,
    pub priority: Option<i64>,
    pub msg_number: Option<i64>,
    pub body: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct Result {
    pub name: Option<String>,
    pub wu_name: Option<String>,
    pub platform: Option<String>,
    pub version_num: Option<i64>,
    pub plan_class: Option<String>,
    pub project_url: Option<String>,
    pub final_cpu_time: Option<f64>,
    pub final_elapsed_time: Option<f64>,
    pub exit_status: Option<i64>,
    pub state: Option<i64>,
    pub report_deadline: Option<f64>,
    pub received_time: Option<f64>,
    pub estimated_cpu_time_remaining: Option<f64>,
    pub completed_time: Option<f64>,
}
