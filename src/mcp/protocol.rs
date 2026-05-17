use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl Response {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    pub fn err(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError { code, message: message.into() }),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub tools: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: Capabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem { kind: "text".into(), text: s.into() }],
            is_error: None,
        }
    }
    pub fn error(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentItem { kind: "text".into(), text: s.into() }],
            is_error: Some(true),
        }
    }
}

pub fn all_tools() -> Value {
    json!([
        {
            "name": "oc_get",
            "description": "Run 'oc get <resource> -n <namespace> -o json' with an optional label selector. Returns parsed JSON. Safer than raw shell — arguments are constructed without injection risk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "resource": { "type": "string", "description": "Resource type (e.g. 'pod', 'virtualmachine', 'datavolume')" },
                    "name": { "type": "string", "description": "Resource name. Omit to list all." },
                    "namespace": { "type": "string", "description": "Namespace. Omit for cluster-scoped." },
                    "label_selector": { "type": "string", "description": "Optional -l label selector" }
                },
                "required": ["resource"]
            }
        },
        {
            "name": "oc_apply_yaml",
            "description": "Apply a YAML manifest string using 'oc apply -f -'. Injects the namespace into resources if namespace is specified. Returns the oc output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "yaml": { "type": "string", "description": "YAML manifest content to apply" },
                    "namespace": { "type": "string", "description": "Namespace to apply into (adds -n flag)" }
                },
                "required": ["yaml"]
            }
        },
        {
            "name": "oc_delete",
            "description": "Delete a Kubernetes resource by type and name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "resource": { "type": "string", "description": "Resource type" },
                    "name": { "type": "string", "description": "Resource name" },
                    "namespace": { "type": "string", "description": "Namespace" },
                    "ignore_not_found": { "type": "boolean", "description": "Suppress error if resource does not exist. Default: true." }
                },
                "required": ["resource", "name"]
            }
        },
        {
            "name": "oc_wait",
            "description": "Run 'oc wait --for=condition=<condition>' on a resource with a timeout. Returns success or timeout message.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "resource": { "type": "string", "description": "Resource type/name (e.g. 'pod/mypod' or 'virtualmachine/myvm')" },
                    "condition": { "type": "string", "description": "Condition to wait for (e.g. 'Ready', 'Available', 'Progressing=False')" },
                    "namespace": { "type": "string", "description": "Namespace" },
                    "timeout_secs": { "type": "number", "description": "Timeout in seconds. Default: 300." }
                },
                "required": ["resource", "condition"]
            }
        },
        {
            "name": "oc_logs",
            "description": "Get pod logs. Returns the last N lines (default: 100).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pod_name": { "type": "string", "description": "Pod name or 'deployment/<name>'" },
                    "namespace": { "type": "string", "description": "Namespace" },
                    "container": { "type": "string", "description": "Container name. Omit for first/only container." },
                    "tail": { "type": "number", "description": "Number of lines from the end. Default: 100." }
                },
                "required": ["pod_name", "namespace"]
            }
        },
        {
            "name": "oc_exec",
            "description": "Execute a single command in a pod container. Returns stdout and stderr.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pod_name": { "type": "string", "description": "Pod name" },
                    "namespace": { "type": "string", "description": "Namespace" },
                    "command": { "type": "string", "description": "Command to run (passed to sh -c)" },
                    "container": { "type": "string", "description": "Container name. Omit for default." }
                },
                "required": ["pod_name", "namespace", "command"]
            }
        },
        {
            "name": "virtctl_migrate",
            "description": "Trigger a live migration of a VirtualMachine using virtctl.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "vm_name": { "type": "string", "description": "VirtualMachine name" },
                    "namespace": { "type": "string", "description": "Namespace" }
                },
                "required": ["vm_name", "namespace"]
            }
        },
        {
            "name": "virtctl_pause",
            "description": "Pause a VirtualMachineInstance using virtctl.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "vm_name": { "type": "string", "description": "VM name" },
                    "namespace": { "type": "string", "description": "Namespace" }
                },
                "required": ["vm_name", "namespace"]
            }
        },
        {
            "name": "virtctl_unpause",
            "description": "Unpause a VirtualMachineInstance using virtctl.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "vm_name": { "type": "string", "description": "VM name" },
                    "namespace": { "type": "string", "description": "Namespace" }
                },
                "required": ["vm_name", "namespace"]
            }
        },
        {
            "name": "virtctl_ssh",
            "description": "Run a single command inside a VM guest via virtctl ssh. Requires SSH access to the VM.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "vm_name": { "type": "string", "description": "VM name" },
                    "namespace": { "type": "string", "description": "Namespace" },
                    "command": { "type": "string", "description": "Command to run on the guest" },
                    "username": { "type": "string", "description": "SSH username. Default: fedora." }
                },
                "required": ["vm_name", "namespace", "command"]
            }
        },
        {
            "name": "cleanup_namespace",
            "description": "Delete all KubeVirt objects in a pw-* test namespace in the correct order (VMIs force-deleted first, then VMs, then DataVolumes, PVCs, templates, secrets). Mirrors OcCliClient.cleanupTestNamespace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "pw-* namespace to clean up" }
                },
                "required": ["namespace"]
            }
        }
    ])
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_ok_no_error() {
        let r = Response::ok(Some(json!(1)), json!({}));
        assert!(r.error.is_none());
        assert!(r.result.is_some());
    }

    #[test]
    fn response_err_no_result() {
        let r = Response::err(Some(json!(2)), -32600, "bad request");
        assert!(r.result.is_none());
        assert_eq!(r.error.as_ref().unwrap().message, "bad request");
    }

    #[test]
    fn null_fields_omitted_in_serialization() {
        let r = Response::ok(None, json!({}));
        let s = serde_json::to_string(&r).unwrap();
        assert!(!s.contains("\"error\""));
        assert!(!s.contains("\"id\""));
    }

    #[test]
    fn tool_result_text_not_error() {
        let t = ToolCallResult::text("output");
        assert!(t.is_error.is_none());
        assert_eq!(t.content[0].kind, "text");
    }

    #[test]
    fn tool_result_error_is_flagged() {
        let t = ToolCallResult::error("failed");
        assert_eq!(t.is_error, Some(true));
        assert_eq!(t.content[0].text, "failed");
    }

    #[test]
    fn all_eleven_tools_present() {
        let tools = all_tools();
        let names: Vec<&str> = tools.as_array().unwrap()
            .iter()
            .filter_map(|t| t.get("name").and_then(|v| v.as_str()))
            .collect();
        for expected in &[
            "oc_get", "oc_apply_yaml", "oc_delete", "oc_wait", "oc_logs", "oc_exec",
            "virtctl_migrate", "virtctl_pause", "virtctl_unpause", "virtctl_ssh",
            "cleanup_namespace",
        ] {
            assert!(names.contains(expected), "missing tool: {}", expected);
        }
        assert_eq!(names.len(), 11);
    }

    #[test]
    fn cleanup_namespace_has_required_param() {
        let tools = all_tools();
        let cleanup = tools.as_array().unwrap()
            .iter()
            .find(|t| t.get("name").and_then(|v| v.as_str()) == Some("cleanup_namespace"))
            .unwrap();
        let required = cleanup.get("inputSchema").and_then(|s| s.get("required")).and_then(|r| r.as_array()).unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("namespace")));
    }
}
