import { Database } from "bun:sqlite";
import { chmodSync, existsSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";

import type {
  ContextStore,
  DeveloperProfile,
  DeveloperProfileFields,
  ContextCapture,
  ImportedDocument,
  OnboardingDraft,
  Project,
} from "@coderune/core";

const DATABASE_VERSION = 4;

interface Migration {
  version: number;
  apply: string;
}

const MIGRATIONS: Migration[] = [
  {
    version: 1,
    apply: `
      CREATE TABLE developer_profiles (
        id TEXT PRIMARY KEY,
        version INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );

      CREATE TABLE developer_profile_versions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        profile_id TEXT NOT NULL,
        version INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE(profile_id, version)
      );

      CREATE TABLE onboarding_draft (
        id TEXT PRIMARY KEY CHECK (id = 'onboarding'),
        data TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );
    `,
  },
  {
    version: 2,
    apply: `
      CREATE TABLE projects (
        id TEXT PRIMARY KEY,
        version INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      );

      CREATE TABLE project_versions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id TEXT NOT NULL,
        version INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE(project_id, version)
      );
    `,
  },
  {
    version: 3,
    apply: `
      CREATE TABLE documents (
        id TEXT PRIMARY KEY,
        project_id TEXT,
        kind TEXT NOT NULL,
        name TEXT NOT NULL,
        path TEXT NOT NULL,
        hash TEXT NOT NULL,
        size INTEGER NOT NULL,
        text TEXT NOT NULL,
        data TEXT NOT NULL,
        imported_at TEXT NOT NULL
      );

      CREATE INDEX documents_project_id ON documents(project_id);
      CREATE INDEX documents_hash ON documents(hash);
    `,
  },
  {
    version: 4,
    apply: `
      CREATE TABLE contents (
        hash TEXT PRIMARY KEY,
        text TEXT NOT NULL,
        size INTEGER NOT NULL
      );

      CREATE TABLE captures (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        version INTEGER NOT NULL,
        fingerprint TEXT NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE(project_id, version)
      );

      CREATE INDEX captures_project_id ON captures(project_id);

      CREATE TABLE capture_files (
        capture_id TEXT NOT NULL,
        path TEXT NOT NULL,
        content_hash TEXT NOT NULL,
        size INTEGER NOT NULL,
        category TEXT NOT NULL,
        PRIMARY KEY (capture_id, path)
      );

      CREATE TABLE capture_skipped (
        capture_id TEXT NOT NULL,
        path TEXT NOT NULL,
        reason TEXT NOT NULL,
        PRIMARY KEY (capture_id, path)
      );
    `,
  },
];

interface ProfileRow {
  id: string;
  version: number;
  data: string;
  created_at: string;
  updated_at: string;
}

interface DraftRow {
  data: string;
}

export interface SqliteStoreOptions {
  databasePath: string;
}

export class SqliteContextStore implements ContextStore {
  readonly #database: Database;

  constructor(options: SqliteStoreOptions) {
    mkdirSync(dirname(options.databasePath), { recursive: true });
    const created = !existsSync(options.databasePath);

    this.#database = new Database(options.databasePath, { create: true });
    this.#database.exec("PRAGMA journal_mode = WAL;");
    this.#database.exec("PRAGMA foreign_keys = ON;");
    this.#database.exec("PRAGMA busy_timeout = 5000;");

    if (created) {
      try {
        chmodSync(options.databasePath, 0o600);
      } catch {
        // File permissions are best-effort on platforms without POSIX modes.
      }
    }
  }

  initialize(): void {
    this.#database.exec(`
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL
      );
    `);

    for (const migration of MIGRATIONS) {
      const applied = this.#database
        .query<{ version: number }, [number]>(
          "SELECT version FROM schema_migrations WHERE version = ?"
        )
        .get(migration.version);

      if (applied) {
        continue;
      }

      const apply = this.#database.transaction(() => {
        this.#database.exec(migration.apply);
        this.#database
          .query(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)"
          )
          .run(migration.version, new Date().toISOString());
      });

      apply();
    }

    const latest = this.#database
      .query<{ version: number }, []>(
        "SELECT MAX(version) AS version FROM schema_migrations"
      )
      .get();

    if (!latest || latest.version !== DATABASE_VERSION) {
      throw new Error(
        `Unsupported Coderune database version: ${latest?.version}`
      );
    }
  }

  close(): void {
    this.#database.close();
  }

  getProfile(): DeveloperProfile | null {
    const row = this.#database
      .query<ProfileRow, []>(
        "SELECT id, version, data, created_at, updated_at FROM developer_profiles WHERE id = 'developer'"
      )
      .get();

    return row ? (JSON.parse(row.data) as DeveloperProfile) : null;
  }

  getProfileVersions(): DeveloperProfile[] {
    const rows = this.#database
      .query<{ data: string }, []>(
        "SELECT data FROM developer_profile_versions WHERE profile_id = 'developer' ORDER BY version ASC"
      )
      .all();

    return rows.map((row) => JSON.parse(row.data) as DeveloperProfile);
  }

  saveProfile(fields: DeveloperProfileFields): DeveloperProfile {
    const current = this.getProfile();
    const now = new Date().toISOString();

    const profile: DeveloperProfile = {
      ...fields,
      id: "developer",
      version: (current?.version ?? 0) + 1,
      createdAt: current?.createdAt ?? now,
      updatedAt: now,
    };

    const data = JSON.stringify(profile);

    const save = this.#database.transaction(() => {
      this.#database
        .query(
          `INSERT INTO developer_profiles (id, version, data, created_at, updated_at)
           VALUES ('developer', ?, ?, ?, ?)
           ON CONFLICT(id) DO UPDATE SET
             version = excluded.version,
             data = excluded.data,
             updated_at = excluded.updated_at`
        )
        .run(profile.version, data, profile.createdAt, profile.updatedAt);

      this.#database
        .query(
          `INSERT INTO developer_profile_versions (profile_id, version, data, created_at)
           VALUES ('developer', ?, ?, ?)`
        )
        .run(profile.version, data, now);
    });

    save();

    return profile;
  }

  getOnboardingDraft(): OnboardingDraft | null {
    const row = this.#database
      .query<DraftRow, []>(
        "SELECT data FROM onboarding_draft WHERE id = 'onboarding'"
      )
      .get();

    return row ? (JSON.parse(row.data) as OnboardingDraft) : null;
  }

  saveOnboardingDraft(draft: OnboardingDraft): OnboardingDraft {
    this.#database
      .query(
        `INSERT INTO onboarding_draft (id, data, updated_at)
         VALUES ('onboarding', ?, ?)
         ON CONFLICT(id) DO UPDATE SET
           data = excluded.data,
           updated_at = excluded.updated_at`
      )
      .run(JSON.stringify(draft), draft.updatedAt);

    return draft;
  }

  clearOnboardingDraft(): void {
    this.#database
      .query("DELETE FROM onboarding_draft WHERE id = 'onboarding'")
      .run();
  }
  createProject(project: Project): Project {
    const save = this.#database.transaction(() => {
      this.#database
        .query(
          `INSERT INTO projects (id, version, data, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?)`
        )
        .run(
          project.id,
          project.version,
          JSON.stringify(project),
          project.createdAt,
          project.updatedAt
        );
      this.#database
        .query(
          `INSERT INTO project_versions (project_id, version, data, created_at)
           VALUES (?, ?, ?, ?)`
        )
        .run(
          project.id,
          project.version,
          JSON.stringify(project),
          project.updatedAt
        );
    });

    save();

    return project;
  }

  updateProject(project: Project): Project {
    const save = this.#database.transaction(() => {
      this.#database
        .query(
          `UPDATE projects SET version = ?, data = ?, updated_at = ? WHERE id = ?`
        )
        .run(
          project.version,
          JSON.stringify(project),
          project.updatedAt,
          project.id
        );
      this.#database
        .query(
          `INSERT INTO project_versions (project_id, version, data, created_at)
           VALUES (?, ?, ?, ?)`
        )
        .run(
          project.id,
          project.version,
          JSON.stringify(project),
          project.updatedAt
        );
    });

    save();

    return project;
  }

  getProject(id: string): Project | null {
    const row = this.#database
      .query<{ data: string }, [string]>(
        "SELECT data FROM projects WHERE id = ?"
      )
      .get(id);

    return row ? (JSON.parse(row.data) as Project) : null;
  }

  getProjectVersions(id: string): Project[] {
    const rows = this.#database
      .query<{ data: string }, [string]>(
        "SELECT data FROM project_versions WHERE project_id = ? ORDER BY version ASC"
      )
      .all(id);

    return rows.map((row) => JSON.parse(row.data) as Project);
  }

  listProjects(): Project[] {
    const rows = this.#database
      .query<{ data: string }, []>(
        "SELECT data FROM projects ORDER BY created_at ASC"
      )
      .all();

    return rows.map((row) => JSON.parse(row.data) as Project);
  }

  saveCapture(capture: ContextCapture): ContextCapture {
    const save = this.#database.transaction(() => {
      for (const file of capture.files) {
        this.#database
          .query(
            `INSERT INTO contents (hash, text, size) VALUES (?, ?, ?)
             ON CONFLICT(hash) DO NOTHING`
          )
          .run(file.hash, file.content, file.size);
      }

      const data = JSON.stringify({
        ...capture,
        files: capture.files.map(({ content: _content, ...file }) => file),
      });

      this.#database
        .query(
          `INSERT INTO captures (id, project_id, version, fingerprint, data, created_at)
           VALUES (?, ?, ?, ?, ?, ?)`
        )
        .run(
          capture.id,
          capture.projectId,
          capture.version,
          capture.fingerprint,
          data,
          capture.createdAt
        );

      for (const file of capture.files) {
        this.#database
          .query(
            `INSERT INTO capture_files (capture_id, path, content_hash, size, category)
             VALUES (?, ?, ?, ?, ?)`
          )
          .run(capture.id, file.path, file.hash, file.size, file.category);
      }

      for (const skipped of capture.skipped) {
        this.#database
          .query(
            `INSERT INTO capture_skipped (capture_id, path, reason)
             VALUES (?, ?, ?)`
          )
          .run(capture.id, skipped.path, skipped.reason);
      }
    });

    save();

    return capture;
  }

  getCapture(id: string): ContextCapture | null {
    const row = this.#database
      .query<{ data: string }, [string]>(
        "SELECT data FROM captures WHERE id = ?"
      )
      .get(id);

    return row ? this.#hydrateCapture(row.data) : null;
  }

  listCaptures(projectId: string): ContextCapture[] {
    const rows = this.#database
      .query<{ data: string }, [string]>(
        "SELECT data FROM captures WHERE project_id = ? ORDER BY version DESC"
      )
      .all(projectId);

    return rows.map((row) => this.#hydrateCapture(row.data));
  }

  getCaptureContent(hash: string): string | null {
    const row = this.#database
      .query<{ text: string }, [string]>(
        "SELECT text FROM contents WHERE hash = ?"
      )
      .get(hash);

    return row?.text ?? null;
  }

  deleteCapture(id: string): void {
    const remove = this.#database.transaction(() => {
      this.#database
        .query("DELETE FROM capture_skipped WHERE capture_id = ?")
        .run(id);
      this.#database
        .query("DELETE FROM capture_files WHERE capture_id = ?")
        .run(id);
      this.#database.query("DELETE FROM captures WHERE id = ?").run(id);
      this.#deleteOrphanContents();
    });

    remove();
  }

  deleteProjectCaptures(projectId: string): void {
    const remove = this.#database.transaction(() => {
      const ids = this.#database
        .query<{ id: string }, [string]>(
          "SELECT id FROM captures WHERE project_id = ?"
        )
        .all(projectId)
        .map((row) => row.id);

      for (const id of ids) {
        this.#database
          .query("DELETE FROM capture_skipped WHERE capture_id = ?")
          .run(id);
        this.#database
          .query("DELETE FROM capture_files WHERE capture_id = ?")
          .run(id);
        this.#database.query("DELETE FROM captures WHERE id = ?").run(id);
      }

      this.#deleteOrphanContents();
    });

    remove();
  }

  deleteAllData(): void {
    const remove = this.#database.transaction(() => {
      this.#database.exec(`
        DELETE FROM capture_skipped;
        DELETE FROM capture_files;
        DELETE FROM captures;
        DELETE FROM contents;
        DELETE FROM documents;
        DELETE FROM project_versions;
        DELETE FROM projects;
        DELETE FROM developer_profile_versions;
        DELETE FROM developer_profiles;
        DELETE FROM onboarding_draft;
      `);
    });

    remove();
  }

  #deleteOrphanContents(): void {
    this.#database
      .query(
        "DELETE FROM contents WHERE hash NOT IN (SELECT content_hash FROM capture_files)"
      )
      .run();
  }

  #hydrateCapture(data: string): ContextCapture {
    const capture = JSON.parse(data) as ContextCapture;

    return {
      ...capture,
      files: capture.files.map((file) => ({
        ...file,
        content: this.getCaptureContent(file.hash) ?? "",
      })),
    };
  }

  saveDocument(document: ImportedDocument): ImportedDocument {
    this.#database
      .query(
        `INSERT INTO documents (id, project_id, kind, name, path, hash, size, text, data, imported_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
           project_id = excluded.project_id,
           kind = excluded.kind,
           name = excluded.name,
           path = excluded.path,
           hash = excluded.hash,
           size = excluded.size,
           text = excluded.text,
           data = excluded.data,
           imported_at = excluded.imported_at`
      )
      .run(
        document.id,
        document.projectId,
        document.kind,
        document.name,
        document.path,
        document.hash,
        document.size,
        document.text,
        JSON.stringify(document),
        document.importedAt
      );

    return document;
  }

  getDocument(id: string): ImportedDocument | null {
    const row = this.#database
      .query<{ data: string }, [string]>(
        "SELECT data FROM documents WHERE id = ?"
      )
      .get(id);

    return row ? (JSON.parse(row.data) as ImportedDocument) : null;
  }

  listDocuments(projectId?: string | null): ImportedDocument[] {
    const rows =
      projectId === undefined
        ? this.#database
            .query<{ data: string }, []>(
              "SELECT data FROM documents ORDER BY imported_at ASC"
            )
            .all()
        : this.#database
            .query<{ data: string }, [string | null]>(
              "SELECT data FROM documents WHERE project_id IS ? ORDER BY imported_at ASC"
            )
            .all(projectId);

    return rows.map((row) => JSON.parse(row.data) as ImportedDocument);
  }

  deleteDocument(id: string): void {
    this.#database.query("DELETE FROM documents WHERE id = ?").run(id);
  }

  deleteProject(id: string): void {
    const remove = this.#database.transaction(() => {
      const captureIds = this.#database
        .query<{ id: string }, [string]>(
          "SELECT id FROM captures WHERE project_id = ?"
        )
        .all(id)
        .map((row) => row.id);

      for (const captureId of captureIds) {
        this.#database
          .query("DELETE FROM capture_skipped WHERE capture_id = ?")
          .run(captureId);
        this.#database
          .query("DELETE FROM capture_files WHERE capture_id = ?")
          .run(captureId);
        this.#database
          .query("DELETE FROM captures WHERE id = ?")
          .run(captureId);
      }

      this.#database
        .query("DELETE FROM project_versions WHERE project_id = ?")
        .run(id);
      this.#database
        .query("DELETE FROM documents WHERE project_id = ?")
        .run(id);
      this.#database.query("DELETE FROM projects WHERE id = ?").run(id);
      this.#deleteOrphanContents();
    });

    remove();
  }
}
