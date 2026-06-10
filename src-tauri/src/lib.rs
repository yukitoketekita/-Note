mod config;

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalSize, Size, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    Window, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

#[derive(Serialize)]
// Document data returned to the frontend. More metadata can be added here later.
struct OpenedDocument {
    name: String,
    path: String,
    kind: DocumentKind,
    content: String,
}

#[derive(Clone, Serialize)]
// Lightweight note list entry without file content.
struct NoteEntry {
    name: String,
    path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteListUpdate {
    note: NoteEntry,
    previous_path: Option<String>,
}

#[derive(Default)]
struct QuickNoteState {
    paths: Mutex<HashMap<String, PathBuf>>,
}

#[derive(Serialize)]
struct QuickNoteDraft {
    name: String,
    path: String,
}

const QUICK_NOTE_IMAGE_WIDTH: f64 = 1695.0;
const QUICK_NOTE_IMAGE_HEIGHT: f64 = 1440.0;
const QUICK_NOTE_IMAGE_RATIO: f64 = QUICK_NOTE_IMAGE_WIDTH / QUICK_NOTE_IMAGE_HEIGHT;
const QUICK_NOTE_TITLEBAR_HEIGHT: f64 = 30.0;
const QUICK_NOTE_INITIAL_CONTENT_WIDTH: f64 = 380.0;
const QUICK_NOTE_MIN_CONTENT_WIDTH: f64 = 260.0;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
// Supported document types. Add more variants here as the reader grows.
enum DocumentKind {
    Text,
    Markdown,
    Unknown,
}

#[tauri::command]
// Opens a file picker, reads the selected document, and returns it to the frontend.
fn open_doc() -> Result<Option<OpenedDocument>, String> {
    let Some(path) = pick_document_file() else {
        return Ok(None);
    };

    read_document(&path).map(Some)
}

#[tauri::command]
// Saves the current editor content and renames the source file when the title changed.
fn save_doc(path: String, name: String, content: String) -> Result<OpenedDocument, String> {
    let saved_path = rename_document_if_needed(Path::new(&path), &name)?;

    write_document(&saved_path, &content)?;
    read_document(&saved_path)
}

// Opens the system file picker with the currently supported document formats.
fn pick_document_file() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Document")
        .add_filter("Text document", &["txt"])
        .add_filter("Markdown document", &["md", "markdown"])
        .add_filter("All files", &["*"])
        .pick_file()
}

// Reads a document after detecting its type.
fn read_document(path: &Path) -> Result<OpenedDocument, String> {
    let kind = detect_document_kind(path);
    let content = match kind {
        DocumentKind::Text | DocumentKind::Markdown => read_text_document(path)?,
        DocumentKind::Unknown => {
            return Err("This file type is not supported for reading yet.".to_string());
        }
    };

    Ok(OpenedDocument {
        name: file_name(path),
        path: path.display().to_string(),
        kind,
        content,
    })
}

// Writes a document after detecting its type.
fn write_document(path: &Path, content: &str) -> Result<(), String> {
    match detect_document_kind(path) {
        DocumentKind::Text | DocumentKind::Markdown => write_text_document(path, content),
        DocumentKind::Unknown => Err("This file type is not supported for writing yet.".to_string()),
    }
}

// Renames the file in place when the title no longer matches the current file name.
fn rename_document_if_needed(path: &Path, next_name: &str) -> Result<PathBuf, String> {
    let next_path = renamed_document_path(path, next_name)?;

    if paths_equal(path, &next_path) {
        return Ok(path.to_path_buf());
    }

    if next_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    fs::rename(path, &next_path).map_err(|error| format!("Failed to rename file: {error}"))?;

    Ok(next_path)
}

// Builds the next path from the title and preserves the current extension when omitted.
fn renamed_document_path(path: &Path, next_name: &str) -> Result<PathBuf, String> {
    let clean_name = next_name.trim();

    if clean_name.is_empty() {
        return Err("File name cannot be empty.".to_string());
    }

    if has_invalid_file_name_char(clean_name) {
        return Err("File name cannot contain \\ / : * ? \" < > |".to_string());
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let mut file_name = clean_name.to_string();

    if Path::new(clean_name).extension().is_none() {
        if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
            file_name.push('.');
            file_name.push_str(extension);
        }
    }

    Ok(parent.join(file_name))
}

// Windows file names cannot contain these characters.
fn has_invalid_file_name_char(file_name: &str) -> bool {
    file_name.chars().any(|character| {
        matches!(
            character,
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
        )
    })
}

