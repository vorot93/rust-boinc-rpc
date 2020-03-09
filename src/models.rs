use super::util;

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
pub struct TaskResult {
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
    pub active_task: Option<ActiveTask>,
}

#[derive(Clone, Debug, Default)]
pub struct ActiveTask {
    pub active_task_state: Option<String>,
    pub app_version_num: Option<String>,
    pub slot: Option<u64>,
    pub pid: Option<u64>,
    pub scheduler_state: Option<String>,
    pub checkpoint_cpu_time: Option<f64>,
    pub fraction_done: Option<f64>,
    pub current_cpu_time: Option<f64>,
    pub elapsed_time: Option<f64>,
    pub swap_size: Option<f64>,
    pub working_set_size: Option<f64>,
    pub working_set_size_smoothed: Option<f64>,
    pub page_fault_rate: Option<f64>,
    pub bytes_sent: Option<f64>,
    pub bytes_received: Option<f64>,
    pub progress_rate: Option<f64>,
}

impl<'a> From<&'a treexml::Element> for ActiveTask {
    fn from(node: &treexml::Element) -> Self {
        let mut e = Self::default();
        for n in &node.children {
            match &*n.name {
                "active_task_state" => {
                    e.active_task_state = util::trimmed_optional(&n.text);
                }
                "app_version_num" => {
                    e.app_version_num = util::trimmed_optional(&n.text);
                }
                "slot" => {
                    e.slot = util::eval_node_contents(n);
                }
                "pid" => {
                    e.pid = util::eval_node_contents(n);
                }
                "scheduler_state" => {
                    e.scheduler_state = util::trimmed_optional(&n.text);
                }
                "checkpoint_cpu_time" => {
                    e.checkpoint_cpu_time = util::eval_node_contents(n);
                }
                "fraction_done" => {
                    e.fraction_done = util::eval_node_contents(n);
                }
                "current_cpu_time" => {
                    e.current_cpu_time = util::eval_node_contents(n);
                }
                "elapsed_time" => {
                    e.elapsed_time = util::eval_node_contents(n);
                }
                "swap_size" => {
                    e.swap_size = util::eval_node_contents(n);
                }
                "working_set_size" => {
                    e.working_set_size = util::eval_node_contents(n);
                }
                "working_set_size_smoothed" => {
                    e.working_set_size_smoothed = util::eval_node_contents(n);
                }
                "page_fault_rate" => {
                    e.page_fault_rate = util::eval_node_contents(n);
                }
                "bytes_sent" => {
                    e.bytes_sent = util::eval_node_contents(n);
                }
                "bytes_received" => {
                    e.bytes_received = util::eval_node_contents(n);
                }
                "progress_rate" => {
                    e.progress_rate = util::eval_node_contents(n);
                }
                _ => {}
            }
        }
        e
    }
}
