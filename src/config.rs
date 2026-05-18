use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub kubeconfig: Option<PathBuf>,
    pub oc_path: String,
    pub virtctl_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        let kubeconfig = std::env::var("KUBECONFIG").ok().map(PathBuf::from).or_else(|| {
            // Try kubevirt-ui playwright kubeconfig if KUBEVIRT_PROJECT_ROOT is set
            if let Ok(root) = std::env::var("KUBEVIRT_PROJECT_ROOT") {
                let pw_cfg = PathBuf::from(root).join(".kubeconfigs/test-config");
                if pw_cfg.exists() {
                    return Some(pw_cfg);
                }
            }
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            let default = PathBuf::from(home).join(".kube/config");
            if default.exists() { Some(default) } else { None }
        });

        let oc_path = std::env::var("OC_PATH").unwrap_or_else(|_| "oc".into());
        let virtctl_path = std::env::var("VIRTCTL_PATH").unwrap_or_else(|_| "virtctl".into());

        Self { kubeconfig, oc_path, virtctl_path }
    }
}
