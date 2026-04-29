
use crate::constants::*;
use crate::format::*;
use crate::models::*;

pub(crate) fn build_session_hot_turns(turns: &[TurnDigest]) -> Vec<TurnHotspot> {
    let mut items = turns.iter().map(turn_hotspot_from_digest).collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .response_elapsed_ms
            .unwrap_or_default()
            .cmp(&left.response_elapsed_ms.unwrap_or_default())
            .then_with(|| right.total_tokens.cmp(&left.total_tokens))
    });
    items.truncate(SESSION_HOT_MESSAGES_LIMIT);
    items
}

pub(crate) fn build_pivotal_turns(turns: &[TurnDigest]) -> Vec<usize> {
    let mut items = turns
        .iter()
        .enumerate()
        .filter_map(|(idx, turn)| {
            let mut score = 0usize;
            if turn.recommended_attention == "inspect-artifacts" {
                score += 3;
            }
            if turn.delegation_count > 0 {
                score += 2;
            }
            if !turn.change_intents.is_empty() {
                score += 1;
            }
            if idx > 0 && turns[idx - 1].agent_strategy != turn.agent_strategy {
                score += 2;
            }
            (score > 0).then_some((score, turn.turn_index))
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    items.into_iter().map(|(_, turn_index)| turn_index).take(6).collect()
}

pub(crate) fn turn_hotspot_from_digest(turn: &TurnDigest) -> TurnHotspot {
    let mut hot_reasons = Vec::new();
    if turn.response_elapsed_ms.unwrap_or_default() >= HOT_TURN_SLOW_MS_THRESHOLD {
        hot_reasons.push(String::from("slow"));
    }
    if turn.total_tokens >= HOT_TURN_TOKEN_THRESHOLD {
        hot_reasons.push(String::from("token-heavy"));
    }
    if turn.tool_call_count >= HOT_TURN_TOOL_COUNT_THRESHOLD {
        hot_reasons.push(String::from("tool-heavy"));
    }
    if matches!(turn.turn_cost_tier.as_str(), "heavy" | "extreme") {
        hot_reasons.push(String::from("expensive"));
    }
    if turn.delegation_count > 0 {
        hot_reasons.push(String::from("delegation"));
    }
    if turn.error_count > 0 {
        hot_reasons.push(String::from("errors"));
    }
    TurnHotspot {
        session_path: turn.session_path.clone(),
        turn_index: turn.turn_index,
        user_message_index: turn.user_message_index,
        user_intent: turn.user_intent.clone(),
        user_intent_confidence: turn.user_intent_confidence,
        hot_reasons,
        response_elapsed_ms: turn.response_elapsed_ms,
        wall_to_next_user_ms: turn.wall_to_next_user_ms,
        total_tokens: turn.total_tokens,
        tool_calls: turn.tool_call_count,
        error_count: turn.error_count,
        delegation_count: turn.delegation_count,
        cache_hit_ratio: turn.cache_hit_ratio,
        tokens_per_tool_call: turn.tokens_per_tool_call,
        agent_strategy: turn.agent_strategy.clone(),
        outcome: turn.outcome.clone(),
        turn_cost_tier: turn.turn_cost_tier.clone(),
        turn_effectiveness: turn.turn_effectiveness.clone(),
        recommended_attention: turn.recommended_attention.clone(),
        user_text_preview: turn.user_text_preview.clone(),
        final_assistant_kind: turn.final_assistant_kind.clone(),
        final_assistant_text_preview: turn.final_assistant_text_preview.clone(),
    }
}

pub(crate) fn build_session_hot_messages(messages: &[MessageDigest]) -> Vec<MessageHotspot> {
    let mut items = build_message_hotspots(messages);
    items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| right.message_count.cmp(&left.message_count))
    });
    items.truncate(SESSION_HOT_MESSAGES_LIMIT);
    items
}