// Compares paths consistently across case-insensitive Windows paths.
fn paths_equal(left: &Path, right: &Path) -> bool {
    if cfg!(windows) {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    } else {
        left == right
    }
}

// Detects the document type from the file extension.
fn detect_document_kind(path: &Path) -> DocumentKind {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("txt") => DocumentKind::Text,
        Some("md" | "markdown") => DocumentKind::Markdown,
        _ => DocumentKind::Unknown,
    }
}

// Reads UTF-8 text content.
fn read_text_document(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("Failed to read file: {error}"))
}

// Overwrites text content with the complete editor buffer.
fn write_text_document(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|error| format!("Failed to write file: {error}"))
}

// Extracts a display file name from a path.
fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled Document")
        .to_string()
}

//==================================================================

#[tauri::command]
// Creates a new Markdown note and returns it as the active document.
fn new_doc() -> Result<OpenedDocument, String> {
    let notes_dir = ensure_notes_dir()?;
    let note_path = create_next_note_file(&notes_dir)?;

    read_document(&note_path)
}

#[tauri::command]
// Loads the Markdown files in Notes once at startup.
fn list_notes() -> Result<Vec<NoteEntry>, String> {
    let notes_dir = ensure_notes_dir()?;
    let mut notes = markdown_files_in_dir(&notes_dir)?;

    notes.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));

    Ok(notes)
}

#[tauri::command]
// Opens a Markdown file from the Notes directory.
fn open_note(path: String) -> Result<OpenedDocument, String> {
    let note_path = PathBuf::from(path);

    ensure_path_is_in_notes_dir(&note_path)?;
    read_document(&note_path)
}

fn ensure_notes_dir() -> Result<PathBuf, String> {
    config::ensure_notes_dir()
}

// Reads Markdown file entries for the sidebar.
fn markdown_files_in_dir(notes_dir: &Path) -> Result<Vec<NoteEntry>, String> {
    let entries =
        fs::read_dir(notes_dir).map_err(|error| format!("Failed to read Notes directory: {error}"))?;
    let mut notes = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| format!("Failed to read Notes directory entry: {error}"))?;
        let path = entry.path();

        if entry
            .file_type()
            .map_err(|error| format!("Failed to read file type: {error}"))?
            .is_file()
            && matches!(detect_document_kind(&path), DocumentKind::Markdown)
        {
            notes.push(NoteEntry {
                name: file_name(&path),
                path: path.display().to_string(),
            });
        }
    }

    Ok(notes)
}

// Restricts sidebar opens to files inside Notes.
fn ensure_path_is_in_notes_dir(path: &Path) -> Result<(), String> {
    let notes_dir = ensure_notes_dir()?
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Notes directory path: {error}"))?;
    let note_path = path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve note file path: {error}"))?;

    if !note_path.starts_with(notes_dir) {
        return Err("Only notes inside the Notes directory can be opened.".to_string());
    }

    Ok(())
}

// Creates the next note file: note 1.md, note 2.md, and so on.
fn create_next_note_file(notes_dir: &Path) -> Result<PathBuf, String> {
    let mut next_index = count_files_in_dir(notes_dir)? + 1;

    loop {
        let note_path = notes_dir.join(format!("note {next_index}.md"));

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&note_path)
        {
            Ok(_) => return Ok(note_path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                next_index += 1;
            }
            Err(error) => return Err(format!("Failed to create new note: {error}")),
        }
    }
}

// Counts regular files in a directory.
fn count_files_in_dir(dir: &Path) -> Result<usize, String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("Failed to read Notes directory: {error}"))?;
    let mut file_count = 0;

    for entry in entries {
        let entry = entry.map_err(|error| format!("Failed to read Notes directory entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Failed to read file type: {error}"))?;

        if file_type.is_file() {
            file_count += 1;
        }
    }

    Ok(file_count)
}

//================================

#[tauri::command]
fn quick_note_info(
    window: WebviewWindow,
    quick_notes: State<'_, QuickNoteState>,
) -> Result<QuickNoteDraft, String> {
    let label = window.label().to_string();
    let paths = quick_notes
        .paths
        .lock()
        .map_err(|_| "Failed to read quick note state.".to_string())?;
    let path = paths
        .get(&label)
        .ok_or_else(|| "No file path is registered for this quick note window.".to_string())?;

    Ok(quick_note_draft(path))
}

