import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import isekaiImage from "./assets/isekai.png";
import isekai2Image from "./assets/isekai2.png";
import listImage from "./assets/list.jpg";
import "./App.css";

type DocumentKind = "text" | "markdown" | "unknown";

type OpenedDocument = {
  name: string;
  path: string;
  kind: DocumentKind;
  content: string;
};

type NoteEntry = {
  name: string;
  path: string;
};

type QuickNoteDraft = {
  name: string;
  path: string;
};

type NoteListUpdate = {
  note: NoteEntry;
  previousPath?: string;
};

function AppTitleBar() {
  async function startDragging(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return;
    }

    await invoke("titlebar_start_dragging");
  }

  async function toggleMaximize(event: React.MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    await invoke("titlebar_toggle_maximize");
  }

  async function minimizeWindow(event: React.MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    await invoke("titlebar_minimize");
  }

  async function closeWindow(event: React.MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    await invoke("titlebar_close");
  }

  return (
    <header className="app-titlebar">
      <div className="app-titlebar-drag" onMouseDown={startDragging}>
        <div className="app-titlebar-name">ヰnote</div>
      </div>
      <div className="window-controls">
        <button
          type="button"
          onClick={toggleMaximize}
          onMouseDown={(event) => event.stopPropagation()}
          aria-label="Toggle maximize"
        >
          □
        </button>
        <button
          type="button"
          onClick={minimizeWindow}
          onMouseDown={(event) => event.stopPropagation()}
          aria-label="Minimize"
        >
          -
        </button>
        <button
          type="button"
          onClick={closeWindow}
          onMouseDown={(event) => event.stopPropagation()}
          aria-label="Close"
        >
          ×
        </button>
      </div>
    </header>
  );
}

function hideFileExtension(name: string) {
  return name.replace(/\.[^./\\]+$/, "");
}

function displayDocument(document: OpenedDocument): OpenedDocument {
  return {
    ...document,
    name: hideFileExtension(document.name),
  };
}

function displayNote(note: NoteEntry): NoteEntry {
  return {
    ...note,
    name: hideFileExtension(note.name),
  };
}

function displayQuickNote(draft: QuickNoteDraft): QuickNoteDraft {
  return {
    ...draft,
    name: hideFileExtension(draft.name),
  };
}

