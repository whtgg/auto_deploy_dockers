//! 容器客户端
//!
//! Auth: Mr.Wht
//! Date: 2023/02/18
//! Description: 容器客户端
//! ```

use std::cell::RefCell;
use std::net::SocketAddr;

use time::UtcOffset;
use time::macros::format_description;
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::field::MakeExt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

use common::CONF;

pub mod router;
pub mod common;
pub mod api;
pub mod service;
pub mod schema;

thread_local!(static TRACE: RefCell<String> = RefCell::new(String::default()));

#[tokio::main]
async fn main() {
    // 自定义日志中的traceId格式
    let formatter = format::debug_fn(|writer, field, value| {
        TRACE.with(|f| write!(writer, "[traceId = {}] {} {:?}", *f.borrow(), field, value))
    })
    .delimited(", ");

    // 设置日志级别
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 日志最左侧的时间格式 设置东八区并格式化
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"),
    );

    // 输出到控制台的配置
    let formatting_layer = fmt::layer()
        .with_thread_ids(true)
        .with_line_number(false)
        .fmt_fields(formatter.clone())
        .with_timer(local_time.clone())
        .with_writer(std::io::stderr);


    // 每天生成日志文件 详情见tracing-appender-0.2.2\src\rolling.rs注释
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "log", "dockers.log");

    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    // 输出到日志的配置
    let file_layer = fmt::layer()
        .with_thread_ids(true)
        .with_line_number(true)
        .fmt_fields(formatter)
        .with_ansi(false)
        .with_timer(local_time)
        .with_writer(non_blocking_appender);

    // 两个打印的配置全部注册
    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
        .init();

    let addr = CONF.server.addr.parse::<SocketAddr>().unwrap();
    tracing::info!("addr---{}",addr);
    axum::Server::bind(&addr)
        .serve(router::api_router().into_make_service())
        .await
        .unwrap();
}