#[tauri::command]
fn save_quick_note(
    window: WebviewWindow,
    quick_notes: State<'_, QuickNoteState>,
    title: String,
    content: String,
) -> Result<QuickNoteDraft, String> {
    let label = window.label().to_string();
    let current_path = {
        let paths = quick_notes
            .paths
            .lock()
            .map_err(|_| "Failed to read quick note state.".to_string())?;

        paths
            .get(&label)
            .cloned()
            .ok_or_else(|| "No file path is registered for this quick note window.".to_string())?
    };
    let next_path = quick_note_path_for_title(&current_path, &title)?;
    let previous_path = if paths_equal(&current_path, &next_path) {
        None
    } else {
        Some(current_path.display().to_string())
    };

    if !paths_equal(&current_path, &next_path) {
        if next_path.exists() {
            return Err("A quick note with this name already exists.".to_string());
        }

        if current_path.exists() {
            fs::rename(&current_path, &next_path)
                .map_err(|error| format!("Failed to rename quick note: {error}"))?;
        }
    }

    write_text_document(&next_path, &content)?;

    {
        let mut paths = quick_notes
            .paths
            .lock()
            .map_err(|_| "Failed to write quick note state.".to_string())?;
        paths.insert(label, next_path.clone());
    }

    let _ = window.app_handle().emit_to(
        "main",
        "note-list-updated",
        NoteListUpdate {
            note: NoteEntry {
                name: file_name(&next_path),
                path: next_path.display().to_string(),
            },
            previous_path,
        },
    );

    Ok(quick_note_draft(&next_path))
}

#[tauri::command]
fn titlebar_toggle_maximize(window: WebviewWindow) -> Result<(), String> {
    let is_maximized = window
        .is_maximized()
        .map_err(|error| format!("Failed to read maximize state: {error}"))?;

    if is_maximized {
        window
            .unmaximize()
            .map_err(|error| format!("Failed to restore window: {error}"))
    } else {
        window
            .maximize()
            .map_err(|error| format!("Failed to maximize window: {error}"))
    }
}

#[tauri::command]
fn titlebar_minimize(window: WebviewWindow) -> Result<(), String> {
    window
        .minimize()
        .map_err(|error| format!("Failed to minimize window: {error}"))
}

#[tauri::command]
fn titlebar_close(window: WebviewWindow) -> Result<(), String> {
    window
        .hide()
        .map_err(|error| format!("Failed to hide window: {error}"))
}

#[tauri::command]
fn titlebar_start_dragging(window: WebviewWindow) -> Result<(), String> {
    window
        .start_dragging()
        .map_err(|error| format!("Failed to start window drag: {error}"))
}

#[tauri::command]
// Returns the current notes directory path.
fn get_notes_dir() -> Result<String, String> {
    config::notes_dir_path().map(|path| path.display().to_string())
}

#[tauri::command]
// Sets a custom notes directory and persists it.
fn set_notes_dir(path: String) -> Result<(), String> {
    let dir = PathBuf::from(&path);
    if !dir.is_absolute() {
        return Err("The notes directory path must be absolute.".to_string());
    }

    // Ensure the directory exists
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create directory: {error}"))?;

    let mut config = config::read_config();
    config.notes_dir = Some(path);
    config::write_config(&config)
}

#[tauri::command]
// Deletes a note file from disk.
fn delete_note(path: String) -> Result<(), String> {
    let note_path = PathBuf::from(path);
    if !note_path.exists() {
        return Err("File does not exist.".to_string());
    }
    
    fs::remove_file(&note_path)
        .map_err(|error| format!("Failed to delete file: {error}"))
}

#[tauri::command]
// Opens a folder picker so the user can choose where notes are stored.
fn pick_notes_dir() -> Result<Option<String>, String> {
    let path = rfd::FileDialog::new()
        .set_title("Select Notes Directory")
        .pick_folder();
    Ok(path.map(|p| p.display().to_string()))
}

fn open_quick_note_window(app: &tauri::AppHandle) {
    let Ok(path) = create_next_quick_note_path() else {
        return;
    };
    let label = format!("quick-note-{}", timestamp_millis());
    let initial_content_height = QUICK_NOTE_INITIAL_CONTENT_WIDTH / QUICK_NOTE_IMAGE_RATIO;
    let min_content_height = QUICK_NOTE_MIN_CONTENT_WIDTH / QUICK_NOTE_IMAGE_RATIO;

    if WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::App("index.html?quick-note=1".into()),
    )
    .title("Quick Note")
    .inner_size(
        QUICK_NOTE_INITIAL_CONTENT_WIDTH,
        initial_content_height + QUICK_NOTE_TITLEBAR_HEIGHT,
    )
    .min_inner_size(
        QUICK_NOTE_MIN_CONTENT_WIDTH,
        min_content_height + QUICK_NOTE_TITLEBAR_HEIGHT,
    )
    .decorations(false)
    .resizable(true)
    .always_on_top(true)
    .build()
    .is_ok()
    {
        let state = app.state::<QuickNoteState>();
        let paths_lock = state.paths.lock();

        if let Ok(mut paths) = paths_lock {
            paths.insert(label, path);
        }
    }
}

