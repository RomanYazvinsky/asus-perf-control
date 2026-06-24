use log::{debug, warn};
use std::collections::HashMap;
use zbus::Connection;

const POLKIT_AUTHORITY: &str = "org.freedesktop.PolicyKit1";
const POLKIT_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
const POLKIT_INTERFACE: &str = "org.freedesktop.PolicyKit1.Authority";

/// Check polkit authorization for a given sender and action
pub async fn check_authorization(
    conn: &Connection,
    sender: &str,
    action_id: &str,
) -> zbus::Result<()> {
    debug!(
        "Checking polkit authorization for sender={}, action={}",
        sender, action_id
    );

    // Get sender's PID
    let pid = get_sender_pid(conn, sender).await?;
    debug!("Sender PID: {}", pid);

    // Call polkit to check authorization
    let proxy = zbus::Proxy::new(conn, POLKIT_AUTHORITY, POLKIT_PATH, POLKIT_INTERFACE)
        .await
        .map_err(|e| {
            warn!("Failed to create polkit proxy: {}", e);
            zbus::Error::Failure(format!("Failed to access polkit: {}", e))
        })?;

    // Build subject (unix-process)
    let mut subject_details: HashMap<&str, zbus::zvariant::Value> = HashMap::new();
    subject_details.insert("pid", zbus::zvariant::Value::U32(pid));
    subject_details.insert("start-time", zbus::zvariant::Value::U64(0));

    let subject = ("unix-process", subject_details);

    // Build details (empty)
    let details: HashMap<&str, &str> = HashMap::new();

    // Flags: AllowUserInteraction = 1
    let flags = 1u32;

    // Call CheckAuthorization
    let response = proxy
        .call_method(
            "CheckAuthorization",
            &(subject, action_id, details, flags, ""),
        )
        .await
        .map_err(|e| {
            warn!("Polkit CheckAuthorization failed: {}", e);
            zbus::Error::Failure(format!("Authorization check failed: {}", e))
        })?;

    let result: (bool, bool, HashMap<String, String>) =
        response.body().deserialize().map_err(|e| {
            warn!("Failed to deserialize polkit response: {}", e);
            zbus::Error::Failure(format!("Invalid polkit response: {}", e))
        })?;

    let (is_authorized, _is_challenge, _details) = result;

    if !is_authorized {
        warn!("Authorization denied for action: {}", action_id);
        return Err(zbus::Error::Failure(format!(
            "Authorization required for {}",
            action_id
        )));
    }

    debug!("Authorization granted for action: {}", action_id);
    Ok(())
}

/// Get the PID of the D-Bus sender
async fn get_sender_pid(conn: &Connection, sender: &str) -> zbus::Result<u32> {
    let dbus_proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await
    .map_err(|e| zbus::Error::Failure(format!("Failed to access D-Bus: {}", e)))?;

    let response = dbus_proxy
        .call_method("GetConnectionUnixProcessID", &(sender,))
        .await
        .map_err(|e| zbus::Error::Failure(format!("Failed to get sender PID: {}", e)))?;

    let pid: u32 = response
        .body()
        .deserialize()
        .map_err(|e| zbus::Error::Failure(format!("Invalid PID response: {}", e)))?;

    Ok(pid)
}
