---
description: "Usage: /pm <prompt...>. Forwards your request directly to the 'project-manager' (pm-pam) agent."
allowed-tools:
  - Bash
model: haiku # Use Haiku for fast dispatching
---

# Dispatching to Project Manager

The user wants to speak directly to 'pm-pam'. Their full prompt is:
"$ARGUMENTS"

Please execute the following command to call the 'pm-pam' agent with this prompt:

`!claude use project-manager --prompt "$ARGUMENTS"`