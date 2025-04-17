#![allow(unused)]

use anyhow::{Ok, Result};

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:5173")?;
    println!(">>> New test started");
    let res = hc.do_get("/").await?;
    println!(">>> Status: {}", res.status());
    res.print().await?;
    Ok(())
}