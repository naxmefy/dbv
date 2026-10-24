# Roadmap

| #     | Milestone             | Content                                                                                                                | Release |
|-------|-----------------------|------------------------------------------------------------------------------------------------------------------------|---------|
| M0    | Project foundation    | OSS docs, ADRs, CI consolidation, release pipeline, cleanup                                                            | –       |
| M1    | SQLite + TUI skeleton | Connections, explorer, query history, scripts, editor, results, command log, keybinding config                         | v0.1    |
| M2    | Headless CLI          | `dbv query -c <conn> "…" --format json\|csv\|table`, `dbv connections list`, `dbv schema …`; stable output, exit codes | v0.2    |
| M3    | Postgres              | 2nd driver, async + query cancellation, schemas/functions, password storage                                            | v0.3    |
| M4    | Data editing          | Grid edit/insert/delete with statement preview + confirmation                                                          | v0.4    |
| M5    | Object editing        | Create/alter/truncate/drop tables, views, indexes; DDL preview + confirmation                                          | v0.5    |
| M6    | MySQL/MariaDB         | 3rd driver                                                                                                             | v0.6    |
| M7    | Distribution          | Homebrew, Scoop, winget, APT/.deb, `cargo install`                                                                     | –       |
| M8    | Responsive layout     | lazygit-style: unfocused panels collapse to their title line, focused panel grows                                      | –       |
| M9    | Comfort               | Bookmarks, per-connection scripts dir, persistent result cache, export, themes, syntax highlighting                    | –       |
| later |                       | MSSQL, Oracle (optional), ODBC, NoSQL, diagrams/dashboards, GitHub Sponsors                                            |         |

Milestones are refined as they start, and the order may change.
