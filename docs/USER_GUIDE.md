# User Guide - Wiki

## 1. What Wiki Is For

Wiki stores the important materials around SDLC tasks: requirements, design notes, implementation notes, test evidence, release notes and incident notes.

## 2. Main Concepts

| Concept | Meaning |
|---|---|
| Space | Area for a product, team or workflow context |
| Document | Markdown page with revisions |
| Task dossier | Knowledge folder linked to an external task |
| Phase dossier | Materials for one project-workflow phase |
| Evidence | Link or uploaded file proving a task or phase result |

## 3. Daily Flow

1. Open dashboard.
2. Find the task dossier by task key.
3. Open linked documents or create a new document.
4. Publish a revision when the document is ready.
5. Attach evidence to the relevant phase.
6. Search by task key, title, tag or evidence source.

## 4. Document States

| State | Meaning |
|---|---|
| draft | Work in progress |
| published | Stable revision visible to readers |
| archived | Hidden from normal navigation, restorable by editors/admins |

## 5. Evidence Types

- `external_url` - ссылка на CI job, PR, artifact, release check или другой внешний материал.
- `uploaded_file` - загруженный файл со checksum и metadata.

`source_type` can additionally classify URL/file evidence as CI job, pull request, deployment, test artifact or release proof without adding a separate MVP entity.

## 6. Search

Use `/search` to find:

- documents by title/body;
- task dossier by external task key;
- phase dossier by workflow phase;
- evidence by source type or reference.

## 7. Permissions

Readers can view published content. Editors can create drafts, publish revisions and attach evidence. Owners can manage members, templates and retention.
