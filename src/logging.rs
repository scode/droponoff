use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// Custom event formatter that adds emoji prefixes based on message content
pub struct EmojiFormatter;

impl<S, N> FormatEvent<S, N> for EmojiFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);
        let message = visitor.message;

        let emoji_message = add_emoji_prefix(&message, event.metadata().level());

        writeln!(writer, "{}", emoji_message)
    }
}

struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove quotes added by Debug formatting
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        }
    }
}

/// Add emoji prefix based on content patterns and log level
fn add_emoji_prefix(message: &str, level: &Level) -> String {
    // Priority 1: Success messages (already have ✓, keep as-is or replace with ✅)
    if message.contains('✓') {
        // Replace ✓ with ✅ (green checkbox)
        return message.replace('✓', "✅");
    }

    // Priority 2: Progress step indicators (→)
    if message.starts_with('→') {
        // Replace → with ℹ️ (information)
        return message.replacen('→', "ℹ️", 1);
    }

    // Priority 3: Level-based emoji prefixing
    match *level {
        Level::ERROR => format!("❌ {}", message),
        Level::WARN => format!("⚠️  {}", message),
        Level::INFO => {
            format!("ℹ️  {}", message)
        }
        _ => message.to_string(),
    }
}

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(false)
        .without_time()
        .event_format(EmojiFormatter)
        .init();
}