pub(crate) fn build_hotspots(
    sessions: &[SessionHotspot],
    turns: &[TurnDigest],
    messages: &[MessageDigest],
    tool_calls: &[ToolCallDigest],
) -> ExportHotspots {
    let mut slowest_sessions = sessions.to_vec();
    slowest_sessions.sort_by_key(|spot| std::cmp::Reverse(spot.duration_ms));
    slowest_sessions.truncate(EXPORT_HOTSPOT_LIMIT);

    let mut turn_items = turns.iter().map(turn_hotspot_from_digest).collect::<Vec<_>>();
    turn_items.retain(|item| !item.hot_reasons.is_empty());
    turn_items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.response_elapsed_ms.unwrap_or_default().cmp(&left.response_elapsed_ms.unwrap_or_default()))
            .then_with(|| right.tool_calls.cmp(&left.tool_calls))
            .then_with(|| right.delegation_count.cmp(&left.delegation_count))
    });
    turn_items.truncate(EXPORT_HOTSPOT_LIMIT);

    let mut message_items = build_message_hotspots(messages);
    message_items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| right.message_count.cmp(&left.message_count))
            .then_with(|| right.tool_calls.cmp(&left.tool_calls))
    });
    message_items.truncate(EXPORT_HOTSPOT_LIMIT);

    let all_tool_hotspots = tool_calls
        .iter()
        .cloned()
        .map(|tool| {
            let mut hot_reasons = Vec::new();
            if tool.duration_ms.unwrap_or_default() >= HOT_TOOL_SLOW_MS_THRESHOLD {
                hot_reasons.push(String::from("slow"));
            }
            if tool.output_chars.unwrap_or_default() >= HOT_TOOL_OUTPUT_CHARS_THRESHOLD {
                hot_reasons.push(String::from("large-output"));
            }
            if tool.status == "error" {
                hot_reasons.push(String::from("error"));
            }
            ToolHotspot {
                session_path: tool.session_path,
                message_index: tool.message_index,
                tool_index: tool.tool_index,
                tool: tool.tool,
                hot_reasons,
                status: tool.status,
                duration_ms: tool.duration_ms,
                output_chars: tool.output_chars,
                output_preview: tool.output_preview,
                output_file: tool.output_file,
                error_type: tool.error_type,
                error_file: tool.error_file,
            }
        })
        .collect::<Vec<_>>();
    let mut tool_items = all_tool_hotspots;
    tool_items.retain(|item| !item.hot_reasons.is_empty());
    tool_items.sort_by(|left, right| {
        right
            .output_chars
            .unwrap_or_default()
            .cmp(&left.output_chars.unwrap_or_default())
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| (right.status == "error").cmp(&(left.status == "error")))
    });
    tool_items.truncate(EXPORT_HOTSPOT_LIMIT);

    ExportHotspots {
        slowest_sessions,
        turns: turn_items,
        messages: message_items,
        tools: tool_items,
    }
}

pub(crate) fn build_message_hotspots(messages: &[MessageDigest]) -> Vec<MessageHotspot> {
    let mut items: Vec<MessageHotspot> = Vec::new();
    for message in messages {
        let hotspot = message_hotspot_from_digest(message);
        if hotspot.hot_reasons.is_empty() {
            continue;
        }
        if let Some(previous) = items.last_mut()
            && previous.session_path == hotspot.session_path
            && previous.role == hotspot.role
            && previous.turn_index == hotspot.turn_index
            && previous.hot_reasons == hotspot.hot_reasons
            && previous.end_message_index.unwrap_or(previous.start_message_index) + 1 == hotspot.start_message_index
        {
            previous.end_message_index = Some(hotspot.start_message_index);
            previous.message_count += 1;
            previous.duration_ms = Some(previous.duration_ms.unwrap_or_default() + hotspot.duration_ms.unwrap_or_default());
            previous.total_tokens += hotspot.total_tokens;
            previous.tool_calls += hotspot.tool_calls;
            if previous.sample_text_preview.is_none() {
                previous.sample_text_preview = hotspot.sample_text_preview.clone();
            }
            previous.pattern = message_hotspot_pattern(previous, message.activity_summary.as_deref());
            continue;
        }
        items.push(hotspot);
    }
    items
}

pub(crate) fn message_hotspot_pattern(hotspot: &MessageHotspot, activity_summary: Option<&str>) -> Option<String> {
    if hotspot.message_count <= 1 {
        return None;
    }
    if activity_summary.map(|activity| activity.starts_with("read ")).unwrap_or(false)
        && hotspot.hot_reasons.iter().any(|reason| reason == "token-heavy")
    {
        return Some(String::from("same-file-read-loop"));
    }
    Some(String::from("multi-message-hot-span"))
}

pub(crate) fn message_hotspot_from_digest(message: &MessageDigest) -> MessageHotspot {
    let mut hot_reasons = Vec::new();
    if message.duration_ms.unwrap_or_default() >= HOT_MESSAGE_SLOW_MS_THRESHOLD {
        hot_reasons.push(String::from("slow"));
    }
    if token_total(message.tokens.as_ref()) >= HOT_MESSAGE_TOKEN_THRESHOLD {
        hot_reasons.push(String::from("token-heavy"));
    }
    if message.tool_count >= HOT_MESSAGE_TOOL_COUNT_THRESHOLD {
        hot_reasons.push(String::from("tool-heavy"));
    }
    MessageHotspot {
        session_path: message.session_path.clone(),
        start_message_index: message.message_index,
        end_message_index: None,
        turn_index: message.turn_index,
        message_count: 1,
        role: message.role.clone(),
        hot_reasons,
        duration_ms: message.duration_ms,
        total_tokens: token_total(message.tokens.as_ref()),
        tool_calls: message.tool_count,
        pattern: None,
        sample_text_preview: message
            .text_preview
            .as_deref()
            .map(|text| truncate_text(text, TOOL_PART_PREVIEW_LIMIT)),
    }
}

pub(crate) fn trim_session_hotspot(mut session: SessionHotspot) -> SessionHotspot {
    if session.input_tokens + session.output_tokens + session.reasoning_tokens < HOT_SESSION_TOKEN_THRESHOLD {
        session.input_tokens = 0;
        session.output_tokens = 0;
        session.reasoning_tokens = 0;
    }
    session
}
