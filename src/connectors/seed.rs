//! # Seed connectors — 8 platforms, ~50 procedures pre-loaded
//!
//! These are seed bundles for the most commonly-requested platforms.
//! Each procedure is a composition of primitives — the bundle contains
//! metadata + step definitions, not raw implementation code.
//!
//! ZETS loads these at startup; more can be added via the ingestion
//! pipeline (Phase C) that reads external API docs.
//!
//! Storage impact: all 8 bundles together ~30KB of graph edges.

use crate::secrets::SecretKind;

use super::bundle::{AuthKind, ConnectorBundle, ConnectorProcedure, ProcedureStep};
use super::primitive::PrimitiveId;

fn step(p: PrimitiveId, args: &[&str], label: Option<&str>) -> ProcedureStep {
    ProcedureStep {
        primitive: p,
        args: args.iter().map(|s| s.to_string()).collect(),
        output_label: label.map(|s| s.to_string()),
    }
}

fn proc(
    id: &str,
    description: &str,
    steps: Vec<ProcedureStep>,
    sense_keys: &[&str],
    required_secrets: &[SecretKind],
) -> ConnectorProcedure {
    ConnectorProcedure {
        id: id.to_string(),
        description: description.to_string(),
        steps,
        sense_keys: sense_keys.iter().map(|s| s.to_string()).collect(),
        required_secrets: required_secrets.to_vec(),
        estimated_size_bytes: 0,
    }
}