function App() {
  const isQuickNoteWindow = new URLSearchParams(window.location.search).has(
    "quick-note",
  );
  const [document, setDocument] = useState<OpenedDocument | null>(null);
  const [notes, setNotes] = useState<NoteEntry[]>([]);
  const [quickNotePath, setQuickNotePath] = useState("");
  const [quickNoteTitle, setQuickNoteTitle] = useState("");
  const [quickNoteContent, setQuickNoteContent] = useState("");
  const [quickNoteStatus, setQuickNoteStatus] = useState("");
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [isOpening, setIsOpening] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [markdownEditorWidth, setMarkdownEditorWidth] = useState(50);
  const isMarkdown = document?.kind === "markdown";

  function resetForMd() {
    setMarkdownEditorWidth(50);
  }

  function loadDocument(nextDocument: OpenedDocument, nextStatus: string) {
    setDocument(displayDocument(nextDocument));
    setStatus(nextStatus);

    if (nextDocument.kind === "markdown") {
      resetForMd();
    }
  }

  function upsertNote(nextDocument: OpenedDocument, previousPath?: string) {
    if (nextDocument.kind !== "markdown") {
      return;
    }

    setNotes((currentNotes) => {
      const filteredNotes = currentNotes.filter(
        (note) => note.path !== nextDocument.path && note.path !== previousPath,
      );
      const nextNotes = [
        ...filteredNotes,
        displayNote({ name: nextDocument.name, path: nextDocument.path }),
      ];

      return nextNotes.sort((left, right) =>
        left.name.localeCompare(right.name, "en"),
      );
    });
  }

  function upsertNoteEntry(nextNote: NoteEntry, previousPath?: string) {
    setNotes((currentNotes) => {
      const filteredNotes = currentNotes.filter(
        (note) => note.path !== nextNote.path && note.path !== previousPath,
      );
      const nextNotes = [...filteredNotes, nextNote];

      return nextNotes.sort((left, right) =>
        left.name.localeCompare(right.name, "en"),
      );
    });
  }

  async function SaveQuickNote() {
    setError("");
    setQuickNoteStatus("Saving...");

    try {
      const savedDraft = await invoke<QuickNoteDraft>("save_quick_note", {
        title: quickNoteTitle,
        content: quickNoteContent,
      });
      const displayDraft = displayQuickNote(savedDraft);
      setQuickNotePath(displayDraft.path);

      if (!quickNoteTitle.trim()) {
        setQuickNoteTitle(displayDraft.name);
      }

      setQuickNoteStatus("Saved");
    } catch (error) {
      setQuickNoteStatus("");
      setError(error instanceof Error ? error.message : String(error));
    }
  }

  async function OpenDoc() {
    setIsOpening(true);
    setError("");
    setStatus("");

    try {
      const selectedDocument = await invoke<OpenedDocument | null>("open_doc");

      if (selectedDocument) {
        loadDocument(selectedDocument, "Opened");
      }
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsOpening(false);
    }
  }

  async function NewDoc() {
    setIsOpening(true);
    setError("");
    setStatus("");

    try {
      const createdDocument = await invoke<OpenedDocument>("new_doc");
      loadDocument(createdDocument, "Created");
      upsertNote(createdDocument);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsOpening(false);
    }
  }

  async function OpenNote(path: string) {
    setIsOpening(true);
    setError("");
    setStatus("");

    try {
      const selectedDocument = await invoke<OpenedDocument>("open_note", {
        path,
      });
      loadDocument(selectedDocument, "Opened");
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsOpening(false);
    }
  }

  async function SaveDoc() {
    if (!document || isSaving) {
      return;
    }

    setIsSaving(true);
    setError("");
    setStatus("Saving...");

    try {
      const previousPath = document.path;
      const savedDocument = await invoke<OpenedDocument>("save_doc", {
        path: document.path,
        name: document.name,
        content: document.content,
      });
      loadDocument(savedDocument, "Saved");
      upsertNote(savedDocument, previousPath);
    } catch (error) {
      setStatus("");
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }

  function updateDocumentContent(content: string) {
    setDocument((currentDocument) =>
      currentDocument ? { ...currentDocument, content } : currentDocument,
    );
    setStatus("Unsaved");
  }

  function updateDocumentName(name: string) {
    setDocument((currentDocument) =>
      currentDocument ? { ...currentDocument, name } : currentDocument,
    );
    setStatus("Unsaved");
  }

  function startMarkdownResize(event: React.PointerEvent<HTMLDivElement>) {
    const container = event.currentTarget.parentElement;

    if (!container) {
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    const containerRect = container.getBoundingClientRect();

    function updateWidth(clientX: number) {
      const nextWidth =
        ((clientX - containerRect.left) / containerRect.width) * 100;
      setMarkdownEditorWidth(Math.min(75, Math.max(25, nextWidth)));
    }

    function handlePointerMove(moveEvent: PointerEvent) {
      updateWidth(moveEvent.clientX);
    }

    function handlePointerUp() {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    }

    updateWidth(event.clientX);
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);
  }

  useEffect(() => {
    async function loadNotes() {
      if (isQuickNoteWindow) {
        return;
      }

      try {
        const noteEntries = await invoke<NoteEntry[]>("list_notes");
        setNotes(noteEntries.map(displayNote));
      } catch (error) {
        setError(error instanceof Error ? error.message : String(error));
      }
    }

    loadNotes();
  }, [isQuickNoteWindow]);

  useEffect(() => {
    async function loadQuickNoteInfo() {
      if (!isQuickNoteWindow) {
        return;
      }

      try {
        const draft = await invoke<QuickNoteDraft>("quick_note_info");
        const displayDraft = displayQuickNote(draft);
        setQuickNoteTitle(displayDraft.name);
        setQuickNotePath(displayDraft.path);
      } catch (error) {
        setError(error instanceof Error ? error.message : String(error));
      }
    }

    loadQuickNoteInfo();
  }, [isQuickNoteWindow]);

  useEffect(() => {
    if (isQuickNoteWindow) {
      return;
    }

    const unlistenPromise = listen<NoteListUpdate>(
      "note-list-updated",
      (event) => {
        upsertNoteEntry(displayNote(event.payload.note), event.payload.previousPath);
      },
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [isQuickNoteWindow]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();

        if (isQuickNoteWindow) {
          SaveQuickNote();
        } else {
          SaveDoc();
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    document,
    isSaving,
    isQuickNoteWindow,
    quickNoteTitle,
    quickNoteContent,
  ]);

  if (isQuickNoteWindow) {
    return (
      <>
        <AppTitleBar />
        <main className="quick-note-shell">
          <img className="quick-note-watermark" src={isekai2Image} alt="" />
          <input
            className="quick-note-title"
            value={quickNoteTitle}
            onChange={(event) => {
              setQuickNoteTitle(event.currentTarget.value);
              setQuickNoteStatus("Unsaved");
            }}
            placeholder="Title"
            aria-label="Quick note title"
          />
          <textarea
            className="quick-note-body"
            value={quickNoteContent}
            onChange={(event) => {
              setQuickNoteContent(event.currentTarget.value);
              setQuickNoteStatus("Unsaved");
            }}
            placeholder="Body"
            aria-label="Quick note body"
            autoFocus
          />
          <footer className="quick-note-footer">
            <span>{quickNotePath}</span>
            {quickNoteStatus && <strong>{quickNoteStatus}</strong>}
          </footer>
          {error && <p className="document-error">{error}</p>}
        </main>
      </>
    );
  }

  return (
    <>
      <AppTitleBar />
      <main className="app-shell">
      <aside className="sidebar" aria-label="Document actions">
        <button
          className="menu-button"
          type="button"
          onClick={OpenDoc}
          disabled={isOpening}
        >
          {isOpening ? "Opening..." : "Open"}
        </button>
        <button
          className="menu-button"
          type="button"
          onClick={NewDoc}
          disabled={isOpening}
        >
          New Note
        </button>

        <div className="notes-panel" aria-label="Notes list">
          <img className="notes-watermark" src={listImage} alt="" />
          <div className="notes-panel-title">Notes</div>
          <div className="notes-list">
            {notes.length > 0 ? (
              notes.map((note) => (
                <button
                  className={`note-list-item${
                    document?.path === note.path ? " active" : ""
                  }`}
                  type="button"
                  key={note.path}
                  onClick={() => OpenNote(note.path)}
                  disabled={isOpening}
                >
                  {note.name}
                </button>
              ))
            ) : (
              <p className="notes-empty">No notes yet</p>
            )}
          </div>
        </div>
      </aside>

      <section className="workspace" aria-label="Document content">
        <article className="document-viewer">
          <img className="document-watermark" src={isekaiImage} alt="" />
          {document ? (
            <>
              <header className="document-header">
                <div>
                  <input
                    className="document-title-input"
                    value={document.name}
                    onChange={(event) =>
                      updateDocumentName(event.currentTarget.value)
                    }
                    aria-label="Note title"
                  />
                  <p>{document.path}</p>
                </div>
                {status && <span className="document-status">{status}</span>}
              </header>

              {isMarkdown ? (
                <div className="markdown-workspace">
                  <textarea
                    className="document-editor markdown-source"
                    style={{ flexBasis: `${markdownEditorWidth}%` }}
                    value={document.content}
                    onChange={(event) =>
                      updateDocumentContent(event.currentTarget.value)
                    }
                    spellCheck={false}
                    aria-label="Markdown source"
                  />
                  <div
                    className="markdown-resizer"
                    role="separator"
                    aria-orientation="vertical"
                    aria-label="Resize Markdown editor and preview"
                    onPointerDown={startMarkdownResize}
                  />
                  <div
                    className="markdown-preview"
                    style={{ flexBasis: `${100 - markdownEditorWidth}%` }}
                    aria-label="Markdown preview"
                  >
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>
                      {document.content}
                    </ReactMarkdown>
                  </div>
                </div>
              ) : (
                <textarea
                  className="document-editor"
                  value={document.content}
                  onChange={(event) =>
                    updateDocumentContent(event.currentTarget.value)
                  }
                  spellCheck={false}
                  aria-label="Document content"
                />
              )}
            </>
          ) : (
            <p className="document-placeholder">
              Open or create a txt/md document to start writing
            </p>
          )}

          {error && <p className="document-error">{error}</p>}
        </article>
      </section>
      </main>
    </>
  );
}

export default App;
