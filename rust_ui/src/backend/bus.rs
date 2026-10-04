//! Thin helpers over a shared system-bus connection.

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::OnceCell;
use zbus::zvariant::{DynamicType, OwnedValue, Type, Value};
use zbus::Connection;

const PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";
const INTROSPECTABLE_IFACE: &str = "org.freedesktop.DBus.Introspectable";

/// Lazily connected system bus, shared by all clients.
#[derive(Default)]
pub struct SystemBus {
    conn: OnceCell<Connection>,
}

impl SystemBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connection(&self) -> Result<&Connection> {
        self.conn
            .get_or_try_init(|| async { Connection::system().await })
            .await
            .context("Failed to connect to the D-Bus system bus")
    }

    /// Call a method and deserialize its reply body.
    pub async fn call<B, R>(&self, dest: &str, path: &str, iface: &str, method: &str, body: &B) -> Result<R>
    where
        B: Serialize + DynamicType,
        R: DeserializeOwned + Type,
    {
        let conn = self.connection().await?;
        let reply = conn
            .call_method(Some(dest), path, Some(iface), method, body)
            .await
            .map_err(|e| anyhow!("{dest} {iface}.{method}: {e}"))?;
        reply
            .body()
            .deserialize::<R>()
            .map_err(|e| anyhow!("{dest} {iface}.{method}: unexpected reply: {e}"))
    }

    /// Call a method that returns nothing.
    pub async fn call_void<B>(&self, dest: &str, path: &str, iface: &str, method: &str, body: &B) -> Result<()>
    where
        B: Serialize + DynamicType,
    {
        let conn = self.connection().await?;
        conn.call_method(Some(dest), path, Some(iface), method, body)
            .await
            .map_err(|e| anyhow!("{dest} {iface}.{method}: {e}"))?;
        Ok(())
    }

    pub async fn get_property(&self, dest: &str, path: &str, iface: &str, prop: &str) -> Result<OwnedValue> {
        self.call(dest, path, PROPERTIES_IFACE, "Get", &(iface, prop))
            .await
            .with_context(|| format!("Get {iface}.{prop} at {path}"))
    }

    pub async fn set_property(&self, dest: &str, path: &str, iface: &str, prop: &str, value: Value<'_>) -> Result<()> {
        self.call_void(dest, path, PROPERTIES_IFACE, "Set", &(iface, prop, value))
            .await
            .with_context(|| format!("Set {iface}.{prop} at {path}"))
    }

    /// True when the object answers `Introspect`.
    pub async fn object_exists(&self, dest: &str, path: &str) -> bool {
        self.call::<_, String>(dest, path, INTROSPECTABLE_IFACE, "Introspect", &())
            .await
            .is_ok()
    }
}

/// Strip any `v` wrappers around a property value.
fn inner<'a>(mut value: &'a Value<'a>) -> &'a Value<'a> {
    while let Value::Value(boxed) = value {
        value = boxed;
    }
    value
}

/// Integer view of a property value regardless of its D-Bus integer width.
pub fn value_as_i64(value: &OwnedValue) -> Result<i64> {
    match inner(value) {
        Value::U8(v) => Ok(i64::from(*v)),
        Value::I16(v) => Ok(i64::from(*v)),
        Value::U16(v) => Ok(i64::from(*v)),
        Value::I32(v) => Ok(i64::from(*v)),
        Value::U32(v) => Ok(i64::from(*v)),
        Value::I64(v) => Ok(*v),
        Value::U64(v) => i64::try_from(*v).context("integer out of range"),
        other => Err(anyhow!("expected an integer, got {}", other.value_signature())),
    }
}

pub fn value_as_bool(value: &OwnedValue) -> Result<bool> {
    match inner(value) {
        Value::Bool(v) => Ok(*v),
        other => Err(anyhow!("expected a boolean, got {}", other.value_signature())),
    }
}
