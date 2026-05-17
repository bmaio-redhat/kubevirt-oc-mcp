# kubevirt-oc-mcp

MCP server providing structured `oc` (OpenShift CLI) and `virtctl` (KubeVirt CLI) operations for the [kubevirt-ui](https://github.com/kubevirt-ui/kubevirt-ui) Playwright E2E project.

Wraps common CLI operations as typed MCP tools with safe argument construction — no shell injection, no raw string interpolation. Agents can run `oc get`, apply YAML, wait for conditions, stream logs, exec into pods, trigger VM migrations, SSH into guest VMs, and bulk-clean test namespaces using the same patterns as `OcCliClient` and `VirtctlClient` in the test framework.

## Tools

| Tool | Description |
|------|-------------|
| `oc_get` | `oc get <resource> [-n ns] [-l selector] -o json` |
| `oc_apply_yaml` | `oc apply -f -` with YAML string as stdin |
| `oc_delete` | `oc delete <resource> <name>` with `--ignore-not-found` |
| `oc_wait` | `oc wait --for=condition=<cond>` with configurable timeout |
| `oc_logs` | Get pod logs (`--tail=N`) |
| `oc_exec` | `oc exec <pod> -- sh -c <command>` |
| `virtctl_migrate` | `virtctl migrate <vm> -n <ns>` |
| `virtctl_pause` | `virtctl pause vm <vm> -n <ns>` |
| `virtctl_unpause` | `virtctl unpause vm <vm> -n <ns>` |
| `virtctl_ssh` | `virtctl ssh <user>@<vm> -n <ns> -- <command>` |
| `cleanup_namespace` | Delete all KubeVirt objects in a `pw-*` namespace in the correct order |

## Installation

### Prerequisites

- [Rust](https://rustup.rs) 1.75 or later
- `oc` CLI on `PATH` (or set `OC_PATH`)
- `virtctl` CLI on `PATH` (or set `VIRTCTL_PATH`)

### Build

```bash
git clone https://github.com/kubevirt-ui/kubevirt-oc-mcp
cd kubevirt-oc-mcp
cargo build --release
```

The binary is produced at `target/release/kubevirt-oc-mcp`.

### Add to Cursor `mcp.json`

In your project's `.cursor/mcp.json`:

```json
"kubevirt-oc-mcp": {
  "command": "/path/to/kubevirt-oc-mcp/target/release/kubevirt-oc-mcp",
  "args": [],
  "env": {
    "KUBEVIRT_PROJECT_ROOT": "/path/to/kubevirt-ui"
  }
}
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `KUBECONFIG` | See resolution order below | Kubeconfig path, forwarded as env to all `oc`/`virtctl` child processes |
| `KUBEVIRT_PROJECT_ROOT` | `~/Developer/Projects/kubevirt-ui` | Used for fallback kubeconfig resolution |
| `OC_PATH` | `oc` | Full path to the `oc` binary |
| `VIRTCTL_PATH` | `virtctl` | Full path to the `virtctl` binary |
| `OC_MCP_LOG` | `info` | Log level for stderr |

### Kubeconfig resolution order

1. `KUBECONFIG` environment variable
2. `$KUBEVIRT_PROJECT_ROOT/.kubeconfigs/test-config`
3. `~/.kube/config`

## Safety

`cleanup_namespace` enforces a safety check: it refuses to operate on any namespace that does not begin with `pw-` or `test-`. This prevents accidental bulk deletion in production namespaces.

Deletion order mirrors `OcCliClient.cleanupTestNamespace` from the kubevirt-ui framework:
1. VirtualMachineInstances (force delete, `--grace-period=0`)
2. VirtualMachines
3. VirtualMachineInstanceMigrations
4. DataVolumes → VMSnapshots → Templates → InstanceTypes → Preferences
5. PVCs → Secrets → ConfigMaps → NetworkAttachmentDefinitions

## Running tests

```bash
cargo test
```
