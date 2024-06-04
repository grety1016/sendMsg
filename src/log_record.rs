//这段代码是一体的，结构体跟实现的trait是为了让日志追踪打印的日期跟当前系统一致，用于日志追踪结构体实现
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};
pub use tracing::{info,event,warn,trace,Level,info_span};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::{EnvFilter,prelude::*};


struct LocalTime;
impl FormatTime for LocalTime {
    fn format_time(&self, w: &mut Writer<'_>) -> crate::std_Result<(), std::fmt::Error> {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
///日志追踪
pub fn init() {
    // 创建一个小时滚动的日志记录器
 
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY) 
        .filename_prefix("Rocket") 
        .filename_suffix("log")
        .build("./LOGFILES") 
        .expect("initializing rolling file appender failed");

    let subscriber = tracing_subscriber::FmtSubscriber::builder() 
        .with_max_level(tracing::Level::INFO)        
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(file_appender)                   
        .finish();
        // .with_file(true)
        // .with_line_number(true)  
        //.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // .with_timer(LocalTime)
        // .with_target(false)
        // .with_thread_ids(true)
        // .with_thread_names(true)
        // .fmt_fields(tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
        //     write!(writer, "{} : {:?}\t", field, value)
        // }))
        
    tracing::subscriber::set_global_default(subscriber).unwrap();
}