/// ─── GMAIL ─────────────────────────────────────────────────
pub fn gmail_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "gmail",
        AuthKind::OAuth2,
        "https://gmail.googleapis.com/gmail/v1",
        "https://developers.google.com/gmail/api",
        250,
    );
    b.add_procedure(proc(
        "send",
        "Send an email via Gmail",
        vec![
            step(PrimitiveId::BuildJson, &["{caller.message}"], Some("body")),
            step(PrimitiveId::Base64Encode, &["{body}"], Some("raw")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/users/me/messages/send", "{auth}", "{raw}"], None),
        ],
        &["communication.send.email", "gmail.send", "email.send"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "list",
        "List messages in inbox",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/users/me/messages?q={caller.query}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["communication.list.email", "gmail.list"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "read",
        "Read a specific email by id",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/users/me/messages/{caller.message_id}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["communication.read.email", "gmail.read"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "label_add",
        "Add a label to a message",
        vec![
            step(PrimitiveId::BuildJson, &["{addLabelIds:[caller.label]}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/users/me/messages/{caller.message_id}/modify", "{auth}", "{body}"], None),
        ],
        &["communication.label.email", "gmail.label"],
        &[SecretKind::OAuth],
    ));
    b
}

/// ─── GOOGLE CALENDAR ───────────────────────────────────────
pub fn calendar_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "google_calendar",
        AuthKind::OAuth2,
        "https://www.googleapis.com/calendar/v3",
        "https://developers.google.com/calendar/api",
        500,
    );
    b.add_procedure(proc(
        "event_create",
        "Create a calendar event",
        vec![
            step(PrimitiveId::BuildJson, &["{caller.event}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/calendars/primary/events", "{auth}", "{body}"], None),
        ],
        &["schedule.create.event", "calendar.event.create"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "events_list",
        "List events in a date range",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/calendars/primary/events?timeMin={caller.from}&timeMax={caller.to}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["schedule.list.events", "calendar.events.list"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "event_update",
        "Update a calendar event",
        vec![
            step(PrimitiveId::BuildJson, &["{caller.event}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPatch, &["{base}/calendars/primary/events/{caller.event_id}", "{auth}", "{body}"], None),
        ],
        &["schedule.update.event"],
        &[SecretKind::OAuth],
    ));
    b
}

/// ─── SLACK ─────────────────────────────────────────────────
pub fn slack_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "slack",
        AuthKind::Bearer,
        "https://slack.com/api",
        "https://api.slack.com",
        60,
    );
    b.add_procedure(proc(
        "message_send",
        "Post a message to a Slack channel",
        vec![
            step(PrimitiveId::BuildJson, &["{channel:caller.channel,text:caller.text}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.bot_token}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/chat.postMessage", "{auth}", "{body}"], None),
        ],
        &["communication.send.chat", "slack.message.send"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "thread_reply",
        "Reply in a thread",
        vec![
            step(PrimitiveId::BuildJson, &["{channel:caller.channel,thread_ts:caller.parent_ts,text:caller.text}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.bot_token}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/chat.postMessage", "{auth}", "{body}"], None),
        ],
        &["communication.reply.thread", "slack.thread.reply"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "channel_list",
        "List channels",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.bot_token}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/conversations.list", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["communication.list.channels", "slack.channel.list"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "user_lookup",
        "Look up a user by email",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.bot_token}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/users.lookupByEmail?email={caller.email}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["communication.lookup.user", "slack.user.find"],
        &[SecretKind::ApiKey],
    ));
    b
}

/// ─── TELEGRAM ──────────────────────────────────────────────
pub fn telegram_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "telegram",
        AuthKind::ApiKey,
        "https://api.telegram.org/bot{token}",
        "https://core.telegram.org/bots/api",
        30,
    );
    b.add_procedure(proc(
        "message_send",
        "Send a message via Telegram bot",
        vec![
            step(PrimitiveId::BuildJson, &["{chat_id:caller.chat_id,text:caller.text}"], Some("body")),
            step(PrimitiveId::HttpPost, &["{base}/sendMessage", "{}", "{body}"], None),
        ],
        &["communication.send.chat", "telegram.send"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "photo_send",
        "Send a photo via Telegram bot",
        vec![
            step(PrimitiveId::MultipartForm, &["{chat_id:caller.chat_id,photo:caller.photo_url,caption:caller.caption}"], Some("body")),
            step(PrimitiveId::HttpPost, &["{base}/sendPhoto", "{}", "{body}"], None),
        ],
        &["communication.send.photo", "telegram.photo"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "updates_get",
        "Pull incoming messages",
        vec![
            step(PrimitiveId::HttpGet, &["{base}/getUpdates?offset={caller.offset}", "{}"], None),
            step(PrimitiveId::ParseJson, &["{step_0}"], None),
        ],
        &["communication.receive.chat", "telegram.updates"],
        &[SecretKind::ApiKey],
    ));
    b
}

/// ─── WHATSAPP via GreenAPI ─────────────────────────────────
pub fn whatsapp_greenapi_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "whatsapp_greenapi",
        AuthKind::ApiKey,
        "https://api.green-api.com/waInstance{instance}",
        "https://green-api.com/en/docs",
        60,
    );
    b.add_procedure(proc(
        "message_send",
        "Send a WhatsApp text message via GreenAPI",
        vec![
            step(PrimitiveId::BuildJson, &["{chatId:caller.chat_id,message:caller.text}"], Some("body")),
            step(PrimitiveId::HttpPost, &["{base}/sendMessage/{secret.token}", "{}", "{body}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["communication.send.chat", "whatsapp.send", "whatsapp.greenapi.send"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "file_send",
        "Send a file via WhatsApp",
        vec![
            step(PrimitiveId::BuildJson, &["{chatId:caller.chat_id,urlFile:caller.file_url,fileName:caller.file_name}"], Some("body")),
            step(PrimitiveId::HttpPost, &["{base}/sendFileByUrl/{secret.token}", "{}", "{body}"], None),
        ],
        &["communication.send.file", "whatsapp.file"],
        &[SecretKind::ApiKey],
    ));
    b
}

/// ─── SMTP (generic email fallback) ─────────────────────────
pub fn smtp_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "smtp",
        AuthKind::Basic,
        "smtp://{host}:{port}",
        "https://tools.ietf.org/html/rfc5321",
        300,
    );
    b.add_procedure(proc(
        "send",
        "Send an email via SMTP",
        vec![
            step(PrimitiveId::BuildBasicAuth, &["{secret.user}", "{secret.pass}"], Some("auth")),
            // SMTP isn't HTTP — this is a specialized step the VM handles.
            // For the schema level, we record it as a custom primitive call.
            step(PrimitiveId::HttpPost, &["smtp://{host}:{port}", "{auth}", "{caller.message}"], None),
        ],
        &["communication.send.email", "email.smtp.send"],
        &[SecretKind::Password],
    ));
    b
}

/// ─── GOOGLE DRIVE ──────────────────────────────────────────
pub fn drive_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "google_drive",
        AuthKind::OAuth2,
        "https://www.googleapis.com/drive/v3",
        "https://developers.google.com/drive/api",
        1000,
    );
    b.add_procedure(proc(
        "file_upload",
        "Upload a file to Drive",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::MultipartForm, &["{caller.metadata}", "{caller.bytes}"], Some("body")),
            step(PrimitiveId::HttpPost, &["{base}/files?uploadType=multipart", "{auth}", "{body}"], None),
        ],
        &["storage.upload.file", "drive.file.upload"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "file_list",
        "List files, optionally filtered",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/files?q={caller.query}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["storage.list.files", "drive.file.list"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "file_get",
        "Download a file by id",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/files/{caller.file_id}?alt=media", "{auth}"], None),
        ],
        &["storage.download.file", "drive.file.get"],
        &[SecretKind::OAuth],
    ));
    b
}

