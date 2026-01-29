//! SAP Cloud ALM MCP Server
//!
//! Bridges SAP Cloud ALM APIs to the Model Context Protocol.

mod api;
mod auth;
mod config;
mod debug;
mod error;
mod odata;
mod server;

use std::sync::Arc;

use clap::Parser;
use rmcp::{transport::stdio, ServiceExt};

use crate::api::{
    AnalyticsClient, DocumentsClient, FeaturesClient, LogsClient, ProcessHierarchyClient,
    ProcessMonitoringClient, ProjectsClient, TasksClient, TestManagementClient,
};
use crate::auth::OAuth2Client;
use crate::config::Config;
use crate::debug::DebugLogger;
use crate::odata::ODataClient;
use crate::server::{ApiClients, SapCloudAlmServer};

#[derive(Parser, Debug)]
#[command(name = "sap-cloud-alm-mcp")]
#[command(author, version, about = "SAP Cloud ALM MCP Server", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.json")]
    config: String,

    /// Enable debug mode (logs all MCP messages)
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration
    let config = Config::load(&args.config)?;
    let debug_enabled = args.debug || config.debug;

    // Initialize debug logger
    let debug = Arc::new(DebugLogger::new(debug_enabled));

    if debug_enabled {
        debug.log("SAP Cloud ALM MCP Server starting...");
        debug.log(&format!("Config file: {}", args.config));
        if config.sandbox {
            debug.log("Mode: Sandbox");
            debug.log(&format!("Base URL: {}", config.api_base_url()));
        } else {
            debug.log("Mode: OAuth2 (Production)");
            debug.log(&format!(
                "Tenant: {}",
                config.tenant.as_deref().unwrap_or("N/A")
            ));
            debug.log(&format!(
                "Region: {}",
                config.region.as_deref().unwrap_or("N/A")
            ));
        }
        if let Some(path) = debug.trace_path() {
            eprintln!("[DEBUG] Trace file: {}", path.display());
        }
    }

    // Create OAuth2 client
    let auth_client = OAuth2Client::new(config.clone())?;

    // Create API clients
    // OData-based clients
    let features_odata = ODataClient::new(
        config.features_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let features_client = FeaturesClient::new(features_odata);

    let documents_odata = ODataClient::new(
        config.documents_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let documents_client = DocumentsClient::new(documents_odata);

    let testmanagement_odata = ODataClient::new(
        config.testmanagement_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let testmanagement_client = TestManagementClient::new(testmanagement_odata);

    let processhierarchy_odata = ODataClient::new(
        config.processhierarchy_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let processhierarchy_client = ProcessHierarchyClient::new(processhierarchy_odata);

    let analytics_odata = ODataClient::new(
        config.analytics_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let analytics_client = AnalyticsClient::new(analytics_odata);

    let processmonitoring_odata = ODataClient::new(
        config.processmonitoring_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;
    let processmonitoring_client = ProcessMonitoringClient::new(processmonitoring_odata);

    // REST-based clients
    let tasks_client =
        TasksClient::new(config.tasks_api_url(), auth_client.clone(), debug_enabled)?;

    let projects_client = ProjectsClient::new(
        config.projects_api_url(),
        auth_client.clone(),
        debug_enabled,
    )?;

    let logs_client = LogsClient::new(config.logs_api_url(), auth_client.clone(), debug_enabled)?;

    // Create MCP server
    let clients = ApiClients {
        features: features_client,
        documents: documents_client,
        tasks: tasks_client,
        projects: projects_client,
        testmanagement: testmanagement_client,
        processhierarchy: processhierarchy_client,
        analytics: analytics_client,
        processmonitoring: processmonitoring_client,
        logs: logs_client,
    };

    let server = SapCloudAlmServer::new(clients, debug.clone());

    if debug_enabled {
        debug.log("All API clients initialized");
        debug.log("Starting MCP server on stdio transport...");
    }

    // Run MCP server on stdio transport
    let service = server.serve(stdio()).await?;

    if debug_enabled {
        debug.log("MCP server started, waiting for messages...");
    }

    // Wait for the service to complete
    service.waiting().await?;

    if debug_enabled {
        debug.log("MCP server shutting down");
    }

    Ok(())
}
