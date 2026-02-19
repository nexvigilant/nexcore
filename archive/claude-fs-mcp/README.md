# claude-fs-mcp

MCP server for full-access .claude filesystem operations with automatic backup creation per session start.

Tools:
- `claude_fs_list`
- `claude_fs_read`
- `claude_fs_write`
- `claude_fs_delete`
- `claude_fs_search`
- `claude_fs_tail`
- `claude_fs_diff`
- `claude_fs_stat`
- `claude_backup_now`

Backups are stored at `/home/matthew/.claude/backup/sessions/`. Set `CLAUDE_BACKUP_KEEP` to control retention (default 10).
