use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::discovery::RepoIdentity;
use crate::error::Result;
use crate::extract::{EdgeFact, FileMetrics, SymbolFact};
use crate::parser::ParseDiagnostic;

pub const SCHEMA_VERSION: &str = "1";

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub file_id: i64,
    pub rel_path: String,
    pub abs_path: String,
    pub sha256: Option<String>,
    pub lang: String,
    pub parser_id: Option<String>,
    pub is_binary: bool,
    pub parse_status: String,
    pub last_changed_scan_id: Option<i64>,
}

#[derive(Debug)]
pub struct FileUpsert {
    pub repo_id: i64,
    pub rel_path: String,
    pub abs_path: String,
    pub size_bytes: u64,
    pub mtime_ns: i64,
    pub sha256: String,
    pub lang: String,
    pub parser_id: Option<String>,
    pub is_binary: bool,
    pub parse_status: String,
    pub scan_id: i64,
    pub changed_in_scan: bool,
}

#[derive(Debug, Serialize)]
pub struct ScanRunRecord {
    pub scan_id: i64,
    pub repo_id: i64,
    pub started_ts: i64,
    pub finished_ts: Option<i64>,
    pub status: String,
    pub files_seen: u64,
    pub files_changed: u64,
    pub files_parsed: u64,
    pub errors_json: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RepoRecord {
    pub repo_id: i64,
    pub root_path: String,
    pub vcs_type: Option<String>,
    pub head_ref: Option<String>,
    pub head_commit: Option<String>,
    pub last_scan_ts: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct FileExportRecord {
    pub file_id: i64,
    pub rel_path: String,
    pub abs_path: String,
    pub size_bytes: u64,
    pub mtime_ns: i64,
    pub sha256: Option<String>,
    pub lang: String,
    pub parser_id: Option<String>,
    pub is_binary: bool,
    pub parse_status: String,
    pub last_indexed_ts: i64,
    pub last_seen_scan_id: Option<i64>,
    pub last_changed_scan_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SymbolExportRecord {
    pub symbol_id: i64,
    pub file_id: i64,
    pub rel_path: String,
    pub kind: String,
    pub name: String,
    pub qualname: String,
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u32,
    pub end_line: u32,
    pub visibility: Option<String>,
    pub signature: Option<String>,
    pub doc_excerpt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EdgeExportRecord {
    pub edge_id: i64,
    pub repo_id: i64,
    pub src_symbol_id: Option<i64>,
    pub dst_symbol_id: Option<i64>,
    pub edge_kind: String,
    pub src_file_id: Option<i64>,
    pub dst_file_id: Option<i64>,
    pub payload_json: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticExportRecord {
    pub diagnostic_id: i64,
    pub file_id: i64,
    pub rel_path: String,
    pub severity: String,
    pub code: String,
    pub message: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    pub payload_json: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct MetricsExportRecord {
    pub file_id: i64,
    pub rel_path: String,
    pub total_lines: u32,
    pub blank_lines: u32,
    pub comment_lines: u32,
    pub todo_count: u32,
    pub fixme_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ScanExportPackage {
    pub schema_version: String,
    pub repo: RepoRecord,
    pub scan_run: ScanRunRecord,
    pub files: Vec<FileExportRecord>,
    pub symbols: Vec<SymbolExportRecord>,
    pub edges: Vec<EdgeExportRecord>,
    pub diagnostics: Vec<DiagnosticExportRecord>,
    pub metrics: Vec<MetricsExportRecord>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let store = Self {
            path: path.to_path_buf(),
            conn,
        };
        store.bootstrap()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bootstrap(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS schema_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT INTO schema_meta(key, value)
            VALUES ('schema_version', '1')
            ON CONFLICT(key) DO UPDATE SET value = excluded.value;

            CREATE TABLE IF NOT EXISTS repos (
                repo_id INTEGER PRIMARY KEY AUTOINCREMENT,
                root_path TEXT NOT NULL UNIQUE,
                vcs_type TEXT,
                head_ref TEXT,
                head_commit TEXT,
                last_scan_ts INTEGER
            );

            CREATE TABLE IF NOT EXISTS scan_runs (
                scan_id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL REFERENCES repos(repo_id),
                started_ts INTEGER NOT NULL,
                finished_ts INTEGER,
                status TEXT NOT NULL,
                files_seen INTEGER NOT NULL DEFAULT 0,
                files_changed INTEGER NOT NULL DEFAULT 0,
                files_parsed INTEGER NOT NULL DEFAULT 0,
                errors_json TEXT NOT NULL DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS files (
                file_id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL REFERENCES repos(repo_id),
                rel_path TEXT NOT NULL,
                abs_path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                mtime_ns INTEGER NOT NULL,
                sha256 TEXT,
                lang TEXT NOT NULL,
                parser_id TEXT,
                is_binary INTEGER NOT NULL,
                parse_status TEXT NOT NULL,
                last_indexed_ts INTEGER NOT NULL,
                last_seen_scan_id INTEGER REFERENCES scan_runs(scan_id),
                last_changed_scan_id INTEGER REFERENCES scan_runs(scan_id),
                UNIQUE(repo_id, rel_path)
            );

            CREATE TABLE IF NOT EXISTS symbols (
                symbol_id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_id INTEGER NOT NULL REFERENCES scan_runs(scan_id),
                file_id INTEGER NOT NULL REFERENCES files(file_id),
                kind TEXT NOT NULL,
                name TEXT NOT NULL,
                qualname TEXT NOT NULL,
                start_byte INTEGER NOT NULL,
                end_byte INTEGER NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                visibility TEXT,
                signature TEXT,
                doc_excerpt TEXT
            );

            CREATE TABLE IF NOT EXISTS edges (
                edge_id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_id INTEGER NOT NULL REFERENCES scan_runs(scan_id),
                repo_id INTEGER NOT NULL REFERENCES repos(repo_id),
                src_symbol_id INTEGER REFERENCES symbols(symbol_id),
                dst_symbol_id INTEGER REFERENCES symbols(symbol_id),
                edge_kind TEXT NOT NULL,
                src_file_id INTEGER REFERENCES files(file_id),
                dst_file_id INTEGER REFERENCES files(file_id),
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS parse_diagnostics (
                diagnostic_id INTEGER PRIMARY KEY AUTOINCREMENT,
                scan_id INTEGER NOT NULL REFERENCES scan_runs(scan_id),
                file_id INTEGER NOT NULL REFERENCES files(file_id),
                severity TEXT NOT NULL,
                code TEXT NOT NULL,
                message TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_metrics (
                file_id INTEGER PRIMARY KEY REFERENCES files(file_id),
                scan_id INTEGER NOT NULL REFERENCES scan_runs(scan_id),
                total_lines INTEGER NOT NULL,
                blank_lines INTEGER NOT NULL,
                comment_lines INTEGER NOT NULL,
                todo_count INTEGER NOT NULL,
                fixme_count INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_files_repo_rel ON files(repo_id, rel_path);
            CREATE INDEX IF NOT EXISTS idx_files_repo_lang_status ON files(repo_id, lang, parse_status);
            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_edges_repo_kind ON edges(repo_id, edge_kind);
            CREATE INDEX IF NOT EXISTS idx_diagnostics_file ON parse_diagnostics(file_id);
            ",
        )?;
        Ok(())
    }

    pub fn ensure_repo(&self, identity: &RepoIdentity) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO repos(root_path, vcs_type, head_ref, head_commit, last_scan_ts)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(root_path) DO UPDATE SET
                vcs_type = excluded.vcs_type,
                head_ref = excluded.head_ref,
                head_commit = excluded.head_commit,
                last_scan_ts = excluded.last_scan_ts
            ",
            params![
                identity.root_path.to_string_lossy(),
                identity.vcs_type,
                identity.head_ref,
                identity.head_commit,
                unix_ts(),
            ],
        )?;
        let repo_id = self.conn.query_row(
            "SELECT repo_id FROM repos WHERE root_path = ?1",
            params![identity.root_path.to_string_lossy()],
            |row| row.get(0),
        )?;
        Ok(repo_id)
    }

    pub fn begin_scan_run(&self, repo_id: i64) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO scan_runs(repo_id, started_ts, status, errors_json)
            VALUES (?1, ?2, 'running', '[]')
            ",
            params![repo_id, unix_ts()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn finish_scan_run(
        &self,
        scan_id: i64,
        status: &str,
        files_seen: u64,
        files_changed: u64,
        files_parsed: u64,
        errors: &[String],
    ) -> Result<()> {
        self.conn.execute(
            "
            UPDATE scan_runs
            SET finished_ts = ?2,
                status = ?3,
                files_seen = ?4,
                files_changed = ?5,
                files_parsed = ?6,
                errors_json = ?7
            WHERE scan_id = ?1
            ",
            params![
                scan_id,
                unix_ts(),
                status,
                files_seen as i64,
                files_changed as i64,
                files_parsed as i64,
                serde_json::to_string(errors)?,
            ],
        )?;
        Ok(())
    }

    pub fn get_file(&self, repo_id: i64, rel_path: &str) -> Result<Option<FileRecord>> {
        self.conn
            .query_row(
                "
                SELECT file_id, rel_path, abs_path, sha256, lang, parser_id, is_binary,
                       parse_status, last_changed_scan_id
                FROM files
                WHERE repo_id = ?1 AND rel_path = ?2
                ",
                params![repo_id, rel_path],
                map_file_record,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn active_files(&self, repo_id: i64) -> Result<Vec<FileRecord>> {
        let mut statement = self.conn.prepare(
            "
            SELECT file_id, rel_path, abs_path, sha256, lang, parser_id, is_binary,
                   parse_status, last_changed_scan_id
            FROM files
            WHERE repo_id = ?1 AND parse_status != 'deleted'
            ORDER BY rel_path
            ",
        )?;
        let rows = statement.query_map(params![repo_id], map_file_record)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn upsert_file(&self, upsert: &FileUpsert) -> Result<i64> {
        let now = unix_ts();
        let last_changed_scan_id = if upsert.changed_in_scan {
            Some(upsert.scan_id)
        } else {
            self.get_file(upsert.repo_id, &upsert.rel_path)?
                .and_then(|record| record.last_changed_scan_id)
        };
        self.conn.execute(
            "
            INSERT INTO files(
                repo_id, rel_path, abs_path, size_bytes, mtime_ns, sha256, lang,
                parser_id, is_binary, parse_status, last_indexed_ts, last_seen_scan_id,
                last_changed_scan_id
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(repo_id, rel_path) DO UPDATE SET
                abs_path = excluded.abs_path,
                size_bytes = excluded.size_bytes,
                mtime_ns = excluded.mtime_ns,
                sha256 = excluded.sha256,
                lang = excluded.lang,
                parser_id = excluded.parser_id,
                is_binary = excluded.is_binary,
                parse_status = excluded.parse_status,
                last_indexed_ts = excluded.last_indexed_ts,
                last_seen_scan_id = excluded.last_seen_scan_id,
                last_changed_scan_id = excluded.last_changed_scan_id
            ",
            params![
                upsert.repo_id,
                upsert.rel_path,
                upsert.abs_path,
                upsert.size_bytes as i64,
                upsert.mtime_ns,
                upsert.sha256,
                upsert.lang,
                upsert.parser_id,
                if upsert.is_binary { 1 } else { 0 },
                upsert.parse_status,
                now,
                upsert.scan_id,
                last_changed_scan_id,
            ],
        )?;
        let file_id = self.conn.query_row(
            "SELECT file_id FROM files WHERE repo_id = ?1 AND rel_path = ?2",
            params![upsert.repo_id, upsert.rel_path],
            |row| row.get(0),
        )?;
        Ok(file_id)
    }

    pub fn mark_deleted(&self, file_id: i64, scan_id: i64) -> Result<()> {
        self.replace_file_facts(file_id)?;
        self.conn.execute(
            "
            UPDATE files
            SET parse_status = 'deleted',
                sha256 = NULL,
                parser_id = NULL,
                last_indexed_ts = ?2,
                last_changed_scan_id = ?3,
                last_seen_scan_id = ?3
            WHERE file_id = ?1
            ",
            params![file_id, unix_ts(), scan_id],
        )?;
        Ok(())
    }

    pub fn replace_file_facts(&self, file_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM edges WHERE src_file_id = ?1 OR dst_file_id = ?1",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM parse_diagnostics WHERE file_id = ?1",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM file_metrics WHERE file_id = ?1",
            params![file_id],
        )?;
        self.conn
            .execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])?;
        Ok(())
    }

    pub fn insert_symbols(&self, scan_id: i64, file_id: i64, symbols: &[SymbolFact]) -> Result<()> {
        for symbol in symbols {
            self.conn.execute(
                "
                INSERT INTO symbols(
                    scan_id, file_id, kind, name, qualname, start_byte, end_byte,
                    start_line, end_line, visibility, signature, doc_excerpt
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ",
                params![
                    scan_id,
                    file_id,
                    symbol.kind,
                    symbol.name,
                    symbol.qualname,
                    symbol.start_byte as i64,
                    symbol.end_byte as i64,
                    symbol.start_line as i64,
                    symbol.end_line as i64,
                    symbol.visibility,
                    symbol.signature,
                    symbol.doc_excerpt,
                ],
            )?;
        }
        Ok(())
    }

    pub fn insert_edges(
        &self,
        scan_id: i64,
        repo_id: i64,
        file_id: i64,
        edges: &[EdgeFact],
    ) -> Result<()> {
        for edge in edges {
            self.conn.execute(
                "
                INSERT INTO edges(
                    scan_id, repo_id, src_symbol_id, dst_symbol_id, edge_kind,
                    src_file_id, dst_file_id, payload_json
                )
                VALUES (?1, ?2, NULL, NULL, ?3, ?4, NULL, ?5)
                ",
                params![
                    scan_id,
                    repo_id,
                    edge.edge_kind,
                    file_id,
                    serde_json::to_string(&edge.payload_json)?,
                ],
            )?;
        }
        Ok(())
    }

    pub fn insert_diagnostics(
        &self,
        scan_id: i64,
        file_id: i64,
        diagnostics: &[ParseDiagnostic],
    ) -> Result<()> {
        for diagnostic in diagnostics {
            self.conn.execute(
                "
                INSERT INTO parse_diagnostics(
                    scan_id, file_id, severity, code, message, start_line, end_line,
                    payload_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ",
                params![
                    scan_id,
                    file_id,
                    diagnostic.severity,
                    diagnostic.code,
                    diagnostic.message,
                    diagnostic.start_line.map(i64::from),
                    diagnostic.end_line.map(i64::from),
                    serde_json::to_string(&diagnostic.payload_json)?,
                ],
            )?;
        }
        Ok(())
    }

    pub fn upsert_metrics(&self, scan_id: i64, file_id: i64, metrics: &FileMetrics) -> Result<()> {
        self.conn.execute(
            "
            INSERT INTO file_metrics(
                file_id, scan_id, total_lines, blank_lines, comment_lines, todo_count, fixme_count
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(file_id) DO UPDATE SET
                scan_id = excluded.scan_id,
                total_lines = excluded.total_lines,
                blank_lines = excluded.blank_lines,
                comment_lines = excluded.comment_lines,
                todo_count = excluded.todo_count,
                fixme_count = excluded.fixme_count
            ",
            params![
                file_id,
                scan_id,
                metrics.total_lines as i64,
                metrics.blank_lines as i64,
                metrics.comment_lines as i64,
                metrics.todo_count as i64,
                metrics.fixme_count as i64,
            ],
        )?;
        Ok(())
    }

    pub fn repo_by_root(&self, root: &Path) -> Result<Option<RepoRecord>> {
        self.conn
            .query_row(
                "
                SELECT repo_id, root_path, vcs_type, head_ref, head_commit, last_scan_ts
                FROM repos
                WHERE root_path = ?1
                ",
                params![root.to_string_lossy()],
                map_repo_record,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn repo_for_scan(&self, scan_id: i64) -> Result<RepoRecord> {
        self.conn
            .query_row(
                "
                SELECT r.repo_id, r.root_path, r.vcs_type, r.head_ref, r.head_commit, r.last_scan_ts
                FROM repos r
                JOIN scan_runs s ON s.repo_id = r.repo_id
                WHERE s.scan_id = ?1
                ",
                params![scan_id],
                map_repo_record,
            )
            .map_err(Into::into)
    }

    pub fn scan_run(&self, scan_id: i64) -> Result<ScanRunRecord> {
        self.conn
            .query_row(
                "
                SELECT scan_id, repo_id, started_ts, finished_ts, status, files_seen,
                       files_changed, files_parsed, errors_json
                FROM scan_runs
                WHERE scan_id = ?1
                ",
                params![scan_id],
                map_scan_run_record,
            )
            .map_err(Into::into)
    }

    pub fn query_symbols(&self, repo_id: i64, name: &str) -> Result<Vec<SymbolExportRecord>> {
        let pattern = format!("%{name}%");
        let mut statement = self.conn.prepare(
            "
            SELECT s.symbol_id, s.file_id, f.rel_path, s.kind, s.name, s.qualname,
                   s.start_byte, s.end_byte, s.start_line, s.end_line, s.visibility,
                   s.signature, s.doc_excerpt
            FROM symbols s
            JOIN files f ON f.file_id = s.file_id
            WHERE f.repo_id = ?1 AND s.name LIKE ?2
            ORDER BY f.rel_path, s.start_line, s.kind, s.name
            ",
        )?;
        let rows = statement.query_map(params![repo_id, pattern], map_symbol_export_record)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn query_files(
        &self,
        repo_id: i64,
        lang: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<FileExportRecord>> {
        let mut sql = "
            SELECT file_id, rel_path, abs_path, size_bytes, mtime_ns, sha256, lang,
                   parser_id, is_binary, parse_status, last_indexed_ts,
                   last_seen_scan_id, last_changed_scan_id
            FROM files
            WHERE repo_id = ?1
        "
        .to_string();
        if lang.is_some() {
            sql.push_str(" AND lang = ?2");
        }
        if status.is_some() {
            sql.push_str(if lang.is_some() {
                " AND parse_status = ?3"
            } else {
                " AND parse_status = ?2"
            });
        }
        sql.push_str(" ORDER BY rel_path");

        let mut statement = self.conn.prepare(&sql)?;
        let rows = match (lang, status) {
            (Some(lang), Some(status)) => {
                statement.query_map(params![repo_id, lang, status], map_file_export_record)?
            }
            (Some(lang), None) => {
                statement.query_map(params![repo_id, lang], map_file_export_record)?
            }
            (None, Some(status)) => {
                statement.query_map(params![repo_id, status], map_file_export_record)?
            }
            (None, None) => statement.query_map(params![repo_id], map_file_export_record)?,
        };
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn export_scan(
        &self,
        scan_id: i64,
        include_diagnostics: bool,
    ) -> Result<ScanExportPackage> {
        let repo = self.repo_for_scan(scan_id)?;
        let scan_run = self.scan_run(scan_id)?;
        let files = self.files_for_scan(scan_id)?;
        let file_ids = files.iter().map(|file| file.file_id).collect::<Vec<_>>();
        let symbols = self.symbols_for_files(&file_ids)?;
        let edges = self.edges_for_scan(scan_id, &file_ids)?;
        let diagnostics = if include_diagnostics {
            self.diagnostics_for_files(&file_ids)?
        } else {
            Vec::new()
        };
        let metrics = self.metrics_for_files(&file_ids)?;
        Ok(ScanExportPackage {
            schema_version: SCHEMA_VERSION.to_string(),
            repo,
            scan_run,
            files,
            symbols,
            edges,
            diagnostics,
            metrics,
        })
    }

    fn files_for_scan(&self, scan_id: i64) -> Result<Vec<FileExportRecord>> {
        let mut statement = self.conn.prepare(
            "
            SELECT file_id, rel_path, abs_path, size_bytes, mtime_ns, sha256, lang,
                   parser_id, is_binary, parse_status, last_indexed_ts,
                   last_seen_scan_id, last_changed_scan_id
            FROM files
            WHERE last_seen_scan_id = ?1 OR last_changed_scan_id = ?1
            ORDER BY rel_path
            ",
        )?;
        let rows = statement.query_map(params![scan_id], map_file_export_record)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    fn symbols_for_files(&self, file_ids: &[i64]) -> Result<Vec<SymbolExportRecord>> {
        let mut output = Vec::new();
        let mut statement = self.conn.prepare(
            "
            SELECT s.symbol_id, s.file_id, f.rel_path, s.kind, s.name, s.qualname,
                   s.start_byte, s.end_byte, s.start_line, s.end_line, s.visibility,
                   s.signature, s.doc_excerpt
            FROM symbols s
            JOIN files f ON f.file_id = s.file_id
            WHERE s.file_id = ?1
            ORDER BY f.rel_path, s.start_line, s.kind, s.name
            ",
        )?;
        for file_id in file_ids {
            let rows = statement.query_map(params![file_id], map_symbol_export_record)?;
            for row in rows {
                output.push(row?);
            }
        }
        output.sort_by(|left, right| {
            (&left.rel_path, left.start_line, &left.kind, &left.name).cmp(&(
                &right.rel_path,
                right.start_line,
                &right.kind,
                &right.name,
            ))
        });
        Ok(output)
    }

    fn edges_for_scan(&self, scan_id: i64, file_ids: &[i64]) -> Result<Vec<EdgeExportRecord>> {
        let mut output = Vec::new();
        let mut statement = self.conn.prepare(
            "
            SELECT edge_id, repo_id, src_symbol_id, dst_symbol_id, edge_kind,
                   src_file_id, dst_file_id, payload_json
            FROM edges
            WHERE scan_id = ?1 AND src_file_id = ?2
            ORDER BY edge_kind, payload_json
            ",
        )?;
        for file_id in file_ids {
            let rows = statement.query_map(params![scan_id, file_id], map_edge_export_record)?;
            for row in rows {
                output.push(row?);
            }
        }
        output.sort_by(|left, right| {
            (&left.edge_kind, left.payload_json.to_string())
                .cmp(&(&right.edge_kind, right.payload_json.to_string()))
        });
        Ok(output)
    }

    fn diagnostics_for_files(&self, file_ids: &[i64]) -> Result<Vec<DiagnosticExportRecord>> {
        let mut output = Vec::new();
        let mut statement = self.conn.prepare(
            "
            SELECT d.diagnostic_id, d.file_id, f.rel_path, d.severity, d.code, d.message,
                   d.start_line, d.end_line, d.payload_json
            FROM parse_diagnostics d
            JOIN files f ON f.file_id = d.file_id
            WHERE d.file_id = ?1
            ORDER BY f.rel_path, d.start_line, d.code
            ",
        )?;
        for file_id in file_ids {
            let rows = statement.query_map(params![file_id], map_diagnostic_export_record)?;
            for row in rows {
                output.push(row?);
            }
        }
        output.sort_by(|left, right| {
            (&left.rel_path, left.start_line, &left.code).cmp(&(
                &right.rel_path,
                right.start_line,
                &right.code,
            ))
        });
        Ok(output)
    }

    fn metrics_for_files(&self, file_ids: &[i64]) -> Result<Vec<MetricsExportRecord>> {
        let mut output = Vec::new();
        let mut statement = self.conn.prepare(
            "
            SELECT m.file_id, f.rel_path, m.total_lines, m.blank_lines, m.comment_lines,
                   m.todo_count, m.fixme_count
            FROM file_metrics m
            JOIN files f ON f.file_id = m.file_id
            WHERE m.file_id = ?1
            ORDER BY f.rel_path
            ",
        )?;
        for file_id in file_ids {
            let rows = statement.query_map(params![file_id], map_metrics_export_record)?;
            for row in rows {
                output.push(row?);
            }
        }
        output.sort_by(|left, right| left.rel_path.cmp(&right.rel_path));
        Ok(output)
    }
}

fn map_file_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileRecord> {
    Ok(FileRecord {
        file_id: row.get(0)?,
        rel_path: row.get(1)?,
        abs_path: row.get(2)?,
        sha256: row.get(3)?,
        lang: row.get(4)?,
        parser_id: row.get(5)?,
        is_binary: row.get::<_, i64>(6)? != 0,
        parse_status: row.get(7)?,
        last_changed_scan_id: row.get(8)?,
    })
}

fn map_repo_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepoRecord> {
    Ok(RepoRecord {
        repo_id: row.get(0)?,
        root_path: row.get(1)?,
        vcs_type: row.get(2)?,
        head_ref: row.get(3)?,
        head_commit: row.get(4)?,
        last_scan_ts: row.get(5)?,
    })
}

fn map_scan_run_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScanRunRecord> {
    let errors_raw: String = row.get(8)?;
    Ok(ScanRunRecord {
        scan_id: row.get(0)?,
        repo_id: row.get(1)?,
        started_ts: row.get(2)?,
        finished_ts: row.get(3)?,
        status: row.get(4)?,
        files_seen: row.get::<_, i64>(5)? as u64,
        files_changed: row.get::<_, i64>(6)? as u64,
        files_parsed: row.get::<_, i64>(7)? as u64,
        errors_json: serde_json::from_str(&errors_raw).unwrap_or_else(|_| serde_json::json!([])),
    })
}

fn map_file_export_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileExportRecord> {
    Ok(FileExportRecord {
        file_id: row.get(0)?,
        rel_path: row.get(1)?,
        abs_path: row.get(2)?,
        size_bytes: row.get::<_, i64>(3)? as u64,
        mtime_ns: row.get(4)?,
        sha256: row.get(5)?,
        lang: row.get(6)?,
        parser_id: row.get(7)?,
        is_binary: row.get::<_, i64>(8)? != 0,
        parse_status: row.get(9)?,
        last_indexed_ts: row.get(10)?,
        last_seen_scan_id: row.get(11)?,
        last_changed_scan_id: row.get(12)?,
    })
}

fn map_symbol_export_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<SymbolExportRecord> {
    Ok(SymbolExportRecord {
        symbol_id: row.get(0)?,
        file_id: row.get(1)?,
        rel_path: row.get(2)?,
        kind: row.get(3)?,
        name: row.get(4)?,
        qualname: row.get(5)?,
        start_byte: row.get::<_, i64>(6)? as u64,
        end_byte: row.get::<_, i64>(7)? as u64,
        start_line: row.get::<_, i64>(8)? as u32,
        end_line: row.get::<_, i64>(9)? as u32,
        visibility: row.get(10)?,
        signature: row.get(11)?,
        doc_excerpt: row.get(12)?,
    })
}

fn map_edge_export_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<EdgeExportRecord> {
    let payload_raw: String = row.get(7)?;
    Ok(EdgeExportRecord {
        edge_id: row.get(0)?,
        repo_id: row.get(1)?,
        src_symbol_id: row.get(2)?,
        dst_symbol_id: row.get(3)?,
        edge_kind: row.get(4)?,
        src_file_id: row.get(5)?,
        dst_file_id: row.get(6)?,
        payload_json: serde_json::from_str(&payload_raw).unwrap_or_else(|_| serde_json::json!({})),
    })
}

fn map_diagnostic_export_record(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<DiagnosticExportRecord> {
    let payload_raw: String = row.get(8)?;
    Ok(DiagnosticExportRecord {
        diagnostic_id: row.get(0)?,
        file_id: row.get(1)?,
        rel_path: row.get(2)?,
        severity: row.get(3)?,
        code: row.get(4)?,
        message: row.get(5)?,
        start_line: row.get::<_, Option<i64>>(6)?.map(|value| value as u32),
        end_line: row.get::<_, Option<i64>>(7)?.map(|value| value as u32),
        payload_json: serde_json::from_str(&payload_raw).unwrap_or_else(|_| serde_json::json!({})),
    })
}

fn map_metrics_export_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetricsExportRecord> {
    Ok(MetricsExportRecord {
        file_id: row.get(0)?,
        rel_path: row.get(1)?,
        total_lines: row.get::<_, i64>(2)? as u32,
        blank_lines: row.get::<_, i64>(3)? as u32,
        comment_lines: row.get::<_, i64>(4)? as u32,
        todo_count: row.get::<_, i64>(5)? as u32,
        fixme_count: row.get::<_, i64>(6)? as u32,
    })
}

pub fn unix_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
