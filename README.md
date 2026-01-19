# SAP Cloud ALM MCP Server

A Model Context Protocol (MCP) server that bridges SAP Cloud ALM APIs to AI assistants like Claude. This enables natural language interaction with SAP Cloud ALM for managing features, documents, tasks, projects, test cases, and more.

## Features

- **Full SAP Cloud ALM API Coverage**: Access to 9 different APIs including Features, Documents, Tasks, Projects, Test Management, Process Hierarchy, Analytics, Process Monitoring, and Logs
- **Two Authentication Modes**:
  - OAuth2 client credentials for production environments
  - Static API key for SAP API Business Hub sandbox testing
- **60+ MCP Tools**: Comprehensive toolset for CRUD operations across all supported APIs
- **OData v4 Support**: Full query builder with $filter, $select, $expand, $orderby, $top, $skip
- **Debug Mode**: Detailed logging for troubleshooting

## Prerequisites

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **SAP Cloud ALM Access**: Either:
  - Production: Service binding credentials (client_id, client_secret, tenant, region)
  - Sandbox: API key from [SAP API Business Hub](https://api.sap.com/)

## Installation

### Build from Source

```bash
# Clone and navigate to the project directory
git clone https://github.com/consetto/sap-cloud-alm-odata-mcp.git
cd sap-cloud-alm-odata-mcp

# Build the release binary
cargo build --release

# The binary will be at: ./target/release/sap-cloud-alm-mcp
```

## Configuration

Create a `config.json` file in the project directory (or specify a custom path with `--config`).

### Option A: OAuth2 Mode (Production)

For connecting to your SAP Cloud ALM tenant:

```json
{
  "tenant": "your-tenant-name",
  "region": "eu10",
  "client_id": "sb-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx!bxxxxx|calm!bxxxxx",
  "client_secret": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx=",
  "debug": false,
  "timeout_seconds": 30
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| `tenant` | Yes | Your SAP Cloud ALM tenant identifier |
| `region` | Yes | SAP region (eu10, eu20, us10, ap10, jp10, ca10, eu11, cn20) |
| `client_id` | Yes | OAuth2 client ID from service binding |
| `client_secret` | Yes | OAuth2 client secret from service binding |
| `debug` | No | Enable debug logging (default: false) |
| `timeout_seconds` | No | HTTP request timeout (default: 30) |

### Option B: Sandbox Mode (Testing)

For testing against the SAP API Business Hub sandbox:

```json
{
  "sandbox": true,
  "api_key": "your-sap-sandbox-api-key",
  "debug": true,
  "timeout_seconds": 30
}
```

**Fields:**
| Field | Required | Description |
|-------|----------|-------------|
| `sandbox` | Yes | Must be `true` to enable sandbox mode |
| `api_key` | Yes | Your API key from SAP API Business Hub |
| `debug` | No | Enable debug logging (default: false) |
| `timeout_seconds` | No | HTTP request timeout (default: 30) |

**Getting a Sandbox API Key:**
1. Visit [SAP API Business Hub](https://api.sap.com/)
2. Log in with your SAP account
3. Navigate to any SAP Cloud ALM API (e.g., CALM_CDM_ODATA)
4. Click "Show API Key" to reveal your key

## Running the Server

### Standalone (for testing)

```bash
# With default config.json
./target/release/sap-cloud-alm-mcp

# With custom config file
./target/release/sap-cloud-alm-mcp --config /path/to/config.json

# With debug mode enabled
./target/release/sap-cloud-alm-mcp --debug
```

### With Claude Desktop

Add the server to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "sap-cloud-alm": {
      "command": "/absolute/path/to/sap-cloud-alm-mcp",
      "args": ["--config", "/absolute/path/to/config.json"]
    }
  }
}
```

After updating the configuration, restart Claude Desktop.

### With MCP Inspector (for debugging)

```bash
npx @modelcontextprotocol/inspector ./target/release/sap-cloud-alm-mcp --config config.json
```

## Available Tools

### Features API (OData)
| Tool | Description |
|------|-------------|
| `list_features` | List features with OData filtering |
| `get_feature` | Get a single feature by UUID |
| `create_feature` | Create a new feature |
| `update_feature` | Update an existing feature |
| `delete_feature` | Delete a feature |
| `list_feature_statuses` | List available status codes |
| `list_feature_priorities` | List available priority codes |
| `list_external_references` | List external references |
| `create_external_reference` | Create an external reference |
| `delete_external_reference` | Delete an external reference |

### Documents API (OData)
| Tool | Description |
|------|-------------|
| `list_documents` | List documents with filtering |
| `get_document` | Get a single document |
| `create_document` | Create a new document |
| `update_document` | Update a document |
| `delete_document` | Delete a document |
| `list_document_types` | List available document types |
| `list_document_statuses` | List available statuses |

### Tasks API (REST)
| Tool | Description |
|------|-------------|
| `list_tasks` | List tasks for a project |
| `get_task` | Get task details |
| `create_task` | Create a new task |
| `update_task` | Update a task |
| `delete_task` | Delete a task |
| `list_task_comments` | List comments on a task |
| `create_task_comment` | Add a comment to a task |
| `list_task_references` | List task references |
| `list_workstreams` | List workstreams |
| `list_deliverables` | List deliverables |

### Projects API (REST)
| Tool | Description |
|------|-------------|
| `list_projects` | List all projects |
| `get_project` | Get project details |
| `create_project` | Create a new project |
| `list_project_timeboxes` | List sprints/timeboxes |
| `list_project_teams` | List team members |
| `list_programs` | List all programs |
| `get_program` | Get program details |

### Test Management API (OData)
| Tool | Description |
|------|-------------|
| `list_testcases` | List manual test cases |
| `get_testcase` | Get test case details |
| `create_testcase` | Create a test case |
| `update_testcase` | Update a test case |
| `delete_testcase` | Delete a test case |
| `list_test_activities` | List test activities |
| `create_test_activity` | Create a test activity |
| `list_test_actions` | List test actions |
| `create_test_action` | Create a test action |

### Process Hierarchy API (OData)
| Tool | Description |
|------|-------------|
| `list_hierarchy_nodes` | List process hierarchy nodes |
| `get_hierarchy_node` | Get a hierarchy node |
| `create_hierarchy_node` | Create a hierarchy node |
| `update_hierarchy_node` | Update a hierarchy node |
| `delete_hierarchy_node` | Delete a hierarchy node |

### Analytics API (OData)
| Tool | Description |
|------|-------------|
| `query_analytics` | Query analytics datasets |
| `list_analytics_providers` | List data providers |

### Process Monitoring API (OData)
| Tool | Description |
|------|-------------|
| `list_monitoring_events` | List monitoring events |
| `get_monitoring_event` | Get event details |
| `list_monitoring_services` | List monitored services |

### Logs API (REST)
| Tool | Description |
|------|-------------|
| `get_logs` | Get logs (OpenTelemetry format) |
| `post_logs` | Post logs |

## Example Usage with Claude

Once configured with Claude Desktop, you can interact naturally:

```
"List all my SAP Cloud ALM projects"

"Show me the features in project XYZ"

"Create a new task in project ABC with title 'Fix login bug' and assign it to John"

"What are the open test cases for the current sprint?"

"Show me the process hierarchy for our S/4HANA implementation"
```

## Debug Mode

Enable debug mode for troubleshooting:

1. Set `"debug": true` in config.json, or
2. Use the `--debug` command line flag

Debug output includes:
- All MCP messages (sent/received)
- API requests and responses
- Authentication flow details
- A trace file at `/tmp/sap_calm_mcp_trace_{timestamp}.log`

## API Endpoints

The server connects to the following SAP Cloud ALM APIs:

| API | Type | Base Path |
|-----|------|-----------|
| Features | OData v4 | `/api/calm-features/v1` |
| Documents | OData v4 | `/api/calm-documents/v1` |
| Tasks | REST | `/api/calm-tasks/v1` |
| Projects | REST | `/api/calm-projects/v1` |
| Test Management | OData v4 | `/api/calm-testmanagement/v1` |
| Process Hierarchy | OData v4 | `/api/calm-processhierarchy/v1` |
| Analytics | OData v4 | `/api/calm-analytics/v1` |
| Process Monitoring | OData v4 | `/api/calm-processmonitoring/v1` |
| Logs | REST | `/api/calm-logs/v1` |

**Production URL Pattern**: `https://{tenant}.{region}.alm.cloud.sap`
**Sandbox URL**: `https://sandbox.api.sap.com/SAPCALM`

## Troubleshooting

### Authentication Errors

**OAuth2 Mode:**
- Verify your client_id and client_secret are correct
- Ensure your service binding has the required scopes
- Check that the tenant and region match your SAP Cloud ALM instance

**Sandbox Mode:**
- Verify your API key is valid and not expired
- Check that sandbox mode is enabled (`"sandbox": true`)

### Connection Issues

- Ensure your network allows connections to SAP Cloud ALM endpoints
- Check if a proxy is required and configure it at the OS level
- Increase `timeout_seconds` for slow connections

### Debug Tips

1. Enable debug mode to see detailed logs
2. Check the trace file for full request/response details
3. Use MCP Inspector to test tools interactively

## License

MIT License

## Contributing

Contributions are welcome! Please ensure your code:
- Builds without errors (`cargo build`)
- Passes all tests (`cargo test`)
- Follows Rust conventions (`cargo fmt` and `cargo clippy`)
