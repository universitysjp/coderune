//! The documents screen.

use crate::app::CoderuneApp;
use crate::widgets::*;
use coderune_domain::{AgentAvailability, DocumentKind};
use coderune_ui::components as ui;
use coderune_ui::theme::{self, CODE_SM, LABEL};
use coderune_ui::WhenExt;
use rune::{col, row, text, El};
use rune_kit::{Badge, Kit, TextInput};

pub(crate) fn documents(s: &CoderuneApp, k: &Kit) -> El {
    let workspace = s.workspace.as_ref();
    let documents = workspace
        .map(|workspace| workspace.documents.as_slice())
        .unwrap_or(&[]);
    let projects = workspace
        .map(|workspace| workspace.projects.as_slice())
        .unwrap_or(&[]);

    let mut kind_chips = Vec::new();
    for kind in [DocumentKind::Cv, DocumentKind::Srs] {
        kind_chips.push(option_chip(
            format!("doc-kind-{kind:?}"),
            kind.label(),
            s.document_form.kind == kind,
            move |s: &mut CoderuneApp| {
                s.document_form.kind = kind;
                if kind == DocumentKind::Cv {
                    s.document_form.project_id = None;
                }
                s.document_form.extracted = None;
                s.document_form.draft_preview = None;
            },
        ));
    }
    let mut project_chips = Vec::new();
    for project in projects {
        let id = project.id.clone();
        project_chips.push(option_chip(
            format!("doc-project-{}", project.id),
            &project.name,
            s.document_form.project_id.as_deref() == Some(project.id.as_str()),
            move |s: &mut CoderuneApp| {
                s.document_form.project_id = Some(id.clone());
                s.document_form.extracted = None;
                s.document_form.draft_preview = None;
            },
        ));
    }

    let mut import_children = vec![
        ui::field(
            "Document path",
            Some("PDF, DOCX, Markdown, and plain text are extracted locally. The original file is not copied."),
            TextInput::new(&s.document_form.path)
                .placeholder("/path/to/cv.md")
                .label("Document path")
                .width(None)
                .height(32.0)
                .on_edit(|s: &mut CoderuneApp| &mut s.document_form.path)
                .on_change(|s: &mut CoderuneApp, _| {
                    s.document_form.extracted = None;
                    s.document_form.draft_preview = None;
                })
                .build(k),
        ),
        ui::field("Kind", None, row().gap(8.0).children(kind_chips)),
    ];
    if s.document_form.kind == DocumentKind::Srs {
        import_children.push(ui::field(
            "Project",
            Some("An SRS belongs to one project."),
            row().gap(8.0).children(project_chips),
        ));
    }
    import_children.push(secondary("Extract text", |s| s.extract_document(), k));
    if let Some(extracted) = &s.document_form.extracted {
        import_children.push(
            col()
                .w_full()
                .gap(12.0)
                .pt(12.0)
                .border_sides([1.0, 0.0, 0.0, 0.0], theme::row_line())
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(text(&extracted.name).font(LABEL).color(theme::ink()))
                        .child(ui::badge(&format!("{} bytes", extracted.bytes), k)),
                )
                .child(ui::code_well_tall(&extracted.text))
                .child(primary(
                    "Save reviewed import",
                    |s| s.save_document_import(),
                    k,
                )),
        );
    }

    let mut document_rows = Vec::new();
    for document in documents {
        let selected = s.document_form.selected_document.as_deref() == Some(document.id.as_str());
        let id = document.id.clone();
        let project_name = document
            .project_id
            .as_deref()
            .and_then(|id| projects.iter().find(|project| project.id == id))
            .map(|project| project.name.as_str())
            .unwrap_or("No project");
        document_rows.push(
            col()
                .key(format!("document-{}", document.id))
                .hit()
                .focusable(true)
                .role("button")
                .label(&document.name)
                .w_full()
                .padding(12.0)
                .gap(4.0)
                .radius(6.0)
                .when(selected, |this| this.fill(theme::accent_soft()))
                .when(!selected, |this| this.hover(theme::hover()))
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(
                            text(&document.name)
                                .font(LABEL)
                                .color(theme::ink())
                                .a11y_hidden(),
                        )
                        .child(Badge::new(document.kind.label()).build(k).a11y_hidden())
                        .child(
                            Badge::new(format!("{} bytes", document.text_bytes))
                                .build(k)
                                .a11y_hidden(),
                        ),
                )
                .child(
                    text(format!(
                        "{project_name} · {}",
                        &document.sha256[..8.min(document.sha256.len())]
                    ))
                    .font(CODE_SM)
                    .color(theme::faint())
                    .a11y_hidden(),
                )
                .on_press(move |s: &mut CoderuneApp| {
                    s.document_form.selected_document = Some(id.clone());
                    s.document_form.draft_preview = None;
                }),
        );
    }
    if document_rows.is_empty() {
        document_rows.push(ui::subtle("No imported documents yet."));
    }

    let selected_document = s
        .document_form
        .selected_document
        .as_deref()
        .and_then(|id| documents.iter().find(|document| document.id == id));
    let agents = workspace
        .map(|workspace| workspace.agents.as_slice())
        .unwrap_or(&[]);
    let mut agent_chips = Vec::new();
    for status in agents {
        let id = status.id;
        let enabled = status.availability == AgentAvailability::Ready;
        agent_chips.push(option_chip(
            format!("draft-agent-{:?}", status.id),
            status.id.label(),
            s.document_form.agent == Some(status.id),
            move |s: &mut CoderuneApp| {
                if enabled {
                    s.document_form.agent = Some(id);
                    s.document_form.draft_preview = None;
                }
            },
        ));
    }
    let mut draft_children = vec![];
    if let Some(document) = selected_document {
        draft_children.push(ui::subtle(&format!(
            "Selected: {} ({})",
            document.name,
            document.kind.label()
        )));
    } else {
        draft_children.push(ui::subtle(
            "Select an imported document from the list below.",
        ));
    }
    draft_children.push(row().gap(8.0).children(agent_chips));
    draft_children.push(secondary(
        "Prepare sharing preview",
        |s| s.prepare_draft(),
        k,
    ));
    if let Some(preview) = &s.document_form.draft_preview {
        draft_children.push(ui::subtle(&format!(
            "{} bytes · limit {} bytes · {}",
            preview.estimated_bytes,
            preview.max_input_bytes,
            if preview.fits { "fits" } else { "too large" }
        )));
        draft_children.push(ui::code_well_tall(&preview.text));
        draft_children.push(ui::tiny("Draft execution runs through Claude Code's headless mode in the service layer. Review the exact text before requesting."));
    }
    if let Some(error) = &s.error {
        draft_children.push(ui::alert(Some(error.code.as_str()), &error.message));
    }
    if let Some(notice) = &s.notice {
        draft_children.push(ui::alert(None, notice));
    }

    col()
        .w_full()
        .gap(24.0)
        .child(ui::page_header(
            "Documents",
            "Import CV and SRS source documents, review the extracted text, and prepare agent drafts.",
            vec![],
        ))
        .child(ui::card(Some("Import a document"), Some("Extraction is local. The original file is not copied into application storage."), import_children))
        .child(ui::card(Some("Imported documents"), Some("Approved text retained locally. Select one to prepare a draft."), document_rows))
        .child(ui::card(Some("Agent-assisted draft"), Some("Coderune shares only the exact text you preview. Sign-in, providers, and models stay with the agent."), draft_children))
}
