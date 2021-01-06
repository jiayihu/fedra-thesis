use once_cell::sync::Lazy;

use k8s_openapi::api::core::v1::{Pod as KubePod, Volume as KubeVolume};
use kube::api::Meta;

static EMPTY_MAP: Lazy<std::collections::BTreeMap<String, String>> =
    Lazy::new(|| std::collections::BTreeMap::new());

/// A Kubernetes Pod
///
/// This is a new type around the k8s_openapi Pod definition
/// providing convenient accessor methods
///
/// Copied over from `krustlet/crates/kubelet/src/pod/mod.rs`. Some methods have been removed from
/// the source, e.g. container info and deletion timestamp.
#[derive(Default, Debug, Clone)]
pub struct Pod {
    kube_pod: KubePod,
}

impl Pod {
    /// Get the name of the pod
    pub fn name(&self) -> &str {
        self.kube_pod
            .metadata
            .name
            .as_deref()
            .expect("Pod name should always be set but was not")
    }

    /// Get the pod's namespace
    ///
    /// Returns "default" if no namespace was explictily set
    pub fn namespace(&self) -> &str {
        self.kube_pod
            .metadata
            .namespace
            .as_deref()
            .unwrap_or("default")
    }

    /// Get the pod's node_selector map
    pub fn node_selector(&self) -> Option<&std::collections::BTreeMap<String, String>> {
        self.kube_pod.spec.as_ref()?.node_selector.as_ref()
    }

    /// Get the pod's service account name
    pub fn service_account_name(&self) -> Option<&str> {
        let spec = self.kube_pod.spec.as_ref()?;
        spec.service_account_name.as_deref()
    }

    /// Get the pod volumes
    pub fn volumes(&self) -> Option<&Vec<KubeVolume>> {
        let spec = self.kube_pod.spec.as_ref()?;
        spec.volumes.as_ref()
    }

    /// Get the pod's host ip
    pub fn host_ip(&self) -> Option<&str> {
        let status = self.kube_pod.status.as_ref()?;
        status.host_ip.as_deref()
    }

    /// Get the pod's ip
    pub fn pod_ip(&self) -> Option<&str> {
        let status = self.kube_pod.status.as_ref()?;
        status.pod_ip.as_deref()
    }

    /// Get an iterator over the pod's labels
    pub fn labels(&self) -> &std::collections::BTreeMap<String, String> {
        self.kube_pod.meta().labels.as_ref().unwrap_or(&EMPTY_MAP)
    }

    ///  Get the pod's annotations
    pub fn annotations(&self) -> &std::collections::BTreeMap<String, String> {
        self.kube_pod
            .meta()
            .annotations
            .as_ref()
            .unwrap_or(&EMPTY_MAP)
    }

    /// Get the names of the pod's image pull secrets
    pub fn image_pull_secrets(&self) -> Vec<String> {
        match self.kube_pod.spec.as_ref() {
            None => vec![],
            Some(spec) => match spec.image_pull_secrets.as_ref() {
                None => vec![],
                Some(objrefs) => objrefs
                    .iter()
                    .filter_map(|objref| objref.name.clone())
                    .collect(),
            },
        }
    }

    /// Indicate if this pod is a static pod.
    /// TODO: A missing owner_references field was an indication of static pod in my testing but I
    /// dont know how reliable this is.
    pub fn is_static(&self) -> bool {
        self.kube_pod.meta().owner_references.is_none()
    }

    /// Indicate if this pod is part of a Daemonset
    pub fn is_daemonset(&self) -> bool {
        if let Some(owners) = &self.kube_pod.meta().owner_references {
            for owner in owners {
                if owner.kind == "DaemonSet" {
                    return true;
                }
            }
        }
        false
    }

    ///  Get a specific annotation from the pod
    pub fn get_annotation(&self, key: &str) -> Option<&str> {
        Some(self.annotations().get(key)?.as_str())
    }

    /// Turn the Pod into the Kubernetes API version of a Pod
    pub fn into_kube_pod(self) -> KubePod {
        self.kube_pod
    }

    /// Turn a reference to a Pod into a reference to the Kubernetes API version of a Pod
    pub fn as_kube_pod(&self) -> &KubePod {
        &self.kube_pod
    }
}

impl std::convert::From<KubePod> for Pod {
    fn from(api_pod: KubePod) -> Self {
        Self { kube_pod: api_pod }
    }
}

impl<'a> std::convert::From<&'a Pod> for &'a KubePod {
    fn from(pod: &'a Pod) -> Self {
        &pod.kube_pod
    }
}
impl std::convert::From<Pod> for KubePod {
    fn from(pod: Pod) -> Self {
        pod.kube_pod
    }
}

/// PodKey is a unique human readable key for storing a handle to a pod in a hash.
///
/// Copied over from `krustlet/crates/kubelet/src/pod/mod.rs`
#[derive(Hash, Ord, Eq, PartialOrd, PartialEq, Debug, Clone, Default)]
pub struct PodKey {
    name: String,
    namespace: String,
}

impl PodKey {
    /// Returns the name of the pod in the pod key
    #[allow(unused)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the namespace of the pod in the pod key
    #[allow(unused)]
    pub fn namespace(&self) -> String {
        self.namespace.clone()
    }
}

impl From<Pod> for PodKey {
    fn from(p: Pod) -> Self {
        PodKey {
            name: p.name().to_owned(),
            namespace: p.namespace().to_owned(),
        }
    }
}

impl From<&Pod> for PodKey {
    fn from(p: &Pod) -> Self {
        PodKey {
            name: p.name().to_owned(),
            namespace: p.namespace().to_owned(),
        }
    }
}

impl From<KubePod> for PodKey {
    fn from(p: KubePod) -> Self {
        PodKey {
            name: p.name(),
            namespace: p.namespace().unwrap_or_else(|| "default".to_string()),
        }
    }
}

impl From<&KubePod> for PodKey {
    fn from(p: &KubePod) -> Self {
        PodKey {
            name: p.name(),
            namespace: p.namespace().unwrap_or_else(|| "default".to_string()),
        }
    }
}