/// ─── GOOGLE SHEETS ─────────────────────────────────────────
pub fn sheets_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "google_sheets",
        AuthKind::OAuth2,
        "https://sheets.googleapis.com/v4/spreadsheets",
        "https://developers.google.com/sheets/api",
        500,
    );
    b.add_procedure(proc(
        "range_read",
        "Read a range of cells",
        vec![
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpGet, &["{base}/{caller.sheet_id}/values/{caller.range}", "{auth}"], None),
            step(PrimitiveId::ParseJson, &["{step_1}"], None),
        ],
        &["data.read.spreadsheet", "sheets.range.read"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "range_write",
        "Write values to a range",
        vec![
            step(PrimitiveId::BuildJson, &["{values:caller.values}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPut, &["{base}/{caller.sheet_id}/values/{caller.range}?valueInputOption=USER_ENTERED", "{auth}", "{body}"], None),
        ],
        &["data.write.spreadsheet", "sheets.range.write"],
        &[SecretKind::OAuth],
    ));
    b.add_procedure(proc(
        "row_append",
        "Append a row to the end of a sheet",
        vec![
            step(PrimitiveId::BuildJson, &["{values:[caller.row]}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.oauth}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/{caller.sheet_id}/values/{caller.range}:append?valueInputOption=USER_ENTERED", "{auth}", "{body}"], None),
        ],
        &["data.append.row"],
        &[SecretKind::OAuth],
    ));
    b
}

/// ─── OPENAI ────────────────────────────────────────────────
pub fn openai_bundle() -> ConnectorBundle {
    let mut b = ConnectorBundle::new(
        "openai",
        AuthKind::Bearer,
        "https://api.openai.com/v1",
        "https://platform.openai.com/docs",
        500,
    );
    b.add_procedure(proc(
        "chat_complete",
        "Chat completion",
        vec![
            step(PrimitiveId::BuildJson, &["{model:caller.model,messages:caller.messages,max_tokens:caller.max_tokens}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.api_key}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/chat/completions", "{auth}", "{body}"], None),
            step(PrimitiveId::ParseJson, &["{step_2}"], None),
        ],
        &["ai.chat.complete", "openai.chat"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "embedding",
        "Compute an embedding vector",
        vec![
            step(PrimitiveId::BuildJson, &["{model:caller.model,input:caller.text}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.api_key}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/embeddings", "{auth}", "{body}"], None),
        ],
        &["ai.embedding", "openai.embed"],
        &[SecretKind::ApiKey],
    ));
    b.add_procedure(proc(
        "tts",
        "Text-to-speech",
        vec![
            step(PrimitiveId::BuildJson, &["{model:caller.model,voice:caller.voice,input:caller.text}"], Some("body")),
            step(PrimitiveId::BuildBearerAuth, &["{secret.api_key}"], Some("auth")),
            step(PrimitiveId::HttpPost, &["{base}/audio/speech", "{auth}", "{body}"], None),
        ],
        &["ai.tts", "openai.tts"],
        &[SecretKind::ApiKey],
    ));
    b
}

/// All seed bundles in one vector.
pub fn all_seed_bundles() -> Vec<ConnectorBundle> {
    vec![
        gmail_bundle(),
        calendar_bundle(),
        slack_bundle(),
        telegram_bundle(),
        whatsapp_greenapi_bundle(),
        smtp_bundle(),
        drive_bundle(),
        sheets_bundle(),
        openai_bundle(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_9_bundles_load() {
        let all = all_seed_bundles();
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn test_total_procedures_count() {
        let all = all_seed_bundles();
        let total: usize = all.iter().map(|b| b.procedure_count()).sum();
        // 9 bundles with 2-4 procedures each → expect 20-30
        assert!(total >= 20);
        assert!(total <= 40);
    }

    #[test]
    fn test_total_storage_under_budget() {
        let all = all_seed_bundles();
        let total: u32 = all.iter().map(|b| b.total_size_bytes()).sum();
        // All connectors combined should be under 15KB
        assert!(total < 15_000, "too large: {} bytes", total);
    }

    #[test]
    fn test_gmail_send_discoverable() {
        let gmail = gmail_bundle();
        let send = gmail.find("send").unwrap();
        assert!(send.sense_keys.iter().any(|k| k == "communication.send.email"));
    }

    #[test]
    fn test_slack_thread_reply_distinct_from_message() {
        let slack = slack_bundle();
        assert!(slack.find("message_send").is_some());
        assert!(slack.find("thread_reply").is_some());
    }

    #[test]
    fn test_sense_key_communication_send_chat_hit_multiple() {
        // Both Slack, Telegram, and WhatsApp should answer to
        // "communication.send.chat"
        let bundles = [slack_bundle(), telegram_bundle(), whatsapp_greenapi_bundle()];
        for b in &bundles {
            let hits = b.find_by_sense("communication.send.chat");
            assert!(!hits.is_empty(), "no send.chat in {}", b.platform);
        }
    }

    #[test]
    fn test_all_procedures_have_sense_keys() {
        for bundle in all_seed_bundles() {
            for p in &bundle.procedures {
                assert!(
                    !p.sense_keys.is_empty(),
                    "procedure {}/{} missing sense_keys",
                    bundle.platform,
                    p.id
                );
            }
        }
    }

    #[test]
    fn test_oauth_bundles_require_oauth_secret() {
        let oauth_bundles = [
            gmail_bundle(),
            calendar_bundle(),
            drive_bundle(),
            sheets_bundle(),
        ];
        for bundle in &oauth_bundles {
            for p in &bundle.procedures {
                assert!(
                    p.required_secrets.contains(&SecretKind::OAuth),
                    "{}/{} should require OAuth",
                    bundle.platform,
                    p.id
                );
            }
        }
    }

    #[test]
    fn test_openai_has_chat_embedding_tts() {
        let oa = openai_bundle();
        assert!(oa.find("chat_complete").is_some());
        assert!(oa.find("embedding").is_some());
        assert!(oa.find("tts").is_some());
    }
}
