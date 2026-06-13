# Interaction

## Interaction Created

Raw SSE event:

```
event: interaction.created
data: {"interaction":{"id":"v1_...","status":"in_progress","model":"gemini-3-flash-preview"},"event_type":"interaction.created","metadata":{...}}
```

Parsed `data` object:

```json
{
  "event": "interaction.created",
  "data": {
    "interaction": {
      "id": "v1_ChdqS0FzYXBYY0hPZWsxTWtQdTZhbXNBcxIXaktBc2FwWGNIT2VrMU1rUHU2YW1zQXM",
      "status": "in_progress",
      "object": "interaction",
      "model": "gemini-3-flash-preview"
    },
    "event_type": "interaction.created",
    "metadata": {
      "total_usage": {
        "total_tokens": 3886,
        "total_input_tokens": 3886,
        "input_tokens_by_modality": [
          {
            "modality": "text",
            "tokens": 3886
          }
        ],
        "total_cached_tokens": 0,
        "total_output_tokens": 0,
        "total_tool_use_tokens": 0,
        "total_thought_tokens": 0
      }
    },
    "id": "",
    "retry": "None"
  }
}
```

## Interaction Status Update

```json
{
  "event": "interaction.status_update",
  "data": {
    "interaction_id": "v1_ChdqS0FzYXBYY0hPZWsxTWtQdTZhbXNBcxIXaktBc2FwWGNIT2VrMU1rUHU2YW1zQXM",
    "status": "in_progress",
    "event_type": "interaction.status_update"
  },
  "id": "",
  "retry": "None"
}
```

## Step

## Step Start (Thought)

```json
{
  "event": "step.start",
  "data": {
    "index": 0,
    "step": {
      "type": "thought"
    },
    "event_type": "step.start"
  },
  "id": "",
  "retry": "None"
}
```

## Step Delta

### (Though Summary)

```json
{
  "event": "step.delta",
  "data": {
    "index": 0,
    "delta": {
      "content": {
        "text": "**Assessing AAPL's Potential**\\n\\nI've determined that calling screening tools isn't necessary because the request focuses on a specific ticker. I also realize I don't need to call the `ticker_taxonomy` tool because it isn't seeking specific financial information about AAPL's sector. I am ready to move on to the next set of instructions.\\n\\n\\n",
        "type": "text"
      },
      "type": "thought_summary"
    },
    "event_type": "step.delta"
  },
  "id": "",
  "retry": "None"
}
```

### (Though Signature)

```json
{
  "event": "step.delta",
  "data": {
    "index": 0,
    "delta": {
      "signature": "",
      "type": "thought_signature"
    },
    "event_type": "step.delta"
  },
  "id": "",
  "retry": "None"
}
```

### (Arguments delta)

```json
{
  "event": "step.delta",
  "data": {
    "index": 2,
    "delta": {
      "arguments": {
        "symbols": ["AAPL"]
      },
      "type": "arguments_delta"
    },
    "event_type": "step.delta"
  },
  "id": "",
  "retry": "None"
}
```

## Step Stop

```json
{
  "event": "step.stop",
  "data": {
    "index": 0,
    "event_type": "step.stop",
    "metadata": {
      "total_usage": {
        "total_tokens": 4180,
        "total_input_tokens": 3886,
        "input_tokens_by_modality": [{ "modality": "text", "tokens": 3886 }],
        "total_cached_tokens": 0,
        "total_output_tokens": 16,
        "total_tool_use_tokens": 0,
        "total_thought_tokens": 278
      }
    }
  },
  "id": "",
  "retry": "None"
}
```

## Interaction Completed

```json
{
  "event": "interaction.completed",
  "data": {
    "interaction": {
      "id": "v1_ChdqS0FzYXBYY0hPZWsxTWtQdTZhbXNBcxIXaktBc2FwWGNIT2VrMU1rUHU2YW1zQXM",
      "status": "requires_action",
      "usage": {
        "total_tokens": 4221,
        "total_input_tokens": 3886,
        "input_tokens_by_modality": [{ "modality": "text", "tokens": 3886 }],
        "total_cached_tokens": 0,
        "total_output_tokens": 57,
        "total_tool_use_tokens": 0,
        "total_thought_tokens": 278
      },
      "created": "2026-06-13T00:13:02Z",
      "updated": "2026-06-13T00:13:02Z",
      "service_tier": "standard",
      "object": "interaction",
      "model": "gemini-3-flash-preview"
    },
    "event_type": "interaction.completed"
  },
  "id": "",
  "retry": "None"
}
```