fn is_quick_note_window(window: &Window) -> bool {
    window.label().starts_with("quick-note-")
}

// Keeps quick note windows aligned to the background image aspect ratio.
fn enforce_quick_note_aspect_ratio(window: &Window, size: PhysicalSize<u32>) {
    if !is_quick_note_window(window) {
        return;
    }

    let scale_factor = window.scale_factor().unwrap_or(1.0);
    let titlebar_height = QUICK_NOTE_TITLEBAR_HEIGHT * scale_factor;
    let width = size.width as f64;
    let height = size.height as f64;
    let content_height = (height - titlebar_height).max(1.0);
    let current_ratio = width / content_height;

    let (next_width, next_height) = if current_ratio > QUICK_NOTE_IMAGE_RATIO {
        (content_height * QUICK_NOTE_IMAGE_RATIO, height)
    } else {
        (width, width / QUICK_NOTE_IMAGE_RATIO + titlebar_height)
    };

    let next_width = next_width.round().max(1.0) as u32;
    let next_height = next_height.round().max(1.0) as u32;

    if next_width.abs_diff(size.width) <= 2 && next_height.abs_diff(size.height) <= 2 {
        return;
    }

    let _ = window.set_size(Size::Physical(PhysicalSize::new(next_width, next_height)));
}

fn create_next_quick_note_path() -> Result<PathBuf, String> {
    let notes_dir = ensure_notes_dir()?;
    let mut next_index = count_files_in_dir(&notes_dir)? + 1;

    loop {
        let path = notes_dir.join(format!("quick note {next_index}.md"));

        if !path.exists() {
            return Ok(path);
        }

        next_index += 1;
    }
}

fn quick_note_path_for_title(current_path: &Path, title: &str) -> Result<PathBuf, String> {
    let clean_title = title.trim();

    if clean_title.is_empty() {
        return Ok(current_path.to_path_buf());
    }

    if has_invalid_file_name_char(clean_title) {
        return Err("File name cannot contain \\ / : * ? \" < > |".to_string());
    }

    let parent = current_path.parent().unwrap_or_else(|| Path::new(""));
    let mut file_name = clean_title.to_string();

    if Path::new(clean_title).extension().is_none() {
        file_name.push_str(".md");
    }

    Ok(parent.join(file_name))
}

fn quick_note_draft(path: &Path) -> QuickNoteDraft {
    QuickNoteDraft {
        name: file_name(path),
        path: path.display().to_string(),
    }
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

//================================

// Shows the main window and restores focus.
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// Hides the main window while keeping the app alive in the tray.
fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

// Ctrl+Alt+A toggle logic.
fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                hide_main_window(app);
            }
            _ => {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }
    }
}

// Creates the system tray icon.
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
    let icon = app
        .default_window_icon()
        .expect("missing default window icon")
        .clone();

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("ipad")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

// Registers global shortcuts for toggling the main window and opening quick notes.
fn setup_global_shortcut(app: &tauri::App) -> tauri::Result<()> {
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_shortcuts(["ctrl+alt+a", "ctrl+alt+q"])
            .expect("invalid global shortcut")
            .with_handler(|app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyA) {
                        toggle_main_window(app);
                    }

                    if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyQ) {
                        open_quick_note_window(app);
                    }
                }
            })
            .build(),
    )?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(QuickNoteState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup_tray(app)?;
            setup_global_shortcut(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }

            if let WindowEvent::Resized(size) = event {
                enforce_quick_note_aspect_ratio(window, *size);
            }
        })
        .invoke_handler(tauri::generate_handler![
            open_doc,
            save_doc,
            new_doc,
            list_notes,
            open_note,
            quick_note_info,
            save_quick_note,
            titlebar_toggle_maximize,
            titlebar_minimize,
            titlebar_close,
            titlebar_start_dragging,
            get_notes_dir,
            set_notes_dir,
            delete_note,
            pick_notes_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
